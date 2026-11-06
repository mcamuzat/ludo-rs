#!/usr/bin/env python3
"""Exports twixtbot's network for games/twixt.

    uv run --with torch --with numpy scripts/twixt-export.py <twixtbot-ui clone> <repo root>

Reads <clone>/model/torch.pt (twixtbot-ui, MIT, converted from twixtbot,
MIT), folds every BatchNorm into the preceding layer and writes:
  web/public/twixt/net-f16.bin        the weights, float16 (format below)
  games/twixt/tests/fixtures/net-ref.json
                                      reference positions: twixtbot's encoded
                                      inputs and the outputs of the folded
                                      network with the float16 weights.

File format (little endian): b"TWXN", u32 version (1), u32 channels,
u32 blocks, u32 value_hidden, u32 value_reductions, then f16 tensors:
  stem: location w [C,2,1,1], pegs w [C,2,5,5], links w [C,8,4,4], bias [C]
  each block: conv1 w [C,C,5,5], b [C], conv2 w [C,C,5,5], b [C]
  policy: conv1 w [2,C,1,1], b [2], conv2 w [1,2,1,1]
  each value reduction: w [C,C,5,5], b [C]
  value: fc w [H, C*2*2], b [H], out w [3,H]
"""
import json
import pathlib
import struct
import sys

import numpy as np
import torch
import torch.nn.functional as F

BN_EPS = 1e-3


def load(clone):
    sys.path.insert(0, str(clone))
    from src.backend import naf, twixt  # noqa: E402
    from src.backend.torchmodel import TwixtNet  # noqa: E402

    ckpt = torch.load(clone / "model" / "torch.pt", map_location="cpu", weights_only=True)
    config = dict(ckpt["config"])
    model = TwixtNet(**config)
    model.load_state_dict(ckpt["state_dict"])
    model.eval()
    return naf, twixt, model, config


def fold(sd, bn):
    g, b = sd[f"{bn}.weight"], sd[f"{bn}.bias"]
    s = g / torch.sqrt(sd[f"{bn}.running_var"] + BN_EPS)
    return s, b - sd[f"{bn}.running_mean"] * s


def folded_tensors(model, config):
    """The tensors of the file, in order, as float32 numpy arrays."""
    sd = {k: v.detach().float() for k, v in model.state_dict().items()}
    out = []
    s, t = fold(sd, "primary_bn")
    sc = s[:, None, None, None]
    out += [sd["location.weight"] * sc, sd["pegs.weight"] * sc, sd["links.weight"] * sc, t]
    for i in range(config["blocks"]):
        for j in (1, 2):
            s, t = fold(sd, f"blocks.{i}.bn{j}")
            out += [sd[f"blocks.{i}.conv{j}.weight"] * s[:, None, None, None], t]
    s, t = fold(sd, "policy_bn")
    out += [sd["policy_conv1.weight"] * s[:, None, None, None], t, sd["policy_conv2.weight"]]
    for i in range(config["value_reductions"]):
        s, t = fold(sd, f"value_bn.{i}")
        out += [sd[f"value_conv.{i}.weight"] * s[:, None, None, None], t]
    s, t = fold(sd, "value_bn_fc")
    out += [sd["value_fc.weight"] * s[:, None], t, sd["value_out.weight"]]
    return [x.numpy().astype(np.float32) for x in out]


def same_pad(n):
    """SAME padding of a 5x5 stride-2 convolution, as in torchmodel.py."""
    out = (n + 1) // 2
    total = max((out - 1) * 2 + 5 - n, 0)
    return total // 2, total - total // 2


def forward(tensors, config, pegs, links, locs):
    """The folded network; inputs NHWC numpy arrays (H = x), batch of one."""
    t = [torch.from_numpy(x) for x in tensors]
    it = iter(t)

    def nchw(a):
        return torch.from_numpy(np.ascontiguousarray(a, dtype=np.float32)).permute(2, 0, 1)[None]

    pegs, links, locs = nchw(pegs), F.pad(nchw(links), (1, 2, 1, 2)), nchw(locs)
    wl, wp, wk, b = next(it), next(it), next(it), next(it)
    h = F.conv2d(locs, wl) + F.conv2d(pegs, wp, padding=2) + F.conv2d(links, wk)
    h = torch.abs(h + b[None, :, None, None])
    for _ in range(config["blocks"]):
        w1, b1, w2, b2 = next(it), next(it), next(it), next(it)
        r = torch.abs(F.conv2d(h, w1, b1, padding=2))
        r = torch.abs(F.conv2d(r, w2, b2, padding=2))
        h = h + r
    w1, b1, w2 = next(it), next(it), next(it)
    p = F.conv2d(torch.abs(F.conv2d(h, w1, b1)), w2)
    policy = p[:, :, 1:-1, :].reshape(-1)
    v = h
    for _ in range(config["value_reductions"]):
        w, bb = next(it), next(it)
        ph, pw = same_pad(v.shape[-2]), same_pad(v.shape[-1])
        v = F.pad(v, (pw[0], pw[1], ph[0], ph[1]))
        v = torch.abs(F.conv2d(v, w, bb, stride=2))
    v = v.permute(0, 2, 3, 1).flatten(1)
    wf, bf, wo = next(it), next(it), next(it)
    v = torch.abs(v @ wf.T + bf)
    value = (v @ wo.T).reshape(-1)
    return policy.detach().numpy(), value.detach().numpy()


def point_name(p):
    return f"{chr(ord('a') + p.x)}{p.y + 1}"


def greedy(naf, twixt, model, game, plies):
    """Plays `plies` moves of the original network's best policy move."""
    from src.backend.point import Point  # noqa: E402

    for _ in range(plies):
        if game.result is not None or game.just_won():
            break
        pegs, links, locs = naf.NetInputs(game).to_input_arrays(False)
        inputs = [torch.from_numpy(np.asarray(x, dtype=np.float32))[None] for x in (pegs, links, locs)]
        with torch.inference_mode():
            _, logits = model(*inputs)
        logits = logits[0].numpy()
        legal = naf.legal_move_policy_array(game)
        logits[legal == 0] = -np.inf
        game.play(naf.policy_index_point(game, int(np.argmax(logits))))
    return game


def play(twixt, moves):
    from src.backend.point import Point  # noqa: E402

    game = twixt.Game(True)  # Little Golem: own links may cross.
    for m in moves:
        game.play(twixt.SWAP if m == "swap" else Point(m))
    return game


def history_names(twixt, game):
    return ["swap" if h == twixt.SWAP else point_name(h) for h in game.history]


def main():
    clone, root = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
    naf, twixt, model, config = load(clone)
    assert config["value_triple"] and not config["use_recents"]
    assert config["value_padding"].upper() == "SAME"

    tensors = folded_tensors(model, config)
    print("parameters:", sum(x.size for x in tensors))
    half = [x.astype("<f2") for x in tensors]
    rounded = [x.astype(np.float32) for x in half]

    out = root / "web" / "public" / "twixt" / "net-f16.bin"
    out.parent.mkdir(parents=True, exist_ok=True)
    with open(out, "wb") as f:
        f.write(b"TWXN")
        f.write(struct.pack("<5I", 1, config["channels"], config["blocks"], config["value_hidden"], config["value_reductions"]))
        for x in half:
            f.write(x.tobytes())
    print("wrote", out, out.stat().st_size, "bytes")

    # Openings, then greedy play by the original network.
    specs = [
        ([], 0),
        (["k12"], 0),
        (["c8", "swap"], 0),
        (["l12"], 20),
        (["e6", "swap"], 25),
        (["m13"], 39),
        (["j14"], 31),
        (["d20"], 41),
        # Own links crossing (Task 3's own_links_may_cross, twixtbot notation).
        (["f6", "p16", "g8", "p20", "g6", "t16", "f8"], 0),
    ]
    games = [greedy(naf, twixt, model, play(twixt, m), n) for m, n in specs]
    # Near the end of a won game: replay a finished greedy game minus 2 plies.
    finished = greedy(naf, twixt, model, play(twixt, ["k11"]), 400)
    names = history_names(twixt, finished)
    games.append(play(twixt, names[:-2]))

    positions = []
    for game in games:
        pegs, links, locs = naf.NetInputs(game).to_input_arrays(False)
        inputs = [torch.from_numpy(np.asarray(x, dtype=np.float32))[None] for x in (pegs, links, locs)]
        with torch.inference_mode():
            v_ref, p_ref = model(*inputs)
        p32, v32 = forward(tensors, config, pegs, links, locs)
        fold_err = max(np.abs(p32 - p_ref[0].numpy()).max(), np.abs(v32 - v_ref[0].numpy()).max())
        p16, v16 = forward(rounded, config, pegs, links, locs)
        f16_err = max(np.abs(p16 - p32).max(), np.abs(v16 - v32).max())
        turn = "red" if game.turn == twixt.Game.WHITE else "black"
        print(f"{len(game.history):3} plies, {turn} to move: fold {fold_err:.2e}, f16 {f16_err:.2e}")
        positions.append(
            {
                "moves": history_names(twixt, game),
                "pegs": np.asarray(pegs).astype(int).reshape(-1).tolist(),
                "links": np.asarray(links).astype(int).reshape(-1).tolist(),
                "policy": [round(float(x), 6) for x in p16],
                "value": [round(float(x), 6) for x in v16],
            }
        )

    fixture = root / "games" / "twixt" / "tests" / "fixtures" / "net-ref.json"
    fixture.parent.mkdir(parents=True, exist_ok=True)
    fixture.write_text(json.dumps({"positions": positions}, separators=(",", ":")))
    print("wrote", fixture)


if __name__ == "__main__":
    main()

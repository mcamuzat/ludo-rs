import { useEffect, useState } from "react";
import type { BoardProps } from "../registry";
import { TileBack, TileImg } from "./tiles";
import type { HandResult, MeldView, PlayerView, RiichiAction, RiichiView, RoundView, Tile, Wind } from "./types";
import "./riichi.css";

const WIND: Record<Wind, string> = { E: "Est", S: "Sud", W: "Ouest", N: "Nord" };
const DRAW: Record<string, string> = { Exhaustive: "Mur épuisé", NineTerminals: "Neuf terminaux", Abortive: "Nul abortif" };
const POSITIONS = ["bottom", "right", "top", "left"] as const;

function callLabel(a: RiichiAction): string {
  switch (a.type) {
    case "Riichi":
      return "Riichi";
    case "Tsumo":
      return "Tsumo";
    case "Ron":
      return "Ron";
    case "Pass":
      return "Passer";
    case "KyushuKyuhai":
      return "Neuf terminaux";
    case "Chi":
      return "Chi";
    case "Pon":
      return "Pon";
    case "Daiminkan":
    case "Ankan":
    case "Kakan":
      return "Kan";
    case "Discard":
    case "Tsumogiri":
      return "Défausser";
  }
}

function callTiles(a: RiichiAction): Tile[] {
  switch (a.type) {
    case "Chi":
    case "Pon":
      return [...a.with, a.tile];
    case "Daiminkan":
    case "Ankan":
    case "Kakan":
      return [a.tile];
    default:
      return [];
  }
}

function Melds({ melds }: { melds: MeldView[] }) {
  return (
    <div className="melds">
      {melds.map((m, i) => (
        <span key={i} className="meld">
          {m.tiles.map((t, j) => (
            <TileImg key={j} tile={t} size="sm" />
          ))}
        </span>
      ))}
    </div>
  );
}

function Pond({ player, position }: { player: PlayerView; position: (typeof POSITIONS)[number] }) {
  return (
    <div className={`pond pond-${position}`}>
      {player.discards.map((d, i) => (
        <TileImg key={i} tile={d.tile} size="sm" sideways={d.riichi} dim={d.tsumogiri} />
      ))}
    </div>
  );
}

function Seat({ player, name, dealer }: { player: PlayerView; name: string; dealer: boolean }) {
  return (
    <div className="seat-label">
      <strong>
        {WIND[player.seatWind]}
        {dealer ? " (donneur)" : ""}
      </strong>{" "}
      {name} — {player.score}
      {player.riichi && <span className="riichi-stick"> riichi</span>}
      <div className="concealed">
        {Array.from({ length: player.handSize }, (_, i) => (
          <TileBack key={i} size="sm" />
        ))}
      </div>
      <Melds melds={player.melds} />
    </div>
  );
}

function Center({ round }: { round: RoundView }) {
  return (
    <div className="center">
      <div className="round">
        {WIND[round.wind]} {round.hand}
      </div>
      <div>
        Honba {round.honba} · Riichi {round.riichiSticks}
      </div>
      <div>Mur : {round.wallLeft}</div>
      <div className="dora">
        Dora :{" "}
        {round.doraIndicators.map((t, i) => (
          <TileImg key={i} tile={t} size="sm" />
        ))}
      </div>
    </div>
  );
}

function ResultPanel({ result, names, onClose }: { result: HandResult; names: string[]; onClose(): void }) {
  return (
    <div className="hand-result" role="dialog">
      {result.wins.map((w, i) => (
        <div key={i} className="win">
          <h3>
            {names[w.seat]} — {w.from === null ? "Tsumo" : `Ron sur ${names[w.from]}`}
          </h3>
          <div className="row">
            {w.hand.map((t, j) => (
              <TileImg key={j} tile={t} size="sm" />
            ))}
            <Melds melds={w.melds} />
          </div>
          <ul className="yaku">
            {w.yaku.map((y) => (
              <li key={y.name}>
                {y.name} <span className="ja">{y.nameJa}</span>
              </li>
            ))}
          </ul>
          <p>{w.yakuman ? `Yakuman — ${w.points} points` : `${w.han} han ${w.fu} fu — ${w.points} points`}</p>
        </div>
      ))}
      {result.draw && <h3>{DRAW[result.draw]}</h3>}
      {result.uraIndicators.length > 0 && (
        <div className="row">
          Ura-dora :{" "}
          {result.uraIndicators.map((t, i) => (
            <TileImg key={i} tile={t} size="sm" />
          ))}
        </div>
      )}
      <ul className="deltas">
        {result.deltas.map((d, seat) => (
          <li key={seat}>
            {names[seat]} : {d > 0 ? `+${d}` : d}
          </li>
        ))}
      </ul>
      <button className="primary" onClick={onClose}>
        Continuer
      </button>
    </div>
  );
}

export function RiichiBoard({ view, legalActions, seatNames, submit }: BoardProps) {
  const v = view as RiichiView;
  const legal = legalActions as RiichiAction[];
  const self = v.me ?? 0;
  const order = [0, 1, 2, 3].map((k) => (self + k) % 4);
  const discardable = new Set(legal.flatMap((a) => (a.type === "Discard" ? [a.tile] : [])));
  const drawn = v.drawn;
  const tsumogiri = legal.some((a) => a.type === "Tsumogiri");
  const calls = legal.filter((a) => a.type !== "Discard" && a.type !== "Tsumogiri");
  const [showHelp, setShowHelp] = useState(true);
  const [dismissed, setDismissed] = useState(0);
  const result = v.lastHand && v.lastHand.number !== dismissed ? v.lastHand : null;

  useEffect(() => {
    if (!result) return;
    const number = result.number;
    const timer = setTimeout(() => setDismissed(number), 8000);
    return () => clearTimeout(timer);
  }, [result?.number]);

  const discard = (tile: Tile) => (discardable.has(tile) ? () => submit({ type: "Discard", tile }) : undefined);

  return (
    <div className="riichi">
      <div className="riichi-table">
        {order.map((seat, k) => (
          <div key={seat} className={`area area-${POSITIONS[k]}`}>
            <Pond player={v.players[seat]} position={POSITIONS[k]} />
          </div>
        ))}
        <Center round={v.round} />
      </div>
      <div className="seats-row">
        {order.slice(1).map((seat) => (
          <Seat key={seat} player={v.players[seat]} name={seatNames[seat]} dealer={v.round.dealer === seat} />
        ))}
      </div>
      <div className="my-area">
        <div className="my-hand">
          {v.hand.map((t, i) => (
            <TileImg key={i} tile={t} size="lg" onClick={discard(t)} dim={discardable.size > 0 && !discardable.has(t)} />
          ))}
          {drawn && (
            <span className="drawn">
              <TileImg
                tile={drawn}
                size="lg"
                onClick={tsumogiri ? () => submit({ type: "Tsumogiri", tile: drawn }) : undefined}
                dim={legal.length > 0 && !tsumogiri}
              />
            </span>
          )}
          <Melds melds={v.players[self].melds} />
        </div>
        <div className="calls">
          {calls.map((a, i) => (
            <button key={i} className={a.type === "Pass" ? undefined : "primary"} onClick={() => submit(a)}>
              {callLabel(a)}{" "}
              {callTiles(a).map((t, j) => (
                <TileImg key={j} tile={t} size="sm" />
              ))}
            </button>
          ))}
          {v.answered && <span className="status">En attente des autres joueurs…</span>}
        </div>
        <div className="help">
          <button onClick={() => setShowHelp((x) => !x)}>{showHelp ? "Masquer l'aide" : "Aide"}</button>
          {showHelp && v.help && (
            <span>
              {v.help.shanten <= 0 && v.help.waits.length > 0 ? (
                <>
                  Tenpai — attentes :{" "}
                  {v.help.waits.map((t) => (
                    <TileImg key={t} tile={t} size="sm" />
                  ))}
                </>
              ) : (
                `Shanten : ${Math.max(0, v.help.shanten)}`
              )}
            </span>
          )}
        </div>
        <div className="seat-label mine">
          <strong>{WIND[v.players[self].seatWind]}</strong> {seatNames[self]} — {v.players[self].score}
          {v.players[self].riichi && <span className="riichi-stick"> riichi</span>}
        </div>
      </div>
      {result && <ResultPanel result={result} names={seatNames} onClose={() => setDismissed(result.number)} />}
    </div>
  );
}

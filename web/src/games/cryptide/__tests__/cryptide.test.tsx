import { isValidElement, type ReactElement, type ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { CryptideBoard, TurnControls, actionsAt, describeEvent, place } from "../Board";
import { HintHelp } from "../HintHelp";
import { RiddleBoard, freshAttempt, sameHint } from "../Riddle";
import { hintLabel } from "../hints";
import type { Clue, CryptideAction, CryptideView, Hint, Riddle, Terrain } from "../types";

type AnyElement = ReactElement<Record<string, unknown>>;

function hostElements(node: ReactNode, out: AnyElement[] = []): AnyElement[] {
  if (Array.isArray(node)) {
    node.forEach((child) => hostElements(child, out));
    return out;
  }
  if (!isValidElement(node)) return out;
  const el = node as AnyElement;
  if (typeof el.type === "function") {
    return hostElements((el.type as (props: unknown) => ReactNode)(el.props), out);
  }
  out.push(el);
  return hostElements(el.props.children as ReactNode, out);
}

const NAMES = ["Ana", "Hugo", "Cléo"];
const TERRAINS: Terrain[] = ["Forest", "Desert", "Lake", "Swamp", "Mountain"];

/** The 24 positive clues, in the engine's order (black last). */
function clues(): Clue[] {
  const out: Clue[] = [];
  for (let i = 0; i < 5; i++) for (let j = i + 1; j < 5; j++) out.push({ Terrains: [TERRAINS[i], TERRAINS[j]] });
  (["Forest", "Desert", "Swamp", "Mountain", "Lake"] as Terrain[]).forEach((t) => out.push({ NearTerrain: t }));
  out.push("NearAnimals", { NearKind: "Stone" }, { NearKind: "Shack" }, { NearAnimal: "Bear" }, { NearAnimal: "Cougar" });
  out.push({ NearColor: "White" }, { NearColor: "Blue" }, { NearColor: "Green" }, { NearColor: "Black" });
  return out;
}

const NORMAL: Hint[] = clues()
  .slice(0, 23)
  .map((clue) => ({ clue, negated: false }));
const ADVANCED: Hint[] = [...clues(), ...clues()].map((clue, i) => ({ clue, negated: i >= 24 }));

const VIEW: CryptideView = {
  players: 3,
  mode: "Normal",
  squares: Array.from({ length: 108 }, (_, i) => ({
    terrain: TERRAINS[i % 5],
    animal: i === 3 ? "Bear" : null,
    structure: i === 7 ? ["Stone", "White"] : null,
  })),
  tokens: Array.from({ length: 108 }, (_, i) => ({ cube: i === 5 ? 1 : null, discs: i === 6 ? [0, 2] : [] })),
  phase: { type: "Turn" },
  turn: 0,
  focus: 6,
  my_hint: NORMAL[0],
  hints: NORMAL,
  log: [],
  reveal: null,
};

describe("Cryptide board", () => {
  it("words each family of clues as the rulebook does", () => {
    expect(hintLabel({ clue: { Terrains: ["Forest", "Swamp"] }, negated: false })).toBe(
      "L'habitat se trouve dans une Forêt ou un Marécage.",
    );
    expect(hintLabel({ clue: { NearTerrain: "Desert" }, negated: false })).toBe(
      "L'habitat se trouve dans, ou à 1 case d'un Désert.",
    );
    expect(hintLabel({ clue: "NearAnimals", negated: false })).toBe(
      "L'habitat se trouve dans, ou à 1 case du territoire d'un animal.",
    );
    expect(hintLabel({ clue: { NearKind: "Shack" }, negated: false })).toBe(
      "L'habitat se trouve dans, ou jusqu'à 2 cases d'une cabane abandonnée.",
    );
    expect(hintLabel({ clue: { NearAnimal: "Cougar" }, negated: false })).toBe(
      "L'habitat se trouve dans, ou jusqu'à 2 cases du territoire d'un puma.",
    );
    expect(hintLabel({ clue: { NearColor: "Blue" }, negated: true })).toBe(
      "L'habitat ne se trouve pas dans, ou jusqu'à 3 cases d'une structure bleue.",
    );
  });

  it("tells what happened", () => {
    expect(place(13)).toBe("ligne 2, colonne 2");
    expect(describeEvent({ type: "Asked", by: 0, target: 1, cell: 0 }, NAMES)).toBe(
      "Ana demande à Hugo : ligne 1, colonne 1 ?",
    );
    expect(describeEvent({ type: "Placed", seat: 2, cell: 1, answer: "Cube", why: "Penalty" }, NAMES)).toBe(
      "Cléo pose un cube ligne 1, colonne 2 (pénalité).",
    );
    expect(describeEvent({ type: "Won", seat: 1 }, NAMES)).toBe("Hugo a trouvé l'habitat !");
  });

  it("draws the 108 spaces, the tokens and one's own clue", () => {
    const html = renderToStaticMarkup(
      <CryptideBoard view={VIEW} legalActions={[]} pending={[0]} me={0} seatNames={NAMES} submit={() => {}} />,
    );
    expect(html.match(/data-cell=/g)?.length).toBe(108);
    expect(html).toContain("Ton indice :");
    expect(html).toContain("cr-bear");
    expect(html).not.toContain("Les indices");
  });

  it("offers only the legal actions on a space", () => {
    const legal: CryptideAction[] = [{ Ask: { cell: 4, target: 1 } }, { Ask: { cell: 4, target: 2 } }, { Search: 9 }];
    const here = actionsAt(legal, 4);
    const sent: CryptideAction[] = [];
    const buttons = hostElements(<TurnControls view={VIEW} actions={here} names={NAMES} submit={(a) => sent.push(a)} />).filter(
      (el) => el.type === "button",
    );
    expect(buttons).toHaveLength(2);
    buttons.forEach((b) => (b.props.onClick as () => void)());
    expect(sent).toEqual([{ Ask: { cell: 4, target: 1 } }, { Ask: { cell: 4, target: 2 } }]);
    const search = renderToStaticMarkup(
      <TurnControls view={VIEW} actions={actionsAt(legal, 9)} names={NAMES} submit={() => {}} />,
    );
    expect(search).toContain("Fouiller ici");
    expect(renderToStaticMarkup(<TurnControls view={VIEW} actions={here} names={NAMES} submit={() => {}} />)).not.toContain(
      "Fouiller",
    );
  });

  it("lists every clue of the mode in the player aid, without marks", () => {
    const normal = renderToStaticMarkup(<HintHelp hints={NORMAL} />);
    expect(normal.match(/<li>/g)?.length).toBe(23);
    expect(normal).not.toContain("noire");
    const advanced = renderToStaticMarkup(<HintHelp hints={ADVANCED} />);
    expect(advanced.match(/<li>/g)?.length).toBe(48);
    expect(advanced).toContain("ne se trouve pas");
  });

  it("shows everyone's clue and the habitat at the end", () => {
    const over: CryptideView = {
      ...VIEW,
      phase: { type: "Over" },
      reveal: { hints: [NORMAL[0], NORMAL[11], NORMAL[20]], habitat: 30 },
      log: [{ type: "Won", seat: 0 }],
    };
    const html = renderToStaticMarkup(
      <CryptideBoard view={over} legalActions={[]} pending={[]} me={0} seatNames={NAMES} submit={() => {}} />,
    ).replaceAll("&#x27;", "'");
    expect(html).toContain("Les indices");
    expect(html).toContain("Hugo : L'habitat se trouve dans, ou à 1 case d'un Désert.");
    expect(html).toContain("L'habitat était en ligne 3, colonne 7.");
    expect(html).toContain("Ana a trouvé l'habitat !");
  });

  describe("riddles", () => {
    const RIDDLE: Riddle = {
      level: "Full",
      players: 3,
      mode: "Normal",
      squares: VIEW.squares,
      tokens: VIEW.tokens,
      log: [{ type: "Asked", by: 2, target: 0, cell: 6 }],
      solver: 1,
      hint: NORMAL[11],
      habitat: 30,
      hints: [NORMAL[0], NORMAL[11], NORMAL[20]],
      given: [null, NORMAL[11], null],
      hint_cells: [[30, 31], [29, 30], [30]],
      deduction: [[NORMAL[0], NORMAL[3]], [], [NORMAL[20]]],
      eliminations: [],
      combinations: {
        working: [{ hints: [NORMAL[0], NORMAL[11], NORMAL[20]], outcome: { Habitat: 30 } }],
        fallen: [{ hints: [NORMAL[3], NORMAL[11], NORMAL[20]], outcome: "Nowhere" }],
        nowhere: 1,
        several: 0,
        repeated: 0,
        complete: true,
      },
      mode_hints: NORMAL,
    };
    const EMPTY = Array.from({ length: 108 }, () => ({ cube: null, discs: [] as number[] }));
    const WARMUP: Riddle = {
      ...RIDDLE,
      level: "Warmup",
      tokens: EMPTY,
      log: [],
      solver: 0,
      hint: NORMAL[0],
      given: [NORMAL[0], NORMAL[11], NORMAL[20]],
      deduction: [[], [], []],
      combinations: { ...RIDDLE.combinations, working: [], fallen: [], nowhere: 0 },
    };
    const HINT: Riddle = {
      ...RIDDLE,
      level: "Hint",
      solver: 2,
      hint: NORMAL[20],
      given: [null, null, null],
      deduction: [[], [], []],
      eliminations: [{ hint: NORMAL[0], cell: 6, mark: "Disc" }],
      combinations: WARMUP.combinations,
    };
    type Extra = { hintMisses?: (typeof NORMAL)[number][]; onGuessHint?(h: unknown): void; shaded?: number | null };
    const element = (riddle: Riddle, misses: number[], solved: "found" | "shown" | null, extra: Extra = {}) => (
      <RiddleBoard
        riddle={riddle}
        misses={misses}
        hintMisses={extra.hintMisses ?? []}
        solved={solved}
        shaded={extra.shaded ?? null}
        onGuess={() => {}}
        onGuessHint={extra.onGuessHint ?? (() => {})}
        onShade={() => {}}
        onGiveUp={() => {}}
      />
    );
    const board = (misses: number[], solved: "found" | "shown" | null, riddle = RIDDLE, extra: Extra = {}) =>
      hostElements(element(riddle, misses, solved, extra));
    const html = (misses: number[], solved: "found" | "shown" | null, riddle = RIDDLE, extra: Extra = {}) =>
      renderToStaticMarkup(element(riddle, misses, solved, extra)).replaceAll("&#x27;", "'");
    const clickable = (els: AnyElement[]) => {
      const map = els.find((el) => el.type === "svg");
      return hostElements(map)
        .filter((el) => el.props["data-cell"] !== undefined && el.props.onClick)
        .map((el) => el.props["data-cell"]);
    };

    it("gives the solver's clue and keeps the others hidden", () => {
      const page = html([], null);
      expect(page).toContain("Hugo (IA)</strong> sait où se trouve l'habitat");
      expect(page).toContain("Indice de Hugo (IA) :</strong> L'habitat se trouve dans, ou à 1 case d'un Désert.");
      expect(page).not.toContain("pouvait déduire");
      expect(page).not.toContain("habitat est en");
      expect(page).toContain("Lina (IA) demande à Margot (IA)");
    });

    it("lets every space without a cube or a miss be guessed", () => {
      const cells = clickable(board([12], null));
      expect(cells).toHaveLength(106);
      expect(cells).not.toContain(5);
      expect(cells).not.toContain(12);
      expect(html([12], null)).toContain("Raté : 1 essai");
    });

    it("shows the answer and what the solver could tell of the others", () => {
      const page = html([12], "found");
      expect(page).toContain("Trouvé en 2 essais !");
      expect(page).toContain("L'habitat est en ligne 3, colonne 7.");
      expect(page).toContain("Margot (IA) : un indice parmi 2");
      expect(page).toContain("Lina (IA) : un seul indice possible");
      expect(page).toContain(`<li class="true">${hintLabel(NORMAL[20])}</li>`);
    });

    it("the full solution shows the combinations", () => {
      const page = html([], "shown");
      expect(page).toContain("Combinaisons qui marchent");
      expect(page).toContain("1 sans case commune");
      expect(page).toContain(hintLabel(NORMAL[3]).replace(/\.$/, ""));
      expect(page).not.toContain(". ·");
      expect(page).not.toContain(". →");
    });

    it("warmup lists every hint and lets every space be picked", () => {
      const page = html([], null, WARMUP);
      for (const h of WARMUP.hints) expect(page).toContain(hintLabel(h));
      expect(page).toContain("Trouve la seule case qui les vérifie tous");
      expect(clickable(board([], null, WARMUP))).toHaveLength(108);
      expect(clickable(board([4], null, WARMUP))).not.toContain(4);
    });

    it("warmup's solution shades the spaces a hint keeps", () => {
      const map = board([], "shown", WARMUP, { shaded: 1 }).find((el) => el.type === "svg");
      const shaded = hostElements(map)
        .filter((el) => typeof el.props.className === "string" && el.props.className.includes("cr-shaded"))
        .map((el) => el.props["data-cell"]);
      expect(shaded).toHaveLength(106);
      expect(shaded).not.toContain(29);
      expect(shaded).not.toContain(30);
    });

    it("a hint riddle hides every hint and offers the mode's hints to pick", () => {
      const els = board([], null, HINT, { hintMisses: [NORMAL[4]] });
      const page = html([], null, HINT);
      expect(page).not.toContain("Indice de");
      expect(page).toContain("Les jetons de <strong>Lina (IA)</strong> ne laissent qu'un indice possible");
      expect(clickable(els)).toHaveLength(0);
      const picks = els.filter((el) => el.type === "button" && el.props["data-hint"] !== undefined);
      expect(picks).toHaveLength(NORMAL.length);
      const missed = picks.find((el) => el.props["data-hint"] === JSON.stringify(NORMAL[4]));
      expect(missed?.props.disabled).toBe(true);
    });

    it("a hint riddle's solution names the token that rules each hint out", () => {
      const page = html([], "shown", HINT);
      expect(page).toContain(`L'indice de Lina (IA) : ${hintLabel(NORMAL[20])}`);
      expect(page).toContain(
        `${hintLabel(NORMAL[0]).replace(/\.$/, "")} : écarté par le disque de Lina (IA) en ligne 1, colonne 7`,
      );
      expect(page).not.toContain(". : écarté");
    });

    it("a hint is matched whole, negation included", () => {
      expect(sameHint(ADVANCED[3], ADVANCED[3])).toBe(true);
      expect(sameHint(ADVANCED[3], ADVANCED[27])).toBe(false);
      expect(sameHint({ clue: { Terrains: ["Forest", "Desert"] }, negated: false }, ADVANCED[0])).toBe(true);
    });

    it("a new riddle clears the misses and the answer", () => {
      expect(freshAttempt()).toEqual({ misses: [], hintMisses: [], solved: null, shaded: null });
    });
  });
});

describe("Cryptide map legend", () => {
  it("says which outline is a cougar's and which a bear's", async () => {
    const { HexMap } = await import("../HexMap");
    const html = renderToStaticMarkup(
      <HexMap squares={[]} tokens={[]} playable={new Set()} selected={null} focus={null} habitat={null} onPick={() => {}} />,
    );
    expect(html).toMatch(/class="cr-cougar"[^]*Territoire de puma/);
    expect(html).toMatch(/class="cr-bear"[^]*Territoire d&#x27;ours/);
  });
});

import { isValidElement, type ReactElement, type ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { CryptideBoard, TurnControls, actionsAt, describeEvent, place } from "../Board";
import { HintHelp } from "../HintHelp";
import { hintLabel } from "../hints";
import type { Clue, CryptideAction, CryptideView, Hint, Terrain } from "../types";

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
});

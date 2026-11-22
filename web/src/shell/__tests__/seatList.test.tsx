import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import type { Update } from "../../engine/types";
import type { SeatInfo } from "../../session/session";
import { SeatList } from "../TableScreen";

const over = (outcome: number[]): Update => ({ rev: 1, view: null, legalActions: [], pending: [], outcome, clocks: [] });
const seat = (name: string): SeatInfo => ({ name, kind: "local", connected: true });

describe("SeatList", () => {
  it("ranks the seats of a finished game", () => {
    const html = renderToStaticMarkup(<SeatList seats={[seat("Ana"), seat("Hugo")]} update={over([1, 0])} me={0} receivedAt={0} />);
    expect(html).toContain("1er");
  });

  it("does not rank the only seat of a solo game", () => {
    const html = renderToStaticMarkup(<SeatList seats={[seat("Ana")]} update={over([0])} me={0} receivedAt={0} />);
    expect(html).not.toContain("1er");
  });
});

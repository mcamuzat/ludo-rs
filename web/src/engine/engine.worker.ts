// Holds the match. Never searches: the AI has its own worker, so a long
// search never delays network messages.
import init, * as wasm from "../wasm/ludo.js";
import { serve } from "./rpc";
import type { EngineRequest, Snapshot } from "./types";

let handle: number | null = null;

function current(): number {
  if (handle === null) throw new Error("Aucune partie en cours.");
  return handle;
}

function replace(next: number) {
  if (handle !== null) wasm.dropMatch(handle);
  handle = next;
}

serve<EngineRequest>(init(), (req) => {
  switch (req.kind) {
    case "listGames":
      return JSON.parse(wasm.listGames());
    case "newMatch":
      replace(wasm.newMatch(req.gameId, JSON.stringify(req.config ?? null), req.players, req.seed));
      return null;
    case "importLog":
      replace(wasm.importLog(req.gameId, JSON.stringify(req.log)));
      return null;
    case "apply":
      wasm.apply(current(), req.seat, JSON.stringify(req.action));
      return null;
    case "snapshot": {
      const h = current();
      const snapshot: Snapshot = {
        rev: wasm.rev(h),
        pending: Array.from(wasm.pending(h)),
        outcome: JSON.parse(wasm.outcome(h)),
        views: req.viewers.map((viewer) => ({
          viewer,
          view: JSON.parse(wasm.view(h, viewer)),
          legalActions: viewer === null ? [] : JSON.parse(wasm.legalActions(h, viewer)),
        })),
      };
      return snapshot;
    }
    case "exportState":
      return JSON.parse(wasm.exportState(current()));
    case "exportLog":
      return JSON.parse(wasm.exportLog(current()));
  }
});

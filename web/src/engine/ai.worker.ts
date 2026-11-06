import init, {
  aiMove,
  twixtAdvise,
  twixtInstallNet,
  twixtPosition,
  twixtReviewPoint,
  twixtReview,
} from "../wasm/ludo.js";
import { serve } from "./rpc";
import { fetchTwixtNet, TwixtNetLoader } from "./twixtNet";
import type { AiWorkerRequest } from "./types";

const twixtNet = new TwixtNetLoader(fetchTwixtNet, twixtInstallNet);

serve<AiWorkerRequest>(init(), async (req) => {
  switch (req.kind) {
    case "move":
      // TwixT's AI is its network: never let it play without it.
      if (req.gameId === "twixt") await twixtNet.ensure();
      return JSON.parse(aiMove(req.gameId, JSON.stringify(req.state), req.seat, req.level, req.seed));
    case "twixtAdvise":
      await twixtNet.ensure();
      return JSON.parse(twixtAdvise(JSON.stringify(req.moves)));
    case "twixtReviewPoint":
      await twixtNet.ensure();
      return JSON.parse(twixtReviewPoint(JSON.stringify(req.moves), req.ply));
    case "twixtReview":
      return JSON.parse(twixtReview(JSON.stringify(req.moves), JSON.stringify(req.points)));
    case "twixtPosition":
      return JSON.parse(twixtPosition(JSON.stringify(req.moves)));
  }
});

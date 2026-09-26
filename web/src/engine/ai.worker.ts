import init, { aiMove } from "../wasm/ludo.js";
import { serve } from "./rpc";
import type { AiRequest } from "./types";

serve<AiRequest>(init(), (req) =>
  JSON.parse(aiMove(req.gameId, JSON.stringify(req.state), req.seat, req.level, req.seed)),
);

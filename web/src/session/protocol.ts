// Host ↔ guest protocol. The host runs the engine (the authority); a guest
// only sends actions and displays the updates of its own seat.
import type { Update } from "../engine/types";
import type { SeatInfo } from "./session";

/** Bumped on any incompatible change: host and guest must match. */
export const PROTOCOL_VERSION = 2;

/** Prefix of PeerJS ids, so a short code doesn't collide with other apps. */
export const PEER_PREFIX = "ludo-";

export type GuestMessage =
  | { type: "hello"; protocol: number; name: string; seatToken?: string }
  | { type: "action"; rev: number; action: unknown }
  | { type: "ping"; t: number };

export type HostMessage =
  | { type: "lobby"; gameId: string; seats: SeatInfo[]; started: boolean }
  | { type: "seated"; seat: number; seatToken: string }
  | { type: "update"; update: Update }
  /** `fatal`: the connection is refused and the guest must leave. */
  | { type: "rejected"; reason: string; fatal: boolean }
  | { type: "pong"; t: number };

function isObject(data: unknown): data is Record<string, unknown> {
  return typeof data === "object" && data !== null;
}

/** The host trusts nothing from the network: shape check here, legality in the engine. */
export function parseGuestMessage(data: unknown): GuestMessage | null {
  if (!isObject(data)) return null;
  switch (data.type) {
    case "hello": {
      if (typeof data.protocol !== "number" || typeof data.name !== "string") return null;
      // PeerJS's binary encoding turns an absent (undefined) token into null.
      const seatToken = data.seatToken ?? undefined;
      if (seatToken !== undefined && typeof seatToken !== "string") return null;
      return { type: "hello", protocol: data.protocol, name: data.name, seatToken };
    }
    case "action":
      if (!Number.isInteger(data.rev) || !("action" in data)) return null;
      return { type: "action", rev: data.rev as number, action: data.action };
    case "ping":
      return typeof data.t === "number" ? { type: "ping", t: data.t } : null;
    default:
      return null;
  }
}

const HOST_TYPES = new Set(["lobby", "seated", "update", "rejected", "pong"]);

export function parseHostMessage(data: unknown): HostMessage | null {
  if (!isObject(data) || typeof data.type !== "string" || !HOST_TYPES.has(data.type)) return null;
  return data as HostMessage;
}

/** Short code to share, without ambiguous characters (0/O, 1/I…). */
export function newCode(): string {
  const alphabet = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
  const bytes = crypto.getRandomValues(new Uint8Array(6));
  return Array.from(bytes, (b) => alphabet[b % alphabet.length]).join("");
}

export function normalizeCode(code: string): string {
  return code.trim().toUpperCase();
}

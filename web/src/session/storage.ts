// Per-device conveniences. Storage may be unavailable (private window,
// blocked site data): every access is guarded and the page works without it.
import type { HostSeat } from "./seats";

export interface SavedHost {
  code: string;
  gameId: string;
  seats: HostSeat[];
  log: unknown;
}

const NAME_KEY = "ludo-name";
const HOST_KEY = "ludo-host";
const guestKey = (code: string) => `ludo-guest:${code}`;

function read(key: string): string | null {
  try {
    return localStorage.getItem(key);
  } catch {
    return null;
  }
}

function write(key: string, value: string | null) {
  try {
    if (value === null) localStorage.removeItem(key);
    else localStorage.setItem(key, value);
  } catch {
    // Storage unavailable: nothing is kept.
  }
}

export function loadName(): string {
  return read(NAME_KEY) ?? "";
}

export function saveName(name: string) {
  write(NAME_KEY, name);
}

function isSavedHost(v: unknown): v is SavedHost {
  if (typeof v !== "object" || v === null) return false;
  const s = v as Partial<SavedHost>;
  return typeof s.code === "string" && typeof s.gameId === "string" && Array.isArray(s.seats) && "log" in s;
}

export function loadHost(): SavedHost | null {
  const raw = read(HOST_KEY);
  if (!raw) return null;
  try {
    const value: unknown = JSON.parse(raw);
    return isSavedHost(value) ? value : null;
  } catch {
    return null;
  }
}

export function saveHost(saved: SavedHost) {
  write(HOST_KEY, JSON.stringify(saved));
}

export function clearHost() {
  write(HOST_KEY, null);
}

export function loadGuestToken(code: string): string | null {
  return read(guestKey(code));
}

export function saveGuestToken(code: string, token: string) {
  write(guestKey(code), token);
}

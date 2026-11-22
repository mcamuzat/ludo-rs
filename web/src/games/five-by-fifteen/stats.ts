import type { Route } from "./types";

export type Stats = Record<Route, { played: number; won: number }>;

const KEY = "five-by-fifteen-stats";

export type Store = Pick<Storage, "getItem" | "setItem">;

/** The browser's storage; reading it may throw (blocked site data). */
function browser(): Store {
  return localStorage;
}

function empty(): Stats {
  return { Short: { played: 0, won: 0 }, Medium: { played: 0, won: 0 }, Long: { played: 0, won: 0 } };
}

/** Games played and won on each route, on this browser (empty if storage is unavailable). */
export function loadStats(store: () => Store = browser): Stats {
  const stats = empty();
  try {
    const saved = JSON.parse(store().getItem(KEY) ?? "{}") as Partial<Stats>;
    for (const route of Object.keys(stats) as Route[]) {
      const s = saved[route];
      if (s && Number.isFinite(s.played) && Number.isFinite(s.won)) stats[route] = { played: s.played, won: s.won };
    }
  } catch {
    // Private window or blocked storage: no statistics.
  }
  return stats;
}

export function recordResult(route: Route, won: boolean, store: () => Store = browser): void {
  const stats = loadStats(store);
  stats[route].played += 1;
  if (won) stats[route].won += 1;
  try {
    store().setItem(KEY, JSON.stringify(stats));
  } catch {
    // Not saved: the game itself is unaffected.
  }
}

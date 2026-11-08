import type { GameMeta } from "../engine/types";
import { t } from "../i18n";

/** « 2–4 » for a run of table sizes, « 2 ou 4 » (« 2 or 4 ») otherwise. */
export function playersLabel(meta: GameMeta): string {
  const counts = meta.playerCounts;
  if (counts.length === 1) return `${counts[0]}`;
  const run = counts.every((n, i) => i === 0 || n === counts[i - 1] + 1);
  return run ? `${counts[0]}–${counts[counts.length - 1]}` : counts.join(` ${t().players.or} `);
}

/** « 2–4 joueurs » / « 2–4 players ». */
export function playersCount(meta: GameMeta): string {
  return t().players.count(playersLabel(meta));
}

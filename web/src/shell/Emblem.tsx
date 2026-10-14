import type { GameFamily } from "../games/registry";

/** The die beside the site's name. */
export function Die() {
  return (
    <svg viewBox="0 0 30 30" aria-hidden="true">
      <rect x="2" y="2" width="26" height="26" rx="7" style={{ fill: "var(--accent)" }} />
      <g style={{ fill: "var(--accent-ink)" }}>
        <circle cx="9.5" cy="9.5" r="2.4" />
        <circle cx="15" cy="15" r="2.4" />
        <circle cx="20.5" cy="20.5" r="2.4" />
      </g>
    </svg>
  );
}

/** A game's thumbnail on the home screen: one drawing per family, the game's initial on Pairs cards. */
export function Emblem({ family, name }: { family: GameFamily; name: string }) {
  if (family === "board") {
    return (
      <svg viewBox="0 0 44 44" aria-hidden="true">
        <rect width="44" height="44" rx="10" style={{ fill: "var(--accent-soft)" }} />
        <g style={{ stroke: "var(--accent)" }} strokeWidth="1.6" fill="none">
          <polygon points="22,8 30,12.5 30,21.5 22,26 14,21.5 14,12.5" />
          <polygon points="14,21.5 22,26 22,35 14,39.5 6,35 6,26" opacity=".45" />
          <polygon points="30,21.5 38,26 38,35 30,39.5 22,35 22,26" opacity=".45" />
        </g>
        <circle cx="22" cy="17" r="4" style={{ fill: "var(--accent)" }} />
      </svg>
    );
  }
  if (family === "tiles") {
    return (
      <svg viewBox="0 0 44 44" aria-hidden="true">
        <rect width="44" height="44" rx="10" style={{ fill: "var(--accent-soft)" }} />
        <rect x="13" y="8" width="18" height="26" rx="3" style={{ fill: "var(--face)", stroke: "var(--accent)" }} strokeWidth="1.6" />
        <rect x="13" y="30" width="18" height="6" rx="2" style={{ fill: "var(--accent)" }} />
        <circle cx="22" cy="19" r="4.5" fill="none" stroke="#c2453a" strokeWidth="1.8" />
      </svg>
    );
  }
  const initial = name.replace(/^L['’]/, "").charAt(0).toUpperCase();
  return (
    <svg viewBox="0 0 44 44" aria-hidden="true">
      <rect width="44" height="44" rx="10" style={{ fill: "var(--brass-soft)" }} />
      <rect x="9" y="10" width="15" height="22" rx="3" style={{ fill: "var(--face)", stroke: "var(--brass)" }} strokeWidth="1.5" transform="rotate(-9 16 21)" />
      <rect x="19" y="11" width="15" height="22" rx="3" style={{ fill: "var(--face)", stroke: "var(--brass)" }} strokeWidth="1.5" transform="rotate(7 26 22)" />
      <text x="26.5" y="26.5" fontWeight="800" fontSize="11" fill="#8a5c10" style={{ fontFamily: "var(--f-display)" }} textAnchor="middle">
        {initial}
      </text>
    </svg>
  );
}

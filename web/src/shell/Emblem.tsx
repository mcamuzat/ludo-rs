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

/** A game's thumbnail on the home screen: one drawing per family, the game's initial on cards. */
export function Emblem({ family, name }: { family: GameFamily; name: string }) {
  const initial = name.replace(/^L['’]/, "").charAt(0).toUpperCase();
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
  if (family === "coins") {
    return (
      <svg viewBox="0 0 44 44" aria-hidden="true">
        <rect width="44" height="44" rx="10" style={{ fill: "var(--accent-soft)" }} />
        <ellipse cx="22" cy="31" rx="11" ry="4.5" fill="#7a4a26" stroke="#fff" strokeWidth="1.2" strokeDasharray="2 2" />
        <ellipse cx="22" cy="25" rx="11" ry="4.5" fill="#3b5bab" stroke="#fff" strokeWidth="1.2" strokeDasharray="2 2" />
        <ellipse cx="22" cy="19" rx="11" ry="4.5" fill="#c8322b" stroke="#fff" strokeWidth="1.2" strokeDasharray="2 2" />
      </svg>
    );
  }
  if (family === "trade") {
    return (
      <svg viewBox="0 0 44 44" aria-hidden="true">
        <rect width="44" height="44" rx="10" style={{ fill: "var(--accent-soft)" }} />
        <circle cx="22" cy="22" r="13" fill="#0083bf" />
        {[0, 1, 2, 3, 4].map((k) => {
          const a = -Math.PI / 2 + (2 * Math.PI * k) / 5;
          return <line key={k} x1="22" y1="22" x2={22 + 14 * Math.cos(a)} y2={22 + 14 * Math.sin(a)} stroke="#fff" strokeWidth="2.2" />;
        })}
      </svg>
    );
  }
  if (family === "modern") {
    return (
      <svg viewBox="0 0 44 44" aria-hidden="true">
        <rect width="44" height="44" rx="10" style={{ fill: "var(--accent-soft)" }} />
        {["#c8322b", "#2e8b57", "#2f6db5", "#e0b021", "#7b4fa8"].map((fill, k) => (
          <rect key={k} x={7 + k * 5} y={9 + (k % 2) * 2} width="14" height="22" rx="2.5" fill={fill} stroke="#fff" strokeWidth="1.2" />
        ))}
        <text x="22" y="39" fontWeight="800" fontSize="8" style={{ fill: "var(--accent)", fontFamily: "var(--f-display)" }} textAnchor="middle">
          {initial}
        </text>
      </svg>
    );
  }
  if (family === "cards") {
    return (
      <svg viewBox="0 0 44 44" aria-hidden="true">
        <rect width="44" height="44" rx="10" style={{ fill: "var(--accent-soft)" }} />
        <rect x="12" y="7" width="20" height="30" rx="3" style={{ fill: "var(--face)", stroke: "var(--accent)" }} strokeWidth="1.5" />
        <text x="22" y="22" fontWeight="800" fontSize="11" fill="#b3261e" style={{ fontFamily: "var(--f-display)" }} textAnchor="middle">
          {initial}
        </text>
        <text x="22" y="33" fontSize="9" fill="#b3261e" textAnchor="middle">
          ♥
        </text>
      </svg>
    );
  }
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

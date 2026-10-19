import type { Card, Color, Rank } from "./types";

/** The colours of the print-and-play sheets of the French rules. */
export const COLOR_HEX: Record<Color, string> = {
  Red: "#ff2a00",
  Orange: "#ff8a00",
  Yellow: "#f2cf00",
  Navy: "#1b1774",
  Sky: "#0083bf",
  Green: "#3aac34",
};

const COLOR_NAME: Record<Color, string> = {
  Red: "rouge",
  Orange: "orange",
  Yellow: "jaune",
  Navy: "bleu marine",
  Sky: "bleu ciel",
  Green: "vert",
};

export const RANK_NAME: Record<Rank, string> = { Unit: "Unité", Bi: "Bi", Tri: "Tri", Quad: "Quad", Quint: "Quint" };
export const RANK_VALUE: Record<Rank, number> = { Unit: 1, Bi: 3, Tri: 7, Quad: 16, Quint: 36 };
const PARTS: Record<Rank, number> = { Unit: 1, Bi: 2, Tri: 3, Quad: 4, Quint: 5 };

/** "Unité verte", "Bi bleu ciel". */
export function cardName({ rank, color }: Card): string {
  const name = rank === "Unit" && color === "Green" ? "verte" : COLOR_NAME[color];
  return `${RANK_NAME[rank]} ${name}`;
}

export function colorName(color: Color): string {
  return COLOR_NAME[color];
}

/** The rank symbol: a disc cut in 1 to 5 parts. */
export function Disc({ rank, color, r = 11 }: { rank: Rank; color: Color; r?: number }) {
  const n = PARTS[rank];
  const start = n === 2 ? 0 : -Math.PI / 2;
  const cuts = n === 1 ? [] : Array.from({ length: n }, (_, k) => start + (2 * Math.PI * k) / n);
  return (
    <g>
      <circle r={r} fill={COLOR_HEX[color]} />
      {cuts.map((a) => (
        <line key={a} x1={0} y1={0} x2={(r + 1) * Math.cos(a)} y2={(r + 1) * Math.sin(a)} stroke="#fff" strokeWidth={r / 6} />
      ))}
    </g>
  );
}

export function MonadCard({
  card,
  selected = false,
  onClick,
}: {
  card: Card;
  selected?: boolean;
  onClick?(): void;
}) {
  const face = (
    <svg viewBox="0 0 44 62" className="monad-card-face" aria-hidden="true">
      <rect x="0.5" y="0.5" width="43" height="61" rx="5" fill="#fff" stroke="#c9c2b4" />
      <g transform="translate(12 13)">
        <Disc rank={card.rank} color={card.color} r={8} />
      </g>
      <text x="22" y="36" textAnchor="middle" fontSize="15" fontWeight="700" fill={COLOR_HEX[card.color]}>
        {RANK_VALUE[card.rank]}
      </text>
      <g transform="translate(32 51) rotate(180)">
        <Disc rank={card.rank} color={card.color} r={6} />
      </g>
    </svg>
  );
  const label = cardName(card);
  return onClick ? (
    <button className={`monad-card${selected ? " selected" : ""}`} onClick={onClick} aria-label={label} aria-pressed={selected}>
      {face}
    </button>
  ) : (
    <span className="monad-card" aria-label={label} role="img">
      {face}
    </span>
  );
}

/** The Monad disc: a yin-yang marked 80. */
export function MonadDisc({ size = 40 }: { size?: number }) {
  return (
    <svg viewBox="-21 -21 42 42" width={size} height={size} className="monad-disc" aria-hidden="true">
      <circle r="20" fill="#fff" stroke="#111" strokeWidth="1.5" />
      <path d="M 0 -20 A 20 20 0 0 0 0 20 A 10 10 0 0 0 0 0 A 10 10 0 0 1 0 -20 Z" fill="#111" />
      <text x="9" y="4" textAnchor="middle" fontSize="8.5" fontWeight="800" fill="#111">
        80
      </text>
    </svg>
  );
}

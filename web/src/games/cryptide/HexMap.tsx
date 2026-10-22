import type { Cell, Kind, Square, StructureColor, Terrain, Tokens } from "./types";

const R = 20;
const SQRT3 = Math.sqrt(3);
const COLS = 12;
const ROWS = 9;

/** The rulebook's colours (p. 4). */
export const TERRAIN_FILL: Record<Terrain, string> = {
  Lake: "#3f86c9",
  Mountain: "#a3a29c",
  Forest: "#3c8a3a",
  Swamp: "#5c3a4f",
  Desert: "#e8c24a",
};

const STRUCTURE_FILL: Record<StructureColor, string> = {
  White: "#f7f5ef",
  Green: "#2f9e4f",
  Blue: "#2f63c9",
  Black: "#262626",
};

/** Token colours, one per seat. */
export const SEAT_FILL = ["#e0352b", "#f28c1c", "#22c3d6", "#d940b0", "#7b5cff"];

/** Flat-top hexes; odd columns sit half a space lower. */
export function center(cell: Cell) {
  const row = Math.floor(cell / COLS);
  const col = cell % COLS;
  return { x: R + col * 1.5 * R, y: (SQRT3 / 2) * R + row * SQRT3 * R + (col % 2 === 1 ? (SQRT3 / 2) * R : 0) };
}

function hexPoints(x: number, y: number, r: number): string {
  return Array.from({ length: 6 }, (_, k) => {
    const angle = (Math.PI / 3) * k;
    return `${x + r * Math.cos(angle)},${y + r * Math.sin(angle)}`;
  }).join(" ");
}

function structure(kind: Kind, color: StructureColor, x: number, y: number) {
  const fill = STRUCTURE_FILL[color];
  const stroke = color === "Black" ? "#bbb" : "#222";
  if (kind === "Stone") {
    const r = R * 0.42;
    const pts = Array.from({ length: 8 }, (_, k) => {
      const a = (Math.PI / 4) * k + Math.PI / 8;
      return `${x + r * Math.cos(a)},${y - R * 0.12 + r * Math.sin(a)}`;
    }).join(" ");
    return <polygon className="cr-structure" points={pts} fill={fill} stroke={stroke} strokeWidth={1.5} />;
  }
  const s = R * 0.5;
  return (
    <polygon
      className="cr-structure"
      points={`${x},${y - R * 0.12 - s} ${x - s},${y - R * 0.12 + s * 0.7} ${x + s},${y - R * 0.12 + s * 0.7}`}
      fill={fill}
      stroke={stroke}
      strokeWidth={1.5}
    />
  );
}

function tokensAt(t: Tokens, x: number, y: number) {
  const discs = t.discs.map((seat, i) => {
    const dx = (i - (t.discs.length - 1) / 2) * R * 0.42;
    return <circle key={`d${seat}`} cx={x + dx} cy={y + R * 0.5} r={R * 0.2} fill={SEAT_FILL[seat]} stroke="#111" strokeWidth={1} />;
  });
  const cube =
    t.cube !== null ? (
      <rect
        x={x - R * 0.28}
        y={y - R * 0.28}
        width={R * 0.56}
        height={R * 0.56}
        rx={2}
        fill={SEAT_FILL[t.cube]}
        stroke="#111"
        strokeWidth={1.2}
      />
    ) : null;
  return (
    <g className="cr-tokens">
      {cube}
      {discs}
    </g>
  );
}

export interface HexMapProps {
  squares: Square[];
  tokens: Tokens[];
  /** Spaces that can be clicked now. */
  playable: Set<Cell>;
  selected: Cell | null;
  focus: Cell | null;
  habitat: Cell | null;
  /** Wrong guesses, crossed out (riddles). */
  misses?: Cell[];
  onPick(cell: Cell): void;
}

export function HexMap({ squares, tokens, playable, selected, focus, habitat, misses = [], onPick }: HexMapProps) {
  const width = R * 2 + (COLS - 1) * 1.5 * R;
  const height = SQRT3 * R * ROWS + (SQRT3 / 2) * R;
  return (
    <svg className="cr-map" viewBox={`0 0 ${width} ${height}`} role="img" aria-label="Carte de Cryptide">
      {squares.map((sq, cell) => {
        const { x, y } = center(cell);
        const can = playable.has(cell);
        const classes = ["cr-space", can ? "playable" : "", cell === selected ? "selected" : "", cell === focus ? "focus" : "", cell === habitat ? "habitat" : ""]
          .filter(Boolean)
          .join(" ");
        return (
          <g key={cell} className={classes} data-cell={cell} onClick={can ? () => onPick(cell) : undefined}>
            <polygon points={hexPoints(x, y, R)} fill={TERRAIN_FILL[sq.terrain]} className="cr-hex" />
            {sq.animal && (
              <polygon
                points={hexPoints(x, y, R * 0.8)}
                className={sq.animal === "Bear" ? "cr-bear" : "cr-cougar"}
                fill="none"
              />
            )}
            {sq.structure && structure(sq.structure[0], sq.structure[1], x, y)}
            {tokensAt(tokens[cell], x, y)}
            {misses.includes(cell) && (
              <path
                className="cr-miss"
                d={`M${x - R * 0.45},${y - R * 0.45} L${x + R * 0.45},${y + R * 0.45} M${x + R * 0.45},${y - R * 0.45} L${x - R * 0.45},${y + R * 0.45}`}
              />
            )}
          </g>
        );
      })}
    </svg>
  );
}

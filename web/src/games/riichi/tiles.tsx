import type { Tile } from "./types";

const HONORS: Record<string, string> = { E: "Ton", S: "Nan", W: "Shaa", N: "Pei", P: "Haku", F: "Hatsu", C: "Chun" };
const SUITS: Record<string, string> = { m: "Man", p: "Pin", s: "Sou" };
const BASE = `${import.meta.env.BASE_URL}tiles/`;

export function tileFile(tile: Tile): string {
  if (HONORS[tile]) return `${HONORS[tile]}.svg`;
  const suit = SUITS[tile[1]];
  return tile.endsWith("r") ? `${suit}5-Dora.svg` : `${suit}${tile[0]}.svg`;
}

type Size = "sm" | "md" | "lg";

interface TileProps {
  tile: Tile;
  size?: Size;
  /** Riichi declaration tile, or a called tile. */
  sideways?: boolean;
  dim?: boolean;
  onClick?: () => void;
}

export function TileImg({ tile, size = "md", sideways, dim, onClick }: TileProps) {
  const cls = ["tile", `tile-${size}`, sideways && "sideways", dim && "dim", onClick && "clickable"].filter(Boolean).join(" ");
  return (
    <span className={cls} onClick={onClick} title={tile} role={onClick ? "button" : undefined}>
      <img src={`${BASE}Front.svg`} alt="" />
      <img className="face" src={`${BASE}${tileFile(tile)}`} alt={tile} />
    </span>
  );
}

export function TileBack({ size = "sm" }: { size?: Size }) {
  return (
    <span className={`tile tile-${size}`}>
      <img src={`${BASE}Back.svg`} alt="" />
    </span>
  );
}

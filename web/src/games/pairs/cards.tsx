import type { CSSProperties, ReactNode } from "react";
import "./pairs.css";

/** A face-up card: a button when it can be chosen, plain otherwise. */
export function Card({
  rank,
  onClick,
  title,
  small,
  raised,
}: {
  rank: number;
  onClick?: () => void;
  title?: string;
  small?: boolean;
  /** Picked out of a hand (to discard, say). */
  raised?: boolean;
}) {
  const cls = `pairs-card r${rank}${small ? " small" : ""}${raised ? " raised" : ""}`;
  if (!onClick) return <span className={cls}>{rank}</span>;
  return (
    <button type="button" className={`${cls} playable`} onClick={onClick} title={title} aria-label={title}>
      {rank}
    </button>
  );
}

/** A face-down pile with its size. */
export function CardBack({ count, label }: { count: number; label: string }) {
  return (
    <span className="pairs-pile">
      <span className="pairs-card back">{count}</span>
      {label}
    </span>
  );
}

/**
 * Seats around a centre. `first` is the seat drawn at the bottom (the
 * viewer's). On narrow screens the CSS turns the ring into a list.
 */
export function Table({ seats, first, center }: { seats: ReactNode[]; first: number; center: ReactNode }) {
  const n = seats.length;
  return (
    <div className="pairs-table felt">
      <div className="pairs-center">{center}</div>
      {seats.map((seat, i) => {
        const angle = Math.PI / 2 + (2 * Math.PI * ((i - first + n) % n)) / n;
        const style = {
          "--x": `${50 + 39 * Math.cos(angle)}%`,
          "--y": `${50 + 38 * Math.sin(angle)}%`,
        } as CSSProperties;
        return (
          <div key={i} className="pairs-place" style={style}>
            {seat}
          </div>
        );
      })}
    </div>
  );
}

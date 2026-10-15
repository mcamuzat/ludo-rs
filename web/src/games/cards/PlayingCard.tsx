import type { Card, Rank, Suit } from "./types";
import "./cards.css";

/** French indices: V(alet), D(ame), R(oi). */
const INDEX: Record<Rank, string> = {
  Two: "2",
  Three: "3",
  Four: "4",
  Five: "5",
  Six: "6",
  Seven: "7",
  Eight: "8",
  Nine: "9",
  Ten: "10",
  Jack: "V",
  Queen: "D",
  King: "R",
  Ace: "A",
};

const RANK_NAME: Record<Rank, string> = {
  ...INDEX,
  Jack: "valet",
  Queen: "dame",
  King: "roi",
  Ace: "as",
};

const SYMBOL: Record<Suit, string> = { Spades: "♠", Hearts: "♥", Diamonds: "♦", Clubs: "♣" };
const SUIT_NAME: Record<Suit, string> = { Spades: "pique", Hearts: "cœur", Diamonds: "carreau", Clubs: "trèfle" };

/** "7 de cœur", "dame de pique", "joker". */
export function cardName(card: Card): string {
  return card.kind === "Joker" ? "joker" : `${RANK_NAME[card.rank]} de ${SUIT_NAME[card.suit]}`;
}

/** A playing card: a toggle button when it can be picked, plain otherwise. */
export function PlayingCard({
  card,
  small,
  selected,
  onClick,
}: {
  card: Card;
  small?: boolean;
  selected?: boolean;
  onClick?: () => void;
}) {
  const red = card.kind === "Natural" && (card.suit === "Hearts" || card.suit === "Diamonds");
  const cls = `playing-card${red ? " red" : ""}${card.kind === "Joker" ? " joker" : ""}${small ? " small" : ""}${
    selected ? " selected" : ""
  }`;
  const face =
    card.kind === "Joker" ? (
      <span>Joker</span>
    ) : (
      <>
        <span>{INDEX[card.rank]}</span>
        <span className="suit">{SYMBOL[card.suit]}</span>
      </>
    );
  if (!onClick) {
    return (
      <span className={cls} title={cardName(card)}>
        {face}
      </span>
    );
  }
  return (
    <button
      type="button"
      className={`${cls} playable`}
      aria-pressed={selected ?? false}
      aria-label={cardName(card)}
      onClick={onClick}
    >
      {face}
    </button>
  );
}

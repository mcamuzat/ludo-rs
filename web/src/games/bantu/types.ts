// Mirrors games/bantu/src (View, Action, Spot, Dest, End, LastMove) and
// board.json (written by the test board_json_is_up_to_date).

export type Spot = { kind: "Home" } | { kind: "On"; cell: number } | { kind: "Finished" };

export type Dest = { Cell: number } | "Finish";

export type BantuAction = { Move: { piece: number; to: Dest } };

export type End = { Won: number } | "QuietLimit" | "Blocked";

export interface LastMove {
  seat: number;
  piece: number;
  from: Spot;
  to: Spot;
  /** [seat, piece] sent back to its start square. */
  captured: [number, number] | null;
}

export interface BantuView {
  players: number;
  /** pieces[seat][number - 1] */
  pieces: Spot[][];
  turn: number | null;
  /** How far each piece of the player to move goes (0 when in). */
  lengths: number[];
  protected: number[];
  last: LastMove | null;
  /** Rounds of the table without progress left before the quiet limit. */
  quietLeft: number;
  end: End | null;
  legal: BantuAction[];
}

export interface BoardCell {
  x: number;
  y: number;
  arrow: boolean;
}

export interface BoardData {
  cells: BoardCell[];
  /** startLines[n - 1]: line of piece n, from the centre outwards. */
  startLines: number[][];
  rows: number[][];
  /** Oriented the way pieces move. */
  links: [number, number][];
}

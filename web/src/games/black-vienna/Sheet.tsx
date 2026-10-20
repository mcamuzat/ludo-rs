import type { Column, Note, Suspect } from "./types";

/** A mark the player sets by hand. */
export type ManualMark = "+" | "−" | "?";

/** The key of a sheet cell: "A:0" for suspect A at seat 0, "A:BV" for Black Vienna. */
export function cellKey(suspect: Suspect, column: Column): string {
  return `${suspect}:${column === "Trio" ? "BV" : column.Seat}`;
}

/**
 * The investigation sheet: one row per suspect, one column per player, then
 * Black Vienna. Automatic notes (direct facts, from the engine) are fixed;
 * the other cells show the player's own marks and cycle them on click.
 */
export function Sheet({
  suspects,
  players,
  names,
  notes,
  marks,
  onCycle,
}: {
  suspects: Suspect[];
  players: number;
  names: string[];
  notes: Note[];
  marks: Record<string, ManualMark>;
  /** Absent for a spectator. */
  onCycle?: (key: string) => void;
}) {
  const auto = new Map(notes.map((n) => [cellKey(n.suspect, n.column), n.mark === "Plus" ? "+" : "−"]));
  const columns: Column[] = [...Array.from({ length: players }, (_, seat) => ({ Seat: seat })), "Trio"];
  return (
    <table className="bv-sheet">
      <thead>
        <tr>
          <th />
          {columns.map((c) => (
            <th key={cellKey("", c)} scope="col">
              {c === "Trio" ? "Black Vienna" : names[c.Seat]}
            </th>
          ))}
        </tr>
      </thead>
      <tbody>
        {suspects.map((s) => (
          <tr key={s}>
            <th scope="row">{s}</th>
            {columns.map((c) => {
              const key = cellKey(s, c);
              const fixed = auto.get(key);
              if (fixed) {
                return (
                  <td key={key} data-cell={key} className="auto" title="Fait établi">
                    {fixed}
                  </td>
                );
              }
              const mark = marks[key];
              return (
                <td
                  key={key}
                  data-cell={key}
                  className={mark ? "manual" : undefined}
                  onClick={onCycle ? () => onCycle(key) : undefined}
                >
                  {mark ?? ""}
                </td>
              );
            })}
          </tr>
        ))}
      </tbody>
    </table>
  );
}

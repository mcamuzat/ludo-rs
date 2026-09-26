import { useState } from "react";
import type { GameMeta, Level } from "../engine/types";
import { aiName } from "../session/names";
import { CLOCK_PRESETS, type ClockSettings } from "../session/clock";
import type { TableSeat } from "../session/table";

/** What the host picks for each seat after their own. */
export type SeatChoice = "local" | "open" | Level;

interface Props {
  game: GameMeta;
  onLocal(seats: TableSeat[]): void;
  /** Absent: online play is not offered. */
  onHost?(choices: SeatChoice[], clock: ClockSettings | null): void;
  onBack(): void;
}

const LEVELS: Level[] = ["Easy", "Medium", "Hard"];

export function Lobby({ game, onLocal, onHost, onBack }: Props) {
  const [players, setPlayers] = useState(game.minPlayers);
  const [choices, setChoices] = useState<SeatChoice[]>(() => Array(game.maxPlayers - 1).fill("Medium"));
  const [clockId, setClockId] = useState(CLOCK_PRESETS[0].id);
  const others = choices.slice(0, players - 1);

  const options: { value: SeatChoice; label: string }[] = [
    ...LEVELS.map((level) => ({ value: level as SeatChoice, label: aiName(level) })),
    // Hot-seat only without hidden information: everyone sees the same screen.
    ...(game.hiddenInfo ? [] : [{ value: "local" as SeatChoice, label: "Humain (même écran)" }]),
    ...(onHost ? [{ value: "open" as SeatChoice, label: "Invité (en ligne)" }] : []),
  ];

  const localSeats = (): TableSeat[] => [
    { kind: "local", name: "Toi" },
    ...others.map((c, i): TableSeat =>
      c === "local" || c === "open" ? { kind: "local", name: `Joueur ${i + 2}` } : { kind: "ai", level: c, name: aiName(c) },
    ),
  ];

  const choose = (i: number, value: SeatChoice) =>
    setChoices((current) => current.map((c, j) => (j === i ? value : c)));

  return (
    <main>
      <div className="topbar">
        <button onClick={onBack}>← Menu</button>
        <h2>{game.name}</h2>
      </div>
      <div className="row">
        <label htmlFor="players">Joueurs :</label>
        <select id="players" value={players} onChange={(e) => setPlayers(Number(e.target.value))}>
          {Array.from({ length: game.maxPlayers - game.minPlayers + 1 }, (_, k) => game.minPlayers + k).map((n) => (
            <option key={n} value={n}>
              {n}
            </option>
          ))}
        </select>
      </div>
      <ol className="seats">
        <li>Toi</li>
        {others.map((c, i) => (
          <li key={i}>
            <select value={c} onChange={(e) => choose(i, e.target.value as SeatChoice)}>
              {options.map((o) => (
                <option key={o.value} value={o.value}>
                  {o.label}
                </option>
              ))}
            </select>
          </li>
        ))}
      </ol>
      {onHost && game.clock && (
        <div className="row">
          <label htmlFor="clock">Chrono en ligne :</label>
          <select id="clock" value={clockId} onChange={(e) => setClockId(e.target.value)}>
            {CLOCK_PRESETS.map((p) => (
              <option key={p.id} value={p.id}>
                {p.label}
              </option>
            ))}
          </select>
        </div>
      )}
      <div className="row">
        <button className="primary" disabled={others.includes("open")} onClick={() => onLocal(localSeats())}>
          Jouer sur cet appareil
        </button>
        {onHost && (
          <button disabled={others.includes("local")} onClick={() =>
              onHost(others, game.clock ? (CLOCK_PRESETS.find((p) => p.id === clockId)?.settings ?? null) : null)
            }>
            Héberger en ligne
          </button>
        )}
      </div>
    </main>
  );
}

import { useState } from "react";
import type { GameConfig, GameMeta, Level } from "../engine/types";
import { aiName, aiSeatName } from "../session/names";
import { CLOCK_PRESETS, clockPresetLabel, type ClockSettings } from "../session/clock";
import { gameName, useT } from "../i18n";
import type { TableSeat } from "../session/table";
import { defaultConfig } from "../games/options";
import { gameUi } from "../games/registry";
import { GameOptions } from "./GameOptions";
import { playersCount } from "./players";
import { RulesReminder } from "./RulesReminder";

/** What the host picks for each seat after their own. */
export type SeatChoice = "local" | "open" | Level;

interface Props {
  game: GameMeta;
  onLocal(seats: TableSeat[], config: GameConfig | null): void;
  /** Absent: online play is not offered. */
  onHost?(choices: SeatChoice[], clock: ClockSettings | null, config: GameConfig | null): void;
  onBack(): void;
}

const LEVELS: Level[] = ["Easy", "Medium", "Hard"];

export function Lobby({ game, onLocal, onHost, onBack }: Props) {
  const [players, setPlayers] = useState(game.playerCounts[0]);
  const [choices, setChoices] = useState<SeatChoice[]>(() => Array(game.maxPlayers - 1).fill("Medium"));
  const [clockId, setClockId] = useState(CLOCK_PRESETS[0].id);
  const [config, setConfig] = useState(() => defaultConfig(game));
  const others = choices.slice(0, players - 1);
  const t = useT();

  const options: { value: SeatChoice; label: string }[] = [
    ...LEVELS.map((level) => ({ value: level as SeatChoice, label: aiName(level) })),
    // Hot-seat only without hidden information: everyone sees the same screen.
    ...(!game.sharedScreen ? [] : [{ value: "local" as SeatChoice, label: t.lobby.sameScreen }]),
    ...(onHost ? [{ value: "open" as SeatChoice, label: t.lobby.guest }] : []),
  ];

  const localSeats = (): TableSeat[] => [
    { kind: "local", name: t.lobby.you },
    ...others.map((c, i): TableSeat =>
      c === "local" || c === "open" ? { kind: "local", name: t.lobby.player(i + 2) } : { kind: "ai", level: c, name: aiSeatName(i + 1) },
    ),
  ];

  const choose = (i: number, value: SeatChoice) =>
    setChoices((current) => current.map((c, j) => (j === i ? value : c)));

  return (
    <main>
      <div className="topbar">
        <button className="ghost" onClick={onBack}>
          {t.lobby.back}
        </button>
        <h2>{gameName(game)}</h2>
        <span className="pill">{playersCount(game)}</span>
      </div>
      <RulesReminder rules={gameUi(game.id)?.rules} open />
      <div className="lobby">
        <section className="panel">
          <div className="panel-head">
            <h3>{t.lobby.seats}</h3>
            <div className="row">
              <label htmlFor="players">{t.lobby.players}</label>
              <select id="players" value={players} onChange={(e) => setPlayers(Number(e.target.value))}>
                {game.playerCounts.map((n) => (
                  <option key={n} value={n}>
                    {n}
                  </option>
                ))}
              </select>
            </div>
          </div>
          <ol className="seats">
            <li>{t.lobby.you}</li>
            {others.map((c, i) => (
              <li key={i}>
                <select value={c} aria-label={t.lobby.seat(i + 2)} onChange={(e) => choose(i, e.target.value as SeatChoice)}>
                  {options.map((o) => (
                    <option key={o.value} value={o.value}>
                      {o.label}
                    </option>
                  ))}
                </select>
              </li>
            ))}
          </ol>
        </section>
        {config && (
          <section className="panel">
            <h3>{t.lobby.options}</h3>
            <GameOptions meta={game} config={config} onChange={setConfig} />
          </section>
        )}
        <section className="panel">
          <h3>{t.lobby.start}</h3>
          {onHost && game.clock && (
            <div className="field">
              <label htmlFor="clock">{t.lobby.clock}</label>
              <select id="clock" value={clockId} onChange={(e) => setClockId(e.target.value)}>
                {CLOCK_PRESETS.map((p) => (
                  <option key={p.id} value={p.id}>
                    {clockPresetLabel(p)}
                  </option>
                ))}
              </select>
            </div>
          )}
          <div className="row">
            <button className="primary" disabled={others.includes("open")} onClick={() => onLocal(localSeats(), config)}>
              {t.lobby.playHere}
            </button>
            {onHost && (
              <button
                disabled={others.includes("local")}
                onClick={() =>
                  onHost(
                    others,
                    game.clock ? (CLOCK_PRESETS.find((p) => p.id === clockId)?.settings ?? null) : null,
                    config,
                  )
                }
              >
                {t.lobby.host}
              </button>
            )}
          </div>
          {onHost && <p className="hint">{t.lobby.hostHint}</p>}
        </section>
      </div>
    </main>
  );
}

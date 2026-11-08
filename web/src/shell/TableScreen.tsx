import { useEffect, useMemo, useState, useSyncExternalStore } from "react";
import type { GameMeta, Update } from "../engine/types";
import { clockLabel } from "../session/clock";
import { rankLabel, seatName } from "../session/names";
import { gameName, useT } from "../i18n";
import { gameUi } from "../games/registry";
import { HostSession } from "../session/host";
import type { SeatInfo, Session, SessionState } from "../session/session";
import { GameOptions } from "./GameOptions";
import { HostLobby } from "./HostLobby";
import { RulesReminder } from "./RulesReminder";

interface Props {
  session: Session;
  games: GameMeta[];
  onLeave(): void;
}

export function TableScreen({ session, games, onLeave }: Props) {
  const state = useSyncExternalStore(session.subscribe, session.getState);
  const meta = games.find((g) => g.id === state.gameId);
  const t = useT();
  return (
    <main>
      <div className="topbar">
        <button className="ghost" onClick={onLeave}>
          {t.lobby.back}
        </button>
        <h2>{meta ? gameName(meta) : "…"}</h2>
        <span className="spacer" />
        {state.latencyMs !== null && <span className="pill mono">{state.latencyMs} ms</span>}
      </div>
      {state.error && <p className="error">{state.error}</p>}
      {state.status && <p className="status">{state.status}</p>}
      {state.notice && <p className="notice">{state.notice}</p>}
      {state.phase === "lobby" &&
        (session instanceof HostSession ? (
          <HostLobby session={session} state={state} meta={meta} />
        ) : (
          <GuestLobby state={state} meta={meta} />
        ))}
      {state.phase === "playing" && <Playing session={session} state={state} meta={meta} />}
    </main>
  );
}

function Playing({ session, state, meta }: { session: Session; state: SessionState; meta: GameMeta | undefined }) {
  const update = state.update;
  // Clocks count down from when the update arrived.
  const receivedAt = useMemo(() => performance.now(), [update]);
  const ui = state.gameId ? gameUi(state.gameId) : undefined;
  const t = useT();
  if (state.gameId && !ui) return <p className="error">{t.table.unknownGame}</p>;
  const Board = ui?.Board;
  return (
    <>
      <SeatList seats={state.seats} update={update} me={state.me} receivedAt={receivedAt} />
      <RulesReminder rules={ui?.rules} />
      {meta && <GameOptions meta={meta} config={state.config} />}
      {update && Board && (
        <Board
          view={update.view}
          legalActions={update.legalActions}
          pending={update.pending}
          me={state.me}
          seatNames={state.seats.map(seatName)}
          submit={(action) => {
            if (state.me !== null) session.submit(state.me, action);
          }}
          assist={session.assist() ?? undefined}
        />
      )}
      {update?.outcome && (
        <div className="end">
          <strong>{t.table.over}</strong>{" "}
          {session.canRestart && (
            <button className="primary" onClick={() => session.restart()}>
              {t.table.rematch}
            </button>
          )}
        </div>
      )}
    </>
  );
}

/** Re-renders every `ms` while `active`. */
function useNow(ms: number, active: boolean): number {
  const [now, setNow] = useState(() => performance.now());
  useEffect(() => {
    if (!active) return;
    const id = setInterval(() => setNow(performance.now()), ms);
    return () => clearInterval(id);
  }, [ms, active]);
  return now;
}


function SeatList({
  seats,
  update,
  me,
  receivedAt,
}: {
  seats: SeatInfo[];
  update: Update | null;
  me: number | null;
  receivedAt: number;
}) {
  const now = useNow(250, (update?.clocks.length ?? 0) > 0);
  const t = useT();
  return (
    <ol className="seatbar">
      {seats.map((seat, i) => {
        const pending = update?.pending.includes(i) ?? false;
        const rank = update?.outcome?.[i];
        const clock = update?.clocks.find((x) => x.seat === i);
        return (
          <li key={i} className={pending ? "active" : undefined}>
            <strong>
              {seatName(seat, i)}
              {i === me ? t.table.you : ""}
            </strong>
            {!seat.connected && seat.kind === "remote" && <span className="tag">{t.table.disconnected}</span>}
            {pending && <span className="tag">{seat.kind === "ai" ? t.table.thinking : t.table.toPlay}</span>}
            {clock && <span className="clock">{clockLabel(clock, now - receivedAt)}</span>}
            {rank !== undefined && <span className="tag">{rankLabel(rank)}</span>}
          </li>
        );
      })}
    </ol>
  );
}

function GuestLobby({ state, meta }: { state: SessionState; meta: GameMeta | undefined }) {
  const t = useT();
  return (
    <section className="panel">
      <h3>{t.table.seats}</h3>
      <ol className="seats">
        {state.seats.map((seat, i) => (
          <li key={i}>
            {seatName(seat, i)}
            {i === state.me ? t.table.you : ""}
          </li>
        ))}
      </ol>
      {meta && <GameOptions meta={meta} config={state.config} />}
      <p className="hint">{t.table.hostStarts}</p>
    </section>
  );
}

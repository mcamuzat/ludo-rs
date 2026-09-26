import { useSyncExternalStore } from "react";
import type { GameMeta, Update } from "../engine/types";
import { gameUi } from "../games/registry";
import { HostSession } from "../session/host";
import type { SeatInfo, Session, SessionState } from "../session/session";
import { HostLobby } from "./HostLobby";

interface Props {
  session: Session;
  games: GameMeta[];
  onLeave(): void;
}

export function TableScreen({ session, games, onLeave }: Props) {
  const state = useSyncExternalStore(session.subscribe, session.getState);
  const meta = games.find((g) => g.id === state.gameId);
  return (
    <main>
      <div className="topbar">
        <button onClick={onLeave}>← Menu</button>
        <h2>{meta?.name ?? "…"}</h2>
        {state.latencyMs !== null && <span className="status">{state.latencyMs} ms</span>}
      </div>
      {state.error && <p className="error">{state.error}</p>}
      {state.status && <p className="status">{state.status}</p>}
      {state.notice && <p className="notice">{state.notice}</p>}
      {state.phase === "lobby" &&
        (session instanceof HostSession ? <HostLobby session={session} state={state} /> : <GuestLobby state={state} />)}
      {state.phase === "playing" && <Playing session={session} state={state} />}
    </main>
  );
}

function Playing({ session, state }: { session: Session; state: SessionState }) {
  const ui = state.gameId ? gameUi(state.gameId) : undefined;
  if (state.gameId && !ui) return <p className="error">Jeu inconnu : mets la page à jour.</p>;
  const update = state.update;
  const Board = ui?.Board;
  return (
    <>
      <SeatList seats={state.seats} update={update} me={state.me} />
      {update && Board && (
        <Board
          view={update.view}
          legalActions={update.legalActions}
          pending={update.pending}
          me={state.me}
          seatNames={state.seats.map((s) => s.name)}
          submit={(action) => {
            if (state.me !== null) session.submit(state.me, action);
          }}
        />
      )}
      {update?.outcome && (
        <div className="end">
          <strong>Partie terminée.</strong>{" "}
          {session.canRestart && <button onClick={() => session.restart()}>Revanche</button>}
        </div>
      )}
    </>
  );
}

function rankLabel(rank: number): string {
  return rank === 0 ? "1er" : `${rank + 1}ᵉ`;
}

function SeatList({ seats, update, me }: { seats: SeatInfo[]; update: Update | null; me: number | null }) {
  return (
    <ol className="seats">
      {seats.map((seat, i) => {
        const pending = update?.pending.includes(i) ?? false;
        const rank = update?.outcome?.[i];
        return (
          <li key={i} className={pending ? "active" : undefined}>
            {seat.name}
            {i === me ? " (toi)" : ""}
            {!seat.connected && seat.kind === "remote" ? " (déconnecté)" : ""}
            {pending ? (seat.kind === "ai" ? " — réfléchit…" : " — à jouer") : ""}
            {rank !== undefined ? ` — ${rankLabel(rank)}` : ""}
          </li>
        );
      })}
    </ol>
  );
}

function GuestLobby({ state }: { state: SessionState }) {
  return (
    <ol className="seats">
      {state.seats.map((seat, i) => (
        <li key={i}>
          {seat.name}
          {i === state.me ? " (toi)" : ""}
        </li>
      ))}
    </ol>
  );
}

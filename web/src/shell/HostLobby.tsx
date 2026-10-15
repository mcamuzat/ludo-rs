import type { GameMeta, Level } from "../engine/types";
import type { HostSession } from "../session/host";
import { aiName } from "../session/names";
import type { HostChoice } from "../session/seats";
import type { SessionState } from "../session/session";
import { GameOptions } from "./GameOptions";

const LEVELS: Level[] = ["Easy", "Medium", "Hard"];

function parseChoice(value: string): HostChoice {
  return value === "open" ? { kind: "open" } : { kind: "ai", level: value as Level };
}

export function HostLobby({
  session,
  state,
  meta,
}: {
  session: HostSession;
  state: SessionState;
  meta: GameMeta | undefined;
}) {
  const link = state.shareCode ? `${location.origin}${location.pathname}?join=${state.shareCode}` : null;
  const seats = session.hostSeats();
  return (
    <div className="lobby">
      <section className="panel">
        <h3>Sièges</h3>
        <ol className="seats">
          {seats.map((seat, i) => (
            <li key={i}>
              {seat.kind === "host" && `${seat.name} (toi)`}
              {seat.kind === "guest" && `${seat.name}${seat.connected ? "" : " (déconnecté)"}`}
              {(seat.kind === "open" || seat.kind === "ai") && (
                <select
                  aria-label={`Siège ${i + 1}`}
                  value={seat.kind === "open" ? "open" : seat.level}
                  onChange={(e) => session.setSeat(i, parseChoice(e.target.value))}
                >
                  <option value="open">Place libre (invité)</option>
                  {LEVELS.map((level) => (
                    <option key={level} value={level}>
                      {aiName(level)}
                    </option>
                  ))}
                </select>
              )}
            </li>
          ))}
        </ol>
      </section>
      <section className="panel">
        <h3>Partie en ligne</h3>
        {state.shareCode && link && (
          <>
            <div className="codebox">
              <strong className="code">{state.shareCode}</strong>
              <button onClick={() => void navigator.clipboard?.writeText(state.shareCode ?? "")}>Copier le code</button>
            </div>
            <div className="share">
              <input readOnly aria-label="Lien d'invitation" value={link} onFocus={(e) => e.currentTarget.select()} />
              <button onClick={() => void navigator.clipboard?.writeText(link)}>Copier le lien</button>
            </div>
            <p className="hint">Les autres joueurs saisissent le code dans « Rejoindre une partie », ou ouvrent le lien.</p>
          </>
        )}
        {meta && <GameOptions meta={meta} config={state.config} />}
        <div className="row">
          <button className="primary" disabled={!session.ready()} onClick={() => session.start()}>
            Lancer la partie
          </button>
        </div>
      </section>
    </div>
  );
}

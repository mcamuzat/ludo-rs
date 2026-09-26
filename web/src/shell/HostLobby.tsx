import type { Level } from "../engine/types";
import type { HostSession } from "../session/host";
import { aiName } from "../session/names";
import type { HostChoice } from "../session/seats";
import type { SessionState } from "../session/session";

const LEVELS: Level[] = ["Easy", "Medium", "Hard"];

function parseChoice(value: string): HostChoice {
  return value === "open" ? { kind: "open" } : { kind: "ai", level: value as Level };
}

export function HostLobby({ session, state }: { session: HostSession; state: SessionState }) {
  const link = state.shareCode ? `${location.origin}${location.pathname}?join=${state.shareCode}` : null;
  const seats = session.hostSeats();
  return (
    <div>
      {state.shareCode && link && (
        <>
          <p>
            Code : <strong className="code">{state.shareCode}</strong>
          </p>
          <div className="row">
            <input readOnly value={link} size={40} onFocus={(e) => e.currentTarget.select()} />
            <button onClick={() => void navigator.clipboard?.writeText(link)}>Copier le lien</button>
          </div>
        </>
      )}
      <ol className="seats">
        {seats.map((seat, i) => (
          <li key={i}>
            {seat.kind === "host" && `${seat.name} (toi)`}
            {seat.kind === "guest" && `${seat.name}${seat.connected ? "" : " (déconnecté)"}`}
            {(seat.kind === "open" || seat.kind === "ai") && (
              <select
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
      <button className="primary" disabled={!session.ready()} onClick={() => session.start()}>
        Lancer la partie
      </button>
    </div>
  );
}

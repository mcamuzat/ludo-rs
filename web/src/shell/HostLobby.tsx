import type { GameMeta, Level } from "../engine/types";
import type { HostSession } from "../session/host";
import { aiName } from "../session/names";
import type { HostChoice } from "../session/seats";
import type { SessionState } from "../session/session";
import { GameOptions } from "./GameOptions";
import { useT } from "../i18n";

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
  const t = useT();
  return (
    <div className="lobby">
      <section className="panel">
        <h3>{t.table.seats}</h3>
        <ol className="seats">
          {seats.map((seat, i) => (
            <li key={i}>
              {seat.kind === "host" && `${seat.name || t.seats.hostDefault}${t.table.you}`}
              {seat.kind === "guest" && `${seat.name || t.seats.guestDefault}${seat.connected ? "" : t.table.disconnectedSuffix}`}
              {(seat.kind === "open" || seat.kind === "ai") && (
                <select
                  aria-label={t.lobby.seat(i + 1)}
                  value={seat.kind === "open" ? "open" : seat.level}
                  onChange={(e) => session.setSeat(i, parseChoice(e.target.value))}
                >
                  <option value="open">{t.table.openSeatGuest}</option>
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
        <h3>{t.table.onlineGame}</h3>
        {state.shareCode && link && (
          <>
            <div className="codebox">
              <strong className="code">{state.shareCode}</strong>
              <button onClick={() => void navigator.clipboard?.writeText(state.shareCode ?? "")}>{t.table.copyCode}</button>
            </div>
            <div className="share">
              <input readOnly aria-label={t.table.inviteLink} value={link} onFocus={(e) => e.currentTarget.select()} />
              <button onClick={() => void navigator.clipboard?.writeText(link)}>{t.table.copyLink}</button>
            </div>
            <p className="hint">{t.table.shareHint}</p>
          </>
        )}
        {meta && <GameOptions meta={meta} config={state.config} />}
        <div className="row">
          <button className="primary" disabled={!session.ready()} onClick={() => session.start()}>
            {t.table.startGame}
          </button>
        </div>
      </section>
    </div>
  );
}

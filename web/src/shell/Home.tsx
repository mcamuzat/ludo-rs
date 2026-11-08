import { useState } from "react";
import type { GameMeta } from "../engine/types";
import { gameUi, type GameFamily } from "../games/registry";
import { Die, Emblem } from "./Emblem";
import { playersCount } from "./players";
import { gameName, getLang, setLang, useLang, useT, type Lang } from "../i18n";
import type { SavedHost } from "../session/storage";

interface Props {
  catalog: GameMeta[] | null;
  error: string | null;
  name: string;
  onName(name: string): void;
  onPick(game: GameMeta): void;
  onRiddles(): void;
  onJoin(code: string): void;
  saved: SavedHost | null;
  onResume(): void;
  onAbandon(): void;
}

/** Home screen sections, in display order. */
const SHELVES: { shelf: "board" | "pairs" | "cards" | "modern" | "coins" | "trade"; families: GameFamily[] }[] = [
  { shelf: "board", families: ["board", "tiles"] },
  { shelf: "pairs", families: ["pairs"] },
  { shelf: "cards", families: ["cards"] },
  { shelf: "modern", families: ["modern"] },
  { shelf: "coins", families: ["coins"] },
  { shelf: "trade", families: ["trade"] },
];

const LANGS: Lang[] = ["fr", "en"];
const CRYPTIDE = { id: "cryptide", name: "Cryptide" };

function LanguagePicker() {
  const t = useT();
  const lang = useLang();
  return (
    <div className="field">
      <span className="label" id="language">
        {t.language.label}
      </span>
      <div className="langs" role="group" aria-labelledby="language">
        {LANGS.map((l) => (
          <button key={l} type="button" lang={l} title={t.language[l]} aria-pressed={l === lang} onClick={() => setLang(l)}>
            {l.toUpperCase()}
          </button>
        ))}
      </div>
    </div>
  );
}

export function Home({ catalog, error, name, onName, onPick, onRiddles, onJoin, saved, onResume, onAbandon }: Props) {
  const [code, setCode] = useState("");
  const t = useT();
  const games = (catalog ?? [])
    .flatMap((meta) => {
      const ui = gameUi(meta.id);
      return ui && !ui.hidden ? [{ meta, family: ui.family, name: gameName(meta) }] : [];
    })
    .sort((a, b) => a.name.localeCompare(b.name, getLang(), { sensitivity: "base" }));
  const savedMeta = saved ? catalog?.find((g) => g.id === saved.gameId) : undefined;
  const savedName = saved ? (savedMeta ? gameName(savedMeta) : saved.gameId) : null;
  return (
    <main className="home">
      <header className="appbar">
        <div className="brand">
          <Die />
          <h1>
            ludo<b>.rs</b>
          </h1>
        </div>
        <div className="appbar-tools">
          <div className="field">
            <label htmlFor="name">{t.home.yourName}</label>
            <input id="name" value={name} maxLength={20} onChange={(e) => onName(e.target.value)} />
          </div>
          <form
            className="join"
            onSubmit={(e) => {
              e.preventDefault();
              if (code.trim()) onJoin(code);
            }}
          >
            <div className="field">
              <label htmlFor="code">{t.home.joinGame}</label>
              <input id="code" value={code} placeholder="CODE" onChange={(e) => setCode(e.target.value)} />
            </div>
            <button type="submit">{t.home.join}</button>
          </form>
          <LanguagePicker />
        </div>
      </header>
      {saved && (
        <div className="resume">
          <span className="dot" aria-hidden="true" />
          <p>
            <strong>{t.home.hostedGame}</strong> · {savedName}, {t.home.code} <span className="mono">{saved.code}</span>
          </p>
          <button className="primary" onClick={onResume}>
            {t.home.resume}
          </button>
          <button className="ghost" onClick={onAbandon}>
            {t.home.abandon}
          </button>
        </div>
      )}
      {error && <p className="error">{error}</p>}
      {!catalog && !error && <p className="status">{t.home.loading}</p>}
      {SHELVES.map((shelf) => {
        const shelved = games.filter((g) => shelf.families.includes(g.family));
        if (shelved.length === 0) return null;
        return (
          <section key={shelf.shelf} className="family">
            <div className="family-head">
              <h2>{t.home.shelves[shelf.shelf]}</h2>
              <span>{t.home.games(shelved.length)}</span>
            </div>
            <ul className="games">
              {shelved.map(({ meta, family, name }) => (
                <li key={meta.id}>
                  <button className="game" onClick={() => onPick(meta)}>
                    <Emblem family={family} name={name} />
                    <strong>{name}</strong>
                    <small>{playersCount(meta)}</small>
                  </button>
                </li>
              ))}
            </ul>
          </section>
        );
      })}
      {catalog && (
        <section className="family">
          <div className="family-head">
            <h2>{t.home.riddles}</h2>
            <span>{t.home.training}</span>
          </div>
          <ul className="games">
            <li>
              <button className="game" onClick={onRiddles}>
                <Emblem family="modern" name={gameName(CRYPTIDE)} />
                <strong>{gameName(CRYPTIDE)}</strong>
                <small>{t.home.riddlesHint}</small>
              </button>
            </li>
          </ul>
        </section>
      )}
    </main>
  );
}

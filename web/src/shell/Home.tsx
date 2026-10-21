import { useState } from "react";
import type { GameMeta } from "../engine/types";
import { gameUi, type GameFamily } from "../games/registry";
import { Die, Emblem } from "./Emblem";
import { playersLabel } from "./players";
import type { SavedHost } from "../session/storage";

interface Props {
  catalog: GameMeta[] | null;
  error: string | null;
  name: string;
  onName(name: string): void;
  onPick(game: GameMeta): void;
  onJoin(code: string): void;
  saved: SavedHost | null;
  onResume(): void;
  onAbandon(): void;
}

/** Home screen sections, in display order. */
const SHELVES: { title: string; families: GameFamily[] }[] = [
  { title: "Plateau et tuiles", families: ["board", "tiles"] },
  { title: "Deck Pairs", families: ["pairs"] },
  { title: "Cartes traditionnelles", families: ["cards"] },
  { title: "Jeux de cartes d'auteur", families: ["modern"] },
  { title: "Jeux de pièces", families: ["coins"] },
  { title: "Jeux d'échange", families: ["trade"] },
];

export function Home({ catalog, error, name, onName, onPick, onJoin, saved, onResume, onAbandon }: Props) {
  const [code, setCode] = useState("");
  const games = (catalog ?? [])
    .flatMap((meta) => {
      const ui = gameUi(meta.id);
      return ui && !ui.hidden ? [{ meta, family: ui.family }] : [];
    })
    .sort((a, b) => a.meta.name.localeCompare(b.meta.name, "fr", { sensitivity: "base" }));
  const savedName = saved ? (catalog?.find((g) => g.id === saved.gameId)?.name ?? saved.gameId) : null;
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
            <label htmlFor="name">Ton nom</label>
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
              <label htmlFor="code">Rejoindre une partie</label>
              <input id="code" value={code} placeholder="CODE" onChange={(e) => setCode(e.target.value)} />
            </div>
            <button type="submit">Rejoindre</button>
          </form>
        </div>
      </header>
      {saved && (
        <div className="resume">
          <span className="dot" aria-hidden="true" />
          <p>
            <strong>Partie hébergée en cours</strong> · {savedName}, code <span className="mono">{saved.code}</span>
          </p>
          <button className="primary" onClick={onResume}>
            Reprendre
          </button>
          <button className="ghost" onClick={onAbandon}>
            Abandonner
          </button>
        </div>
      )}
      {error && <p className="error">{error}</p>}
      {!catalog && !error && <p className="status">Chargement…</p>}
      {SHELVES.map((shelf) => {
        const shelved = games.filter((g) => shelf.families.includes(g.family));
        if (shelved.length === 0) return null;
        return (
          <section key={shelf.title} className="family">
            <div className="family-head">
              <h2>{shelf.title}</h2>
              <span>{shelved.length > 1 ? `${shelved.length} jeux` : "1 jeu"}</span>
            </div>
            <ul className="games">
              {shelved.map(({ meta, family }) => (
                <li key={meta.id}>
                  <button className="game" onClick={() => onPick(meta)}>
                    <Emblem family={family} name={meta.name} />
                    <strong>{meta.name}</strong>
                    <small>
                      {playersLabel(meta)}{" "}
                      joueurs
                    </small>
                  </button>
                </li>
              ))}
            </ul>
          </section>
        );
      })}
    </main>
  );
}

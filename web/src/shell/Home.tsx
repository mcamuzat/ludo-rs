import { useState } from "react";
import type { GameMeta } from "../engine/types";
import { gameUi } from "../games/registry";
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
}

export function Home({ catalog, error, name, onName, onPick, onJoin, saved, onResume }: Props) {
  const [code, setCode] = useState("");
  const games = (catalog ?? []).filter((g) => gameUi(g.id));
  const savedName = saved ? (catalog?.find((g) => g.id === saved.gameId)?.name ?? saved.gameId) : null;
  return (
    <main className="home">
      <h1>ludo.rs</h1>
      <div className="row">
        <label htmlFor="name">Ton nom :</label>
        <input id="name" value={name} maxLength={20} onChange={(e) => onName(e.target.value)} />
      </div>
      {saved && (
        <div className="row">
          <span>
            Partie hébergée en cours ({savedName}, code {saved.code}).
          </span>
          <button onClick={onResume}>Reprendre</button>
        </div>
      )}
      {error && <p className="error">{error}</p>}
      {!catalog && !error && <p className="status">Chargement…</p>}
      <ul className="games">
        {games.map((g) => (
          <li key={g.id} className="game-card">
            <h2>{g.name}</h2>
            <p>
              {g.minPlayers === g.maxPlayers ? g.minPlayers : `${g.minPlayers}–${g.maxPlayers}`} joueurs
            </p>
            <button className="primary" onClick={() => onPick(g)}>
              Jouer
            </button>
          </li>
        ))}
      </ul>
      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          if (code.trim()) onJoin(code);
        }}
      >
        <label htmlFor="code">Rejoindre une partie :</label>
        <input id="code" value={code} placeholder="CODE" size={8} onChange={(e) => setCode(e.target.value)} />
        <button type="submit">Rejoindre</button>
      </form>
    </main>
  );
}

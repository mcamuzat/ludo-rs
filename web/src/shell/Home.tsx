import type { GameMeta } from "../engine/types";
import { gameUi } from "../games/registry";

interface Props {
  catalog: GameMeta[] | null;
  error: string | null;
  onPick(game: GameMeta): void;
}

export function Home({ catalog, error, onPick }: Props) {
  const games = (catalog ?? []).filter((g) => gameUi(g.id));
  return (
    <main className="home">
      <h1>ludo.rs</h1>
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
    </main>
  );
}

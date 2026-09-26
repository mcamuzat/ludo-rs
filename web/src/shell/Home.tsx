import type { GameMeta } from "../engine/types";
import { gameUi } from "../games/registry";

interface Props {
  catalog: GameMeta[] | null;
  error: string | null;
}

export function Home({ catalog, error }: Props) {
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
          </li>
        ))}
      </ul>
    </main>
  );
}

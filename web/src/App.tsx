import { useEffect, useState } from "react";
import { loadCatalog } from "./engine/clients";
import type { GameMeta } from "./engine/types";
import { LocalSession } from "./session/local";
import type { Session } from "./session/session";
import { Home } from "./shell/Home";
import { Lobby } from "./shell/Lobby";
import { TableScreen } from "./shell/TableScreen";

type Screen = { name: "home" } | { name: "lobby"; game: GameMeta } | { name: "table"; session: Session };

export function App() {
  const [catalog, setCatalog] = useState<GameMeta[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [screen, setScreen] = useState<Screen>({ name: "home" });

  useEffect(() => {
    loadCatalog().then(setCatalog, (e) => setError(String(e)));
  }, []);

  const home = () => {
    if (screen.name === "table") screen.session.dispose();
    setScreen({ name: "home" });
  };

  switch (screen.name) {
    case "home":
      return <Home catalog={catalog} error={error} onPick={(game) => setScreen({ name: "lobby", game })} />;
    case "lobby":
      return (
        <Lobby
          game={screen.game}
          onBack={home}
          onLocal={(seats) => setScreen({ name: "table", session: new LocalSession(screen.game.id, seats) })}
        />
      );
    case "table":
      return <TableScreen session={screen.session} games={catalog ?? []} onLeave={home} />;
  }
}

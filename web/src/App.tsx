import { useEffect, useState } from "react";
import { loadCatalog } from "./engine/clients";
import type { GameMeta } from "./engine/types";
import { GuestSession } from "./session/guest";
import { HostSession } from "./session/host";
import { LocalSession } from "./session/local";
import type { HostChoice } from "./session/seats";
import type { Session } from "./session/session";
import { loadHost, loadName, saveName } from "./session/storage";
import { Home } from "./shell/Home";
import { Lobby, type SeatChoice } from "./shell/Lobby";
import { TableScreen } from "./shell/TableScreen";

type Screen = { name: "home" } | { name: "lobby"; game: GameMeta } | { name: "table"; session: Session };

function hostChoice(choice: SeatChoice): HostChoice {
  return choice === "open" || choice === "local" ? { kind: "open" } : { kind: "ai", level: choice };
}

/** A `?join=CODE` link opens the game directly. */
function initialScreen(name: string): Screen {
  const code = new URLSearchParams(location.search).get("join");
  return code ? { name: "table", session: new GuestSession(code, name) } : { name: "home" };
}

export function App() {
  const [catalog, setCatalog] = useState<GameMeta[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [name, setName] = useState(loadName);
  const [screen, setScreen] = useState<Screen>(() => initialScreen(loadName()));

  useEffect(() => {
    loadCatalog().then(setCatalog, (e) => setError(String(e)));
  }, []);

  const open = (session: Session) => setScreen({ name: "table", session });

  const home = () => {
    if (screen.name === "table") screen.session.dispose();
    history.replaceState(null, "", location.pathname);
    setScreen({ name: "home" });
  };

  switch (screen.name) {
    case "home": {
      const saved = loadHost();
      return (
        <Home
          catalog={catalog}
          error={error}
          name={name}
          onName={(n) => {
            setName(n);
            saveName(n);
          }}
          onPick={(game) => setScreen({ name: "lobby", game })}
          onJoin={(code) => open(new GuestSession(code, name))}
          saved={saved}
          onResume={() => saved && open(HostSession.resume(saved))}
        />
      );
    }
    case "lobby":
      return (
        <Lobby
          game={screen.game}
          onBack={home}
          onLocal={(seats) => open(new LocalSession(screen.game.id, seats))}
          onHost={(choices) => open(new HostSession(screen.game.id, name, choices.map(hostChoice)))}
        />
      );
    case "table":
      return <TableScreen session={screen.session} games={catalog ?? []} onLeave={home} />;
  }
}

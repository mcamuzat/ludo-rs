import { useEffect, useState } from "react";
import { loadCatalog } from "./engine/clients";
import type { GameMeta } from "./engine/types";
import { Home } from "./shell/Home";

export function App() {
  const [catalog, setCatalog] = useState<GameMeta[] | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadCatalog().then(setCatalog, (e) => setError(String(e)));
  }, []);

  return <Home catalog={catalog} error={error} />;
}

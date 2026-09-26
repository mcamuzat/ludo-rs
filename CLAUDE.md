# CLAUDE.md

ludo-rs : plateforme de jeux traditionnels. Moteur Rust (`crates/ludo-core`, `games/*`), pont WASM (`crates/ludo-wasm`), site React (`web/`).

## Commandes
- `cargo test --workspace --release` : règles, conformance, IA.
- `cargo fmt --all` et `cargo clippy --workspace --all-targets -- -D warnings` avant chaque commit.
- `cargo run --release -p ludo-cli -- arena yavalath 200 hard easy` : Difficile doit marquer ≥ 0.80 contre Facile (0.94 mesuré).
- `cargo run --release -p ludo-cli -- play yavalath 2 medium` : partie en terminal.
- Web : `./scripts/build-wasm.sh` (ou `dev`), puis `cd web && pnpm dev` / `pnpm test` / `pnpm build`.

## Principes
- **Le moteur possède toute la logique.** `ludo-wasm`, `ludo-cli` et `web/` affichent et envoient des actions. Toute valeur dérivée dont l'UI a besoin est calculée par le jeu et placée dans sa `View`.
- **`ludo-core` et `games/*` restent natifs** : jamais de `wasm-bindgen`, `js-sys`, `web-sys`. Un futur serveur les réutilisera tels quels (rejeu du `MatchLog`).
- **`apply` renvoie `Illegal` sans modifier l'état**, ne panique jamais sur une entrée.
- **Information cachée** : `view` ne contient jamais la graine ni les données cachées d'un autre siège ; `determinize` redistribue tout ce que l'observateur ne voit pas. La conformance le vérifie.
- **P2P, l'hôte fait autorité** (`web/src/session/`) : un invité ne reçoit que `updateFor(snapshot, son siège)`. Incrémenter `PROTOCOL_VERSION` (`protocol.ts`) à tout changement du protocole. PeerJS encode en binaire : un champ `undefined` arrive `null` ; ne jamais envoyer `undefined`, et lire `null` comme absent.
- Changer un type sérialisé d'un jeu impose de mettre à jour `web/src/games/<id>/types.ts` dans le même changement.
- **Règles** : chaque jeu a un `RULES.md` qui cite sa source ([R]…) ; un point non tranché est marqué ❓. Toute modification de règle met à jour `RULES.md`, le moteur et un test ensemble.

## Ajouter un jeu
1. `games/<id>/` : implémenter `Game`, `Determinize`, `Heuristics` ; `RULES.md` sourcé ; tests de règles ; `tests/conformance.rs` qui appelle `ludo_core::conformance::check::<G>()` pour chaque nombre de joueurs.
2. `crates/ludo-registry/src/lib.rs` : un `static` et une entrée dans `GAMES` ; ajouter la dépendance dans son `Cargo.toml` et dans le workspace.
3. `web/src/games/<id>/types.ts` + `Board.tsx`, et une entrée dans `web/src/games/registry.ts`.
4. `ludo-cli arena <id> …` pour vérifier que l'IA tient la route.

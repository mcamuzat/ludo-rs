# CLAUDE.md

ludo-rs : plateforme de jeux traditionnels. Moteur Rust (`crates/ludo-core`, `games/*`), pont WASM (`crates/ludo-wasm`), site React (`web/`).

## Commandes
- `cargo test --workspace --release` : règles, conformance, IA.
- `cargo fmt --all` et `cargo clippy --workspace --all-targets -- -D warnings` avant chaque commit.
- `cargo run --release -p ludo-cli -- arena yavalath 200 hard easy` : Difficile doit marquer ≥ 0.80 contre Facile (0.94 mesuré).
- `cargo run --release -p ludo-cli -- arena pairs-continuous 400 hard easy` : Difficile doit marquer ≥ 0.55 contre Facile (0.82 mesuré ; 0.60 à 4 joueurs, où le hasard pèse plus).
- `cargo run --release -p ludo-cli -- arena pairs-auberge 400 hard easy` : Difficile doit marquer ≥ 0.55 contre Facile (0.72 mesuré ; 0.74 à 4 joueurs).
- `cargo run --release -p ludo-cli -- arena pairs-ghost-town 400 hard easy` : Difficile doit marquer ≥ 0.55 contre Facile (0.76 mesuré, à 2 comme à 4 joueurs).
- `cargo run --release -p ludo-cli -- arena pairs-zombie-life 400 hard easy 3` : Difficile doit marquer ≥ 0.55 contre Facile (0.96 mesuré à 3 joueurs).
- `cargo run --release -p ludo-cli -- arena pairs-goblin-poker 400 hard easy 3` : Difficile doit marquer ≥ 0.55 contre Facile (0.70 mesuré à 3 joueurs).
- `cargo run --release -p ludo-cli -- arena pairs-to-the-moon 96 hard easy` : Difficile doit marquer ≥ 0.55 contre Facile (1.00 mesuré à 2 comme à 4 joueurs ; ~100 s CPU par partie de 6 manches, d'où 96 parties plutôt que 400).
- `cargo run --release -p ludo-cli -- arena pairs-deadfall 400 hard easy` : Difficile doit marquer ≥ 0.55 contre Facile (0.74 mesuré ; 0.65 à 4 joueurs, 0.59 à 7).
- `cargo run --release -p ludo-cli -- arena pairs-captains-orders 400 hard easy` : Difficile doit marquer ≥ 0.55 contre Facile (0.57 mesuré à 4 joueurs, 0.60 à 8 ; Facile lit aussi les gestes). Lecture contre ancienne IA : voir la spec Pairs, section 16.
- `cargo run --release -p ludo-cli -- arena pairs-pendleton 100 hard easy` : Difficile doit marquer ≥ 0.55 contre Facile (0.93 mesuré ; ~55 s pour 100 parties de 100 points).
- `cargo run --release -p ludo-cli -- arena pairs-kitty 200 hard easy 4` : Difficile doit marquer ≥ 0.55 contre Facile (0.69 mesuré à 4 joueurs, 0.69 à 3, 0.58 à 6).
- `cargo run --release -p ludo-cli -- arena pairs-octopus 200 hard easy 4` : Difficile doit marquer ≥ 0.55 contre Facile (0.82 mesuré à 4 joueurs, 0.78 à 6).
- `cargo run --release -p ludo-cli -- arena pairs-port 200 hard easy` : Difficile doit marquer ≥ 0.55 contre Facile (0.87 mesuré ; 0.75 à 5 joueurs).
- `cargo run --release -p ludo-cli -- arena pairs-starboard 400 hard easy 3` : Difficile doit marquer ≥ 0.55 contre Facile (0.59 mesuré à 3 joueurs, 0.57 à 2, 0.65 à 6).
- `cargo run --release -p ludo-cli -- arena pairs-sweep 200 hard easy 3` : Difficile doit marquer ≥ 0.55 contre Facile (0.70 mesuré à 3 joueurs, 0.61 à 5).
- `cargo run --release -p ludo-cli -- arena pairs-rocket 400 hard easy` : Difficile doit marquer ≥ 0.55 contre Facile (0.73 mesuré ; 0.70 à 4 joueurs).
- `cargo run --release -p ludo-cli -- arena pairs-monster 200 hard easy 4` : Difficile doit marquer ≥ 0.55 contre Facile (0.68 mesuré à 4 joueurs comme à 6).
- `cargo run --release -p ludo-cli -- arena pairs-troll-guts 400 hard easy 5` : Difficile doit marquer ≥ 0.55 contre Facile (0.58 mesuré à 5 joueurs, 0.61 à 3 ; jeu très chaotique).
- `cargo run --release -p ludo-cli -- play yavalath 2 medium` : partie en terminal.
- Web : `./scripts/build-wasm.sh` (ou `dev`), puis `cd web && pnpm dev` / `pnpm test` / `pnpm build`.

## Principes
- **Le moteur possède toute la logique.** `ludo-wasm`, `ludo-cli` et `web/` affichent et envoient des actions. Toute valeur dérivée dont l'UI a besoin est calculée par le jeu et placée dans sa `View`.
- **`ludo-core` et `games/*` restent natifs** : jamais de `wasm-bindgen`, `js-sys`, `web-sys`. Un futur serveur les réutilisera tels quels (rejeu du `MatchLog`).
- **`apply` renvoie `Illegal` sans modifier l'état**, ne panique jamais sur une entrée.
- **Information cachée** : `view` ne contient jamais la graine ni les données cachées d'un autre siège ; `determinize` redistribue tout ce que l'observateur ne voit pas. La conformance le vérifie.
- **P2P, l'hôte fait autorité** (`web/src/session/`) : un invité ne reçoit que `updateFor(snapshot, son siège)`. Incrémenter `PROTOCOL_VERSION` (`protocol.ts`) à tout changement du protocole. PeerJS encode en binaire : un champ `undefined` arrive `null` ; ne jamais envoyer `undefined`, et lire `null` comme absent.
- Changer un type sérialisé d'un jeu impose de mettre à jour `web/src/games/<id>/types.ts` dans le même changement.
- **Attente visible** : si le simple fait d'être sollicité est une information (qui peut appeler une défausse au Riichi), le jeu restreint `visible_pending` ; l'hôte n'envoie à chacun que cette attente et les chronos correspondants.
- **Riichi** : `games/riichi` enveloppe `riichienv-core` (épinglé `=0.4.10`). Son état se sérialise par rejeu ; le résultat d'une main se lit avant le `step` (riichienv l'efface aussitôt). Monter de version = relancer toute la suite, dont `reported_points_match_the_engine_payments` et la conformance.
- `getrandom` 0.4 (`wasm_js`) n'est déclaré que dans `crates/ludo-wasm`.
- **IA** : un jeu au grand nombre de coups restreint la recherche avec `Heuristics::search_actions` (coups sensés) plutôt que de toucher à l'ISMCTS générique. Tout changement d'IA se mesure avec `ludo arena` (parallèle), Yavalath compris.
- **Pairs** : `games/pairs` porte tous les jeux du deck Pairs, un module par jeu, le matériel commun dans `deck.rs`. La graine des remélanges vit dans le `Supply` : il ne va jamais dans une vue, et `determinize` appelle `redeal_hidden`. Les mélanges du moteur utilisent `ChaCha8Rng`, jamais `SmallRng` (générateur différent en wasm32 et en natif : un journal du navigateur ne se rejouerait pas) ;  `the_deal_of_a_seed_is_pinned` fige la donne. Même raison : ne jamais tirer un `usize` (`gen_range(0..v.len())`), tirer un `u32` et le convertir.
- **Règles** : chaque jeu a un `RULES.md` qui cite sa source ([R]…) ; un point non tranché est marqué ❓. Toute modification de règle met à jour `RULES.md`, le moteur et un test ensemble.

## Ajouter un jeu
1. `games/<id>/` : implémenter `Game`, `Determinize`, `Heuristics` ; `RULES.md` sourcé ; tests de règles ; `tests/conformance.rs` qui appelle `ludo_core::conformance::check::<G>()` pour chaque nombre de joueurs.
2. `crates/ludo-registry/src/lib.rs` : un `static` et une entrée dans `GAMES` ; ajouter la dépendance dans son `Cargo.toml` et dans le workspace.
3. `web/src/games/<id>/types.ts` + `Board.tsx`, et une entrée dans `web/src/games/registry.ts`.
4. `ludo-cli arena <id> …` pour vérifier que l'IA tient la route.

## Ajouter un jeu Pairs
Comme « Ajouter un jeu », mais sans nouvelle crate :
1. `games/pairs/src/<jeu>/` : `mod.rs` (règles, `Game`), `view.rs`, `determinize.rs`, `heuristics.rs`, `testing.rs` ; `pub mod <jeu>;` et la réexportation dans `games/pairs/src/lib.rs`. Le matériel vient de `deck.rs` (`Rank`, `Supply`, `pair_in`…) : l'étendre là plutôt que de le recopier.
2. Une section dans `games/pairs/RULES.md` ; `tests/<jeu>_rules.rs` ; un cas dans `tests/conformance.rs` pour chaque nombre de joueurs.
3. `crates/ludo-registry/src/lib.rs` : un `static` et une entrée dans `GAMES`.
4. `web/src/games/pairs/<jeu>/types.ts` + `Board.tsx` (cartes et table : `../cards.tsx`), un test dans `__tests__/`, une entrée dans `web/src/games/registry.ts` avec son rappel de règles (`web/src/games/pairs/rules.ts`, à garder d'accord avec `RULES.md`). L'accueil trie les jeux par nom ; `hidden: true` retire un jeu de l'accueil (Riichi, pour l'instant).
5. `ludo-cli arena pairs-<jeu> 400 hard easy`, et la ligne correspondante dans « Commandes ».

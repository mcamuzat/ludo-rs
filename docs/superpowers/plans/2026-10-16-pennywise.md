# Pennywise Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Pennywise jouable sur ludo-rs (contre l'IA, sur le même écran, en ligne), 2 à 6 joueurs, personnage choisi dans le lobby.

**Architecture:** Une crate `games/pennywise` (règles, vue, IA) qui implémente `Game`, `Determinize` (identité) et `Heuristics` de `ludo-core`, inscrite dans `ludo-registry` ; un plateau React dans `web/src/games/pennywise/` qui n'affiche que la `View` et envoie des `legalActions`.

**Tech Stack:** Rust 2021 (`ludo-core`, `serde`, `rand`/`rand_chacha`), React 19 + Vitest.

**Spec:** `docs/superpowers/specs/2026-10-16-pennywise-design.md`

## Global Constraints

- Identifiant `pennywise`, nom affiché « Pennywise », 2 à 6 joueurs, `hidden_info: false`, `clock: true`.
- Valeurs de pièces : 1, 2, 3, 5, 10, 20, 25 (cents).
- Mains : Cody (défaut) 4×1, 3×5, 2×10, 1×25 (64¢) ; Darlene 3×1, 3×3, 2×10, 1×20 (52¢) ; Sugar 2×1, 2×2, 2×5, 1×10 (26¢) ; Taylor 3×1, 2×5, 1×10 (23¢).
- Reprise maximale **imposée** ; le joueur choisit l'assortiment quand plusieurs atteignent le maximum.
- Fin dès qu'un joueur n'a plus de pièce à la fin de son tour ; classement par argent restant, égalités = rangs égaux.
- Premier joueur tiré de la graine avec `ChaCha8Rng` (jamais `SmallRng`), en tirant un `u32` (jamais un `usize`).
- `apply` renvoie `Illegal` sans modifier l'état et ne panique jamais.
- `games/pennywise` ne dépend jamais de `wasm-bindgen`, `js-sys`, `web-sys`.
- Rien n'est calculé côté web ; tout libellé dérivé (totaux, classement) vient de la `View`.
- Commits avec l'e-mail du dépôt (`marc.camuzat@gmail.com`, déjà en config locale), terminés par `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Avant chaque commit Rust : `cargo fmt --all` et `cargo clippy --workspace --all-targets -- -D warnings`.

## Review Focus

- Pot qui ne permet aucune reprise (vide, ou seulement des pièces ≥ la pièce posée, ou pose d'un 1¢) : le coup reste légal avec `take` vide. → test `nothing_to_take_is_a_legal_empty_take` (Task 1).
- Un joueur pose sa dernière pièce mais reprend de la monnaie : il n'est pas ruiné, la partie continue. → test `last_coin_with_change_back_does_not_ruin` (Task 1).
- Coups forgés (pièce absente de la main, reprise non maximale, reprise non triée, siège qui n'a pas la main, partie finie) : `Illegal`, état inchangé. → test `forged_moves_are_refused_without_change` (Task 1).
- Gros pot à 6 joueurs : les assortiments proposés sont distincts et tous de même montant maximal. → test `takes_are_distinct_and_all_maximal` (Task 1).
- Égalité d'argent entre survivants à plus de deux : rangs égaux. → test `survivors_with_equal_money_tie` (Task 1).

---

### Task 1: Moteur Pennywise (règles, vue, tests)

**Files:**
- Create: `games/pennywise/Cargo.toml`, `games/pennywise/RULES.md`, `games/pennywise/src/lib.rs`, `games/pennywise/src/purse.rs`, `games/pennywise/src/view.rs`, `games/pennywise/tests/rules.rs`
- Modify: `Cargo.toml` (membre `games/pennywise`, dépendance de workspace `pennywise = { path = "games/pennywise" }`)

**Interfaces:**
- Produces (`pennywise` crate, tout réexporté à la racine) :
  - `pub const COINS: [u8; 7] = [1, 2, 3, 5, 10, 20, 25];`
  - `pub struct Purse` (dans `purse.rs`) : compte par valeur, `Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize`. Méthodes : `fn count(&self, cents: u8) -> u8`, `fn add(&mut self, cents: u8)`, `fn remove(&mut self, cents: u8) -> bool`, `fn total(&self) -> u32`, `fn is_empty(&self) -> bool`, `fn stacks(&self) -> Vec<Stack>` (valeurs non nulles, croissantes), `fn max_takes(&self, below: u8) -> Vec<Vec<u8>>` (voir Step 3).
  - `#[derive(Serialize)] #[serde(rename_all = "camelCase")] pub struct Stack { pub cents: u8, pub count: u8 }`
  - `#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)] pub enum Character { #[default] Cody, Darlene, Sugar, Taylor }` avec `fn purse(self) -> Purse`.
  - `#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)] #[serde(default)] pub struct Config { pub character: Character }`
  - `#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)] pub enum Action { Play { coin: u8, take: Vec<u8> } }`, `take` trié par valeur décroissante.
  - `pub struct State { pub players: u8, pub config: Config, pub purses: Vec<Purse>, pub pot: Purse, pub current: Seat, pub last: Option<LastMove>, pub ruined: Option<Seat> }`
  - `#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)] #[serde(rename_all = "camelCase")] pub struct LastMove { pub seat: Seat, pub coin: u8, pub take: Vec<u8> }`
  - `pub struct Pennywise;` qui implémente `Game` (State, Action, View, Config ci-dessus).
  - `View` (dans `view.rs`, `#[serde(rename_all = "camelCase")]`) : `character: Character, purses: Vec<Vec<Stack>>, totals: Vec<u32>, pot: Vec<Stack>, potTotal: u32, current: Option<Seat>, last: Option<LastMove>, ruined: Option<Seat>, ranks: Option<Vec<u8>>`. `current` vaut `None` en fin de partie ; `ranks` vaut `Some` exactement en fin de partie.

- [ ] **Step 1: Écrire les tests de règles qui échouent** dans `games/pennywise/tests/rules.rs`

Un helper `fn state(purses: &[&[u8]], pot: &[u8], current: u8) -> State` construit un état à la main (à partir de `Pennywise::new` puis en remplaçant les bourses). Tests et assertions :

```rust
#[test] fn every_seat_starts_with_the_character_purse() // Cody : total 64, count(25)==1 ; Darlene 52 ; Sugar 26 ; Taylor 23 ; pot vide, last None, ruined None
#[test] fn first_player_comes_from_the_seed()           // Pennywise::new(&cfg, 4, s).current identique pour une même graine ; sur les graines 0..200 les 4 sièges apparaissent
#[test] fn ten_takes_back_nine_at_most()                // pot [5,3,1,1], pose 10 → legal = [Play{coin:10, take:[5,3,1]}] ; après apply : pot [10,1], bourse +9
#[test] fn equal_maximal_takes_are_offered_as_choices() // pot [3,2,2,1], pose 5 → takes {[3,1],[2,2]} (montant 4), rien d'autre
#[test] fn nothing_to_take_is_a_legal_empty_take()      // pot vide pose 10 → take [] ; pot [10,5] pose 5 → take [] ; pose 1 → take []
#[test] fn turn_passes_to_the_left()                    // à 3 joueurs, après un coup du siège 2, current == Seat(0)
#[test] fn last_coin_with_change_back_does_not_ruin()   // bourse [10], pot [5] : pose 10 reprend [5] → ruined None, pending non vide
#[test] fn playing_the_last_coin_ruins_and_ends()       // bourse [5], pot [10] : pose 5 → ruined Some(seat), pending vide, outcome Some
#[test] fn two_players_ruined_one_loses()               // outcome Ranked([1,0]) quand le siège 0 est ruiné ; view.totals[1] = argent restant
#[test] fn survivors_with_equal_money_tie()             // 3 joueurs, siège 0 ruiné, sièges 1 et 2 à 20¢ chacun → Ranked([2,0,0])
#[test] fn forged_moves_are_refused_without_change()    // coin absente, take non maximal ([3] au lieu de [5,3,1]), take non trié ([1,3,5]), mauvais siège, partie finie → Err, état == copie d'avant
#[test] fn takes_are_distinct_and_all_maximal()         // 6 joueurs, pot [1×4, 2×2, 3×3, 5×3, 10×2, 20] : pour chaque pièce posée, takes sans doublon, même somme, somme < pièce
#[test] fn config_reads_the_lobby_json()                // serde_json::from_value::<Config>(json!({"character":"Darlene"})) == Darlene ; Value::Null / {} → défaut Cody
#[test] fn the_view_shows_everything()                  // view(Seat(1)) == view(Spectator) en JSON ; totals, pot, potTotal corrects
```

- [ ] **Step 2: Lancer les tests pour vérifier qu'ils échouent**

Run: `cargo test --release -p pennywise`
Expected: échec de compilation (crate vide).

- [ ] **Step 3: Implémenter `Purse`, `Character` et `Purse::max_takes` dans `purse.rs`**

`max_takes(&self, below: u8)` énumère chaque sous-multiensemble du pot de somme `< below` (récursion sur les 7 valeurs, de la plus grande à la plus petite, en élaguant dès que la somme atteint `below`), garde ceux de somme maximale, et renvoie chacun en liste décroissante. Le pot vide, ou `below <= 1`, renvoie `vec![vec![]]`. Il n'y a pas de doublon, puisqu'on énumère des comptes par valeur.

- [ ] **Step 4: Implémenter `Pennywise` (`lib.rs`) et `View` (`view.rs`)**

- `meta()` : les valeurs des Global Constraints.
- `options()` : `GameOption { key: "character", kind: OptionKind::Choice { values: &["Cody", "Darlene", "Sugar", "Taylor"], default: "Cody" } }`.
- `new` : le premier joueur est `ChaCha8Rng::seed_from_u64(seed).gen::<u32>() % players` ; on ne garde pas la graine dans l'état.
- `legal_actions` : pour chaque valeur en main du siège actif, `self.pot` augmenté de la pièce posée, puis `max_takes(coin)`. Vide si le siège n'a pas la main ou si la partie est finie.
- `apply` : refuser tout `action` absent de `legal_actions(s, seat)`, puis jouer. Si la bourse est vide après le coup, `ruined = Some(seat)` ; sinon le tour passe au siège suivant.
- `outcome` : `Some(Ranked)` par argent restant (plus riche = rang 0, égalités au même rang ; le ruiné, à 0¢, finit dernier de fait) dès que `ruined` est posé.
- `timeout_action` : `legal_actions(s, seat).into_iter().next()`.

- [ ] **Step 5: Écrire `games/pennywise/RULES.md`**

Les sources [J], [C], [U] de la spec §2 ; une règle par puce, chacune avec sa source ; aucun ❓ restant.

- [ ] **Step 6: Lancer les tests pour vérifier qu'ils passent**

Run: `cargo test --release -p pennywise`
Expected: les 14 tests de règles passent.

- [ ] **Step 7: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
git add Cargo.toml Cargo.lock games/pennywise
git commit -m "pennywise: règles, bourses et vue"
```

### Task 2: IA, conformance, registre et mesure

**Files:**
- Create: `games/pennywise/src/heuristics.rs`, `games/pennywise/tests/conformance.rs`
- Modify: `games/pennywise/src/lib.rs` (`impl Determinize`, `mod heuristics`), `crates/ludo-registry/Cargo.toml`, `crates/ludo-registry/src/lib.rs`, `CLAUDE.md` (« Commandes »), `README.md` (liste des jeux)

**Interfaces:**
- Consumes: `Pennywise`, `Config`, `Character`, `State`, `Action` (Task 1).
- Produces: l'entrée `pennywise` dans `ludo_registry::GAMES` (`static PENNYWISE: GameEntry<pennywise::Pennywise>`), que le WASM et le CLI exposent sans autre changement.

- [ ] **Step 1: Écrire la conformance qui échoue** dans `tests/conformance.rs`

```rust
#[test] fn pennywise_conforms_at_every_table_size()  // for players in 2..=6 : check::<Pennywise>(&Config::default(), players, 40, 300 + players)
#[test] fn pennywise_conforms_with_every_character() // Darlene, Sugar, Taylor à 2 et à 4 joueurs, 30 parties chacun
```

- [ ] **Step 2: Lancer pour vérifier l'échec**

Run: `cargo test --release -p pennywise --test conformance`
Expected: échec de compilation (`Determinize` et `Heuristics` absents).

- [ ] **Step 3: Implémenter `Determinize` (renvoie `s.clone()`) et `Heuristics`**

`rollout_action` tire un coup légal au hasard. On garde `evaluate`, `budget` et `search_actions` par défaut, sauf si la mesure de la Step 6 l'exige ; dans ce cas, `evaluate` renvoie la part de l'argent total détenue par chaque siège.

- [ ] **Step 4: Inscrire le jeu** dans `crates/ludo-registry` (dépendance `pennywise.workspace = true`, un `static PENNYWISE`, l'entrée en fin de `GAMES`)

- [ ] **Step 5: Lancer toute la suite**

Run: `cargo test --workspace --release`
Expected: tout passe, `ids_are_unique_and_every_game_starts` compris.

- [ ] **Step 6: Mesurer l'IA**

Run: `cargo run --release -p ludo-cli -- arena pennywise 200 hard easy`, puis la même commande avec `4` joueurs.
Expected: Difficile ≥ 0.55 dans les deux cas. Noter les deux valeurs mesurées.

- [ ] **Step 7: Documenter et committer**

Dans `CLAUDE.md` (« Commandes ») : `- \`cargo run --release -p ludo-cli -- arena pennywise 200 hard easy\` : Difficile doit marquer ≥ 0.55 contre Facile (<mesuré> ; <mesuré> à 4 joueurs).` Dans `README.md`, après le Président : `- **Pennywise** (2–6 joueurs, James Ernest / Cheapass Games, jeu de pièces ; personnage Cody, Darlene, Sugar ou Taylor en option).`

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
git add games/pennywise crates/ludo-registry Cargo.lock CLAUDE.md README.md
git commit -m "pennywise: IA, conformance et registre"
```

### Task 3: Plateau web

**Files:**
- Create: `web/src/games/pennywise/types.ts`, `Board.tsx`, `pennywise.css`, `rules.ts`, `__tests__/pennywise.test.tsx`
- Modify: `web/src/games/registry.ts` (`GameFamily` + `"coins"`, entrée `{ id: "pennywise", family: "coins", Board: PennywiseBoard, rules: PENNYWISE_RULES }`), `web/src/shell/Home.tsx` (étagère `{ title: "Jeux de pièces", families: ["coins"] }` après « Cartes traditionnelles »), `web/src/shell/Emblem.tsx` (dessin `coins` : trois jetons empilés), `web/src/games/options.ts` (`"pennywise.character": "Personnage"`), `web/src/shell/__tests__/home.test.tsx`

**Interfaces:**
- Consumes: la `View` et l'`Action` de Task 1, sérialisées en JSON.
- Produces (exportés par `Board.tsx`, pour les tests) : `PennywiseBoard`, `takesFor(legal: PennywiseAction[], coin: number): PennywiseAction[]`, `describeTake(take: number[]): string` (`[]` → `"rien"`, `[5,3,1]` → `"5¢ + 3¢ + 1¢"`).

- [ ] **Step 1: Écrire `types.ts`** à l'identique de la `View`, de `LastMove`, de `Stack` et de l'`Action` de Task 1 (`type PennywiseAction = { Play: { coin: number; take: number[] } }`).

- [ ] **Step 2: Écrire les tests qui échouent** dans `__tests__/pennywise.test.tsx` (rendu `renderToStaticMarkup`, comme `cards/__tests__/president.test.tsx`)

```ts
it("describes a take")                         // describeTake([]) === "rien" ; describeTake([5,3,1]) === "5¢ + 3¢ + 1¢"
it("groups legal plays by coin")              // takesFor(legal, 5) renvoie les deux Play{coin:5} et rien d'autre
it("shows every purse, the pot and the totals") // le HTML contient chaque nom de siège, "64¢", le total du pot
it("makes only playable coins clickable")     // à mon tour : les jetons d'une valeur de legalActions portent la classe "playable" ; pas à mon tour : aucun
it("tells the last move")                     // last {seat:1, coin:10, take:[5,3,1]} → "Hugo a posé 10¢ et repris 5¢ + 3¢ + 1¢"
it("announces the end with the ranking")      // ruined: 0, ranks [1,0] → le nom du siège 0 suivi de "est ruiné", et le gagnant en premier
```

Dans `home.test.tsx` : `it("shelves the coin games after the classic cards")`. Avec une entrée `pennywise` dans le catalogue de test, l'index de « Cartes traditionnelles » < celui de « Jeux de pièces » < celui de « Pennywise ».

- [ ] **Step 3: Lancer pour vérifier l'échec**

Run: `cd web && pnpm test`
Expected: les nouveaux tests échouent (module absent).

- [ ] **Step 4: Implémenter le plateau**

- Jetons ronds, avec la valeur au centre et les couleurs de la spec §6 : 1¢ `#f4f1ea`, 2¢ `#b9a3d6`, 3¢ `#3f8a4a`, 5¢ `#c8322b`, 10¢ `#3b5bab`, 20¢ `#5b3a8c`, 25¢ `#7a4a26` (dans `pennywise.css`, une classe par valeur `.coin-1` … `.coin-25`).
- Le pot est au centre, avec `potTotal`. Chaque siège a sa rangée de piles et son total, et le siège actif porte le style laiton commun (`active`).
- Un `useState<number | null>` retient la pièce choisie. Un clic sur une pièce jouable envoie directement l'action si `takesFor` n'en renvoie qu'une. Sinon, une liste de boutons « Reprendre <describeTake> » apparaît, avec « Annuler ».
- `rules.ts` : rappel en 4 puces, d'accord avec `RULES.md` (mains par personnage, pose puis reprise maximale sous la pièce posée, fin au premier ruiné, classement par argent).

- [ ] **Step 5: Vérifier**

Run: `cd web && pnpm test && pnpm type-check && cd .. && ./scripts/build-wasm.sh && cd web && pnpm build`
Expected: tout passe.

- [ ] **Step 6: Contrôle dans le navigateur**

`pnpm preview`, puis `playwright-core` avec le Chromium de `~/.cache/ms-playwright` : depuis l'accueil, ouvrir Pennywise, choisir Darlene, lancer une partie à 2 contre l'IA et jouer une pièce. Le pot et les totaux doivent changer et l'IA doit répondre. Faire une capture d'écran.

- [ ] **Step 7: Commit**

```bash
git add web
git commit -m "web: plateau Pennywise et étagère Jeux de pièces"
```

Puis passer la spec à « Statut : plan livré ».

# ludo-rs : plateforme + Yavalath — plan d'implémentation

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Un site statique en Rust/WASM + React où l'on joue au Yavalath (2–3 joueurs) contre l'IA, en local ou en P2P, sur une plateforme où ajouter un jeu ne touche pas la plateforme.

**Architecture:** Chaque jeu implémente le trait `Game` de `ludo-core` (réducteur pur, vues par siège). `ludo-core` fournit aussi le journal de partie, l'IA générique (ISMCTS max-n) et une batterie de conformance ; une implémentation générique `GameEntry<G>` efface les types en JSON pour `ludo-wasm` et `ludo-cli`. Le web (React/Vite) fait tourner le moteur et l'IA dans deux Workers ; l'hôte fait autorité et envoie à chaque invité PeerJS uniquement sa vue.

**Tech Stack:** Rust stable (edition 2021), `rand 0.8` (`small_rng`), `serde`/`serde_json`, `wasm-bindgen =0.2.121`, `js-sys 0.3` ; React 19, Vite 7, TypeScript 5.9, Vitest 3, PeerJS 1.5, pnpm 12.

**Spec:** `docs/superpowers/specs/2026-09-26-plateforme-yavalath-design.md`

## Global Constraints

- Dépôt : `~/prog/ludo-rs`, branche `main`. Commits terminés par `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Avant chaque commit : `cargo fmt --all` et `cargo clippy --workspace --all-targets -- -D warnings` propres.
- `ludo-core` et `games/*` ne dépendent jamais de `wasm-bindgen`, `js-sys`, `web-sys` ni d'aucune API navigateur ; seul `crates/ludo-wasm` en dépend.
- Le moteur possède toute la logique ; le web affiche des valeurs fournies par le moteur et envoie des actions. Toute valeur dérivée dont l'UI a besoin va dans la `View` du jeu.
- `apply` ne panique jamais sur une entrée invalide : il renvoie `Illegal` **sans modifier l'état**.
- Texte de l'UI et messages `Illegal` en français ; identifiants et commentaires de code en anglais.
- `wasm-bindgen` épinglé à `=0.2.121` (même version que la CLI installée).
- Niveaux IA : budget navigateur Facile 100 ms / Moyen 1000 ms / Difficile 3000 ms ; budget CLI/tests 100 / 1000 / 3000 itérations ; Facile joue un coup aléatoire avec probabilité 0,15.
- P2P : préfixe d'identifiant PeerJS `ludo-`, code de salle de 6 caractères, `PROTOCOL_VERSION = 1`.
- Hot-seat proposé seulement si `meta.hiddenInfo === false`.
- Chaque règle de `games/yavalath/RULES.md` cite **[R]** (livret nestorgames) ; les choix non tranchés sont marqués ❓.
- Frontière WASM : tout passe en **chaînes JSON** (`serde_json` côté Rust, `JSON.parse`/`JSON.stringify` côté JS).

## Écarts assumés par rapport à la spec (reportés dans la spec à la tâche 15)

1. Récompense IA : part des autres sièges devancés, une égalité comptant pour moitié (sinon un nul vaudrait une victoire).
2. `ai_move` prend un niveau (`"Easy" | "Medium" | "Hard"`) au lieu d'un budget en ms, car le niveau fixe aussi la probabilité d'erreur.
3. Conformance : « action hors liste refusée » est testé génériquement en faisant jouer un siège qui n'est pas en attente ; les coups hors liste propres à un jeu (case occupée…) sont couverts par ses tests de règles.
4. `Hello` ne contient pas `game_id` : l'invité ne connaît que le code ; l'hôte envoie le jeu dans `Lobby`, et l'invité refuse un jeu absent de son registre.
5. « Fermé » est réalisé par le choix du nombre de joueurs dans le lobby : les sièges au-delà n'existent pas.

## Review Focus

1. Un invité envoie un message malformé ou une action pour un siège qui n'est pas le sien → l'hôte le refuse, la partie continue (Tâches 3, 13).
2. L'hôte recharge sa page en pleine partie puis reprend : un invité qui revient avec son jeton retrouve **son** siège (Tâche 13, `fromJSON` + reconnexion).
3. Deux invités se présentent pour la dernière place libre → un seul est assis, l'autre reçoit « La table est complète. » (Tâche 13).
4. L'IA a calculé sur un état périmé (un autre siège a joué entre-temps) → le coup est ignoré et l'IA recalcule si elle doit encore jouer (Tâche 12 : `rev.test.ts` fixe la règle ; `Table.playAi` la branche, vérifié à la main en 3 joueurs avec deux IA).
5. À 3 joueurs, la seule case de blocage imposée aligne 3 pierres du bloqueur → il doit la jouer et il est éliminé (Tâche 7).

---

## Structure des fichiers

```
Cargo.toml                         workspace
rust-toolchain.toml
.gitignore
CLAUDE.md, README.md
crates/ludo-core/src/
  lib.rs                           ré-exports
  game.rs                          Seat, Viewer, GameMeta, Pending, Outcome, Illegal, Game, Determinize, Heuristics
  log.rs                           MatchLog, Match, replay
  conformance.rs                   check::<G>()
  ai.rs                            Level, Budget, Search, choose::<G>()
  any.rs                           LudoError, AnyMatch, AnyGame, GameEntry, find
  testgames.rs                     (cfg(test)) Nim, HighCard, Leaky
crates/ludo-registry/src/lib.rs    GAMES, find
crates/ludo-cli/src/main.rs        list, arena, play
crates/ludo-wasm/src/lib.rs        API wasm-bindgen
games/yavalath/
  RULES.md
  src/lib.rs                       Yavalath, Config, Action, State, règles
  src/board.rs                     Cell, Color, Board, Effect
  src/view.rs                      View, CellView
  src/heuristics.rs                impl Heuristics
  tests/rules.rs, tests/three_players.rs, tests/conformance.rs
scripts/build-wasm.sh
web/
  package.json, tsconfig.json, vite.config.ts, index.html
  src/main.tsx, App.tsx, styles.css, vite-env.d.ts
  src/engine/types.ts, rpc.ts, engine.worker.ts, ai.worker.ts, clients.ts
  src/games/registry.ts, games/yavalath/{types.ts, Board.tsx}
  src/session/session.ts, table.ts, local.ts, rev.ts, protocol.ts, seats.ts, storage.ts, names.ts, host.ts, guest.ts
  src/session/__tests__/protocol.test.ts, seats.test.ts
  src/shell/Home.tsx, Lobby.tsx, TableScreen.tsx, HostLobby.tsx
.github/workflows/ci.yml, pages.yml
```

---

### Task 1: Workspace, types du contrat et jeu de test Nim

**Files:**
- Create: `Cargo.toml`, `rust-toolchain.toml`, `.gitignore`
- Create: `crates/ludo-core/Cargo.toml`, `crates/ludo-core/src/lib.rs`, `crates/ludo-core/src/game.rs`, `crates/ludo-core/src/testgames.rs`

**Interfaces:**
- Produces: `Seat(pub u8)` (+ `index()`), `Viewer::{Seat(Seat), Spectator}`, `GameMeta { id, name, min_players, max_players, hidden_info }`, `Pending(pub Vec<Seat>)` (+ `none()`, `one()`, `is_empty()`, `contains()`), `Outcome::Ranked(Vec<u8>)` (+ `rewards() -> Vec<f64>`), `Illegal(pub String)` (+ `new()`), traits `Game`, `Determinize`, `Heuristics` ; en test : `testgames::{Nim, NimConfig, NimState}`.

- [ ] **Step 1: Créer le workspace**

`Cargo.toml` :
```toml
[workspace]
members = ["crates/ludo-core"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT"

[workspace.dependencies]
rand = { version = "0.8", features = ["small_rng"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
ludo-core = { path = "crates/ludo-core" }

[profile.release]
opt-level = 3

# WASM build: `cargo build --profile wasm-release`.
[profile.wasm-release]
inherits = "release"
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

`rust-toolchain.toml` :
```toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
targets = ["wasm32-unknown-unknown"]
```

`.gitignore` :
```
/target
/web/node_modules
/web/dist
# Generated by scripts/build-wasm.sh
/web/src/wasm
```

`crates/ludo-core/Cargo.toml` :
```toml
[package]
name = "ludo-core"
description = "Game contract, match log, generic AI and conformance checks"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
rand.workspace = true
serde.workspace = true
serde_json.workspace = true
```

`crates/ludo-core/src/lib.rs` :
```rust
//! The contract every game implements, and what the platform builds on it.

mod game;

pub use game::{Determinize, Game, GameMeta, Heuristics, Illegal, Outcome, Pending, Seat, Viewer};

#[cfg(test)]
mod testgames;
```

- [ ] **Step 2: Écrire les tests (qui échouent)**

À la fin de `crates/ludo-core/src/game.rs` (fichier créé à l'étape 3, écrire d'abord ce module de tests) :
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewards_count_players_beaten_and_half_ties() {
        assert_eq!(Outcome::Ranked(vec![0, 1]).rewards(), vec![1.0, 0.0]);
        assert_eq!(Outcome::Ranked(vec![0, 0]).rewards(), vec![0.5, 0.5]);
        assert_eq!(Outcome::Ranked(vec![0, 1, 1]).rewards(), vec![1.0, 0.25, 0.25]);
        assert_eq!(Outcome::Ranked(vec![1, 0, 2]).rewards(), vec![0.5, 1.0, 0.0]);
    }

    #[test]
    fn pending_helpers() {
        assert!(Pending::none().is_empty());
        assert!(Pending::one(Seat(2)).contains(Seat(2)));
        assert!(!Pending::one(Seat(2)).contains(Seat(1)));
    }
}
```

Dans `crates/ludo-core/src/testgames.rs`, les tests de Nim (le jeu est écrit à l'étape 3) :
```rust
#[cfg(test)]
mod nim_tests {
    use super::*;
    use crate::{Game, Outcome, Seat};

    #[test]
    fn taking_the_last_stone_wins() {
        let mut s = Nim::new(&NimConfig { pile: 4 }, 2, 0);
        assert_eq!(Nim::legal_actions(&s, Seat(0)), vec![1, 2, 3]);
        Nim::apply(&mut s, Seat(0), &3).unwrap();
        assert_eq!(Nim::legal_actions(&s, Seat(1)), vec![1]);
        Nim::apply(&mut s, Seat(1), &1).unwrap();
        assert_eq!(Nim::outcome(&s), Some(Outcome::Ranked(vec![1, 0])));
        assert!(Nim::pending(&s).is_empty());
    }

    #[test]
    fn illegal_moves_are_refused_without_changing_the_state() {
        let mut s = Nim::new(&NimConfig { pile: 4 }, 2, 0);
        let before = s.clone();
        assert!(Nim::apply(&mut s, Seat(0), &4).is_err());
        assert!(Nim::apply(&mut s, Seat(1), &1).is_err());
        assert_eq!(s, before);
    }
}
```

- [ ] **Step 3: Écrire `game.rs` et `testgames.rs`**

`crates/ludo-core/src/game.rs` (au-dessus du module de tests) :
```rust
use std::cmp::Ordering;
use std::fmt;
use std::hash::Hash;

use rand::Rng;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// A place at the table, numbered from 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Seat(pub u8);

impl Seat {
    pub fn index(self) -> usize {
        usize::from(self.0)
    }
}

/// Who a view is computed for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Viewer {
    Seat(Seat),
    Spectator,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameMeta {
    pub id: &'static str,
    pub name: &'static str,
    pub min_players: u8,
    pub max_players: u8,
    pub hidden_info: bool,
}

/// Seats that must decide now. Several seats when the game waits for
/// simultaneous answers; empty exactly when the game is over.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Pending(pub Vec<Seat>);

impl Pending {
    pub fn none() -> Self {
        Self(Vec::new())
    }

    pub fn one(seat: Seat) -> Self {
        Self(vec![seat])
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn contains(&self, seat: Seat) -> bool {
        self.0.contains(&seat)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Outcome {
    /// Rank of each seat (0 = first); equal ranks are ties.
    Ranked(Vec<u8>),
}

impl Outcome {
    /// Each seat's reward in [0, 1]: the share of other seats it finished
    /// ahead of, a tie counting as half.
    pub fn rewards(&self) -> Vec<f64> {
        let Outcome::Ranked(ranks) = self;
        let n = ranks.len();
        if n <= 1 {
            return vec![1.0; n];
        }
        ranks
            .iter()
            .enumerate()
            .map(|(i, &mine)| {
                let beaten: f64 = ranks
                    .iter()
                    .enumerate()
                    .filter(|&(j, _)| j != i)
                    .map(|(_, &theirs)| match mine.cmp(&theirs) {
                        Ordering::Less => 1.0,
                        Ordering::Equal => 0.5,
                        Ordering::Greater => 0.0,
                    })
                    .sum();
                beaten / (n - 1) as f64
            })
            .collect()
    }
}

/// A refused action. The message is shown to the player.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Illegal(pub String);

impl Illegal {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for Illegal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Illegal {}

/// A game as a pure reducer: no I/O, no clock; randomness comes from the
/// seed given to `new` and kept in the state.
pub trait Game: 'static {
    type State: Clone + fmt::Debug + Serialize + DeserializeOwned;
    type Action: Clone + fmt::Debug + Eq + Hash + Serialize + DeserializeOwned;
    /// What one seat (or a spectator) may see. Never contains the seed nor
    /// another seat's hidden data.
    type View: Serialize;
    type Config: Clone + fmt::Debug + Default + Serialize + DeserializeOwned;

    fn meta() -> GameMeta;
    /// `players` is within `meta()`'s bounds (checked by `Match::new`).
    fn new(config: &Self::Config, players: u8, seed: u64) -> Self::State;
    fn pending(s: &Self::State) -> Pending;
    /// Empty for a seat that is not pending.
    fn legal_actions(s: &Self::State, seat: Seat) -> Vec<Self::Action>;
    /// On error the state is left unchanged.
    fn apply(s: &mut Self::State, seat: Seat, action: &Self::Action) -> Result<(), Illegal>;
    fn view(s: &Self::State, viewer: Viewer) -> Self::View;
    /// `Some` exactly when `pending` is empty.
    fn outcome(s: &Self::State) -> Option<Outcome>;
}

pub trait Determinize: Game {
    /// A copy of `s` where everything `observer` cannot see (other hands,
    /// deck order, other seats' unrevealed simultaneous answers) is redrawn
    /// consistently with what `observer` knows. Identity for a
    /// perfect-information game.
    fn determinize<R: Rng + ?Sized>(s: &Self::State, observer: Seat, rng: &mut R) -> Self::State;
}

pub trait Heuristics: Game {
    /// A legal action for `seat` (which is pending), better than uniform random.
    fn rollout_action<R: Rng + ?Sized>(s: &Self::State, seat: Seat, rng: &mut R) -> Self::Action;

    /// Per-seat estimate in [0, 1] used to cut long simulations.
    fn evaluate(_s: &Self::State) -> Option<Vec<f64>> {
        None
    }
}
```

`crates/ludo-core/src/testgames.rs` (au-dessus du module de tests) :
```rust
//! Tiny games that exercise the platform in tests.

use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{Determinize, Game, GameMeta, Heuristics, Illegal, Outcome, Pending, Seat, Viewer};

/// Take 1 to 3 stones; whoever takes the last one wins, everyone else ties second.
pub struct Nim;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NimConfig {
    pub pile: u8,
}

impl Default for NimConfig {
    fn default() -> Self {
        Self { pile: 10 }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NimState {
    pub pile: u8,
    pub players: u8,
    pub current: Seat,
    pub winner: Option<Seat>,
}

impl Game for Nim {
    type State = NimState;
    type Action = u8;
    type View = NimState;
    type Config = NimConfig;

    fn meta() -> GameMeta {
        GameMeta { id: "nim", name: "Nim", min_players: 2, max_players: 4, hidden_info: false }
    }

    fn new(config: &NimConfig, players: u8, _seed: u64) -> NimState {
        NimState { pile: config.pile, players, current: Seat(0), winner: None }
    }

    fn pending(s: &NimState) -> Pending {
        if s.winner.is_some() {
            Pending::none()
        } else {
            Pending::one(s.current)
        }
    }

    fn legal_actions(s: &NimState, seat: Seat) -> Vec<u8> {
        if s.winner.is_some() || seat != s.current {
            return Vec::new();
        }
        (1..=3.min(s.pile)).collect()
    }

    fn apply(s: &mut NimState, seat: Seat, take: &u8) -> Result<(), Illegal> {
        if !Self::legal_actions(s, seat).contains(take) {
            return Err(Illegal::new("coup interdit"));
        }
        s.pile -= take;
        if s.pile == 0 {
            s.winner = Some(seat);
        } else {
            s.current = Seat((seat.0 + 1) % s.players);
        }
        Ok(())
    }

    fn view(s: &NimState, _viewer: Viewer) -> NimState {
        s.clone()
    }

    fn outcome(s: &NimState) -> Option<Outcome> {
        s.winner
            .map(|w| Outcome::Ranked((0..s.players).map(|i| u8::from(i != w.0)).collect()))
    }
}

impl Determinize for Nim {
    fn determinize<R: Rng + ?Sized>(s: &NimState, _observer: Seat, _rng: &mut R) -> NimState {
        s.clone()
    }
}

impl Heuristics for Nim {
    fn rollout_action<R: Rng + ?Sized>(s: &NimState, seat: Seat, rng: &mut R) -> u8 {
        *Self::legal_actions(s, seat).choose(rng).expect("pending seat has a move")
    }
}
```

- [ ] **Step 4: Lancer les tests**

Run: `cd ~/prog/ludo-rs && cargo test -p ludo-core`
Expected: 4 tests PASS.

- [ ] **Step 5: fmt, clippy, commit**

```bash
cd ~/prog/ludo-rs && cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
git add -A && git commit -m "ludo-core: game contract and Nim test game

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: Journal de partie (`Match`, `MatchLog`, `replay`)

**Files:**
- Create: `crates/ludo-core/src/log.rs`
- Modify: `crates/ludo-core/src/lib.rs`

**Interfaces:**
- Consumes: `Game`, `Seat`, `Illegal` (Tâche 1).
- Produces: `MatchLog<G> { game_id: String, config: G::Config, players: u8, seed: u64, actions: Vec<(Seat, G::Action)> }` (Serialize/Deserialize/Clone) ; `Match<G>` avec `new(config, players, seed) -> Result<Self, Illegal>`, `from_log(MatchLog<G>) -> Result<Self, Illegal>`, `apply(&mut self, Seat, G::Action) -> Result<(), Illegal>`, `state(&self) -> &G::State`, `log(&self) -> &MatchLog<G>`, `rev(&self) -> u32` ; `replay::<G>(&MatchLog<G>) -> Result<G::State, Illegal>`.

- [ ] **Step 1: Écrire les tests (qui échouent)**

À la fin de `crates/ludo-core/src/log.rs` :
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::testgames::{Nim, NimConfig};

    fn played() -> Match<Nim> {
        let mut m = Match::<Nim>::new(NimConfig { pile: 5 }, 2, 42).unwrap();
        m.apply(Seat(0), 2).unwrap();
        m.apply(Seat(1), 3).unwrap();
        m
    }

    #[test]
    fn replay_rebuilds_the_same_state() {
        let m = played();
        assert_eq!(m.rev(), 2);
        assert_eq!(replay::<Nim>(m.log()).unwrap(), *m.state());
    }

    #[test]
    fn a_refused_action_is_not_logged() {
        let mut m = played();
        assert!(m.apply(Seat(0), 1).is_err());
        assert_eq!(m.rev(), 2);
    }

    #[test]
    fn a_tampered_log_is_refused() {
        let mut log = played().log().clone();
        log.actions.push((Seat(0), 1));
        let err = replay::<Nim>(&log).unwrap_err();
        assert!(err.0.starts_with("action 2"), "{}", err.0);
    }

    #[test]
    fn a_log_from_another_game_is_refused() {
        let mut log = played().log().clone();
        log.game_id = "chess".into();
        assert!(replay::<Nim>(&log).is_err());
    }

    #[test]
    fn player_count_is_checked() {
        assert!(Match::<Nim>::new(NimConfig::default(), 1, 0).is_err());
        assert!(Match::<Nim>::new(NimConfig::default(), 5, 0).is_err());
    }

    #[test]
    fn the_log_round_trips_through_json() {
        let m = played();
        let json = serde_json::to_string(m.log()).unwrap();
        let back: MatchLog<Nim> = serde_json::from_str(&json).unwrap();
        assert_eq!(replay::<Nim>(&back).unwrap(), *m.state());
    }
}
```

Note : `NimState` dérive `PartialEq` (Tâche 1), d'où les `assert_eq!` directs.

- [ ] **Step 2: Écrire `log.rs`**

```rust
use serde::{Deserialize, Serialize};

use crate::{Game, Illegal, Seat};

/// Everything needed to rebuild a match: the engine is deterministic, so the
/// setup plus the accepted actions, in order, determine the state.
#[derive(Serialize, Deserialize)]
#[serde(bound = "")]
pub struct MatchLog<G: Game> {
    pub game_id: String,
    pub config: G::Config,
    pub players: u8,
    pub seed: u64,
    pub actions: Vec<(Seat, G::Action)>,
}

impl<G: Game> Clone for MatchLog<G> {
    fn clone(&self) -> Self {
        Self {
            game_id: self.game_id.clone(),
            config: self.config.clone(),
            players: self.players,
            seed: self.seed,
            actions: self.actions.clone(),
        }
    }
}

/// A match in progress: its state and the log that produced it.
pub struct Match<G: Game> {
    state: G::State,
    log: MatchLog<G>,
}

impl<G: Game> Match<G> {
    pub fn new(config: G::Config, players: u8, seed: u64) -> Result<Self, Illegal> {
        let meta = G::meta();
        if !(meta.min_players..=meta.max_players).contains(&players) {
            return Err(Illegal::new(format!(
                "{} se joue de {} à {} joueurs.",
                meta.name, meta.min_players, meta.max_players
            )));
        }
        let state = G::new(&config, players, seed);
        let log = MatchLog { game_id: meta.id.to_owned(), config, players, seed, actions: Vec::new() };
        Ok(Self { state, log })
    }

    pub fn from_log(log: MatchLog<G>) -> Result<Self, Illegal> {
        if log.game_id != G::meta().id {
            return Err(Illegal::new(format!("Journal d'un autre jeu : {}.", log.game_id)));
        }
        let MatchLog { config, players, seed, actions, .. } = log;
        let mut m = Self::new(config, players, seed)?;
        for (i, (seat, action)) in actions.into_iter().enumerate() {
            m.apply(seat, action).map_err(|e| Illegal::new(format!("action {i} : {e}")))?;
        }
        Ok(m)
    }

    /// Applies and logs `action`; a refused action changes nothing.
    pub fn apply(&mut self, seat: Seat, action: G::Action) -> Result<(), Illegal> {
        G::apply(&mut self.state, seat, &action)?;
        self.log.actions.push((seat, action));
        Ok(())
    }

    pub fn state(&self) -> &G::State {
        &self.state
    }

    pub fn log(&self) -> &MatchLog<G> {
        &self.log
    }

    /// Number of accepted actions: identifies a state for staleness checks.
    pub fn rev(&self) -> u32 {
        self.log.actions.len() as u32
    }
}

/// Rebuilds the state described by `log`, refusing it at the first illegal action.
pub fn replay<G: Game>(log: &MatchLog<G>) -> Result<G::State, Illegal> {
    Match::from_log(log.clone()).map(|m| m.state)
}
```

`lib.rs` : ajouter `mod log;` et `pub use log::{replay, Match, MatchLog};`.

- [ ] **Step 3: Lancer les tests**

Run: `cargo test -p ludo-core`
Expected: tous PASS (10 tests).

- [ ] **Step 4: fmt, clippy, commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
git add -A && git commit -m "ludo-core: match log and replay

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: Jeu à information cachée HighCard et batterie de conformance

**Files:**
- Create: `crates/ludo-core/src/conformance.rs`
- Modify: `crates/ludo-core/src/testgames.rs` (ajout HighCard, Leaky), `crates/ludo-core/src/lib.rs`

**Interfaces:**
- Consumes: `Match`, `MatchLog`, `replay` (Tâche 2).
- Produces: `conformance::check::<G: Game + Determinize>(config: &G::Config, players: u8, games: usize, seed: u64)` — panique avec un message explicite au premier manquement. En test : `testgames::{HighCard, HighCardState, Choice, Leaky}`.

- [ ] **Step 1: Ajouter HighCard et Leaky à `testgames.rs`**

Ajouter après Nim (et `use rand::rngs::SmallRng; use rand::SeedableRng;` en tête) :
```rust
/// Each seat is dealt one hidden card from 0..=6, then every seat answers
/// Stay or Fold at the same time. Highest card among stayers wins; folders
/// tie behind the winner; other stayers finish last. Nobody stays: all tie.
pub struct HighCard;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Choice {
    Stay,
    Fold,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HighCardState {
    pub cards: Vec<u8>,
    /// Undealt cards, hidden from everyone.
    pub deck: Vec<u8>,
    pub answers: Vec<Option<Choice>>,
}

#[derive(Debug, Serialize)]
pub struct HighCardView {
    pub my_card: Option<u8>,
    pub my_answer: Option<Choice>,
    pub answered: Vec<bool>,
    /// Cards and answers, once everyone has answered.
    pub revealed: Option<(Vec<u8>, Vec<Choice>)>,
}

impl HighCard {
    fn finished(s: &HighCardState) -> Option<Vec<Choice>> {
        s.answers.iter().copied().collect()
    }
}

impl Game for HighCard {
    type State = HighCardState;
    type Action = Choice;
    type View = HighCardView;
    type Config = ();

    fn meta() -> GameMeta {
        GameMeta { id: "highcard", name: "HighCard", min_players: 2, max_players: 4, hidden_info: true }
    }

    fn new(_config: &(), players: u8, seed: u64) -> HighCardState {
        let mut deck: Vec<u8> = (0..=6).collect();
        deck.shuffle(&mut SmallRng::seed_from_u64(seed));
        let cards = deck.split_off(deck.len() - usize::from(players));
        HighCardState { cards, deck, answers: vec![None; usize::from(players)] }
    }

    fn pending(s: &HighCardState) -> Pending {
        Pending((0..s.answers.len() as u8).map(Seat).filter(|x| s.answers[x.index()].is_none()).collect())
    }

    fn legal_actions(s: &HighCardState, seat: Seat) -> Vec<Choice> {
        match s.answers.get(seat.index()) {
            Some(None) => vec![Choice::Stay, Choice::Fold],
            _ => Vec::new(),
        }
    }

    fn apply(s: &mut HighCardState, seat: Seat, choice: &Choice) -> Result<(), Illegal> {
        match s.answers.get_mut(seat.index()) {
            Some(slot @ None) => {
                *slot = Some(*choice);
                Ok(())
            }
            _ => Err(Illegal::new("déjà répondu")),
        }
    }

    fn view(s: &HighCardState, viewer: Viewer) -> HighCardView {
        let mine = match viewer {
            Viewer::Seat(seat) => Some(seat.index()),
            Viewer::Spectator => None,
        };
        HighCardView {
            my_card: mine.map(|i| s.cards[i]),
            my_answer: mine.and_then(|i| s.answers[i]),
            answered: s.answers.iter().map(Option::is_some).collect(),
            revealed: Self::finished(s).map(|answers| (s.cards.clone(), answers)),
        }
    }

    fn outcome(s: &HighCardState) -> Option<Outcome> {
        let answers = Self::finished(s)?;
        let n = answers.len();
        let stayers: Vec<usize> = (0..n).filter(|&i| answers[i] == Choice::Stay).collect();
        let Some(&best) = stayers.iter().max_by_key(|&&i| s.cards[i]) else {
            return Some(Outcome::Ranked(vec![0; n]));
        };
        let folders = (n - stayers.len()) as u8;
        Some(Outcome::Ranked(
            (0..n)
                .map(|i| match answers[i] {
                    _ if i == best => 0,
                    Choice::Fold => 1,
                    Choice::Stay => 1 + folders,
                })
                .collect(),
        ))
    }
}

impl Determinize for HighCard {
    fn determinize<R: Rng + ?Sized>(s: &HighCardState, observer: Seat, rng: &mut R) -> HighCardState {
        if Self::finished(s).is_some() {
            return s.clone();
        }
        let me = observer.index();
        let mut pool: Vec<u8> = (0..=6).filter(|&c| c != s.cards[me]).collect();
        pool.shuffle(rng);
        let mut d = s.clone();
        for i in (0..s.cards.len()).filter(|&i| i != me) {
            d.cards[i] = pool.pop().expect("enough cards");
            if d.answers[i].is_some() {
                d.answers[i] = Some(if rng.gen_bool(0.5) { Choice::Stay } else { Choice::Fold });
            }
        }
        d.deck = pool;
        d
    }
}

impl Heuristics for HighCard {
    fn rollout_action<R: Rng + ?Sized>(_s: &HighCardState, _seat: Seat, rng: &mut R) -> Choice {
        if rng.gen_bool(0.5) {
            Choice::Stay
        } else {
            Choice::Fold
        }
    }
}

/// HighCard with a bug: every view shows every card.
pub struct Leaky;

impl Game for Leaky {
    type State = HighCardState;
    type Action = Choice;
    type View = (HighCardView, Vec<u8>);
    type Config = ();

    fn meta() -> GameMeta {
        GameMeta { id: "leaky", ..HighCard::meta() }
    }
    fn new(config: &(), players: u8, seed: u64) -> HighCardState {
        HighCard::new(config, players, seed)
    }
    fn pending(s: &HighCardState) -> Pending {
        HighCard::pending(s)
    }
    fn legal_actions(s: &HighCardState, seat: Seat) -> Vec<Choice> {
        HighCard::legal_actions(s, seat)
    }
    fn apply(s: &mut HighCardState, seat: Seat, a: &Choice) -> Result<(), Illegal> {
        HighCard::apply(s, seat, a)
    }
    fn view(s: &HighCardState, viewer: Viewer) -> Self::View {
        (HighCard::view(s, viewer), s.cards.clone())
    }
    fn outcome(s: &HighCardState) -> Option<Outcome> {
        HighCard::outcome(s)
    }
}

impl Determinize for Leaky {
    fn determinize<R: Rng + ?Sized>(s: &HighCardState, observer: Seat, rng: &mut R) -> HighCardState {
        HighCard::determinize(s, observer, rng)
    }
}
```

- [ ] **Step 2: Écrire les tests de conformance (qui échouent)**

À la fin de `crates/ludo-core/src/conformance.rs` :
```rust
#[cfg(test)]
mod tests {
    use super::check;
    use crate::testgames::{HighCard, Leaky, Nim, NimConfig};

    #[test]
    fn nim_conforms() {
        check::<Nim>(&NimConfig::default(), 3, 200, 1);
    }

    #[test]
    fn highcard_conforms() {
        check::<HighCard>(&(), 4, 200, 2);
    }

    #[test]
    #[should_panic(expected = "dépend de données cachées")]
    fn a_leaking_view_is_caught() {
        check::<Leaky>(&(), 3, 20, 3);
    }
}
```

- [ ] **Step 3: Écrire `conformance.rs`**

```rust
//! Checks every game must pass (called from each game's tests).

use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use serde::Serialize;
use serde_json::Value;

use crate::{replay, Determinize, Game, Match, MatchLog, Seat, Viewer};

fn json<T: Serialize + ?Sized>(value: &T) -> Value {
    serde_json::to_value(value).expect("engine types serialize to JSON")
}

/// Plays `games` random matches to the end and panics at the first breach
/// of the `Game` contract.
pub fn check<G: Game + Determinize>(config: &G::Config, players: u8, games: usize, seed: u64) {
    let mut rng = SmallRng::seed_from_u64(seed);
    for g in 0..games {
        let mut m = Match::<G>::new(config.clone(), players, rng.gen())
            .unwrap_or_else(|e| panic!("partie {g} : création refusée : {e}"));
        let mut step = 0usize;
        loop {
            let s = m.state();
            let pending = G::pending(s);
            let outcome = G::outcome(s);
            assert_eq!(pending.is_empty(), outcome.is_some(), "partie {g}, coup {step} : `pending` vide ⇔ `outcome` défini");
            check_serde::<G>(s, g, step);
            if G::meta().hidden_info {
                check_hidden::<G>(s, players, &mut rng, g, step);
            }
            if let Some(crate::Outcome::Ranked(ranks)) = outcome {
                assert_eq!(ranks.len(), usize::from(players), "partie {g} : un rang par siège");
                break;
            }
            let seat = *pending.0.choose(&mut rng).expect("non-empty");
            let legal = G::legal_actions(s, seat);
            assert!(!legal.is_empty(), "partie {g}, coup {step} : le siège {} est en attente sans action légale", seat.0);
            for other in (0..players).map(Seat).filter(|&x| !pending.contains(x)) {
                assert!(G::legal_actions(s, other).is_empty(), "partie {g}, coup {step} : le siège {} n'est pas en attente mais a des actions", other.0);
                let mut copy = s.clone();
                assert!(G::apply(&mut copy, other, &legal[0]).is_err(), "partie {g}, coup {step} : le siège {} a joué hors de son tour", other.0);
                assert_eq!(json(&copy), json(s), "partie {g}, coup {step} : un `apply` refusé a modifié l'état");
            }
            let action = legal.choose(&mut rng).expect("non-empty").clone();
            m.apply(seat, action.clone())
                .unwrap_or_else(|e| panic!("partie {g}, coup {step} : action légale {action:?} refusée : {e}"));
            step += 1;
            assert!(step < 100_000, "partie {g} : la partie ne se termine pas");
        }
        check_log::<G>(&m, g);
    }
}

fn check_serde<G: Game>(s: &G::State, g: usize, step: usize) {
    let text = serde_json::to_string(s).expect("state serializes");
    let back: G::State = serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("partie {g}, coup {step} : l'état ne se relit pas : {e}"));
    assert_eq!(json(&back), json(s), "partie {g}, coup {step} : aller-retour serde de l'état");
}

fn check_hidden<G: Game + Determinize>(s: &G::State, players: u8, rng: &mut SmallRng, g: usize, step: usize) {
    for seat in (0..players).map(Seat) {
        let expected = json(&G::view(s, Viewer::Seat(seat)));
        for _ in 0..2 {
            let d = G::determinize(s, seat, rng);
            assert_eq!(json(&G::view(&d, Viewer::Seat(seat))), expected,
                "partie {g}, coup {step} : la vue du siège {} dépend de données cachées", seat.0);
        }
    }
    let spectator = json(&G::view(s, Viewer::Spectator));
    let d = G::determinize(s, Seat(0), rng);
    assert_eq!(json(&G::view(&d, Viewer::Spectator)), spectator,
        "partie {g}, coup {step} : la vue spectateur dépend de données cachées");
}

fn check_log<G: Game>(m: &Match<G>, g: usize) {
    let final_state = json(m.state());
    let replayed = replay::<G>(m.log()).unwrap_or_else(|e| panic!("partie {g} : le journal ne se rejoue pas : {e}"));
    assert_eq!(json(&replayed), final_state, "partie {g} : le rejeu ne redonne pas l'état final");
    let text = serde_json::to_string(m.log()).expect("log serializes");
    let back: MatchLog<G> = serde_json::from_str(&text).unwrap_or_else(|e| panic!("partie {g} : le journal ne se relit pas : {e}"));
    assert_eq!(json(&replay::<G>(&back).expect("replays")), final_state, "partie {g} : aller-retour serde du journal");
    if let Some(last) = m.log().actions.last().cloned() {
        let mut tampered = m.log().clone();
        tampered.actions.push(last);
        assert!(replay::<G>(&tampered).is_err(), "partie {g} : un journal avec une action après la fin est accepté");
    }
}
```

`lib.rs` : ajouter `pub mod conformance;`.

- [ ] **Step 4: Lancer les tests**

Run: `cargo test -p ludo-core`
Expected: tous PASS, dont `a_leaking_view_is_caught`.

- [ ] **Step 5: fmt, clippy, commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
git add -A && git commit -m "ludo-core: conformance checks, HighCard hidden-information test game

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: IA générique (ISMCTS max-n)

**Files:**
- Create: `crates/ludo-core/src/ai.rs`
- Modify: `crates/ludo-core/src/lib.rs`

**Interfaces:**
- Consumes: `Game`, `Determinize`, `Heuristics`, `Outcome::rewards` (Tâche 1) ; testgames (Tâches 1, 3).
- Produces: `ai::Level::{Easy, Medium, Hard}` (Serialize/Deserialize, noms `"Easy"`…, méthodes `millis()`, `iterations()`, `blunder()`), `ai::Budget::{Iterations(u32), Millis(u32)}`, `ai::Search { budget, blunder: f64, seed: u64 }` avec `Search::timed(Level, u64)` et `Search::counted(Level, u64)`, `ai::choose::<G>(state: &G::State, seat: Seat, search: &Search, now_ms: &dyn Fn() -> f64) -> Result<G::Action, Illegal>`.

- [ ] **Step 1: Écrire les tests (qui échouent)**

À la fin de `crates/ludo-core/src/ai.rs` :
```rust
#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;
    use crate::testgames::{Choice, HighCard, HighCardState, Nim, NimConfig};

    fn no_clock() -> f64 {
        0.0
    }

    fn counted(iterations: u32, seed: u64) -> Search {
        Search { budget: Budget::Iterations(iterations), blunder: 0.0, seed }
    }

    #[test]
    fn nim_leaves_a_multiple_of_four() {
        for (pile, best) in [(5, 1), (6, 2), (7, 3)] {
            let s = Nim::new(&NimConfig { pile }, 2, 0);
            assert_eq!(choose::<Nim>(&s, Seat(0), &counted(3000, 1), &no_clock).unwrap(), best, "pile {pile}");
        }
    }

    #[test]
    fn same_seed_same_move() {
        let s = Nim::new(&NimConfig { pile: 10 }, 3, 0);
        let a = choose::<Nim>(&s, Seat(0), &counted(500, 9), &no_clock).unwrap();
        let b = choose::<Nim>(&s, Seat(0), &counted(500, 9), &no_clock).unwrap();
        assert_eq!(a, b);
    }

    fn highcard(cards: Vec<u8>) -> HighCardState {
        let deck = (0..=6).filter(|c| !cards.contains(c)).collect();
        HighCardState { answers: vec![None; cards.len()], cards, deck }
    }

    #[test]
    fn stays_with_the_best_card() {
        let s = highcard(vec![6, 2]);
        assert_eq!(choose::<HighCard>(&s, Seat(0), &counted(2000, 3), &no_clock).unwrap(), Choice::Stay);
    }

    #[test]
    fn does_not_read_hidden_cards() {
        for seed in 0..10 {
            let a = choose::<HighCard>(&highcard(vec![3, 1]), Seat(0), &counted(300, seed), &no_clock).unwrap();
            let b = choose::<HighCard>(&highcard(vec![3, 5]), Seat(0), &counted(300, seed), &no_clock).unwrap();
            assert_eq!(a, b, "seed {seed}");
        }
    }

    #[test]
    fn a_time_budget_stops_the_search() {
        let t = Cell::new(0.0);
        let clock = || {
            t.set(t.get() + 1.0);
            t.get()
        };
        let s = Nim::new(&NimConfig { pile: 20 }, 2, 0);
        let search = Search { budget: Budget::Millis(50), blunder: 0.0, seed: 0 };
        assert!(choose::<Nim>(&s, Seat(0), &search, &clock).is_ok());
    }

    #[test]
    fn a_blunder_is_still_legal() {
        let s = Nim::new(&NimConfig { pile: 10 }, 2, 0);
        let search = Search { budget: Budget::Iterations(10), blunder: 1.0, seed: 4 };
        let action = choose::<Nim>(&s, Seat(0), &search, &no_clock).unwrap();
        assert!(Nim::legal_actions(&s, Seat(0)).contains(&action));
    }

    #[test]
    fn a_seat_without_decision_is_an_error() {
        let s = Nim::new(&NimConfig { pile: 10 }, 2, 0);
        assert!(choose::<Nim>(&s, Seat(1), &counted(10, 0), &no_clock).is_err());
    }
}
```

- [ ] **Step 2: Écrire `ai.rs`**

```rust
//! Generic search: single-observer ISMCTS with per-seat rewards (max-n).
//! For a perfect-information game the determinization is skipped and this is
//! plain UCT.

use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

use crate::{Determinize, Game, Heuristics, Illegal, Seat};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Level {
    Easy,
    Medium,
    Hard,
}

impl Level {
    /// Budget in the browser.
    pub fn millis(self) -> u32 {
        match self {
            Level::Easy => 100,
            Level::Medium => 1000,
            Level::Hard => 3000,
        }
    }

    /// Reproducible budget for the CLI and tests.
    pub fn iterations(self) -> u32 {
        match self {
            Level::Easy => 100,
            Level::Medium => 1000,
            Level::Hard => 3000,
        }
    }

    /// Probability of playing a random legal move instead of searching.
    pub fn blunder(self) -> f64 {
        match self {
            Level::Easy => 0.15,
            Level::Medium | Level::Hard => 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Budget {
    Iterations(u32),
    Millis(u32),
}

#[derive(Clone, Copy, Debug)]
pub struct Search {
    pub budget: Budget,
    pub blunder: f64,
    pub seed: u64,
}

impl Search {
    pub fn timed(level: Level, seed: u64) -> Self {
        Self { budget: Budget::Millis(level.millis()), blunder: level.blunder(), seed }
    }

    pub fn counted(level: Level, seed: u64) -> Self {
        Self { budget: Budget::Iterations(level.iterations()), blunder: level.blunder(), seed }
    }
}

/// UCB exploration constant, for rewards in [0, 1].
const EXPLORATION: f64 = 0.7;
/// Simulations longer than this use `Heuristics::evaluate` when it answers.
const EVALUATE_AFTER: u32 = 200;
/// The clock is read every this many iterations.
const CLOCK_EVERY: u32 = 16;

struct Node<A> {
    /// The action that leads here from the parent (none for the root).
    action: Option<A>,
    /// Who chose `action`: `reward` is theirs.
    mover: Seat,
    parent: Option<usize>,
    children: Vec<usize>,
    visits: u32,
    availability: u32,
    reward: f64,
}

impl<A> Node<A> {
    fn ucb(&self) -> f64 {
        let visits = f64::from(self.visits);
        self.reward / visits + EXPLORATION * (f64::from(self.availability).ln() / visits).sqrt()
    }
}

/// The action `seat` should take. `now_ms` is only read for `Budget::Millis`.
/// The search starts from a determinization, so it never uses what `seat`
/// cannot see even though it receives the full state.
pub fn choose<G: Game + Determinize + Heuristics>(
    state: &G::State,
    seat: Seat,
    search: &Search,
    now_ms: &dyn Fn() -> f64,
) -> Result<G::Action, Illegal> {
    let legal = G::legal_actions(state, seat);
    if legal.is_empty() {
        return Err(Illegal::new(format!("Le siège {} n'a rien à décider.", seat.0)));
    }
    let mut rng = SmallRng::seed_from_u64(search.seed);
    if legal.len() == 1 {
        return Ok(legal[0].clone());
    }
    if search.blunder > 0.0 && rng.gen_bool(search.blunder) {
        return Ok(legal.choose(&mut rng).expect("non-empty").clone());
    }
    let mut tree = vec![Node {
        action: None,
        mover: seat,
        parent: None,
        children: Vec::new(),
        visits: 0,
        availability: 0,
        reward: 0.0,
    }];
    let start = now_ms();
    let mut done = 0u32;
    loop {
        iterate::<G>(&mut tree, state, seat, &mut rng);
        done += 1;
        let stop = match search.budget {
            Budget::Iterations(n) => done >= n,
            Budget::Millis(ms) => done % CLOCK_EVERY == 0 && now_ms() - start >= f64::from(ms),
        };
        if stop {
            break;
        }
    }
    // The most visited action is the most robust choice.
    let best = tree[0]
        .children
        .iter()
        .map(|&c| &tree[c])
        .filter(|n| n.mover == seat)
        .max_by_key(|n| n.visits)
        .and_then(|n| n.action.clone());
    Ok(best.filter(|a| legal.contains(a)).unwrap_or_else(|| legal[0].clone()))
}

fn iterate<G: Game + Determinize + Heuristics>(
    tree: &mut Vec<Node<G::Action>>,
    root: &G::State,
    me: Seat,
    rng: &mut SmallRng,
) {
    let mut sim = if G::meta().hidden_info { G::determinize(root, me, rng) } else { root.clone() };
    let mut current = 0;
    // Selection, then expansion of one new node.
    loop {
        let pending = G::pending(&sim);
        let Some(&first) = pending.0.first() else { break };
        // At the root the searching seat decides first, even if others are pending too;
        // elsewhere simultaneous decisions are taken in seat order.
        let mover = if current == 0 { me } else { first };
        let legal = G::legal_actions(&sim, mover);
        let available: Vec<usize> = tree[current]
            .children
            .iter()
            .copied()
            .filter(|&c| tree[c].mover == mover && tree[c].action.as_ref().is_some_and(|a| legal.contains(a)))
            .collect();
        for &c in &available {
            tree[c].availability += 1;
        }
        let untried: Vec<&G::Action> = legal
            .iter()
            .filter(|a| !available.iter().any(|&c| tree[c].action.as_ref() == Some(*a)))
            .collect();
        if let Some(&action) = untried.choose(rng) {
            let action = action.clone();
            G::apply(&mut sim, mover, &action).expect("legal action");
            tree.push(Node {
                action: Some(action),
                mover,
                parent: Some(current),
                children: Vec::new(),
                visits: 0,
                availability: 1,
                reward: 0.0,
            });
            let child = tree.len() - 1;
            tree[current].children.push(child);
            current = child;
            break;
        }
        let best = available
            .into_iter()
            .max_by(|&a, &b| tree[a].ucb().total_cmp(&tree[b].ucb()))
            .expect("a pending seat has legal actions");
        let action = tree[best].action.clone().expect("only the root has no action");
        G::apply(&mut sim, mover, &action).expect("legal action");
        current = best;
    }
    let rewards = rollout::<G>(sim, rng);
    let mut node = Some(current);
    while let Some(i) = node {
        tree[i].visits += 1;
        tree[i].reward += rewards[tree[i].mover.index()];
        node = tree[i].parent;
    }
}

fn rollout<G: Game + Heuristics>(mut sim: G::State, rng: &mut SmallRng) -> Vec<f64> {
    let mut steps = 0u32;
    loop {
        if let Some(outcome) = G::outcome(&sim) {
            return outcome.rewards();
        }
        if steps >= EVALUATE_AFTER {
            if let Some(estimate) = G::evaluate(&sim) {
                return estimate;
            }
        }
        let seat = *G::pending(&sim).0.first().expect("an unfinished game has a pending seat");
        let action = G::rollout_action(&sim, seat, rng);
        G::apply(&mut sim, seat, &action).expect("rollout_action returns a legal action");
        steps += 1;
    }
}
```

`lib.rs` : ajouter `pub mod ai;`.

- [ ] **Step 3: Lancer les tests**

Run: `cargo test -p ludo-core --release`
Expected: tous PASS (le mode release rend `nim_leaves_a_multiple_of_four` rapide).

- [ ] **Step 4: fmt, clippy, commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
git add -A && git commit -m "ludo-core: generic ISMCTS with per-seat rewards

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: Effacement de type JSON (`AnyGame`, `AnyMatch`)

**Files:**
- Create: `crates/ludo-core/src/any.rs`
- Modify: `crates/ludo-core/src/lib.rs`

**Interfaces:**
- Consumes: `Match`, `MatchLog` (Tâche 2), `ai::{choose, Search}` (Tâche 4).
- Produces:
  - `LudoError::{UnknownGame(String), BadJson(String), Illegal(String)}` (Display, Error, `From<Illegal>`).
  - `trait AnyMatch { fn game_id(&self) -> &'static str; fn rev(&self) -> u32; fn pending(&self) -> Vec<Seat>; fn legal_actions_json(&self, seat: Seat) -> Value; fn apply_json(&mut self, seat: Seat, action: Value) -> Result<(), LudoError>; fn view_json(&self, viewer: Viewer) -> Value; fn outcome(&self) -> Option<Outcome>; fn state_json(&self) -> Value; fn log_json(&self) -> Value; }`, implémenté par `Match<G>`.
  - `trait AnyGame: Sync { fn meta(&self) -> GameMeta; fn new_match(&self, config: Value, players: u8, seed: u64) -> Result<Box<dyn AnyMatch>, LudoError>; fn import_log(&self, log: Value) -> Result<Box<dyn AnyMatch>, LudoError>; fn ai_move(&self, state: Value, seat: Seat, search: &Search, now_ms: &dyn Fn() -> f64) -> Result<Value, LudoError>; }`.
  - `GameEntry<G>` avec `const fn new()` ; `find(games: &[&'a dyn AnyGame], id: &str) -> Result<&'a dyn AnyGame, LudoError>`.
  - `config` à `Value::Null` ⇒ `G::Config::default()`.

- [ ] **Step 1: Écrire les tests (qui échouent)**

À la fin de `crates/ludo-core/src/any.rs` :
```rust
#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::ai::Level;
    use crate::testgames::Nim;

    static NIM: GameEntry<Nim> = GameEntry::new();
    static GAMES: &[&dyn AnyGame] = &[&NIM];

    #[test]
    fn plays_through_json() {
        let game = find(GAMES, "nim").unwrap();
        let mut m = game.new_match(json!({ "pile": 5 }), 2, 1).unwrap();
        assert_eq!(m.legal_actions_json(Seat(0)), json!([1, 2, 3]));
        m.apply_json(Seat(0), json!(2)).unwrap();
        assert_eq!(m.rev(), 1);
        assert_eq!(m.pending(), vec![Seat(1)]);
        assert_eq!(m.view_json(Viewer::Spectator)["pile"], json!(3));
    }

    #[test]
    fn null_config_means_default() {
        let m = find(GAMES, "nim").unwrap().new_match(Value::Null, 2, 1).unwrap();
        assert_eq!(m.state_json()["pile"], json!(10));
    }

    #[test]
    fn errors_are_typed() {
        let game = find(GAMES, "nim").unwrap();
        let mut m = game.new_match(Value::Null, 2, 1).unwrap();
        assert!(matches!(m.apply_json(Seat(0), json!("x")), Err(LudoError::BadJson(_))));
        assert!(matches!(m.apply_json(Seat(0), json!(9)), Err(LudoError::Illegal(_))));
        assert!(matches!(find(GAMES, "chess"), Err(LudoError::UnknownGame(_))));
        assert!(matches!(game.new_match(Value::Null, 9, 1), Err(LudoError::Illegal(_))));
    }

    #[test]
    fn log_import_resumes_the_match() {
        let game = find(GAMES, "nim").unwrap();
        let mut m = game.new_match(Value::Null, 2, 1).unwrap();
        m.apply_json(Seat(0), json!(3)).unwrap();
        let resumed = game.import_log(m.log_json()).unwrap();
        assert_eq!(resumed.state_json(), m.state_json());
        assert_eq!(resumed.rev(), 1);
    }

    #[test]
    fn ai_move_is_applicable() {
        let game = find(GAMES, "nim").unwrap();
        let mut m = game.new_match(Value::Null, 2, 1).unwrap();
        let action = game.ai_move(m.state_json(), Seat(0), &Search::counted(Level::Easy, 5), &|| 0.0).unwrap();
        m.apply_json(Seat(0), action).unwrap();
    }
}
```

- [ ] **Step 2: Écrire `any.rs`**

```rust
//! Type erasure: every game seen through JSON, for the WASM bridge and the CLI.

use std::fmt;
use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

use crate::ai::{self, Search};
use crate::{Determinize, Game, GameMeta, Heuristics, Illegal, Match, MatchLog, Outcome, Seat, Viewer};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LudoError {
    UnknownGame(String),
    BadJson(String),
    Illegal(String),
}

impl fmt::Display for LudoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LudoError::UnknownGame(id) => write!(f, "Jeu inconnu : {id}."),
            LudoError::BadJson(message) => write!(f, "Données invalides : {message}."),
            LudoError::Illegal(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for LudoError {}

impl From<Illegal> for LudoError {
    fn from(e: Illegal) -> Self {
        LudoError::Illegal(e.0)
    }
}

fn from_json<T: DeserializeOwned>(value: Value) -> Result<T, LudoError> {
    serde_json::from_value(value).map_err(|e| LudoError::BadJson(e.to_string()))
}

fn to_json<T: Serialize + ?Sized>(value: &T) -> Value {
    serde_json::to_value(value).expect("engine types serialize to JSON")
}

pub trait AnyMatch {
    fn game_id(&self) -> &'static str;
    fn rev(&self) -> u32;
    fn pending(&self) -> Vec<Seat>;
    fn legal_actions_json(&self, seat: Seat) -> Value;
    fn apply_json(&mut self, seat: Seat, action: Value) -> Result<(), LudoError>;
    fn view_json(&self, viewer: Viewer) -> Value;
    fn outcome(&self) -> Option<Outcome>;
    fn state_json(&self) -> Value;
    fn log_json(&self) -> Value;
}

impl<G: Game> AnyMatch for Match<G> {
    fn game_id(&self) -> &'static str {
        G::meta().id
    }

    fn rev(&self) -> u32 {
        Match::rev(self)
    }

    fn pending(&self) -> Vec<Seat> {
        G::pending(self.state()).0
    }

    fn legal_actions_json(&self, seat: Seat) -> Value {
        to_json(&G::legal_actions(self.state(), seat))
    }

    fn apply_json(&mut self, seat: Seat, action: Value) -> Result<(), LudoError> {
        let action: G::Action = from_json(action)?;
        Ok(self.apply(seat, action)?)
    }

    fn view_json(&self, viewer: Viewer) -> Value {
        to_json(&G::view(self.state(), viewer))
    }

    fn outcome(&self) -> Option<Outcome> {
        G::outcome(self.state())
    }

    fn state_json(&self) -> Value {
        to_json(self.state())
    }

    fn log_json(&self) -> Value {
        to_json(self.log())
    }
}

pub trait AnyGame: Sync {
    fn meta(&self) -> GameMeta;
    /// `config` = `Value::Null` for the game's default options.
    fn new_match(&self, config: Value, players: u8, seed: u64) -> Result<Box<dyn AnyMatch>, LudoError>;
    fn import_log(&self, log: Value) -> Result<Box<dyn AnyMatch>, LudoError>;
    fn ai_move(&self, state: Value, seat: Seat, search: &Search, now_ms: &dyn Fn() -> f64) -> Result<Value, LudoError>;
}

/// Registers game `G`: `static G: GameEntry<MyGame> = GameEntry::new();`.
pub struct GameEntry<G>(PhantomData<fn() -> G>);

impl<G> GameEntry<G> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<G> Default for GameEntry<G> {
    fn default() -> Self {
        Self::new()
    }
}

impl<G: Game + Determinize + Heuristics> AnyGame for GameEntry<G> {
    fn meta(&self) -> GameMeta {
        G::meta()
    }

    fn new_match(&self, config: Value, players: u8, seed: u64) -> Result<Box<dyn AnyMatch>, LudoError> {
        let config = if config.is_null() { G::Config::default() } else { from_json(config)? };
        Ok(Box::new(Match::<G>::new(config, players, seed)?))
    }

    fn import_log(&self, log: Value) -> Result<Box<dyn AnyMatch>, LudoError> {
        let log: MatchLog<G> = from_json(log)?;
        Ok(Box::new(Match::from_log(log)?))
    }

    fn ai_move(&self, state: Value, seat: Seat, search: &Search, now_ms: &dyn Fn() -> f64) -> Result<Value, LudoError> {
        let state: G::State = from_json(state)?;
        Ok(to_json(&ai::choose::<G>(&state, seat, search, now_ms)?))
    }
}

pub fn find<'a>(games: &[&'a dyn AnyGame], id: &str) -> Result<&'a dyn AnyGame, LudoError> {
    games
        .iter()
        .copied()
        .find(|g| g.meta().id == id)
        .ok_or_else(|| LudoError::UnknownGame(id.to_owned()))
}
```

`lib.rs` : ajouter `mod any;` et `pub use any::{find, AnyGame, AnyMatch, GameEntry, LudoError};`.

- [ ] **Step 3: Lancer les tests**

Run: `cargo test -p ludo-core --release`
Expected: tous PASS.

- [ ] **Step 4: fmt, clippy, commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
git add -A && git commit -m "ludo-core: JSON type erasure for the registry

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 6: Yavalath — plateau et règles à 2 joueurs

**Files:**
- Create: `games/yavalath/Cargo.toml`, `games/yavalath/RULES.md`, `games/yavalath/src/lib.rs`, `games/yavalath/src/board.rs`, `games/yavalath/src/view.rs`
- Create: `games/yavalath/tests/rules.rs`
- Modify: `Cargo.toml` (members + dépendance `yavalath`)

**Interfaces:**
- Consumes: `Game`, `Determinize`, `Seat`, `Viewer`, `Illegal`, `Outcome`, `Pending`, `GameMeta` (Tâche 1).
- Produces (tous `pub`) :
  - `board` : `Cell { q: i8, r: i8 }` (`new`, `is_on_board`, `index`), `Color::{White, Black, Red}` (`TURN_ORDER`), `Board` (`get`, `set`, `is_empty`, `is_full`, `empty_cells`, `effect`, `deciding_lines`), `Effect::{Wins, Loses, Neutral}`, `all_cells()`, `CELL_COUNT = 61`, `RADIUS = 4`, `DIRECTIONS`.
  - `Yavalath` (impl `Game`, `Determinize`), `Config {}`, `Action::{Place(Cell), Swap}` (JSON `{"Place":{"q":0,"r":0}}` / `"Swap"`), `State { board, players, seat_colors, current, eliminated, winner, swapped, stones, last_move, lines }` avec `seat_of`, `is_alive`, `alive`, `next_color`, `forced_blocks`, `legal_cells`, `can_swap`.
  - `View { cells: Vec<CellView>, seat_colors, current: Option<Seat>, eliminated, forced: Vec<Cell>, last_move, lines, swapped }` (JSON camelCase), `CellView { q, r, stone: Option<Color> }`.

- [ ] **Step 1: Déclarer le crate**

`Cargo.toml` racine : `members = ["crates/ludo-core", "games/yavalath"]`, et dans `[workspace.dependencies]` : `yavalath = { path = "games/yavalath" }`.

`games/yavalath/Cargo.toml` :
```toml
[package]
name = "yavalath"
description = "Yavalath rules for ludo-rs"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
ludo-core.workspace = true
rand.workspace = true
serde.workspace = true

[dev-dependencies]
serde_json.workspace = true
```

- [ ] **Step 2: Écrire `RULES.md`**

`games/yavalath/RULES.md` :
```markdown
# Yavalath — règles implémentées

Source : **[R]** le livret officiel nestorgames (règles de Cameron Browne / Ludi),
<https://nestorgames.com/rulebooks/YAVALATH_EN.pdf>.
Les points marqués ❓ ne sont pas tranchés par le livret : ce sont nos choix.

## Matériel et mise en place [R]
- Plateau hexagonal de 61 cases (côté 5), vide au départ.
- Couleurs : Blanc, Noir ; Rouge pour le 3ᵉ joueur.

## Tour [R]
- Blanc commence ; on joue ensuite dans l'ordre Blanc, Noir, Rouge.
- À son tour, on pose une pierre de sa couleur sur une case vide.
- À 2 joueurs, au premier coup de Noir, il peut échanger les couleurs au lieu de poser (swap).
  Il prend alors Blanc et la pierre déjà posée ; l'autre joueur devient Noir et joue.

## Fin de partie [R]
- On gagne en alignant 4 pierres ou plus de sa couleur.
- On perd en alignant 3 pierres de sa couleur sans en aligner 4 par le même coup.
  Aligner 3 et 4 d'un même coup est donc une victoire.
- Plateau plein sans vainqueur : partie nulle.

## Variante à 3 joueurs [R]
- On doit bloquer la victoire du joueur suivant si c'est possible.
- Un joueur qui aligne 3 sans 4 est éliminé ; ses pierres restent sur le plateau.
- Gagne le dernier survivant, ou le premier qui aligne 4.

## Choix sur les points non tranchés
1. ❓ Pas de swap à 3 joueurs (le livret ne le décrit que pour Noir à 2).
2. ❓ Une victoire immédiate prime sur le blocage forcé : un coup qui aligne 4 reste permis même s'il ne bloque pas.
3. ❓ Si le joueur suivant a plusieurs cases gagnantes, on doit jouer sur l'une d'elles ; s'il n'en a aucune, pas de contrainte.
4. ❓ « Joueur suivant » : le prochain joueur non éliminé dans l'ordre du tour.
5. ❓ Après une élimination à 3, les deux restants continuent sans swap, blocage forcé compris.
6. ❓ Classement : le vainqueur 1ᵉʳ, puis les autres survivants (ex æquo), puis les éliminés, le dernier éliminé devant.
   En cas de nul, les survivants sont ex æquo premiers.
```

- [ ] **Step 3: Écrire les tests (qui échouent)**

`games/yavalath/tests/rules.rs` :
```rust
use ludo_core::{Game, Outcome, Seat, Viewer};
use yavalath::{all_cells, Action, Cell, Color, Config, State, Yavalath, CELL_COUNT};

fn new(players: u8) -> State {
    Yavalath::new(&Config::default(), players, 0)
}

fn place(s: &mut State, q: i8, r: i8) {
    let seat = Yavalath::pending(s).0[0];
    Yavalath::apply(s, seat, &Action::Place(Cell::new(q, r))).unwrap_or_else(|e| panic!("({q},{r}) : {e}"));
}

fn play(s: &mut State, moves: &[(i8, i8)]) {
    for &(q, r) in moves {
        place(s, q, r);
    }
}

fn ranks(s: &State) -> Option<Vec<u8>> {
    Yavalath::outcome(s).map(|Outcome::Ranked(r)| r)
}

#[test]
fn the_board_has_61_cells_indexed_densely() {
    let indices: Vec<usize> = all_cells().map(|c| c.index().unwrap()).collect();
    assert_eq!(indices, (0..CELL_COUNT).collect::<Vec<_>>());
    assert!(Cell::new(5, 0).index().is_none());
    assert!(Cell::new(3, 2).index().is_none());
}

#[test]
fn four_in_a_row_wins() {
    let mut s = new(2);
    play(&mut s, &[(0, 0), (0, -4), (1, 0), (4, -4), (3, 0), (2, -4), (2, 0)]);
    assert_eq!(ranks(&s), Some(vec![0, 1]));
    assert_eq!(s.lines.len(), 1);
    assert_eq!(s.lines[0].len(), 4);
}

#[test]
fn three_in_a_row_loses() {
    let mut s = new(2);
    play(&mut s, &[(0, 0), (0, -4), (1, 0), (4, -4), (2, 0)]);
    assert_eq!(ranks(&s), Some(vec![1, 0]));
    assert_eq!(s.eliminated, vec![Seat(0)]);
    assert_eq!(s.winner, Some(Seat(1)));
}

#[test]
fn three_and_four_at_once_wins() {
    let mut s = new(2);
    play(
        &mut s,
        &[(0, 0), (-4, 4), (1, 0), (-2, 4), (3, 0), (0, 4), (2, -1), (-4, 2), (2, 1), (-2, 2), (2, 0)],
    );
    assert_eq!(ranks(&s), Some(vec![0, 1]));
    assert_eq!(s.lines.len(), 1, "only the 4 is highlighted");
    assert_eq!(s.lines[0].len(), 4);
}

#[test]
fn an_occupied_cell_is_refused_and_nothing_changes() {
    let mut s = new(2);
    place(&mut s, 0, 0);
    let before = s.clone();
    assert!(Yavalath::apply(&mut s, Seat(1), &Action::Place(Cell::new(0, 0))).is_err());
    assert!(Yavalath::apply(&mut s, Seat(1), &Action::Place(Cell::new(9, 0))).is_err());
    assert_eq!(s, before);
}

#[test]
fn playing_out_of_turn_is_refused() {
    let mut s = new(2);
    assert!(Yavalath::apply(&mut s, Seat(1), &Action::Place(Cell::new(0, 0))).is_err());
}

#[test]
fn black_may_swap_on_its_first_move_only() {
    let mut s = new(2);
    place(&mut s, 0, 0);
    assert!(Yavalath::legal_actions(&s, Seat(1)).contains(&Action::Swap));
    Yavalath::apply(&mut s, Seat(1), &Action::Swap).unwrap();
    assert_eq!(s.seat_colors, vec![Color::Black, Color::White]);
    assert_eq!(Yavalath::pending(&s).0, vec![Seat(0)]);
    assert_eq!(s.board.get(Cell::new(0, 0)), Some(Color::White));
    place(&mut s, 1, 1);
    assert_eq!(s.board.get(Cell::new(1, 1)), Some(Color::Black));
    assert!(!Yavalath::legal_actions(&s, Seat(1)).contains(&Action::Swap));
}

#[test]
fn no_swap_with_three_players() {
    let mut s = new(3);
    place(&mut s, 0, 0);
    assert!(!Yavalath::legal_actions(&s, Seat(1)).contains(&Action::Swap));
}

/// Two colours with no three in a row in any direction.
fn pattern(c: Cell) -> Color {
    if (c.q + 2 * c.r).rem_euclid(4) < 2 {
        Color::White
    } else {
        Color::Black
    }
}

#[test]
fn a_full_board_is_a_draw() {
    let mut s = new(2);
    let last = Cell::new(0, 0);
    for c in all_cells().filter(|&c| c != last) {
        s.board.set(c, Some(pattern(c)));
    }
    s.stones = 60;
    s.current = Some(pattern(last));
    place(&mut s, 0, 0);
    assert_eq!(ranks(&s), Some(vec![0, 0]));
}

#[test]
fn the_view_shows_the_board_and_whose_turn_it_is() {
    let mut s = new(2);
    place(&mut s, 0, 0);
    let v = Yavalath::view(&s, Viewer::Spectator);
    assert_eq!(v.cells.len(), CELL_COUNT);
    assert_eq!(v.current, Some(Seat(1)));
    assert_eq!(v.last_move, Some(Cell::new(0, 0)));
    let json = serde_json::to_value(&v).unwrap();
    assert!(json.get("seatColors").is_some() && json.get("lastMove").is_some());
    assert_eq!(serde_json::to_value(Action::Place(Cell::new(1, -1))).unwrap(), serde_json::json!({ "Place": { "q": 1, "r": -1 } }));
    assert_eq!(serde_json::to_value(Action::Swap).unwrap(), serde_json::json!("Swap"));
}
```

Run: `cargo test -p yavalath` → Expected: FAIL à la compilation (crate vide).

- [ ] **Step 4: Écrire `board.rs`**

```rust
//! Hexagonal board of side 5 (61 cells) in axial coordinates.

use serde::{Deserialize, Serialize};

pub const RADIUS: i8 = 4;
pub const CELL_COUNT: usize = 61;
/// Index of the first cell of each row r = -4..=4 (row lengths 5,6,7,8,9,8,7,6,5).
const ROW_START: [usize; 9] = [0, 5, 11, 18, 26, 35, 43, 50, 56];
/// The three line directions; each line is walked both ways.
pub const DIRECTIONS: [(i8, i8); 3] = [(1, 0), (0, 1), (1, -1)];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Cell {
    pub q: i8,
    pub r: i8,
}

impl Cell {
    pub const fn new(q: i8, r: i8) -> Self {
        Self { q, r }
    }

    pub fn is_on_board(self) -> bool {
        self.q.abs() <= RADIUS && self.r.abs() <= RADIUS && (self.q + self.r).abs() <= RADIUS
    }

    /// Position in `all_cells()` order, or `None` off the board.
    pub fn index(self) -> Option<usize> {
        if !self.is_on_board() {
            return None;
        }
        let q_min = (-RADIUS).max(-RADIUS - self.r);
        Some(ROW_START[(self.r + RADIUS) as usize] + (self.q - q_min) as usize)
    }

    fn step(self, (dq, dr): (i8, i8), k: i8) -> Cell {
        Cell::new(self.q + dq * k, self.r + dr * k)
    }
}

/// Every cell, row by row (r ascending, then q ascending).
pub fn all_cells() -> impl Iterator<Item = Cell> {
    (-RADIUS..=RADIUS).flat_map(|r| {
        let q_min = (-RADIUS).max(-RADIUS - r);
        let q_max = RADIUS.min(RADIUS - r);
        (q_min..=q_max).map(move |q| Cell::new(q, r))
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Color {
    White,
    Black,
    Red,
}

impl Color {
    /// Turn order [R]: White starts, then Black, then Red.
    pub const TURN_ORDER: [Color; 3] = [Color::White, Color::Black, Color::Red];
}

/// What placing a stone does [R]: 4 or more in a row wins; exactly 3
/// without a 4 loses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    Wins,
    Loses,
    Neutral,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Board {
    stones: Vec<Option<Color>>,
}

impl Default for Board {
    fn default() -> Self {
        Self { stones: vec![None; CELL_COUNT] }
    }
}

impl Board {
    /// `None` for an empty cell or a cell off the board.
    pub fn get(&self, cell: Cell) -> Option<Color> {
        cell.index().and_then(|i| self.stones[i])
    }

    pub fn set(&mut self, cell: Cell, stone: Option<Color>) {
        let i = cell.index().expect("cell on the board");
        self.stones[i] = stone;
    }

    pub fn is_empty(&self, cell: Cell) -> bool {
        cell.is_on_board() && self.get(cell).is_none()
    }

    pub fn is_full(&self) -> bool {
        self.stones.iter().all(Option::is_some)
    }

    pub fn empty_cells(&self) -> impl Iterator<Item = Cell> + '_ {
        all_cells().filter(|&c| self.get(c).is_none())
    }

    /// Length of the run of `color` through `cell` along `dir`, counting `cell` as `color`.
    fn run_len(&self, cell: Cell, color: Color, dir: (i8, i8)) -> usize {
        let mut n = 1;
        for sign in [-1i8, 1] {
            let mut k = 1;
            while self.get(cell.step(dir, sign * k)) == Some(color) {
                n += 1;
                k += 1;
            }
        }
        n
    }

    fn run(&self, cell: Cell, color: Color, dir: (i8, i8)) -> Vec<Cell> {
        let mut back = 1;
        while self.get(cell.step(dir, -back)) == Some(color) {
            back += 1;
        }
        let mut forward = 1;
        while self.get(cell.step(dir, forward)) == Some(color) {
            forward += 1;
        }
        (1 - back..forward).map(|k| cell.step(dir, k)).collect()
    }

    /// What placing `color` on `cell` would do.
    pub fn effect(&self, cell: Cell, color: Color) -> Effect {
        let mut three = false;
        for dir in DIRECTIONS {
            match self.run_len(cell, color, dir) {
                n if n >= 4 => return Effect::Wins,
                3 => three = true,
                _ => {}
            }
        }
        if three {
            Effect::Loses
        } else {
            Effect::Neutral
        }
    }

    /// The runs through `cell` (counted as `color`) that decide the placement:
    /// the runs of 4 or more if any, otherwise the runs of exactly 3.
    pub fn deciding_lines(&self, cell: Cell, color: Color) -> Vec<Vec<Cell>> {
        let runs: Vec<Vec<Cell>> = DIRECTIONS.iter().map(|&d| self.run(cell, color, d)).collect();
        if runs.iter().any(|r| r.len() >= 4) {
            runs.into_iter().filter(|r| r.len() >= 4).collect()
        } else {
            runs.into_iter().filter(|r| r.len() == 3).collect()
        }
    }
}
```

- [ ] **Step 5: Écrire `view.rs`**

```rust
use ludo_core::Seat;
use serde::Serialize;

use crate::{all_cells, Cell, Color, State};

/// Everything is public in Yavalath: every viewer gets the same view.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub cells: Vec<CellView>,
    pub seat_colors: Vec<Color>,
    /// Seat to move; `None` once the game is over.
    pub current: Option<Seat>,
    pub eliminated: Vec<Seat>,
    /// Cells the player to move must choose from to block the next player (3 players).
    pub forced: Vec<Cell>,
    pub last_move: Option<Cell>,
    /// Lines to highlight: 3s that eliminated someone, the 4 that won.
    pub lines: Vec<Vec<Cell>>,
    pub swapped: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CellView {
    pub q: i8,
    pub r: i8,
    pub stone: Option<Color>,
}

impl View {
    pub fn of(s: &State) -> Self {
        Self {
            cells: all_cells().map(|c| CellView { q: c.q, r: c.r, stone: s.board.get(c) }).collect(),
            seat_colors: s.seat_colors.clone(),
            current: s.current.map(|c| s.seat_of(c)),
            eliminated: s.eliminated.clone(),
            forced: s.forced_blocks(),
            last_move: s.last_move,
            lines: s.lines.clone(),
            swapped: s.swapped,
        }
    }
}
```

- [ ] **Step 6: Écrire `lib.rs`**

```rust
//! Yavalath (Cameron Browne / Ludi, 2007). Rules and their sources: RULES.md.

mod board;
mod view;

pub use board::{all_cells, Board, Cell, Color, Effect, CELL_COUNT, DIRECTIONS, RADIUS};
pub use view::{CellView, View};

use ludo_core::{Determinize, Game, GameMeta, Illegal, Outcome, Pending, Seat, Viewer};
use rand::Rng;
use serde::{Deserialize, Serialize};

pub struct Yavalath;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    Place(Cell),
    /// [R] Black's first move, 2 players only: take White's stone and colour.
    Swap,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub board: Board,
    pub players: u8,
    /// Colour of each seat (index = seat). A swap exchanges seats 0 and 1.
    pub seat_colors: Vec<Color>,
    /// Colour to move; `None` once the game is over.
    pub current: Option<Color>,
    /// Eliminated seats, in elimination order.
    pub eliminated: Vec<Seat>,
    pub winner: Option<Seat>,
    pub swapped: bool,
    pub stones: u16,
    pub last_move: Option<Cell>,
    pub lines: Vec<Vec<Cell>>,
}

impl State {
    pub fn seat_of(&self, color: Color) -> Seat {
        let i = self.seat_colors.iter().position(|&c| c == color).expect("colour in play");
        Seat(i as u8)
    }

    pub fn is_alive(&self, seat: Seat) -> bool {
        !self.eliminated.contains(&seat)
    }

    pub fn alive(&self) -> Vec<Seat> {
        (0..self.players).map(Seat).filter(|&s| self.is_alive(s)).collect()
    }

    /// ❓4: the next colour in turn order whose seat is still in the game.
    pub fn next_color(&self, color: Color) -> Color {
        let in_play = &Color::TURN_ORDER[..usize::from(self.players)];
        let start = in_play.iter().position(|&c| c == color).expect("colour in play");
        (1..=in_play.len())
            .map(|k| in_play[(start + k) % in_play.len()])
            .find(|&c| self.is_alive(self.seat_of(c)))
            .unwrap_or(color)
    }

    /// [R] 3 players: cells where the next player would win at once, which
    /// the player to move must block. Empty when the rule does not apply.
    pub fn forced_blocks(&self) -> Vec<Cell> {
        let Some(me) = self.current else { return Vec::new() };
        if self.players != 3 {
            return Vec::new();
        }
        let next = self.next_color(me);
        if next == me {
            return Vec::new();
        }
        self.board.empty_cells().filter(|&c| self.board.effect(c, next) == Effect::Wins).collect()
    }

    /// Cells the colour to move may play.
    pub fn legal_cells(&self) -> Vec<Cell> {
        let Some(me) = self.current else { return Vec::new() };
        let forced = self.forced_blocks();
        let empty = self.board.empty_cells();
        if forced.is_empty() {
            return empty.collect();
        }
        // ❓2: an immediate win is allowed even when it does not block.
        empty.filter(|&c| forced.contains(&c) || self.board.effect(c, me) == Effect::Wins).collect()
    }

    /// [R] + ❓1: Black's first move, 2 players only.
    pub fn can_swap(&self) -> bool {
        self.players == 2 && self.stones == 1 && !self.swapped && self.current == Some(Color::Black)
    }

    fn place(&mut self, seat: Seat, color: Color, cell: Cell) {
        let effect = self.board.effect(cell, color);
        if effect != Effect::Neutral {
            self.lines.extend(self.board.deciding_lines(cell, color));
        }
        self.board.set(cell, Some(color));
        self.stones += 1;
        self.last_move = Some(cell);
        match effect {
            // [R] 4 or more in a row wins, even if the same move makes a 3.
            Effect::Wins => {
                self.winner = Some(seat);
                self.current = None;
                return;
            }
            // [R] 3 without 4: eliminated, the stones stay on the board.
            Effect::Loses => self.eliminated.push(seat),
            Effect::Neutral => {}
        }
        let alive = self.alive();
        if alive.len() == 1 {
            // [R] the last surviving player wins.
            self.winner = Some(alive[0]);
            self.current = None;
        } else if self.board.is_full() {
            // [R] full board: draw.
            self.current = None;
        } else {
            self.current = Some(self.next_color(color));
        }
    }
}

impl Game for Yavalath {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta { id: "yavalath", name: "Yavalath", min_players: 2, max_players: 3, hidden_info: false }
    }

    fn new(_config: &Config, players: u8, _seed: u64) -> State {
        State {
            board: Board::default(),
            players,
            seat_colors: Color::TURN_ORDER[..usize::from(players)].to_vec(),
            current: Some(Color::White),
            eliminated: Vec::new(),
            winner: None,
            swapped: false,
            stones: 0,
            last_move: None,
            lines: Vec::new(),
        }
    }

    fn pending(s: &State) -> Pending {
        match s.current {
            Some(color) => Pending::one(s.seat_of(color)),
            None => Pending::none(),
        }
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        let Some(color) = s.current else { return Vec::new() };
        if s.seat_of(color) != seat {
            return Vec::new();
        }
        let mut actions: Vec<Action> = s.legal_cells().into_iter().map(Action::Place).collect();
        if s.can_swap() {
            actions.push(Action::Swap);
        }
        actions
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        let Some(color) = s.current else { return Err(Illegal::new("La partie est terminée.")) };
        if s.seat_of(color) != seat {
            return Err(Illegal::new("Ce n'est pas ton tour."));
        }
        match *action {
            Action::Swap => {
                if !s.can_swap() {
                    return Err(Illegal::new("L'échange n'est possible qu'au premier coup de Noir, à 2 joueurs."));
                }
                // The swapper takes White and its stone; the other seat becomes Black and moves.
                s.seat_colors.swap(0, 1);
                s.swapped = true;
            }
            Action::Place(cell) => {
                if !s.board.is_empty(cell) {
                    return Err(Illegal::new("Cette case est occupée ou hors du plateau."));
                }
                if !s.legal_cells().contains(&cell) {
                    return Err(Illegal::new("Tu dois bloquer la victoire du joueur suivant."));
                }
                s.place(seat, color, cell);
            }
        }
        Ok(())
    }

    fn view(s: &State, _viewer: Viewer) -> View {
        View::of(s)
    }

    fn outcome(s: &State) -> Option<Outcome> {
        if s.current.is_some() {
            return None;
        }
        let mut ranks = vec![0u8; usize::from(s.players)];
        let alive = s.alive();
        // ❓6: winner first, other survivors tied next, then the eliminated, last eliminated first.
        let mut next = match s.winner {
            Some(winner) => {
                let others: Vec<Seat> = alive.into_iter().filter(|&x| x != winner).collect();
                for seat in &others {
                    ranks[seat.index()] = 1;
                }
                1 + others.len() as u8
            }
            None => alive.len() as u8,
        };
        for seat in s.eliminated.iter().rev() {
            ranks[seat.index()] = next;
            next += 1;
        }
        Some(Outcome::Ranked(ranks))
    }
}

impl Determinize for Yavalath {
    /// Perfect information: nothing to redraw.
    fn determinize<R: Rng + ?Sized>(s: &State, _observer: Seat, _rng: &mut R) -> State {
        s.clone()
    }
}
```

- [ ] **Step 7: Lancer les tests**

Run: `cargo test -p yavalath`
Expected: 10 tests PASS.

- [ ] **Step 8: fmt, clippy, commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
git add -A && git commit -m "yavalath: board and two-player rules

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 7: Yavalath — variante à 3 joueurs

**Files:**
- Create: `games/yavalath/tests/three_players.rs`
- Modify: `games/yavalath/src/lib.rs` seulement si un test échoue (les règles à 3 sont déjà écrites à la Tâche 6 ; cette tâche les prouve).

**Interfaces:**
- Consumes: `State::{forced_blocks, legal_cells, next_color}`, `View::forced` (Tâche 6).

- [ ] **Step 1: Écrire les tests**

`games/yavalath/tests/three_players.rs` :
```rust
use ludo_core::{Game, Outcome, Seat, Viewer};
use yavalath::{Action, Cell, Color, Config, State, Yavalath};

fn three() -> State {
    Yavalath::new(&Config::default(), 3, 0)
}

fn put(s: &mut State, color: Color, cells: &[(i8, i8)]) {
    for &(q, r) in cells {
        s.board.set(Cell::new(q, r), Some(color));
        s.stones += 1;
    }
}

fn places(s: &State, seat: Seat) -> Vec<Cell> {
    let mut cells: Vec<Cell> = Yavalath::legal_actions(s, seat)
        .into_iter()
        .filter_map(|a| match a {
            Action::Place(c) => Some(c),
            Action::Swap => None,
        })
        .collect();
    cells.sort();
    cells
}

fn ranks(s: &State) -> Option<Vec<u8>> {
    Yavalath::outcome(s).map(|Outcome::Ranked(r)| r)
}

#[test]
fn the_next_players_win_must_be_blocked() {
    let mut s = three();
    put(&mut s, Color::Black, &[(0, 0), (1, 0), (3, 0)]);
    assert_eq!(places(&s, Seat(0)), vec![Cell::new(2, 0)]);
    assert_eq!(Yavalath::view(&s, Viewer::Spectator).forced, vec![Cell::new(2, 0)]);
    assert!(Yavalath::apply(&mut s, Seat(0), &Action::Place(Cell::new(-2, 2))).is_err());
}

#[test]
fn a_forced_block_can_eliminate_the_blocker() {
    let mut s = three();
    put(&mut s, Color::Black, &[(0, 0), (1, 0), (3, 0)]);
    put(&mut s, Color::White, &[(2, -1), (2, 1)]);
    Yavalath::apply(&mut s, Seat(0), &Action::Place(Cell::new(2, 0))).unwrap();
    assert_eq!(s.eliminated, vec![Seat(0)]);
    assert_eq!(s.board.get(Cell::new(2, 0)), Some(Color::White), "stones stay");
    assert_eq!(Yavalath::pending(&s).0, vec![Seat(1)]);
    assert_eq!(ranks(&s), None, "two players remain");
}

#[test]
fn an_immediate_win_beats_the_forced_block() {
    let mut s = three();
    put(&mut s, Color::Black, &[(0, 0), (1, 0), (3, 0)]);
    put(&mut s, Color::White, &[(-3, 1), (-2, 1), (0, 1)]);
    assert_eq!(places(&s, Seat(0)), vec![Cell::new(-1, 1), Cell::new(2, 0)]);
}

#[test]
fn the_next_player_skips_the_eliminated() {
    let mut s = three();
    s.eliminated.push(Seat(1));
    assert_eq!(s.next_color(Color::White), Color::Red);
    put(&mut s, Color::Red, &[(0, 0), (1, 0), (3, 0)]);
    assert_eq!(places(&s, Seat(0)), vec![Cell::new(2, 0)]);
}

#[test]
fn the_last_survivor_wins_and_the_eliminated_rank_in_reverse_order() {
    let mut s = three();
    s.eliminated.push(Seat(2));
    put(&mut s, Color::White, &[(0, 0), (1, 0)]);
    Yavalath::apply(&mut s, Seat(0), &Action::Place(Cell::new(2, 0))).unwrap();
    assert_eq!(s.winner, Some(Seat(1)));
    assert_eq!(ranks(&s), Some(vec![1, 0, 2]));
}

#[test]
fn a_four_ranks_the_other_survivor_second() {
    let mut s = three();
    s.eliminated.push(Seat(2));
    put(&mut s, Color::White, &[(0, 0), (1, 0), (3, 0)]);
    Yavalath::apply(&mut s, Seat(0), &Action::Place(Cell::new(2, 0))).unwrap();
    assert_eq!(ranks(&s), Some(vec![0, 1, 2]));
}
```

- [ ] **Step 2: Lancer les tests**

Run: `cargo test -p yavalath --test three_players`
Expected: 6 tests PASS. Si un test échoue, corriger `games/yavalath/src/lib.rs` (pas le test) en suivant `RULES.md`.

- [ ] **Step 3: fmt, clippy, commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
git add -A && git commit -m "yavalath: three-player variant tests

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 8: Yavalath — heuristiques, conformance et registre

**Files:**
- Create: `games/yavalath/src/heuristics.rs`, `games/yavalath/tests/conformance.rs`
- Create: `crates/ludo-registry/Cargo.toml`, `crates/ludo-registry/src/lib.rs`
- Modify: `games/yavalath/src/lib.rs` (`mod heuristics;`), `Cargo.toml` (members + dépendance `ludo-registry`)

**Interfaces:**
- Consumes: `Heuristics` (Tâche 1), `conformance::check` (Tâche 3), `GameEntry`, `AnyGame`, `find`, `LudoError` (Tâche 5).
- Produces: `impl Heuristics for Yavalath` ; `ludo_registry::GAMES: &[&dyn AnyGame]`, `ludo_registry::find(id) -> Result<&'static dyn AnyGame, LudoError>`.

- [ ] **Step 1: Écrire les tests (qui échouent)**

À la fin de `games/yavalath/src/heuristics.rs` :
```rust
#[cfg(test)]
mod tests {
    use ludo_core::{Game, Heuristics, Seat};
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    use crate::{Action, Cell, Color, Config, State, Yavalath};

    fn with(stones: &[(Color, i8, i8)]) -> State {
        let mut s = Yavalath::new(&Config::default(), 2, 0);
        for &(color, q, r) in stones {
            s.board.set(Cell::new(q, r), Some(color));
        }
        s.stones = 2;
        s
    }

    #[test]
    fn takes_an_immediate_win() {
        let s = with(&[(Color::White, 0, 0), (Color::White, 1, 0), (Color::White, 3, 0)]);
        for seed in 0..20 {
            let a = Yavalath::rollout_action(&s, Seat(0), &mut SmallRng::seed_from_u64(seed));
            assert_eq!(a, Action::Place(Cell::new(2, 0)));
        }
    }

    #[test]
    fn blocks_the_next_players_win() {
        let s = with(&[(Color::Black, 0, 0), (Color::Black, 1, 0), (Color::Black, 3, 0)]);
        for seed in 0..20 {
            let a = Yavalath::rollout_action(&s, Seat(0), &mut SmallRng::seed_from_u64(seed));
            assert_eq!(a, Action::Place(Cell::new(2, 0)));
        }
    }

    #[test]
    fn avoids_making_three() {
        let s = with(&[(Color::White, 0, 0), (Color::White, 1, 0)]);
        for seed in 0..50 {
            let a = Yavalath::rollout_action(&s, Seat(0), &mut SmallRng::seed_from_u64(seed));
            assert!(a != Action::Place(Cell::new(2, 0)) && a != Action::Place(Cell::new(-1, 0)), "{a:?}");
        }
    }
}
```

`games/yavalath/tests/conformance.rs` :
```rust
use ludo_core::conformance::check;
use yavalath::{Config, Yavalath};

#[test]
fn two_players_conform() {
    check::<Yavalath>(&Config::default(), 2, 300, 7);
}

#[test]
fn three_players_conform() {
    check::<Yavalath>(&Config::default(), 3, 200, 8);
}
```

- [ ] **Step 2: Écrire `heuristics.rs`**

Au-dessus du module de tests :
```rust
use ludo_core::{Heuristics, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use crate::{Action, Cell, Effect, State, Yavalath};

impl Heuristics for Yavalath {
    /// Win if possible; otherwise block the next player's win with a safe
    /// move; otherwise any move that does not make 3; otherwise anything.
    /// Never swaps: swapping is left to the tree search.
    fn rollout_action<R: Rng + ?Sized>(s: &State, _seat: Seat, rng: &mut R) -> Action {
        let me = s.current.expect("rollout on a finished game");
        let cells = s.legal_cells();
        if let Some(&win) = cells.iter().find(|&&c| s.board.effect(c, me) == Effect::Wins) {
            return Action::Place(win);
        }
        let safe: Vec<Cell> = cells.iter().copied().filter(|&c| s.board.effect(c, me) != Effect::Loses).collect();
        let next = s.next_color(me);
        let blocks: Vec<Cell> = if next == me {
            Vec::new()
        } else {
            safe.iter().copied().filter(|&c| s.board.effect(c, next) == Effect::Wins).collect()
        };
        let pool = if !blocks.is_empty() {
            &blocks
        } else if !safe.is_empty() {
            &safe
        } else {
            &cells
        };
        Action::Place(*pool.choose(rng).expect("an unfinished game has an empty cell"))
    }
}
```

`games/yavalath/src/lib.rs` : ajouter `mod heuristics;` sous `mod board;`.

- [ ] **Step 3: Créer le registre**

`Cargo.toml` racine : `members = ["crates/ludo-core", "crates/ludo-registry", "games/yavalath"]` et `ludo-registry = { path = "crates/ludo-registry" }` dans `[workspace.dependencies]`.

`crates/ludo-registry/Cargo.toml` :
```toml
[package]
name = "ludo-registry"
description = "The list of games the site and the CLI offer"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
ludo-core.workspace = true
yavalath.workspace = true
```

`crates/ludo-registry/src/lib.rs` :
```rust
//! Every game of the platform. Adding a game: one `static` and one entry in `GAMES`.

use ludo_core::{AnyGame, GameEntry, LudoError};

static YAVALATH: GameEntry<yavalath::Yavalath> = GameEntry::new();

pub static GAMES: &[&dyn AnyGame] = &[&YAVALATH];

pub fn find(id: &str) -> Result<&'static dyn AnyGame, LudoError> {
    ludo_core::find(GAMES, id)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use serde_json::Value;

    use super::*;

    #[test]
    fn ids_are_unique_and_every_game_starts() {
        let ids: HashSet<&str> = GAMES.iter().map(|g| g.meta().id).collect();
        assert_eq!(ids.len(), GAMES.len());
        for game in GAMES {
            let meta = game.meta();
            for players in meta.min_players..=meta.max_players {
                let m = game.new_match(Value::Null, players, 1).unwrap();
                assert!(!m.pending().is_empty(), "{} à {players}", meta.id);
            }
        }
    }
}
```
Et `[dev-dependencies] serde_json.workspace = true` dans `crates/ludo-registry/Cargo.toml`.

- [ ] **Step 4: Lancer les tests**

Run: `cargo test --workspace --release`
Expected: tous PASS (conformance Yavalath : 500 parties).

- [ ] **Step 5: fmt, clippy, commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
git add -A && git commit -m "yavalath: rollout heuristics and conformance; game registry

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 9: `ludo-cli` (list, arena, play) et critère de niveau

**Files:**
- Create: `crates/ludo-cli/Cargo.toml`, `crates/ludo-cli/src/main.rs`
- Modify: `Cargo.toml` (members)

**Interfaces:**
- Consumes: `ludo_registry::{GAMES, find}` (Tâche 8), `AnyGame`, `AnyMatch` (Tâche 5), `ai::{Level, Search}` (Tâche 4).

- [ ] **Step 1: Créer le crate**

`Cargo.toml` racine : ajouter `"crates/ludo-cli"` à `members`.

`crates/ludo-cli/Cargo.toml` :
```toml
[package]
name = "ludo-cli"
description = "Terminal tools: list games, AI arena, play against the AI"
version.workspace = true
edition.workspace = true
license.workspace = true

[[bin]]
name = "ludo"
path = "src/main.rs"

[dependencies]
ludo-core.workspace = true
ludo-registry.workspace = true
serde_json.workspace = true
```

- [ ] **Step 2: Écrire `main.rs`**

```rust
//! `ludo list` | `ludo arena <jeu> <parties> <niveau A> <niveau B> [joueurs]` | `ludo play <jeu> <joueurs> <niveau>`

use std::io::{self, BufRead, Write};
use std::str::FromStr;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use ludo_core::ai::{Level, Search};
use ludo_core::{AnyGame, Seat, Viewer};
use serde_json::Value;

const USAGE: &str = "usage :
  ludo list
  ludo arena <jeu> <parties> <easy|medium|hard> <easy|medium|hard> [joueurs]
  ludo play <jeu> <joueurs> <easy|medium|hard>";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(e) = run(&args) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn run(args: &[String]) -> Result<(), String> {
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        ["list"] => {
            for game in ludo_registry::GAMES {
                let m = game.meta();
                println!("{}\t{}\t{}–{} joueurs", m.id, m.name, m.min_players, m.max_players);
            }
            Ok(())
        }
        ["arena", game, games, a, b, rest @ ..] => {
            let players = rest.first().map(|p| parse(p)).transpose()?.unwrap_or(2);
            arena(game, parse(games)?, level(a)?, level(b)?, players)
        }
        ["play", game, players, lvl] => play(game, parse(players)?, level(lvl)?),
        _ => Err(USAGE.into()),
    }
}

fn parse<T: FromStr>(s: &str) -> Result<T, String> {
    s.parse().map_err(|_| format!("valeur invalide : {s}\n{USAGE}"))
}

fn level(s: &str) -> Result<Level, String> {
    match s {
        "easy" => Ok(Level::Easy),
        "medium" => Ok(Level::Medium),
        "hard" => Ok(Level::Hard),
        _ => Err(format!("niveau inconnu : {s}\n{USAGE}")),
    }
}

fn game(id: &str) -> Result<&'static dyn AnyGame, String> {
    ludo_registry::find(id).map_err(|e| e.to_string())
}

/// Level A against level B; A takes each seat in turn. Prints A's mean reward
/// (1 = always first, 0.5 = even).
fn arena(id: &str, games: u32, a: Level, b: Level, players: u8) -> Result<(), String> {
    let game = game(id)?;
    let score = |g: u32| -> Result<f64, String> {
        let a_seat = Seat((g % u32::from(players)) as u8);
        let mut m = game.new_match(Value::Null, players, u64::from(g)).map_err(|e| e.to_string())?;
        while let Some(&seat) = m.pending().first() {
            let lvl = if seat == a_seat { a } else { b };
            let search = Search::counted(lvl, (u64::from(g) << 32) | u64::from(m.rev()));
            let action = game.ai_move(m.state_json(), seat, &search, &|| 0.0).map_err(|e| e.to_string())?;
            m.apply_json(seat, action).map_err(|e| e.to_string())?;
        }
        Ok(m.outcome().expect("finished").rewards()[a_seat.index()])
    };
    let mut total = 0.0;
    for g in 0..games {
        total += score(g)?;
        eprint!("\r{}/{games}  score A = {:.3}", g + 1, total / f64::from(g + 1));
    }
    eprintln!();
    println!("{id} à {players} : {a:?} contre {b:?}, {games} parties, score A = {:.3}", total / f64::from(games));
    Ok(())
}

/// Seat 0 is you (actions chosen by number); every other seat is the AI.
fn play(id: &str, players: u8, lvl: Level) -> Result<(), String> {
    let game = game(id)?;
    let seed = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_nanos() as u64);
    let mut m = game.new_match(Value::Null, players, seed).map_err(|e| e.to_string())?;
    let clock = Instant::now();
    let now = move || clock.elapsed().as_secs_f64() * 1000.0;
    let me = Seat(0);
    let mut input = io::stdin().lock().lines();
    while let Some(&seat) = m.pending().first() {
        if seat == me {
            println!("{}", serde_json::to_string_pretty(&m.view_json(Viewer::Seat(me))).expect("json"));
            let legal = m.legal_actions_json(me).as_array().cloned().unwrap_or_default();
            for (i, action) in legal.iter().enumerate() {
                println!("  {i}: {action}");
            }
            print!("ton coup > ");
            io::stdout().flush().map_err(|e| e.to_string())?;
            let line = input.next().ok_or("entrée fermée")?.map_err(|e| e.to_string())?;
            let Some(action) = line.trim().parse::<usize>().ok().and_then(|i| legal.get(i).cloned()) else {
                println!("numéro invalide");
                continue;
            };
            m.apply_json(me, action).map_err(|e| e.to_string())?;
        } else {
            let search = Search::timed(lvl, seed ^ u64::from(m.rev()));
            let action = game.ai_move(m.state_json(), seat, &search, &now).map_err(|e| e.to_string())?;
            println!("siège {} joue {action}", seat.0);
            m.apply_json(seat, action).map_err(|e| e.to_string())?;
        }
    }
    println!("{}", serde_json::to_string_pretty(&m.view_json(Viewer::Spectator)).expect("json"));
    println!("classement : {:?}", m.outcome());
    Ok(())
}
```

- [ ] **Step 3: Vérifier les commandes**

Run: `cargo run --release -p ludo-cli -- list`
Expected: `yavalath	Yavalath	2–3 joueurs`

Run: `cargo run --release -p ludo-cli -- arena yavalath 10 medium easy`
Expected: termine sans erreur, affiche un score A.

- [ ] **Step 4: Mesurer le critère de réussite (≈ 20–40 min)**

Run: `cargo run --release -p ludo-cli -- arena yavalath 200 hard easy`
Expected: `score A` ≥ 0.800. Si le score est inférieur, ne pas toucher aux niveaux : améliorer `rollout_action` (Tâche 8) ou la constante `EXPLORATION` (Tâche 4), relancer, et noter le score obtenu dans le message de commit.

Run (fumée à 3 joueurs) : `cargo run --release -p ludo-cli -- arena yavalath 20 hard easy 3`
Expected: termine sans erreur.

- [ ] **Step 5: fmt, clippy, commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
git add -A && git commit -m "ludo-cli: list, arena and play (hard vs easy: <score mesuré>)

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 10: Pont WASM et script de build

**Files:**
- Create: `crates/ludo-wasm/Cargo.toml`, `crates/ludo-wasm/src/lib.rs`, `scripts/build-wasm.sh`
- Modify: `Cargo.toml` (members)

**Interfaces:**
- Consumes: `ludo_registry::{GAMES, find}`, `AnyMatch`, `LudoError`, `Outcome`, `ai::{Level, Search}`.
- Produces (module JS `web/src/wasm/ludo.js`, toutes les données en chaînes JSON) :
  - `listGames(): string` → `GameMeta[]`
  - `newMatch(gameId: string, config: string, players: number, seed: number): number` (handle ; `config` = `"null"` pour le défaut)
  - `importLog(gameId: string, log: string): number`
  - `dropMatch(h: number): void`
  - `pending(h): Uint8Array`, `rev(h): number`
  - `legalActions(h, seat): string`, `apply(h, seat, action: string): void` (lève une erreur au message lisible)
  - `view(h, seat?: number | null): string` (absent/null = spectateur)
  - `outcome(h): string` → `number[] | null` (rangs)
  - `exportState(h): string`, `exportLog(h): string`
  - `aiMove(gameId: string, state: string, seat: number, level: "Easy"|"Medium"|"Hard", seed: number): string`

- [ ] **Step 1: Créer le crate**

`Cargo.toml` racine : ajouter `"crates/ludo-wasm"` à `members`.

`crates/ludo-wasm/Cargo.toml` :
```toml
[package]
name = "ludo-wasm"
description = "WASM bridge: thin JSON layer over the game registry"
version.workspace = true
edition.workspace = true
license.workspace = true

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
ludo-core.workspace = true
ludo-registry.workspace = true
serde_json.workspace = true
# Must match the installed wasm-bindgen CLI exactly (`wasm-bindgen --version`).
wasm-bindgen = "=0.2.121"
js-sys = "0.3"
console_error_panic_hook = "0.1"
# rand pulls in getrandom 0.2, which needs its "js" backend to compile on
# wasm32-unknown-unknown (the engine itself only uses seeded generators).
getrandom = { version = "0.2", features = ["js"] }
```

- [ ] **Step 2: Écrire `lib.rs`**

```rust
//! WASM bridge. Zero game logic: every function parses JSON, calls the
//! registry, and returns JSON. Matches live in WASM memory behind handles.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use ludo_core::ai::{Level, Search};
use ludo_core::{AnyMatch, LudoError, Outcome, Seat, Viewer};
use serde_json::Value;
use wasm_bindgen::prelude::*;

thread_local! {
    static MATCHES: RefCell<HashMap<u32, Box<dyn AnyMatch>>> = RefCell::new(HashMap::new());
    static NEXT_HANDLE: Cell<u32> = const { Cell::new(1) };
}

#[wasm_bindgen(start)]
fn start() {
    console_error_panic_hook::set_once();
}

fn err(e: LudoError) -> JsError {
    JsError::new(&e.to_string())
}

fn parse(json: &str) -> Result<Value, JsError> {
    serde_json::from_str(json).map_err(|e| JsError::new(&format!("JSON invalide : {e}")))
}

fn text(value: &impl serde::Serialize) -> String {
    serde_json::to_string(value).expect("engine types serialize to JSON")
}

fn store(m: Box<dyn AnyMatch>) -> u32 {
    let handle = NEXT_HANDLE.get();
    NEXT_HANDLE.set(handle + 1);
    MATCHES.with_borrow_mut(|all| all.insert(handle, m));
    handle
}

fn with_match<R>(handle: u32, f: impl FnOnce(&mut dyn AnyMatch) -> Result<R, JsError>) -> Result<R, JsError> {
    MATCHES.with_borrow_mut(|all| match all.get_mut(&handle) {
        Some(m) => f(m.as_mut()),
        None => Err(JsError::new("Partie inconnue.")),
    })
}

#[wasm_bindgen(js_name = listGames)]
pub fn list_games() -> String {
    text(&ludo_registry::GAMES.iter().map(|g| g.meta()).collect::<Vec<_>>())
}

#[wasm_bindgen(js_name = newMatch)]
pub fn new_match(game_id: &str, config: &str, players: u8, seed: u32) -> Result<u32, JsError> {
    let game = ludo_registry::find(game_id).map_err(err)?;
    Ok(store(game.new_match(parse(config)?, players, seed.into()).map_err(err)?))
}

#[wasm_bindgen(js_name = importLog)]
pub fn import_log(game_id: &str, log: &str) -> Result<u32, JsError> {
    let game = ludo_registry::find(game_id).map_err(err)?;
    Ok(store(game.import_log(parse(log)?).map_err(err)?))
}

#[wasm_bindgen(js_name = dropMatch)]
pub fn drop_match(handle: u32) {
    MATCHES.with_borrow_mut(|all| all.remove(&handle));
}

#[wasm_bindgen]
pub fn pending(handle: u32) -> Result<Vec<u8>, JsError> {
    with_match(handle, |m| Ok(m.pending().into_iter().map(|s| s.0).collect()))
}

#[wasm_bindgen]
pub fn rev(handle: u32) -> Result<u32, JsError> {
    with_match(handle, |m| Ok(m.rev()))
}

#[wasm_bindgen(js_name = legalActions)]
pub fn legal_actions(handle: u32, seat: u8) -> Result<String, JsError> {
    with_match(handle, |m| Ok(text(&m.legal_actions_json(Seat(seat)))))
}

#[wasm_bindgen]
pub fn apply(handle: u32, seat: u8, action: &str) -> Result<(), JsError> {
    let action = parse(action)?;
    with_match(handle, |m| m.apply_json(Seat(seat), action).map_err(err))
}

#[wasm_bindgen]
pub fn view(handle: u32, seat: Option<u8>) -> Result<String, JsError> {
    let viewer = seat.map_or(Viewer::Spectator, |s| Viewer::Seat(Seat(s)));
    with_match(handle, |m| Ok(text(&m.view_json(viewer))))
}

#[wasm_bindgen]
pub fn outcome(handle: u32) -> Result<String, JsError> {
    with_match(handle, |m| Ok(text(&m.outcome().map(|Outcome::Ranked(ranks)| ranks))))
}

#[wasm_bindgen(js_name = exportState)]
pub fn export_state(handle: u32) -> Result<String, JsError> {
    with_match(handle, |m| Ok(text(&m.state_json())))
}

#[wasm_bindgen(js_name = exportLog)]
pub fn export_log(handle: u32) -> Result<String, JsError> {
    with_match(handle, |m| Ok(text(&m.log_json())))
}

/// The AI's action for `seat` in `state` (from `exportState`), within the
/// level's time budget.
#[wasm_bindgen(js_name = aiMove)]
pub fn ai_move(game_id: &str, state: &str, seat: u8, level: &str, seed: u32) -> Result<String, JsError> {
    let level: Level = serde_json::from_value(Value::from(level))
        .map_err(|_| JsError::new(&format!("Niveau inconnu : {level}.")))?;
    let game = ludo_registry::find(game_id).map_err(err)?;
    let search = Search::timed(level, seed.into());
    let action = game.ai_move(parse(state)?, Seat(seat), &search, &js_sys::Date::now).map_err(err)?;
    Ok(text(&action))
}
```

- [ ] **Step 3: Écrire `scripts/build-wasm.sh`**

```bash
#!/usr/bin/env bash
# Builds ludo-wasm and generates the JS glue into web/src/wasm/.
#   ./scripts/build-wasm.sh          optimized build (profile wasm-release + wasm-opt)
#   ./scripts/build-wasm.sh dev      fast build, no optimization
set -euo pipefail
cd "$(dirname "$0")/.."

PROFILE="wasm-release"
[ "${1:-}" = "dev" ] && PROFILE="dev"
OUT="web/src/wasm"
TARGET_DIR="${CARGO_TARGET_DIR:-target}"

cargo build -p ludo-wasm --target wasm32-unknown-unknown --profile "$PROFILE"

# The "dev" profile writes to target/.../debug/.
PROFILE_DIR="$PROFILE"
[ "$PROFILE" = "dev" ] && PROFILE_DIR="debug"

wasm-bindgen --target web --out-dir "$OUT" --out-name ludo \
  "$TARGET_DIR/wasm32-unknown-unknown/$PROFILE_DIR/ludo_wasm.wasm"

if [ "$PROFILE" = "wasm-release" ] && command -v wasm-opt >/dev/null; then
  wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int \
    --enable-sign-ext --enable-mutable-globals --enable-reference-types --enable-multivalue \
    "$OUT/ludo_bg.wasm" -o "$OUT/ludo_bg.wasm"
fi

ls -lh "$OUT"/ludo_bg.wasm
```

Run: `chmod +x scripts/build-wasm.sh`

- [ ] **Step 4: Construire**

Run: `./scripts/build-wasm.sh`
Expected: se termine par la taille de `web/src/wasm/ludo_bg.wasm` ; `web/src/wasm/ludo.d.ts` contient `listGames`, `newMatch`, `aiMove`.

Run: `grep -E "export function (listGames|newMatch|importLog|apply|view|aiMove)" web/src/wasm/ludo.d.ts | wc -l`
Expected: `6`

- [ ] **Step 5: Vérifier que le cœur reste natif**

Run: `cargo tree -p ludo-core -p yavalath -e normal | grep -E "wasm-bindgen|js-sys|web-sys" || echo OK`
Expected: `OK`

- [ ] **Step 6: fmt, clippy, commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
git add -A && git commit -m "ludo-wasm: JSON bridge over the registry; build script

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 11: Squelette web, Workers, clients, registre UI et plateau Yavalath

**Files:**
- Create: `web/package.json`, `web/tsconfig.json`, `web/vite.config.ts`, `web/index.html`, `web/src/main.tsx`, `web/src/vite-env.d.ts`, `web/src/styles.css`, `web/src/App.tsx`
- Create: `web/src/engine/types.ts`, `web/src/engine/rpc.ts`, `web/src/engine/engine.worker.ts`, `web/src/engine/ai.worker.ts`, `web/src/engine/clients.ts`
- Create: `web/src/games/registry.ts`, `web/src/games/yavalath/types.ts`, `web/src/games/yavalath/Board.tsx`
- Create: `web/src/shell/Home.tsx`

**Interfaces:**
- Consumes: le module `web/src/wasm/ludo.js` (Tâche 10).
- Produces:
  - `engine/types.ts` : `Seat = number`, `Level = "Easy" | "Medium" | "Hard"`, `GameMeta { id, name, minPlayers, maxPlayers, hiddenInfo }`, `SeatView { viewer: Seat | null; view; legalActions }`, `Snapshot { rev, pending, outcome: number[] | null, views: SeatView[] }`, `Update { rev, view, legalActions, pending, outcome }`, `EngineRequest`, `AiRequest`, `updateFor(s: Snapshot, viewer: Seat | null): Update`.
  - `engine/clients.ts` : `EngineClient` (`listGames()`, `newMatch(gameId, players, seed, config?)`, `importLog(gameId, log)`, `apply(seat, action)`, `snapshot(viewers)`, `exportState()`, `exportLog()`, `dispose()`), `AiClient` (`move(req: AiRequest)`, `dispose()`), `randomSeed()`, `loadCatalog()`.
  - `games/registry.ts` : `BoardProps { view; legalActions; pending; me; seatNames; submit(action) }`, `GameUi { id; Board }`, `GAME_UIS`, `gameUi(id)`.

- [ ] **Step 1: Configuration du projet**

`web/package.json` :
```json
{
  "name": "ludo-web",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "wasm": "../scripts/build-wasm.sh",
    "wasm:dev": "../scripts/build-wasm.sh dev",
    "dev": "vite",
    "build": "tsc --noEmit && vite build",
    "preview": "vite preview",
    "type-check": "tsc --noEmit",
    "test": "vitest run"
  },
  "dependencies": {
    "peerjs": "^1.5.5",
    "react": "^19.1.0",
    "react-dom": "^19.1.0"
  },
  "devDependencies": {
    "@types/react": "^19.1.0",
    "@types/react-dom": "^19.1.0",
    "@vitejs/plugin-react": "^5.0.0",
    "typescript": "^5.9.0",
    "vite": "^7.1.0",
    "vitest": "^3.2.0"
  }
}
```

`web/tsconfig.json` :
```json
{
  "compilerOptions": {
    "target": "ES2022",
    "lib": ["ES2022", "DOM", "DOM.Iterable", "WebWorker"],
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "skipLibCheck": true,
    "isolatedModules": true,
    "allowJs": false,
    "noEmit": true
  },
  "include": ["src", "vite.config.ts"]
}
```

`web/vite.config.ts` :
```ts
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  // Relative paths: the build works from any folder (GitHub Pages project site).
  base: "./",
  plugins: [react()],
  worker: { format: "es" },
});
```

`web/index.html` :
```html
<!doctype html>
<html lang="fr">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>ludo.rs</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
```

`web/src/vite-env.d.ts` :
```ts
/// <reference types="vite/client" />
```

`web/src/main.tsx` (pas de `StrictMode` : il créerait deux fois les sessions réseau) :
```tsx
import { createRoot } from "react-dom/client";
import { App } from "./App";
import "./styles.css";

createRoot(document.getElementById("root")!).render(<App />);
```

Run: `cd ~/prog/ludo-rs/web && pnpm install`
Expected: crée `pnpm-lock.yaml` sans erreur.

- [ ] **Step 2: Couche moteur**

`web/src/engine/types.ts` :
```ts
// Mirrors the JSON of crates/ludo-core and crates/ludo-wasm.

export type Seat = number;
export type Level = "Easy" | "Medium" | "Hard";

export interface GameMeta {
  id: string;
  name: string;
  minPlayers: number;
  maxPlayers: number;
  hiddenInfo: boolean;
}

/** One viewer's share of a snapshot (`viewer: null` = spectator). */
export interface SeatView {
  viewer: Seat | null;
  view: unknown;
  legalActions: unknown[];
}

/** The game after an action, for every viewer at once. */
export interface Snapshot {
  rev: number;
  pending: Seat[];
  /** Rank of each seat (0 = first), once the game is over. */
  outcome: number[] | null;
  views: SeatView[];
}

/** What one seat receives. */
export interface Update {
  rev: number;
  view: unknown;
  legalActions: unknown[];
  pending: Seat[];
  outcome: number[] | null;
}

export type EngineRequest =
  | { kind: "listGames" }
  | { kind: "newMatch"; gameId: string; players: number; seed: number; config: unknown }
  | { kind: "importLog"; gameId: string; log: unknown }
  | { kind: "apply"; seat: Seat; action: unknown }
  | { kind: "snapshot"; viewers: (Seat | null)[] }
  | { kind: "exportState" }
  | { kind: "exportLog" };

export interface AiRequest {
  gameId: string;
  state: unknown;
  seat: Seat;
  level: Level;
  seed: number;
}

export function updateFor(s: Snapshot, viewer: Seat | null): Update {
  const seen = s.views.find((v) => v.viewer === viewer);
  if (!seen) throw new Error(`Pas de vue pour ${viewer === null ? "le spectateur" : `le siège ${viewer}`}.`);
  return { rev: s.rev, view: seen.view, legalActions: seen.legalActions, pending: s.pending, outcome: s.outcome };
}
```

`web/src/engine/rpc.ts` :
```ts
export type RpcResponse = { id: number; ok: true; value: unknown } | { id: number; ok: false; error: string };

/** Promise-based calls to a worker; requests are answered in order. */
export class WorkerRpc<Req> {
  private nextId = 0;
  private waiting = new Map<number, { resolve: (value: unknown) => void; reject: (error: Error) => void }>();

  constructor(private worker: Worker) {
    worker.onmessage = (event: MessageEvent<RpcResponse>) => {
      const res = event.data;
      const waiter = this.waiting.get(res.id);
      if (!waiter) return;
      this.waiting.delete(res.id);
      if (res.ok) waiter.resolve(res.value);
      else waiter.reject(new Error(res.error));
    };
  }

  call<T>(req: Req): Promise<T> {
    const id = this.nextId++;
    return new Promise<T>((resolve, reject) => {
      this.waiting.set(id, { resolve: (value) => resolve(value as T), reject });
      this.worker.postMessage({ id, req });
    });
  }

  dispose() {
    this.worker.terminate();
    for (const waiter of this.waiting.values()) waiter.reject(new Error("Moteur arrêté."));
    this.waiting.clear();
  }
}

/** Worker side: answers each request with `handle`'s result. */
export function serve<Req>(ready: Promise<unknown>, handle: (req: Req) => unknown) {
  self.onmessage = async (event: MessageEvent<{ id: number; req: Req }>) => {
    await ready;
    const { id, req } = event.data;
    let res: RpcResponse;
    try {
      res = { id, ok: true, value: handle(req) };
    } catch (e) {
      res = { id, ok: false, error: e instanceof Error ? e.message : String(e) };
    }
    self.postMessage(res);
  };
}
```

`web/src/engine/engine.worker.ts` :
```ts
// Holds the match. Never searches: the AI has its own worker, so a long
// search never delays network messages.
import init, * as wasm from "../wasm/ludo.js";
import { serve } from "./rpc";
import type { EngineRequest, Snapshot } from "./types";

let handle: number | null = null;

function current(): number {
  if (handle === null) throw new Error("Aucune partie en cours.");
  return handle;
}

function replace(next: number) {
  if (handle !== null) wasm.dropMatch(handle);
  handle = next;
}

serve<EngineRequest>(init(), (req) => {
  switch (req.kind) {
    case "listGames":
      return JSON.parse(wasm.listGames());
    case "newMatch":
      replace(wasm.newMatch(req.gameId, JSON.stringify(req.config ?? null), req.players, req.seed));
      return null;
    case "importLog":
      replace(wasm.importLog(req.gameId, JSON.stringify(req.log)));
      return null;
    case "apply":
      wasm.apply(current(), req.seat, JSON.stringify(req.action));
      return null;
    case "snapshot": {
      const h = current();
      const snapshot: Snapshot = {
        rev: wasm.rev(h),
        pending: Array.from(wasm.pending(h)),
        outcome: JSON.parse(wasm.outcome(h)),
        views: req.viewers.map((viewer) => ({
          viewer,
          view: JSON.parse(wasm.view(h, viewer)),
          legalActions: viewer === null ? [] : JSON.parse(wasm.legalActions(h, viewer)),
        })),
      };
      return snapshot;
    }
    case "exportState":
      return JSON.parse(wasm.exportState(current()));
    case "exportLog":
      return JSON.parse(wasm.exportLog(current()));
  }
});
```

`web/src/engine/ai.worker.ts` :
```ts
import init, { aiMove } from "../wasm/ludo.js";
import { serve } from "./rpc";
import type { AiRequest } from "./types";

serve<AiRequest>(init(), (req) =>
  JSON.parse(aiMove(req.gameId, JSON.stringify(req.state), req.seat, req.level, req.seed)),
);
```

`web/src/engine/clients.ts` :
```ts
import { WorkerRpc } from "./rpc";
import type { AiRequest, EngineRequest, GameMeta, Seat, Snapshot } from "./types";

export class EngineClient {
  private rpc = new WorkerRpc<EngineRequest>(
    new Worker(new URL("./engine.worker.ts", import.meta.url), { type: "module" }),
  );

  listGames() {
    return this.rpc.call<GameMeta[]>({ kind: "listGames" });
  }

  newMatch(gameId: string, players: number, seed: number, config: unknown = null) {
    return this.rpc.call<null>({ kind: "newMatch", gameId, players, seed, config });
  }

  importLog(gameId: string, log: unknown) {
    return this.rpc.call<null>({ kind: "importLog", gameId, log });
  }

  /** Rejects with the engine's message when the action is illegal. */
  apply(seat: Seat, action: unknown) {
    return this.rpc.call<null>({ kind: "apply", seat, action });
  }

  snapshot(viewers: (Seat | null)[]) {
    return this.rpc.call<Snapshot>({ kind: "snapshot", viewers });
  }

  exportState() {
    return this.rpc.call<unknown>({ kind: "exportState" });
  }

  exportLog() {
    return this.rpc.call<unknown>({ kind: "exportLog" });
  }

  dispose() {
    this.rpc.dispose();
  }
}

export class AiClient {
  private rpc = new WorkerRpc<AiRequest>(new Worker(new URL("./ai.worker.ts", import.meta.url), { type: "module" }));

  move(req: AiRequest) {
    return this.rpc.call<unknown>(req);
  }

  dispose() {
    this.rpc.dispose();
  }
}

export function randomSeed(): number {
  return crypto.getRandomValues(new Uint32Array(1))[0];
}

/** The games the engine offers (asked from a short-lived worker). */
export async function loadCatalog(): Promise<GameMeta[]> {
  const engine = new EngineClient();
  try {
    return await engine.listGames();
  } finally {
    engine.dispose();
  }
}
```

- [ ] **Step 3: Registre UI et plateau Yavalath**

`web/src/games/registry.ts` :
```ts
import type { ComponentType } from "react";
import type { Seat } from "../engine/types";
import { YavalathBoard } from "./yavalath/Board";

/** What every game board receives. It displays and submits; it computes nothing. */
export interface BoardProps {
  view: unknown;
  legalActions: unknown[];
  pending: Seat[];
  me: Seat | null;
  seatNames: string[];
  submit(action: unknown): void;
}

export interface GameUi {
  id: string;
  Board: ComponentType<BoardProps>;
}

/** Adding a game: one entry here (and one in crates/ludo-registry). */
export const GAME_UIS: GameUi[] = [{ id: "yavalath", Board: YavalathBoard }];

export function gameUi(id: string): GameUi | undefined {
  return GAME_UIS.find((g) => g.id === id);
}
```

`web/src/games/yavalath/types.ts` :
```ts
// Mirrors games/yavalath/src (View, Action).

export type Color = "White" | "Black" | "Red";

export interface Cell {
  q: number;
  r: number;
}

export interface CellView extends Cell {
  stone: Color | null;
}

export interface YavalathView {
  cells: CellView[];
  seatColors: Color[];
  current: number | null;
  eliminated: number[];
  forced: Cell[];
  lastMove: Cell | null;
  lines: Cell[][];
  swapped: boolean;
}

export type YavalathAction = { Place: Cell } | "Swap";
```

`web/src/games/yavalath/Board.tsx` :
```tsx
import type { BoardProps } from "../registry";
import type { Cell, Color, YavalathAction, YavalathView } from "./types";

const SIZE = 26;
const SQRT3 = Math.sqrt(3);
const FILL: Record<Color, string> = { White: "#f4f1ea", Black: "#23211f", Red: "#c8322b" };
const NAMES: Record<Color, string> = { White: "Blanc", Black: "Noir", Red: "Rouge" };

/** Pointy-top axial layout. */
function center(c: Cell) {
  return { x: SIZE * SQRT3 * (c.q + c.r / 2), y: SIZE * 1.5 * c.r };
}

function hexPoints(x: number, y: number): string {
  return Array.from({ length: 6 }, (_, k) => {
    const angle = (Math.PI / 180) * (60 * k - 30);
    return `${x + SIZE * Math.cos(angle)},${y + SIZE * Math.sin(angle)}`;
  }).join(" ");
}

function same(a: Cell, b: Cell) {
  return a.q === b.q && a.r === b.r;
}

export function YavalathBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as YavalathView;
  const legal = legalActions as YavalathAction[];
  const placeable = legal.flatMap((a) => (typeof a === "object" ? [a.Place] : []));
  const canSwap = legal.includes("Swap");
  const inLine = v.lines.flat();

  return (
    <div className="yavalath">
      <ul className="legend">
        {v.seatColors.map((color, seat) => (
          <li key={seat} className={v.current === seat ? "active" : undefined}>
            <span className="swatch" style={{ background: FILL[color] }} />
            {NAMES[color]} : {seatNames[seat]}
            {v.eliminated.includes(seat) ? " (éliminé)" : ""}
          </li>
        ))}
      </ul>
      <svg className="hexboard" viewBox="-210 -190 420 380" role="img" aria-label="Plateau de Yavalath">
        {v.cells.map((c) => {
          const { x, y } = center(c);
          const playable = placeable.some((p) => same(p, c));
          const forced = v.forced.some((f) => same(f, c));
          return (
            <g
              key={`${c.q},${c.r}`}
              className={playable ? "cell playable" : "cell"}
              onClick={playable ? () => submit({ Place: { q: c.q, r: c.r } } satisfies YavalathAction) : undefined}
            >
              <polygon points={hexPoints(x, y)} className={forced ? "hex forced" : "hex"} />
              {c.stone && (
                <circle
                  cx={x}
                  cy={y}
                  r={SIZE * 0.62}
                  fill={FILL[c.stone]}
                  className={inLine.some((l) => same(l, c)) ? "stone in-line" : "stone"}
                />
              )}
              {v.lastMove && same(v.lastMove, c) && <circle cx={x} cy={y} r={4} className="last-move" />}
            </g>
          );
        })}
      </svg>
      {canSwap && (
        <button onClick={() => submit("Swap" satisfies YavalathAction)}>Échanger les couleurs</button>
      )}
      {me !== null && v.current === me && v.forced.length > 0 && (
        <p className="hint">Tu dois bloquer la victoire du joueur suivant.</p>
      )}
    </div>
  );
}
```

- [ ] **Step 4: Accueil et App minimale**

`web/src/shell/Home.tsx` :
```tsx
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
```

`web/src/App.tsx` :
```tsx
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
```

`web/src/styles.css` :
```css
:root {
  color-scheme: light dark;
  --bg: #f6f3ee;
  --fg: #1d1b18;
  --muted: #6b655c;
  --card: #fffdf9;
  --accent: #2f6f5e;
  --line: #d8d0c4;
  --danger: #b3261e;
  font-family: system-ui, sans-serif;
}
@media (prefers-color-scheme: dark) {
  :root { --bg: #1b1a18; --fg: #eeeae3; --muted: #a39c90; --card: #262421; --accent: #6fbfa6; --line: #3d3a35; }
}
* { box-sizing: border-box; }
body { margin: 0; background: var(--bg); color: var(--fg); }
main { max-width: 900px; margin: 0 auto; padding: 16px; }
h1 { margin: 8px 0 16px; }
button { font: inherit; padding: 6px 14px; border-radius: 6px; border: 1px solid var(--line); background: var(--card); color: var(--fg); cursor: pointer; }
button:disabled { opacity: 0.5; cursor: default; }
button.primary { background: var(--accent); border-color: var(--accent); color: white; }
select, input { font: inherit; padding: 5px 8px; border-radius: 6px; border: 1px solid var(--line); background: var(--card); color: var(--fg); }
.error { color: var(--danger); }
.status, .hint { color: var(--muted); }
.notice { color: var(--danger); }
.games { list-style: none; padding: 0; display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 12px; }
.game-card { background: var(--card); border: 1px solid var(--line); border-radius: 10px; padding: 12px 16px; }
.game-card button { margin-top: 4px; }
.game-card h2 { margin: 0 0 4px; font-size: 1.2rem; }
.row { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; margin: 8px 0; }
.seats { padding-left: 20px; }
.seats li { margin: 6px 0; }
.seats .active { font-weight: 600; }
.code { font-size: 1.4rem; letter-spacing: 0.15em; }
.topbar { display: flex; gap: 12px; align-items: center; }
.topbar h2 { margin: 0; }
.yavalath { display: flex; flex-direction: column; align-items: center; gap: 8px; }
.legend { list-style: none; padding: 0; display: flex; gap: 16px; flex-wrap: wrap; margin: 0; }
.legend .active { font-weight: 600; }
.swatch { display: inline-block; width: 14px; height: 14px; border-radius: 50%; border: 1px solid var(--muted); margin-right: 6px; vertical-align: -2px; }
.hexboard { width: min(100%, 560px); height: auto; }
.hex { fill: var(--card); stroke: var(--line); stroke-width: 1.5; }
.hex.forced { stroke: var(--danger); stroke-width: 3; }
.cell.playable { cursor: pointer; }
.cell.playable:hover .hex { fill: color-mix(in srgb, var(--accent) 25%, var(--card)); }
.stone { stroke: var(--muted); stroke-width: 1; }
.stone.in-line { stroke: var(--accent); stroke-width: 4; }
.last-move { fill: var(--accent); }
.end { background: var(--card); border: 1px solid var(--line); border-radius: 10px; padding: 12px 16px; margin-top: 12px; }
```

- [ ] **Step 5: Vérifier**

Run: `cd ~/prog/ludo-rs && ./scripts/build-wasm.sh dev && cd web && pnpm build`
Expected: `tsc` sans erreur, `vite build` produit `web/dist`.

Run: `cd web && pnpm dev`, ouvrir l'URL affichée.
Expected: la page affiche « ludo.rs » et une carte « Yavalath — 2–3 joueurs ».

- [ ] **Step 6: Commit**

```bash
cd ~/prog/ludo-rs && git add -A && git commit -m "web: scaffold, engine and AI workers, game registry, Yavalath board

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 12: Sessions locales, `Table`, lobby et cadre de partie (jouable en local)

**Files:**
- Create: `web/src/session/session.ts`, `web/src/session/table.ts`, `web/src/session/local.ts`, `web/src/session/rev.ts`, `web/src/session/names.ts`
- Create: `web/src/session/__tests__/rev.test.ts`
- Create: `web/src/shell/Lobby.tsx`, `web/src/shell/TableScreen.tsx`
- Modify: `web/src/shell/Home.tsx`, `web/src/App.tsx`

**Interfaces:**
- Consumes: `EngineClient`, `AiClient`, `randomSeed`, `updateFor`, `Snapshot`, `gameUi` (Tâche 11).
- Produces:
  - `session.ts` : `SeatKind = "local" | "ai" | "remote" | "open"`, `SeatInfo { name; kind; connected }`, `SessionState { phase: "connecting" | "lobby" | "playing"; gameId; update; me; seats; status; notice; error; shareCode; latencyMs }`, classe abstraite `Session` (`canRestart`, `submit(seat, action)`, `restart()`, `dispose()`, `getState`, `subscribe`, `protected set`, `protected fail`).
  - `table.ts` : `TableSeat = { kind: "local"; name } | { kind: "ai"; level; name } | { kind: "remote"; name }`, classe `Table(gameId, seats, hooks: { onSnapshot(s); onLog?(log, s); onError(message) })` avec `start(seed, config?)`, `resume(log)`, `submit(seat, action, rev?)`, `snapshot`, `dispose()`.
  - `rev.ts` : `StaleActionError`, `isStale(actionRev, currentRev)`.
  - `names.ts` : `LEVEL_LABELS`, `aiName(level)`, `errorMessage(e)`.
  - `local.ts` : `LocalSession(gameId, seats: TableSeat[], config?)`.
  - `Lobby.tsx` : `SeatChoice = "local" | "open" | Level` ; props `{ game; onLocal(seats: TableSeat[]); onHost?(choices: SeatChoice[]); onBack() }`.
  - `TableScreen.tsx` : props `{ session: Session; games: GameMeta[]; onLeave() }`.

- [ ] **Step 1: Écrire le test (qui échoue)**

`web/src/session/__tests__/rev.test.ts` :
```ts
import { describe, expect, it } from "vitest";
import { StaleActionError, isStale } from "../rev";

describe("rev", () => {
  it("only the current revision is fresh", () => {
    expect(isStale(3, 3)).toBe(false);
    expect(isStale(2, 3)).toBe(true);
    expect(isStale(4, 3)).toBe(true);
  });

  it("the stale error is recognisable", () => {
    expect(new StaleActionError()).toBeInstanceOf(StaleActionError);
    expect(new StaleActionError().message).toMatch(/avancé/);
  });
});
```

Run: `cd web && pnpm test` → Expected: FAIL (module `../rev` introuvable).

- [ ] **Step 2: `rev.ts` et `names.ts`**

`web/src/session/rev.ts` :
```ts
/** The action was computed for a state the game has since left. */
export class StaleActionError extends Error {
  constructor() {
    super("La partie a avancé entre-temps : coup ignoré.");
    this.name = "StaleActionError";
  }
}

/** `rev` counts accepted actions: an action is only valid for the revision it was chosen at. */
export function isStale(actionRev: number, currentRev: number): boolean {
  return actionRev !== currentRev;
}
```

`web/src/session/names.ts` :
```ts
import type { Level } from "../engine/types";

export const LEVEL_LABELS: Record<Level, string> = { Easy: "facile", Medium: "moyenne", Hard: "difficile" };

export function aiName(level: Level): string {
  return `IA ${LEVEL_LABELS[level]}`;
}

export function errorMessage(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}
```

Run: `pnpm test` → Expected: PASS.

- [ ] **Step 3: `session.ts`**

```ts
import type { Seat, Update } from "../engine/types";
import { errorMessage } from "./names";

export type SeatKind = "local" | "ai" | "remote" | "open";

export interface SeatInfo {
  name: string;
  kind: SeatKind;
  connected: boolean;
}

export interface SessionState {
  phase: "connecting" | "lobby" | "playing";
  gameId: string | null;
  /** What `me` sees, or null before the game starts. */
  update: Update | null;
  /** The seat this screen plays (the pending one in hot-seat). */
  me: Seat | null;
  seats: SeatInfo[];
  /** Connection state to show. */
  status: string | null;
  /** Non-fatal message (action refused…). */
  notice: string | null;
  /** Fatal error: the session cannot continue. */
  error: string | null;
  /** Room code to share (host) or joined (guest). */
  shareCode: string | null;
  latencyMs: number | null;
}

const INITIAL: SessionState = {
  phase: "connecting",
  gameId: null,
  update: null,
  me: null,
  seats: [],
  status: null,
  notice: null,
  error: null,
  shareCode: null,
  latencyMs: null,
};

/**
 * A game as seen by the page: a state to display, actions to send. Where the
 * engine runs (here or on a host) is the implementation's business.
 */
export abstract class Session {
  abstract readonly canRestart: boolean;

  private state: SessionState = INITIAL;
  private listeners = new Set<() => void>();

  abstract submit(seat: Seat, action: unknown): void;
  restart(): void {}
  dispose(): void {}

  getState = (): SessionState => this.state;

  subscribe = (listener: () => void): (() => void) => {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  };

  protected set(patch: Partial<SessionState>) {
    this.state = { ...this.state, ...patch };
    this.listeners.forEach((l) => l());
  }

  protected fail(e: unknown) {
    this.set({ error: errorMessage(e) });
  }
}
```

- [ ] **Step 4: `table.ts`**

```ts
import { AiClient, EngineClient, randomSeed } from "../engine/clients";
import type { Level, Seat, Snapshot } from "../engine/types";
import { errorMessage } from "./names";
import { StaleActionError, isStale } from "./rev";

export type TableSeat =
  | { kind: "local"; name: string }
  | { kind: "ai"; level: Level; name: string }
  | { kind: "remote"; name: string };

interface TableHooks {
  onSnapshot(s: Snapshot): void;
  /** After each snapshot: the match log, for saving. */
  onLog?(log: unknown, s: Snapshot): void;
  onError(message: string): void;
}

/** AI moves are shown after at least this long, so humans can follow. */
const AI_MIN_DELAY_MS = 400;

/**
 * The authoritative game: engine + AI seats. Used by local play and by the
 * host. Engine calls are serialized, and every change produces a snapshot
 * for all seats.
 */
export class Table {
  private engine = new EngineClient();
  private ai = new AiClient();
  private queue: Promise<unknown> = Promise.resolve();
  private thinking = new Set<Seat>();
  private disposed = false;
  snapshot: Snapshot | null = null;

  constructor(
    readonly gameId: string,
    readonly seats: TableSeat[],
    private hooks: TableHooks,
  ) {}

  start(seed: number, config: unknown = null): Promise<void> {
    return this.serial(async () => {
      await this.engine.newMatch(this.gameId, this.seats.length, seed, config);
      await this.refresh();
    });
  }

  resume(log: unknown): Promise<void> {
    return this.serial(async () => {
      await this.engine.importLog(this.gameId, log);
      await this.refresh();
    });
  }

  /**
   * Applies `action` for `seat`. Rejects with the engine's message when it
   * is illegal, or with `StaleActionError` when `rev` is given and the game
   * has moved on since.
   */
  submit(seat: Seat, action: unknown, rev?: number): Promise<void> {
    return this.serial(async () => {
      if (rev !== undefined && this.snapshot && isStale(rev, this.snapshot.rev)) throw new StaleActionError();
      await this.engine.apply(seat, action);
      await this.refresh();
    });
  }

  dispose() {
    this.disposed = true;
    this.engine.dispose();
    this.ai.dispose();
  }

  private serial<T>(task: () => Promise<T>): Promise<T> {
    const run = this.queue.then(task);
    this.queue = run.catch(() => undefined);
    return run;
  }

  private async refresh() {
    const viewers: (Seat | null)[] = [...this.seats.keys(), null];
    const s = await this.engine.snapshot(viewers);
    if (this.disposed) return;
    this.snapshot = s;
    this.hooks.onSnapshot(s);
    if (this.hooks.onLog) this.hooks.onLog(await this.engine.exportLog(), s);
    this.runAi(s);
  }

  private runAi(s: Snapshot) {
    for (const seat of s.pending) {
      const setup = this.seats[seat];
      if (setup?.kind !== "ai" || this.thinking.has(seat)) continue;
      this.thinking.add(seat);
      void this.playAi(seat, setup.level);
    }
  }

  private async playAi(seat: Seat, level: Level) {
    const started = performance.now();
    try {
      const { state, rev } = await this.serial(async () => ({
        state: await this.engine.exportState(),
        rev: this.snapshot?.rev ?? 0,
      }));
      const action = await this.ai.move({ gameId: this.gameId, state, seat, level, seed: randomSeed() });
      await sleep(AI_MIN_DELAY_MS - (performance.now() - started));
      if (this.disposed) return;
      this.thinking.delete(seat);
      await this.submit(seat, action, rev);
    } catch (e) {
      this.thinking.delete(seat);
      if (this.disposed) return;
      // Computed on an old state: the latest snapshot decides whether this seat still has to play.
      if (e instanceof StaleActionError) {
        if (this.snapshot) this.runAi(this.snapshot);
      } else {
        this.hooks.onError(errorMessage(e));
      }
    }
  }
}

function sleep(ms: number) {
  return new Promise<void>((resolve) => setTimeout(resolve, Math.max(0, ms)));
}
```

- [ ] **Step 5: `local.ts`**

```ts
import { randomSeed } from "../engine/clients";
import { updateFor, type Seat, type Snapshot } from "../engine/types";
import { errorMessage } from "./names";
import { Session } from "./session";
import { Table, type TableSeat } from "./table";

/** Everything on this device: local humans (hot-seat) and AIs. */
export class LocalSession extends Session {
  readonly canRestart = true;
  private table: Table;

  constructor(
    private gameId: string,
    private seats: TableSeat[],
    private config: unknown = null,
  ) {
    super();
    this.set({
      phase: "playing",
      gameId,
      seats: seats.map((s) => ({ name: s.name, kind: s.kind, connected: true })),
    });
    this.table = this.begin();
  }

  private begin(): Table {
    const table = new Table(this.gameId, this.seats, {
      onSnapshot: (s) => this.show(s),
      onError: (message) => this.set({ notice: message }),
    });
    table.start(randomSeed(), this.config).catch((e) => this.fail(e));
    return table;
  }

  private show(s: Snapshot) {
    const locals = this.seats.flatMap((seat, i) => (seat.kind === "local" ? [i] : []));
    // Hot-seat: show the local seat whose decision it is.
    const me = locals.find((i) => s.pending.includes(i)) ?? this.getState().me ?? locals[0] ?? null;
    this.set({ me, update: updateFor(s, me), notice: null });
  }

  submit(seat: Seat, action: unknown) {
    this.table.submit(seat, action).catch((e) => this.set({ notice: errorMessage(e) }));
  }

  restart() {
    this.table.dispose();
    this.set({ update: null, notice: null });
    this.table = this.begin();
  }

  dispose() {
    this.table.dispose();
  }
}
```

- [ ] **Step 6: Lobby et cadre de partie**

`web/src/shell/Lobby.tsx` :
```tsx
import { useState } from "react";
import type { GameMeta, Level } from "../engine/types";
import { aiName } from "../session/names";
import type { TableSeat } from "../session/table";

/** What the host picks for each seat after their own. */
export type SeatChoice = "local" | "open" | Level;

interface Props {
  game: GameMeta;
  onLocal(seats: TableSeat[]): void;
  /** Absent: online play is not offered. */
  onHost?(choices: SeatChoice[]): void;
  onBack(): void;
}

const LEVELS: Level[] = ["Easy", "Medium", "Hard"];

export function Lobby({ game, onLocal, onHost, onBack }: Props) {
  const [players, setPlayers] = useState(game.minPlayers);
  const [choices, setChoices] = useState<SeatChoice[]>(() => Array(game.maxPlayers - 1).fill("Medium"));
  const others = choices.slice(0, players - 1);

  const options: { value: SeatChoice; label: string }[] = [
    ...LEVELS.map((level) => ({ value: level as SeatChoice, label: aiName(level) })),
    // Hot-seat only without hidden information: everyone sees the same screen.
    ...(game.hiddenInfo ? [] : [{ value: "local" as SeatChoice, label: "Humain (même écran)" }]),
    ...(onHost ? [{ value: "open" as SeatChoice, label: "Invité (en ligne)" }] : []),
  ];

  const localSeats = (): TableSeat[] => [
    { kind: "local", name: "Toi" },
    ...others.map((c, i): TableSeat =>
      c === "local" || c === "open" ? { kind: "local", name: `Joueur ${i + 2}` } : { kind: "ai", level: c, name: aiName(c) },
    ),
  ];

  const choose = (i: number, value: SeatChoice) =>
    setChoices((current) => current.map((c, j) => (j === i ? value : c)));

  return (
    <main>
      <div className="topbar">
        <button onClick={onBack}>← Menu</button>
        <h2>{game.name}</h2>
      </div>
      <div className="row">
        <label htmlFor="players">Joueurs :</label>
        <select id="players" value={players} onChange={(e) => setPlayers(Number(e.target.value))}>
          {Array.from({ length: game.maxPlayers - game.minPlayers + 1 }, (_, k) => game.minPlayers + k).map((n) => (
            <option key={n} value={n}>
              {n}
            </option>
          ))}
        </select>
      </div>
      <ol className="seats">
        <li>Toi</li>
        {others.map((c, i) => (
          <li key={i}>
            <select value={c} onChange={(e) => choose(i, e.target.value as SeatChoice)}>
              {options.map((o) => (
                <option key={o.value} value={o.value}>
                  {o.label}
                </option>
              ))}
            </select>
          </li>
        ))}
      </ol>
      <div className="row">
        <button className="primary" disabled={others.includes("open")} onClick={() => onLocal(localSeats())}>
          Jouer sur cet appareil
        </button>
        {onHost && (
          <button disabled={others.includes("local")} onClick={() => onHost(others)}>
            Héberger en ligne
          </button>
        )}
      </div>
    </main>
  );
}
```

`web/src/shell/TableScreen.tsx` :
```tsx
import { useSyncExternalStore } from "react";
import type { GameMeta, Update } from "../engine/types";
import { gameUi } from "../games/registry";
import type { SeatInfo, Session, SessionState } from "../session/session";

interface Props {
  session: Session;
  games: GameMeta[];
  onLeave(): void;
}

export function TableScreen({ session, games, onLeave }: Props) {
  const state = useSyncExternalStore(session.subscribe, session.getState);
  const meta = games.find((g) => g.id === state.gameId);
  return (
    <main>
      <div className="topbar">
        <button onClick={onLeave}>← Menu</button>
        <h2>{meta?.name ?? "…"}</h2>
        {state.latencyMs !== null && <span className="status">{state.latencyMs} ms</span>}
      </div>
      {state.error && <p className="error">{state.error}</p>}
      {state.status && <p className="status">{state.status}</p>}
      {state.phase === "playing" && <Playing session={session} state={state} />}
    </main>
  );
}

function Playing({ session, state }: { session: Session; state: SessionState }) {
  const ui = state.gameId ? gameUi(state.gameId) : undefined;
  if (state.gameId && !ui) return <p className="error">Jeu inconnu : mets la page à jour.</p>;
  const update = state.update;
  const Board = ui?.Board;
  return (
    <>
      <SeatList seats={state.seats} update={update} me={state.me} />
      {update && Board && (
        <Board
          view={update.view}
          legalActions={update.legalActions}
          pending={update.pending}
          me={state.me}
          seatNames={state.seats.map((s) => s.name)}
          submit={(action) => {
            if (state.me !== null) session.submit(state.me, action);
          }}
        />
      )}
      {state.notice && <p className="notice">{state.notice}</p>}
      {update?.outcome && (
        <div className="end">
          <strong>Partie terminée.</strong>{" "}
          {session.canRestart && <button onClick={() => session.restart()}>Revanche</button>}
        </div>
      )}
    </>
  );
}

function rankLabel(rank: number): string {
  return rank === 0 ? "1er" : `${rank + 1}ᵉ`;
}

function SeatList({ seats, update, me }: { seats: SeatInfo[]; update: Update | null; me: number | null }) {
  return (
    <ol className="seats">
      {seats.map((seat, i) => {
        const pending = update?.pending.includes(i) ?? false;
        const rank = update?.outcome?.[i];
        return (
          <li key={i} className={pending ? "active" : undefined}>
            {seat.name}
            {i === me ? " (toi)" : ""}
            {!seat.connected && seat.kind === "remote" ? " (déconnecté)" : ""}
            {pending ? (seat.kind === "ai" ? " — réfléchit…" : " — à jouer") : ""}
            {rank !== undefined ? ` — ${rankLabel(rank)}` : ""}
          </li>
        );
      })}
    </ol>
  );
}
```

- [ ] **Step 7: Brancher l'accueil et l'App**

`web/src/shell/Home.tsx` : ajouter la prop `onPick(game: GameMeta): void` à `Props`, la déstructurer, et remplacer le contenu de chaque `<li className="game-card">` par :
```tsx
            <h2>{g.name}</h2>
            <p>
              {g.minPlayers === g.maxPlayers ? g.minPlayers : `${g.minPlayers}–${g.maxPlayers}`} joueurs
            </p>
            <button className="primary" onClick={() => onPick(g)}>
              Jouer
            </button>
```

`web/src/App.tsx` (remplacement complet) :
```tsx
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
```

- [ ] **Step 8: Vérifier**

Run: `cd web && pnpm test && pnpm build`
Expected: tests PASS, build sans erreur.

Vérification manuelle (`pnpm dev`) :
1. Yavalath → 2 joueurs, IA facile → « Jouer sur cet appareil » : on pose une pierre, l'IA répond après ~0,4 s, les cases occupées ne sont pas cliquables.
2. 2 joueurs, siège 2 « Humain (même écran) » : après le premier coup de Blanc, le bouton « Échanger les couleurs » apparaît pour Noir, puis disparaît.
3. 3 joueurs, deux humains sur le même écran + une IA : le plateau affiche la vue du siège qui doit jouer ; les cases de blocage imposé sont entourées en rouge.
4. Fin de partie : classement affiché, « Revanche » relance une partie.

- [ ] **Step 9: Commit**

```bash
cd ~/prog/ludo-rs && git add -A && git commit -m "web: local sessions, table with AI seats, lobby and game frame

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 13: Protocole P2P, registre des sièges et stockage

**Files:**
- Create: `web/src/session/protocol.ts`, `web/src/session/seats.ts`, `web/src/session/storage.ts`
- Create: `web/src/session/__tests__/protocol.test.ts`, `web/src/session/__tests__/seats.test.ts`

**Interfaces:**
- Consumes: `SeatInfo` (Tâche 12), `TableSeat` (Tâche 12), `aiName` (Tâche 12), `Update`, `Level` (Tâche 11).
- Produces:
  - `protocol.ts` : `PROTOCOL_VERSION = 1`, `PEER_PREFIX = "ludo-"`, types `GuestMessage` (`hello { protocol, name, seatToken? }`, `action { rev, action }`, `ping { t }`) et `HostMessage` (`lobby { gameId, seats, started }`, `seated { seat, seatToken }`, `update { update }`, `rejected { reason, fatal }`, `pong { t }`), `parseGuestMessage(data): GuestMessage | null`, `parseHostMessage(data): HostMessage | null`, `newCode()`, `normalizeCode(code)`.
  - `seats.ts` : `HostSeat` (`host { name }` | `ai { level }` | `open` | `guest { name, token, connected }`), `HostChoice = { kind: "ai"; level } | { kind: "open" }`, `JoinResult`, classe `SeatRegistry` (`fromJSON`, `toJSON`, `all`, `join(name, token, started, newToken)`, `leave(seat, started)`, `setSeat(seat, choice)`, `ready()`, `info()`, `tableSeats()`).
  - `storage.ts` : `SavedHost { code, gameId, seats: HostSeat[], log }`, `loadName`, `saveName`, `loadHost`, `saveHost`, `clearHost`, `loadGuestToken`, `saveGuestToken` (toutes tolèrent un `localStorage` absent).

- [ ] **Step 1: Écrire les tests (qui échouent)**

`web/src/session/__tests__/protocol.test.ts` :
```ts
import { describe, expect, it } from "vitest";
import { PROTOCOL_VERSION, newCode, normalizeCode, parseGuestMessage, parseHostMessage } from "../protocol";

describe("parseGuestMessage", () => {
  it("rejects garbage", () => {
    for (const bad of [null, 42, "hello", {}, { type: "x" }, { type: "action", action: 1 }, { type: "action", rev: 1.5, action: 1 }, { type: "hello", name: "a" }, { type: "hello", protocol: 1, name: 3 }, { type: "ping" }]) {
      expect(parseGuestMessage(bad)).toBeNull();
    }
  });

  it("accepts well-formed messages", () => {
    expect(parseGuestMessage({ type: "hello", protocol: PROTOCOL_VERSION, name: "Ana" })).toEqual({
      type: "hello",
      protocol: PROTOCOL_VERSION,
      name: "Ana",
      seatToken: undefined,
    });
    expect(parseGuestMessage({ type: "hello", protocol: 1, name: "Ana", seatToken: "t" })?.type).toBe("hello");
    expect(parseGuestMessage({ type: "action", rev: 3, action: { Place: { q: 0, r: 0 } } })).toEqual({
      type: "action",
      rev: 3,
      action: { Place: { q: 0, r: 0 } },
    });
    expect(parseGuestMessage({ type: "ping", t: 12 })).toEqual({ type: "ping", t: 12 });
  });
});

describe("parseHostMessage", () => {
  it("needs a known type", () => {
    expect(parseHostMessage({ type: "nope" })).toBeNull();
    expect(parseHostMessage(null)).toBeNull();
    expect(parseHostMessage({ type: "pong", t: 1 })).toEqual({ type: "pong", t: 1 });
  });
});

describe("codes", () => {
  it("are 6 unambiguous characters", () => {
    for (let i = 0; i < 50; i++) expect(newCode()).toMatch(/^[ABCDEFGHJKLMNPQRSTUVWXYZ23456789]{6}$/);
  });

  it("are normalized", () => {
    expect(normalizeCode("  ab3dxy ")).toBe("AB3DXY");
  });
});
```

`web/src/session/__tests__/seats.test.ts` :
```ts
import { describe, expect, it } from "vitest";
import { SeatRegistry, type HostSeat } from "../seats";

function tokens() {
  let n = 0;
  return () => `t${++n}`;
}

function table(...others: HostSeat[]) {
  return new SeatRegistry([{ kind: "host", name: "Hôte" }, ...others]);
}

describe("SeatRegistry", () => {
  it("seats guests in the open seats, in order", () => {
    const r = table({ kind: "open" }, { kind: "ai", level: "Easy" }, { kind: "open" });
    const next = tokens();
    expect(r.join("Ana", undefined, false, next)).toEqual({ ok: true, seat: 1, token: "t1", reconnected: false });
    expect(r.join("Bo", undefined, false, next)).toEqual({ ok: true, seat: 3, token: "t2", reconnected: false });
    expect(r.ready()).toBe(true);
  });

  it("gives the last open seat to one guest only", () => {
    const r = table({ kind: "open" });
    const next = tokens();
    expect(r.join("Ana", undefined, false, next).ok).toBe(true);
    expect(r.join("Bo", undefined, false, next)).toEqual({ ok: false, reason: "La table est complète." });
  });

  it("gives a returning guest their seat back", () => {
    const r = table({ kind: "open" }, { kind: "open" });
    const next = tokens();
    r.join("Ana", undefined, false, next);
    r.join("Bo", undefined, false, next);
    r.leave(1, true);
    expect(r.info()[1].connected).toBe(false);
    expect(r.join("Ana", "t1", true, next)).toEqual({ ok: true, seat: 1, token: "t1", reconnected: true });
    expect(r.info()[1].connected).toBe(true);
  });

  it("refuses newcomers once the game has started", () => {
    const r = table({ kind: "open" });
    r.join("Ana", undefined, false, tokens());
    r.leave(1, true);
    expect(r.join("Eve", "forged", true, tokens())).toEqual({ ok: false, reason: "La partie a déjà commencé." });
  });

  it("frees a seat when a guest leaves the lobby", () => {
    const r = table({ kind: "open" });
    r.join("Ana", undefined, false, tokens());
    r.leave(1, false);
    expect(r.all[1]).toEqual({ kind: "open" });
  });

  it("survives a host reload: guests come back to their seats", () => {
    const r = table({ kind: "open" }, { kind: "ai", level: "Hard" });
    r.join("Ana", undefined, false, tokens());
    const restored = SeatRegistry.fromJSON(JSON.parse(JSON.stringify(r.toJSON())));
    expect(restored.info()[1]).toEqual({ name: "Ana", kind: "remote", connected: false });
    expect(restored.join("Ana", "t1", true, tokens())).toEqual({ ok: true, seat: 1, token: "t1", reconnected: true });
  });

  it("is not ready while a seat is open or a guest is away", () => {
    const r = table({ kind: "open" });
    expect(r.ready()).toBe(false);
    r.join("Ana", undefined, false, tokens());
    expect(r.ready()).toBe(true);
    r.leave(1, true);
    expect(r.ready()).toBe(false);
  });

  it("lets the host change only free seats", () => {
    const r = table({ kind: "open" }, { kind: "open" });
    r.join("Ana", undefined, false, tokens());
    expect(r.setSeat(0, { kind: "open" })).toBe(false);
    expect(r.setSeat(1, { kind: "ai", level: "Easy" })).toBe(false);
    expect(r.setSeat(2, { kind: "ai", level: "Easy" })).toBe(true);
    expect(r.tableSeats().map((s) => s.kind)).toEqual(["local", "remote", "ai"]);
  });

  it("cleans guest names", () => {
    const r = table({ kind: "open" });
    r.join("   ", undefined, false, tokens());
    expect(r.info()[1].name).toBe("Invité");
  });
});
```

Run: `cd web && pnpm test` → Expected: FAIL (modules introuvables).

- [ ] **Step 2: Écrire `protocol.ts`**

```ts
// Host ↔ guest protocol. The host runs the engine (the authority); a guest
// only sends actions and displays the updates of its own seat.
import type { Update } from "../engine/types";
import type { SeatInfo } from "./session";

/** Bumped on any incompatible change: host and guest must match. */
export const PROTOCOL_VERSION = 1;

/** Prefix of PeerJS ids, so a short code doesn't collide with other apps. */
export const PEER_PREFIX = "ludo-";

export type GuestMessage =
  | { type: "hello"; protocol: number; name: string; seatToken?: string }
  | { type: "action"; rev: number; action: unknown }
  | { type: "ping"; t: number };

export type HostMessage =
  | { type: "lobby"; gameId: string; seats: SeatInfo[]; started: boolean }
  | { type: "seated"; seat: number; seatToken: string }
  | { type: "update"; update: Update }
  /** `fatal`: the connection is refused and the guest must leave. */
  | { type: "rejected"; reason: string; fatal: boolean }
  | { type: "pong"; t: number };

function isObject(data: unknown): data is Record<string, unknown> {
  return typeof data === "object" && data !== null;
}

/** The host trusts nothing from the network: shape check here, legality in the engine. */
export function parseGuestMessage(data: unknown): GuestMessage | null {
  if (!isObject(data)) return null;
  switch (data.type) {
    case "hello":
      if (typeof data.protocol !== "number" || typeof data.name !== "string") return null;
      if (data.seatToken !== undefined && typeof data.seatToken !== "string") return null;
      return { type: "hello", protocol: data.protocol, name: data.name, seatToken: data.seatToken };
    case "action":
      if (!Number.isInteger(data.rev) || !("action" in data)) return null;
      return { type: "action", rev: data.rev as number, action: data.action };
    case "ping":
      return typeof data.t === "number" ? { type: "ping", t: data.t } : null;
    default:
      return null;
  }
}

const HOST_TYPES = new Set(["lobby", "seated", "update", "rejected", "pong"]);

export function parseHostMessage(data: unknown): HostMessage | null {
  if (!isObject(data) || typeof data.type !== "string" || !HOST_TYPES.has(data.type)) return null;
  return data as HostMessage;
}

/** Short code to share, without ambiguous characters (0/O, 1/I…). */
export function newCode(): string {
  const alphabet = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
  const bytes = crypto.getRandomValues(new Uint8Array(6));
  return Array.from(bytes, (b) => alphabet[b % alphabet.length]).join("");
}

export function normalizeCode(code: string): string {
  return code.trim().toUpperCase();
}
```

- [ ] **Step 3: Écrire `seats.ts`**

```ts
import type { Level, Seat } from "../engine/types";
import { aiName } from "./names";
import type { SeatInfo } from "./session";
import type { TableSeat } from "./table";

export type HostSeat =
  | { kind: "host"; name: string }
  | { kind: "ai"; level: Level }
  | { kind: "open" }
  | { kind: "guest"; name: string; token: string; connected: boolean };

/** What the host may put in a seat no guest holds. */
export type HostChoice = { kind: "ai"; level: Level } | { kind: "open" };

export type JoinResult =
  | { ok: true; seat: Seat; token: string; reconnected: boolean }
  | { ok: false; reason: string };

function cleanName(name: string): string {
  return name.trim().slice(0, 20) || "Invité";
}

/** The host's seats: who sits where, and each guest's reconnection token. */
export class SeatRegistry {
  constructor(private seats: HostSeat[]) {}

  /** After a host reload: every guest is away until they reconnect. */
  static fromJSON(seats: HostSeat[]): SeatRegistry {
    return new SeatRegistry(seats.map((s) => (s.kind === "guest" ? { ...s, connected: false } : s)));
  }

  toJSON(): HostSeat[] {
    return this.seats.map((s) => ({ ...s }));
  }

  get all(): readonly HostSeat[] {
    return this.seats;
  }

  /**
   * A guest says hello. A known token takes its seat back (even if an old
   * connection still holds it: the caller closes that one). Otherwise the
   * first open seat, before the game starts.
   */
  join(name: string, token: string | undefined, started: boolean, newToken: () => string): JoinResult {
    const back = token === undefined ? -1 : this.seats.findIndex((s) => s.kind === "guest" && s.token === token);
    if (back >= 0) {
      const seat = this.seats[back] as Extract<HostSeat, { kind: "guest" }>;
      this.seats[back] = { ...seat, connected: true };
      return { ok: true, seat: back, token: seat.token, reconnected: true };
    }
    if (started) return { ok: false, reason: "La partie a déjà commencé." };
    const open = this.seats.findIndex((s) => s.kind === "open");
    if (open < 0) return { ok: false, reason: "La table est complète." };
    const fresh = newToken();
    this.seats[open] = { kind: "guest", name: cleanName(name), token: fresh, connected: true };
    return { ok: true, seat: open, token: fresh, reconnected: false };
  }

  /** In the lobby the seat opens again; during the game it waits for the guest. */
  leave(seat: Seat, started: boolean) {
    const s = this.seats[seat];
    if (s?.kind !== "guest") return;
    this.seats[seat] = started ? { ...s, connected: false } : { kind: "open" };
  }

  /** Lobby only: returns false for the host's seat or a guest's seat. */
  setSeat(seat: Seat, choice: HostChoice): boolean {
    const s = this.seats[seat];
    if (!s || s.kind === "host" || s.kind === "guest") return false;
    this.seats[seat] = choice;
    return true;
  }

  ready(): boolean {
    return this.seats.every((s) => s.kind !== "open" && (s.kind !== "guest" || s.connected));
  }

  info(): SeatInfo[] {
    return this.seats.map((s): SeatInfo => {
      switch (s.kind) {
        case "host":
          return { name: s.name, kind: "local", connected: true };
        case "ai":
          return { name: aiName(s.level), kind: "ai", connected: true };
        case "open":
          return { name: "Place libre", kind: "open", connected: false };
        case "guest":
          return { name: s.name, kind: "remote", connected: s.connected };
      }
    });
  }

  /** The table to start: throws if a seat is still open. */
  tableSeats(): TableSeat[] {
    return this.seats.map((s): TableSeat => {
      switch (s.kind) {
        case "host":
          return { kind: "local", name: s.name };
        case "ai":
          return { kind: "ai", level: s.level, name: aiName(s.level) };
        case "guest":
          return { kind: "remote", name: s.name };
        case "open":
          throw new Error("Un siège est encore libre.");
      }
    });
  }
}
```

- [ ] **Step 4: Écrire `storage.ts`**

```ts
// Per-device conveniences. Storage may be unavailable (private window,
// blocked site data): every access is guarded and the page works without it.
import type { HostSeat } from "./seats";

export interface SavedHost {
  code: string;
  gameId: string;
  seats: HostSeat[];
  log: unknown;
}

const NAME_KEY = "ludo-name";
const HOST_KEY = "ludo-host";
const guestKey = (code: string) => `ludo-guest:${code}`;

function read(key: string): string | null {
  try {
    return localStorage.getItem(key);
  } catch {
    return null;
  }
}

function write(key: string, value: string | null) {
  try {
    if (value === null) localStorage.removeItem(key);
    else localStorage.setItem(key, value);
  } catch {
    // Storage unavailable: nothing is kept.
  }
}

export function loadName(): string {
  return read(NAME_KEY) ?? "";
}

export function saveName(name: string) {
  write(NAME_KEY, name);
}

function isSavedHost(v: unknown): v is SavedHost {
  if (typeof v !== "object" || v === null) return false;
  const s = v as Partial<SavedHost>;
  return typeof s.code === "string" && typeof s.gameId === "string" && Array.isArray(s.seats) && "log" in s;
}

export function loadHost(): SavedHost | null {
  const raw = read(HOST_KEY);
  if (!raw) return null;
  try {
    const value: unknown = JSON.parse(raw);
    return isSavedHost(value) ? value : null;
  } catch {
    return null;
  }
}

export function saveHost(saved: SavedHost) {
  write(HOST_KEY, JSON.stringify(saved));
}

export function clearHost() {
  write(HOST_KEY, null);
}

export function loadGuestToken(code: string): string | null {
  return read(guestKey(code));
}

export function saveGuestToken(code: string, token: string) {
  write(guestKey(code), token);
}
```

- [ ] **Step 5: Lancer les tests**

Run: `cd web && pnpm test`
Expected: tous PASS.

- [ ] **Step 6: Commit**

```bash
cd ~/prog/ludo-rs && git add -A && git commit -m "web: P2P protocol, seat registry and storage

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 14: Sessions hôte et invité, lobby en ligne, reprise

**Files:**
- Create: `web/src/session/host.ts`, `web/src/session/guest.ts`, `web/src/shell/HostLobby.tsx`
- Modify: `web/src/shell/TableScreen.tsx`, `web/src/shell/Home.tsx` (remplacement complet), `web/src/App.tsx` (remplacement complet)

**Interfaces:**
- Consumes: `Table`, `Session`, `updateFor` (Tâches 11–12), `protocol.ts`, `SeatRegistry`, `HostChoice`, `storage.ts` (Tâche 13), `SeatChoice` (Tâche 12).
- Produces: `HostSession(gameId, hostName, others: HostChoice[], saved?)` avec `static resume(saved)`, `start()`, `setSeat(seat, choice)`, `ready()`, `hostSeats()` ; `GuestSession(code, name)`.

- [ ] **Step 1: Écrire `host.ts`**

```ts
import Peer, { type DataConnection } from "peerjs";
import { randomSeed } from "../engine/clients";
import { updateFor, type Seat, type Snapshot } from "../engine/types";
import { errorMessage } from "./names";
import {
  PEER_PREFIX,
  PROTOCOL_VERSION,
  newCode,
  parseGuestMessage,
  type GuestMessage,
  type HostMessage,
} from "./protocol";
import { SeatRegistry, type HostChoice, type HostSeat } from "./seats";
import { Session } from "./session";
import { clearHost, saveHost, type SavedHost } from "./storage";
import { Table } from "./table";

const HOST_SEAT: Seat = 0;
const RETRY_MS = 3000;
const MAX_RETRIES = 10;

/**
 * Hosts a game: the engine runs here and is the authority. Each guest is a
 * PeerJS connection to this host only, and receives only its own seat's view.
 */
export class HostSession extends Session {
  readonly canRestart = true;
  private peer: Peer | null = null;
  /** Open connections and the seat each one holds (null: not seated yet). */
  private conns = new Map<DataConnection, Seat | null>();
  private registry: SeatRegistry;
  private table: Table | null = null;
  private code: string;
  private retries = 0;
  private disposed = false;

  constructor(
    private gameId: string,
    hostName: string,
    others: HostChoice[],
    saved?: SavedHost,
  ) {
    super();
    if (saved) {
      this.code = saved.code;
      this.registry = SeatRegistry.fromJSON(saved.seats);
      this.set({ phase: "playing", gameId, seats: this.registry.info() });
      this.table = this.newTable();
      this.table.resume(saved.log).catch((e) => this.fail(e));
    } else {
      this.code = newCode();
      this.registry = new SeatRegistry([{ kind: "host", name: hostName.trim() || "Hôte" }, ...others]);
      this.set({ phase: "lobby", gameId, seats: this.registry.info() });
    }
    this.set({ status: "Ouverture de la salle…" });
    this.listen();
  }

  static resume(saved: SavedHost): HostSession {
    return new HostSession(saved.gameId, "", [], saved);
  }

  hostSeats(): readonly HostSeat[] {
    return this.registry.all;
  }

  ready(): boolean {
    return this.table === null && this.registry.ready();
  }

  setSeat(seat: Seat, choice: HostChoice) {
    if (this.table || !this.registry.setSeat(seat, choice)) return;
    this.set({ seats: this.registry.info() });
    this.broadcastLobby();
  }

  start() {
    if (!this.ready()) return;
    this.table = this.newTable();
    this.set({ phase: "playing", status: null });
    this.broadcastLobby();
    this.table.start(randomSeed()).catch((e) => this.fail(e));
  }

  submit(seat: Seat, action: unknown) {
    this.table?.submit(seat, action).catch((e) => this.set({ notice: errorMessage(e) }));
  }

  restart() {
    if (!this.table) return;
    this.table.dispose();
    this.table = this.newTable();
    this.table.start(randomSeed()).catch((e) => this.fail(e));
  }

  dispose() {
    this.disposed = true;
    this.table?.dispose();
    this.peer?.destroy();
  }

  private newTable(): Table {
    return new Table(this.gameId, this.registry.tableSeats(), {
      onSnapshot: (s) => this.broadcast(s),
      onLog: (log, s) => this.persist(log, s),
      onError: (message) => this.set({ notice: message }),
    });
  }

  private listen() {
    const peer = new Peer(PEER_PREFIX + this.code);
    this.peer = peer;
    peer.on("open", () => {
      this.retries = 0;
      this.set({
        shareCode: this.code,
        status: this.table ? null : "Partage le code ou le lien avec tes invités.",
      });
    });
    peer.on("connection", (conn) => this.accept(conn));
    peer.on("error", (err) => {
      if (err.type === "unavailable-id") {
        peer.destroy();
        if (!this.table) {
          // Fresh room: just take another code.
          this.code = newCode();
          this.listen();
          return;
        }
        // Resumed room: the previous page may still hold the code for a while.
        if (++this.retries > MAX_RETRIES) {
          this.fail("Le code de la salle est encore pris. Réessaie dans une minute.");
          return;
        }
        this.set({ status: "Le code de la salle est encore réservé, nouvel essai…" });
        setTimeout(() => {
          if (!this.disposed) this.listen();
        }, RETRY_MS);
      } else if (err.type !== "peer-unavailable") {
        this.set({ notice: `Connexion : ${err.message}` });
      }
    });
    // Signaling server lost: established connections keep working.
    peer.on("disconnected", () => {
      if (!peer.destroyed) peer.reconnect();
    });
  }

  private accept(conn: DataConnection) {
    this.conns.set(conn, null);
    conn.on("data", (data) => this.onGuestMessage(conn, data));
    conn.on("close", () => this.onClose(conn));
    conn.on("error", () => this.onClose(conn));
  }

  private onClose(conn: DataConnection) {
    if (!this.conns.has(conn)) return;
    const seat = this.conns.get(conn) ?? null;
    this.conns.delete(conn);
    // Unseated, or replaced by a newer connection of the same guest.
    if (seat === null || [...this.conns.values()].includes(seat)) return;
    const name = this.registry.info()[seat].name;
    this.registry.leave(seat, this.table !== null);
    this.set({
      seats: this.registry.info(),
      status: this.table ? `${name} s'est déconnecté : on attend son retour.` : null,
    });
    this.broadcastLobby();
  }

  private onGuestMessage(conn: DataConnection, data: unknown) {
    const msg = parseGuestMessage(data);
    if (!msg) {
      this.send(conn, { type: "rejected", reason: "Message invalide.", fatal: false });
      return;
    }
    switch (msg.type) {
      case "hello":
        this.onHello(conn, msg);
        break;
      case "ping":
        this.send(conn, { type: "pong", t: msg.t });
        break;
      case "action": {
        const seat = this.conns.get(conn) ?? null;
        if (seat === null || !this.table) {
          this.send(conn, { type: "rejected", reason: "Tu n'as pas de siège dans cette partie.", fatal: false });
          return;
        }
        // The engine checks it is this seat's decision and that it is legal.
        this.table.submit(seat, msg.action, msg.rev).catch((e) => {
          this.send(conn, { type: "rejected", reason: errorMessage(e), fatal: false });
          this.sendUpdate(conn, seat);
        });
        break;
      }
    }
  }

  private onHello(conn: DataConnection, msg: Extract<GuestMessage, { type: "hello" }>) {
    if (msg.protocol !== PROTOCOL_VERSION) {
      this.refuse(conn, `Versions différentes (hôte ${PROTOCOL_VERSION}, toi ${msg.protocol}) : recharge la page.`);
      return;
    }
    const joined = this.registry.join(msg.name, msg.seatToken, this.table !== null, () => crypto.randomUUID());
    if (!joined.ok) {
      this.refuse(conn, joined.reason);
      return;
    }
    // A reconnection replaces the seat's previous connection.
    for (const [other, seat] of this.conns) {
      if (other !== conn && seat === joined.seat) {
        this.conns.delete(other);
        other.close();
      }
    }
    this.conns.set(conn, joined.seat);
    this.send(conn, { type: "seated", seat: joined.seat, seatToken: joined.token });
    this.set({ seats: this.registry.info(), status: this.table ? null : this.getState().status });
    this.broadcastLobby();
    this.sendUpdate(conn, joined.seat);
  }

  private refuse(conn: DataConnection, reason: string) {
    this.send(conn, { type: "rejected", reason, fatal: true });
    setTimeout(() => conn.close(), 500);
  }

  /** Each player gets their own seat's update, and only that. */
  private broadcast(s: Snapshot) {
    this.set({ me: HOST_SEAT, update: updateFor(s, HOST_SEAT), notice: null });
    for (const [conn, seat] of this.conns) {
      if (seat !== null) this.send(conn, { type: "update", update: updateFor(s, seat) });
    }
  }

  private sendUpdate(conn: DataConnection, seat: Seat) {
    const s = this.table?.snapshot;
    if (s) this.send(conn, { type: "update", update: updateFor(s, seat) });
  }

  private broadcastLobby() {
    const msg: HostMessage = { type: "lobby", gameId: this.gameId, seats: this.registry.info(), started: this.table !== null };
    for (const conn of this.conns.keys()) this.send(conn, msg);
  }

  /** Saved after every action so the host can resume after a reload; cleared once the game is over. */
  private persist(log: unknown, s: Snapshot) {
    if (s.outcome) clearHost();
    else saveHost({ code: this.code, gameId: this.gameId, seats: this.registry.toJSON(), log });
  }

  private send(conn: DataConnection, msg: HostMessage) {
    if (conn.open) void conn.send(msg);
  }
}
```

- [ ] **Step 2: Écrire `guest.ts`**

```ts
import Peer, { type DataConnection } from "peerjs";
import type { Seat } from "../engine/types";
import { gameUi } from "../games/registry";
import {
  PEER_PREFIX,
  PROTOCOL_VERSION,
  normalizeCode,
  parseHostMessage,
  type GuestMessage,
} from "./protocol";
import { Session } from "./session";
import { loadGuestToken, saveGuestToken } from "./storage";

const RETRY_MS = 3000;
const PING_MS = 5000;

/** Joins a host: sends actions, displays the updates of its own seat. */
export class GuestSession extends Session {
  readonly canRestart = false;
  private peer: Peer | null = null;
  private conn: DataConnection | null = null;
  private code: string;
  private closed = false;
  private retry: ReturnType<typeof setTimeout> | null = null;
  private ping: ReturnType<typeof setInterval>;

  constructor(
    code: string,
    private name: string,
  ) {
    super();
    this.code = normalizeCode(code);
    this.set({ phase: "connecting", shareCode: this.code, status: "Connexion à l'hôte…" });
    this.connect();
    this.ping = setInterval(() => this.send({ type: "ping", t: performance.now() }), PING_MS);
  }

  submit(_seat: Seat, action: unknown) {
    const rev = this.getState().update?.rev;
    if (rev !== undefined) this.send({ type: "action", rev, action });
  }

  dispose() {
    this.closed = true;
    clearInterval(this.ping);
    if (this.retry) clearTimeout(this.retry);
    this.peer?.destroy();
  }

  private connect() {
    this.peer?.destroy();
    const peer = new Peer();
    this.peer = peer;
    peer.on("open", () => {
      const conn = peer.connect(PEER_PREFIX + this.code, { reliable: true });
      this.conn = conn;
      conn.on("open", () =>
        this.send({
          type: "hello",
          protocol: PROTOCOL_VERSION,
          name: this.name,
          seatToken: loadGuestToken(this.code) ?? undefined,
        }),
      );
      conn.on("data", (data) => this.onHostMessage(data));
      conn.on("close", () => this.lost());
    });
    peer.on("error", (err) => {
      if (err.type === "peer-unavailable") {
        this.set({ status: "Hôte introuvable : vérifie le code, ou attends son retour…" });
      }
      this.lost();
    });
  }

  /** Keeps trying until the host is back (it may be reloading). */
  private lost() {
    if (this.closed || this.retry) return;
    if (this.getState().phase !== "connecting") this.set({ status: "Connexion perdue, reconnexion…", latencyMs: null });
    this.retry = setTimeout(() => {
      this.retry = null;
      if (!this.closed) this.connect();
    }, RETRY_MS);
  }

  private onHostMessage(data: unknown) {
    const msg = parseHostMessage(data);
    if (!msg) return;
    switch (msg.type) {
      case "lobby":
        if (!gameUi(msg.gameId)) {
          this.stop("Jeu inconnu : mets la page à jour.");
          return;
        }
        this.set({
          gameId: msg.gameId,
          seats: msg.seats,
          phase: msg.started ? "playing" : "lobby",
          status: msg.started ? null : "En attente du lancement par l'hôte.",
        });
        break;
      case "seated":
        saveGuestToken(this.code, msg.seatToken);
        this.set({ me: msg.seat });
        break;
      case "update":
        this.set({ phase: "playing", update: msg.update, status: null, notice: null });
        break;
      case "rejected":
        if (msg.fatal) this.stop(msg.reason);
        else this.set({ notice: msg.reason });
        break;
      case "pong":
        this.set({ latencyMs: Math.round(performance.now() - msg.t) });
        break;
    }
  }

  private stop(reason: string) {
    this.closed = true;
    this.fail(reason);
    this.peer?.destroy();
  }

  private send(msg: GuestMessage) {
    if (this.conn?.open) void this.conn.send(msg);
  }
}
```

- [ ] **Step 3: Écrire `HostLobby.tsx` et brancher le lobby dans `TableScreen.tsx`**

`web/src/shell/HostLobby.tsx` :
```tsx
import type { Level } from "../engine/types";
import type { HostSession } from "../session/host";
import { aiName } from "../session/names";
import type { HostChoice } from "../session/seats";
import type { SessionState } from "../session/session";

const LEVELS: Level[] = ["Easy", "Medium", "Hard"];

function parseChoice(value: string): HostChoice {
  return value === "open" ? { kind: "open" } : { kind: "ai", level: value as Level };
}

export function HostLobby({ session, state }: { session: HostSession; state: SessionState }) {
  const link = state.shareCode ? `${location.origin}${location.pathname}?join=${state.shareCode}` : null;
  const seats = session.hostSeats();
  return (
    <div>
      {state.shareCode && link && (
        <>
          <p>
            Code : <strong className="code">{state.shareCode}</strong>
          </p>
          <div className="row">
            <input readOnly value={link} size={40} onFocus={(e) => e.currentTarget.select()} />
            <button onClick={() => void navigator.clipboard?.writeText(link)}>Copier le lien</button>
          </div>
        </>
      )}
      <ol className="seats">
        {seats.map((seat, i) => (
          <li key={i}>
            {seat.kind === "host" && `${seat.name} (toi)`}
            {seat.kind === "guest" && `${seat.name}${seat.connected ? "" : " (déconnecté)"}`}
            {(seat.kind === "open" || seat.kind === "ai") && (
              <select
                value={seat.kind === "open" ? "open" : seat.level}
                onChange={(e) => session.setSeat(i, parseChoice(e.target.value))}
              >
                <option value="open">Place libre (invité)</option>
                {LEVELS.map((level) => (
                  <option key={level} value={level}>
                    {aiName(level)}
                  </option>
                ))}
              </select>
            )}
          </li>
        ))}
      </ol>
      <button className="primary" disabled={!session.ready()} onClick={() => session.start()}>
        Lancer la partie
      </button>
    </div>
  );
}
```

`web/src/shell/TableScreen.tsx` : ajouter les imports
```tsx
import { HostSession } from "../session/host";
import { HostLobby } from "./HostLobby";
```
et, juste avant `{state.phase === "playing" && <Playing session={session} state={state} />}`, la ligne :
```tsx
      {state.phase === "lobby" &&
        (session instanceof HostSession ? <HostLobby session={session} state={state} /> : <GuestLobby state={state} />)}
```
puis, en bas du fichier :
```tsx
function GuestLobby({ state }: { state: SessionState }) {
  return (
    <ol className="seats">
      {state.seats.map((seat, i) => (
        <li key={i}>
          {seat.name}
          {i === state.me ? " (toi)" : ""}
        </li>
      ))}
    </ol>
  );
}
```

- [ ] **Step 4: Accueil (remplacement complet de `Home.tsx`)**

```tsx
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
  const savedName = saved ? catalog?.find((g) => g.id === saved.gameId)?.name ?? saved.gameId : null;
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
```

- [ ] **Step 5: App (remplacement complet)**

```tsx
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
```

- [ ] **Step 6: Vérifier**

Run: `cd web && pnpm test && pnpm build`
Expected: tests PASS, build sans erreur.

Vérification manuelle à plusieurs onglets (`pnpm dev`) :
1. Onglet A : nom « Ana » → Yavalath → 2 joueurs, siège 2 « Invité (en ligne) » → « Héberger en ligne » : un code s'affiche.
2. Onglet B : ouvrir le lien copié : B apparaît dans le lobby de A ; « Lancer la partie » devient actif ; lancer.
3. Jouer quelques coups des deux côtés ; B ne peut pas cliquer quand ce n'est pas son tour.
4. Recharger B : il revient sur son siège (même couleur) et la partie continue.
5. Recharger A : l'accueil propose « Reprendre » ; reprendre ; B se reconnecte seul après quelques secondes.
6. 3 joueurs : A hôte, B invité, siège 3 IA difficile : l'IA joue quand c'est son tour.
7. Onglet C avec le même code pendant la partie : message « La partie a déjà commencé. »

- [ ] **Step 7: Commit**

```bash
cd ~/prog/ludo-rs && git add -A && git commit -m "web: P2P host and guest sessions, online lobby, host resume

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 15: Documentation, CI et publication

**Files:**
- Create: `CLAUDE.md`, `README.md`, `.github/workflows/ci.yml`, `.github/workflows/pages.yml`
- Modify: `docs/superpowers/specs/2026-09-26-plateforme-yavalath-design.md` (écarts assumés)

- [ ] **Step 1: `CLAUDE.md`**

```markdown
# CLAUDE.md

ludo-rs : plateforme de jeux traditionnels. Moteur Rust (`crates/ludo-core`, `games/*`), pont WASM (`crates/ludo-wasm`), site React (`web/`).

## Commandes
- `cargo test --workspace --release` : règles, conformance, IA.
- `cargo fmt --all` et `cargo clippy --workspace --all-targets -- -D warnings` avant chaque commit.
- `cargo run --release -p ludo-cli -- arena yavalath 200 hard easy` : Difficile doit marquer ≥ 0.80 contre Facile (≈ 30 min).
- `cargo run --release -p ludo-cli -- play yavalath 2 medium` : partie en terminal.
- Web : `./scripts/build-wasm.sh` (ou `dev`), puis `cd web && pnpm dev` / `pnpm test` / `pnpm build`.

## Principes
- **Le moteur possède toute la logique.** `ludo-wasm`, `ludo-cli` et `web/` affichent et envoient des actions. Toute valeur dérivée dont l'UI a besoin est calculée par le jeu et placée dans sa `View`.
- **`ludo-core` et `games/*` restent natifs** : jamais de `wasm-bindgen`, `js-sys`, `web-sys`. Un futur serveur les réutilisera tels quels (rejeu du `MatchLog`).
- **`apply` renvoie `Illegal` sans modifier l'état**, ne panique jamais sur une entrée.
- **Information cachée** : `view` ne contient jamais la graine ni les données cachées d'un autre siège ; `determinize` redistribue tout ce que l'observateur ne voit pas. La conformance le vérifie.
- **P2P, l'hôte fait autorité** (`web/src/session/`) : un invité ne reçoit que `updateFor(snapshot, son siège)`. Incrémenter `PROTOCOL_VERSION` (`protocol.ts`) à tout changement du protocole.
- Changer un type sérialisé d'un jeu impose de mettre à jour `web/src/games/<id>/types.ts` dans le même changement.
- **Règles** : chaque jeu a un `RULES.md` qui cite sa source ([R]…) ; un point non tranché est marqué ❓. Toute modification de règle met à jour `RULES.md`, le moteur et un test ensemble.

## Ajouter un jeu
1. `games/<id>/` : implémenter `Game`, `Determinize`, `Heuristics` ; `RULES.md` sourcé ; tests de règles ; `tests/conformance.rs` qui appelle `ludo_core::conformance::check::<G>()` pour chaque nombre de joueurs.
2. `crates/ludo-registry/src/lib.rs` : un `static` et une entrée dans `GAMES` ; ajouter la dépendance dans son `Cargo.toml` et dans le workspace.
3. `web/src/games/<id>/types.ts` + `Board.tsx`, et une entrée dans `web/src/games/registry.ts`.
4. `ludo-cli arena <id> …` pour vérifier que l'IA tient la route.
```

- [ ] **Step 2: `README.md`**

~~~markdown
# ludo-rs

Jeux traditionnels dans le navigateur : contre l'IA, sur le même écran, ou en ligne entre amis (P2P, sans serveur de jeu).
Le moteur est en Rust, compilé en WebAssembly ; le site est en React.

Jeux : **Yavalath** (2–3 joueurs, Cameron Browne). Prochain : Mahjong Riichi.

## Lancer en local
```bash
cargo install wasm-bindgen-cli --version 0.2.121 --locked   # une fois
./scripts/build-wasm.sh dev
cd web && pnpm install && pnpm dev
```

## Architecture
- `crates/ludo-core` : le contrat `Game`, le journal de partie (`MatchLog`, `replay`), l'IA générique (ISMCTS), la conformance.
- `games/<jeu>` : les règles d'un jeu, sa vue, ses heuristiques ; `RULES.md` cite les sources.
- `crates/ludo-registry` : la liste des jeux. `crates/ludo-wasm` : le pont vers le navigateur. `crates/ludo-cli` : arène d'IA et partie en terminal.
- `web/` : accueil, lobby, sessions (locale, hôte, invité), un plateau par jeu.

En ligne, l'hôte fait tourner le moteur ; chaque invité, connecté à l'hôte seul via PeerJS, ne reçoit que ce que son siège peut voir.

Voir `docs/superpowers/specs/` pour la conception et `CLAUDE.md` pour les règles du dépôt et la marche à suivre pour ajouter un jeu.

## Licence
MIT.
~~~

- [ ] **Step 3: CI**

`.github/workflows/ci.yml` :
```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:

jobs:
  rust:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --all --check
      - run: cargo clippy --workspace --all-targets -- -D warnings
      - run: cargo test --workspace --release

  web:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
      - uses: Swatinem/rust-cache@v2
      # Must match the wasm-bindgen version in crates/ludo-wasm/Cargo.toml exactly.
      - name: Cache wasm-bindgen-cli
        id: wasm-bindgen
        uses: actions/cache@v4
        with:
          path: ~/.cargo/bin/wasm-bindgen
          key: wasm-bindgen-cli-0.2.121
      - name: Install wasm-bindgen-cli
        if: steps.wasm-bindgen.outputs.cache-hit != 'true'
        run: cargo install wasm-bindgen-cli --version 0.2.121 --locked
      - run: ./scripts/build-wasm.sh
      - uses: pnpm/action-setup@v4
        with:
          version: 12
      - uses: actions/setup-node@v4
        with:
          node-version: 22
          cache: pnpm
          cache-dependency-path: web/pnpm-lock.yaml
      - name: Test and build the site
        working-directory: web
        run: |
          pnpm install --frozen-lockfile
          pnpm test
          pnpm build
```

`.github/workflows/pages.yml` :
```yaml
# Builds the site and publishes it on GitHub Pages.
name: Pages

on:
  push:
    branches: [main]
  workflow_dispatch:

permissions:
  contents: read
  pages: write
  id-token: write

concurrency:
  group: pages
  cancel-in-progress: true

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
      - uses: Swatinem/rust-cache@v2
      - name: Cache wasm-bindgen-cli
        id: wasm-bindgen
        uses: actions/cache@v4
        with:
          path: ~/.cargo/bin/wasm-bindgen
          key: wasm-bindgen-cli-0.2.121
      - name: Install wasm-bindgen-cli
        if: steps.wasm-bindgen.outputs.cache-hit != 'true'
        run: cargo install wasm-bindgen-cli --version 0.2.121 --locked
      - run: ./scripts/build-wasm.sh
      - uses: pnpm/action-setup@v4
        with:
          version: 12
      - uses: actions/setup-node@v4
        with:
          node-version: 22
          cache: pnpm
          cache-dependency-path: web/pnpm-lock.yaml
      - name: Build the site
        working-directory: web
        run: |
          pnpm install --frozen-lockfile
          pnpm build
      - uses: actions/upload-pages-artifact@v3
        with:
          path: web/dist

  deploy:
    needs: build
    runs-on: ubuntu-latest
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - id: deployment
        uses: actions/deploy-pages@v4
```

- [ ] **Step 4: Reporter les écarts dans la spec**

Dans `docs/superpowers/specs/2026-09-26-plateforme-yavalath-design.md`, ajouter à la fin :
```markdown
## 10. Ajustements faits pendant la planification

1. **Récompense IA** : part des autres sièges devancés, une égalité comptant pour moitié (un nul vaut 0,5 à 2 joueurs, pas 1).
2. **`ai_move`** prend un niveau (`Easy`/`Medium`/`Hard`) plutôt qu'un budget en ms, car le niveau fixe aussi la probabilité d'erreur. Budgets CLI : 100 / 1000 / 3000 itérations.
3. **Conformance** : le refus d'une action hors liste est testé génériquement en faisant jouer un siège qui n'est pas en attente ; les coups hors liste propres à un jeu relèvent de ses tests de règles.
4. **`Hello`** ne porte pas `game_id` : l'invité ne connaît que le code ; l'hôte envoie le jeu dans `Lobby`, et l'invité refuse un jeu absent de son registre.
5. **Siège « Fermé »** : réalisé par le choix du nombre de joueurs dans le lobby.
6. **Frontière WASM** : toutes les données passent en chaînes JSON.
7. **`Heuristics::evaluate`** renvoie `Vec<f64>`.
```

- [ ] **Step 5: Vérification finale**

Run: `cd ~/prog/ludo-rs && cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --release && ./scripts/build-wasm.sh && cd web && pnpm test && pnpm build`
Expected: tout passe.

- [ ] **Step 6: Commit**

```bash
cd ~/prog/ludo-rs && git add -A && git commit -m "Docs, CI and GitHub Pages workflow; record planning adjustments in the spec

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

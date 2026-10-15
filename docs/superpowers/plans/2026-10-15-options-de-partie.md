# Options de partie (spec 4, plan 1) — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Un jeu peut décrire ses options (interrupteur, choix, nombre) ; le moteur les valide ; le lobby les affiche, l'hôte les règle et les invités les voient ; Ghost Town (nombre de mains) en est le premier utilisateur.

**Architecture:** `ludo-core` gagne `GameOption`, `Game::options()` et `Game::check_config()` (appelé par `Match::new`), une vérification de conformance générique et `GameInfo` (métadonnées + options) pour `listGames()`. Le site construit un formulaire générique depuis ces options, avec des libellés par clé dans `web/src/games/options.ts` ; la config voyage dans `SessionState`, le message `lobby` et la sauvegarde de l'hôte.

**Tech Stack:** Rust stable (serde, serde_json), wasm-bindgen 0.2.121, React 19 + TypeScript, vitest 3.

**Spec:** `docs/superpowers/specs/2026-10-15-president-design.md`, §5 (options de partie), §7 (tests) et §8 (livraison, plan 1).

## Global Constraints

- Branche `options-de-partie` (une autre session travaille sur `main` : ne rien modifier hors des fichiers listés par tâche). `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --release` avant chaque commit Rust ; `cd web && pnpm test && pnpm type-check` avant chaque commit web.
- Le moteur possède toute la logique : le site affiche les options reçues et envoie la config ; il ne valide rien, `new_match` refuse une config hors bornes.
- `apply` et `new_match` renvoient une erreur sans paniquer. `Match::new` reste le seul point d'entrée (création comme rejeu d'un journal).
- PeerJS : ne jamais envoyer `undefined`, lire `null` comme absent. Tout changement du protocole incrémente `PROTOCOL_VERSION`.
- Messages d'erreur et textes du site en français ; code et commentaires en anglais.
- Commits avec l'adresse `marc.camuzat@gmail.com` (config locale du dépôt), terminés par `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.

## Review Focus

1. **Journal de Ghost Town écrit avant ce plan** (`"config":{"hands":null}`) : il se rejoue avec 5 mains. → `an_old_log_with_null_hands_replays_with_five`.
2. **Config hors bornes envoyée au moteur** (`{"hands":0}`, `{"hands":16}`, champ inconnu) : `new_match` refuse avec un message lisible, rien n'est créé. → `a_config_out_of_bounds_is_refused` (Ghost Town) et `check_config_guards_new_matches` (cœur).
3. **Option annoncée qui ne correspond pas au `Config`** (mauvais défaut, clé absente, valeur de choix non désérialisable) : la conformance échoue avec un message qui nomme l'option. → `wrong_option_defaults_are_caught`.
4. **Sauvegarde d'hôte antérieure** (sans champ `config`) : la reprise fonctionne, `config` vaut `null`. → `resumes_a_save_without_config` (`savedConfig`, utilisé par `HostSession.resume`).
5. **Jeu sans options** (Yavalath, tous les autres Pairs) : lobby, salle d'attente et table inchangés, aucune section « Options ». → `renders_nothing_without_options`.

---

### Task 1: Options et validation de config dans `ludo-core`

**Files:**
- Modify: `crates/ludo-core/src/game.rs` (types `GameOption`, `OptionKind` ; méthodes `options`, `check_config` du trait `Game`)
- Modify: `crates/ludo-core/src/log.rs` (`Match::new` appelle `check_config`)
- Modify: `crates/ludo-core/src/any.rs` (`GameInfo`, `AnyGame::options`, `AnyGame::info`)
- Modify: `crates/ludo-core/src/lib.rs` (réexportations)
- Modify: `crates/ludo-core/src/testgames.rs` (Nim déclare son option `pile`)

**Interfaces:**
- Produces:
  ```rust
  // ludo_core
  pub struct GameOption { pub key: &'static str, pub kind: OptionKind }
  pub enum OptionKind {
      Toggle { default: bool },
      Choice { values: &'static [&'static str], default: &'static str },
      Number { min: u32, max: u32, default: u32 },
  }
  // trait Game
  fn options() -> Vec<GameOption> { Vec::new() }
  fn check_config(_config: &Self::Config, _players: u8) -> Result<(), Illegal> { Ok(()) }
  // trait AnyGame
  fn options(&self) -> Vec<GameOption>;
  fn info(&self) -> GameInfo; // provided
  pub struct GameInfo { #[serde(flatten)] pub meta: GameMeta, pub options: Vec<GameOption> }
  ```
  JSON d'une option : `{"key":"hands","kind":{"type":"number","min":1,"max":15,"default":5}}`.

- [ ] **Step 1: Write the failing tests**

Dans `crates/ludo-core/src/log.rs`, module `tests` (à côté du test existant qui crée `Match::<Nim>::new(NimConfig { pile: 5 }, 2, 42)`) :

```rust
    #[test]
    fn check_config_guards_new_matches() {
        assert!(Match::<Nim>::new(NimConfig { pile: 30 }, 2, 1).is_ok());
        for pile in [0, 31] {
            let e = Match::<Nim>::new(NimConfig { pile }, 2, 1).err().expect("refused");
            assert_eq!(e.0, "Nim se joue avec un tas de 1 à 30.");
        }
    }
```

Dans `crates/ludo-core/src/any.rs`, module `tests` :

```rust
    #[test]
    fn info_lists_meta_and_options() {
        let info = serde_json::to_value(find(GAMES, "nim").unwrap().info()).unwrap();
        assert_eq!(info["id"], json!("nim"));
        assert_eq!(info["minPlayers"], json!(2));
        assert_eq!(
            info["options"],
            json!([{ "key": "pile", "kind": { "type": "number", "min": 1, "max": 30, "default": 10 } }])
        );
    }

    #[test]
    fn new_match_refuses_a_config_out_of_bounds() {
        let game = find(GAMES, "nim").unwrap();
        assert!(matches!(
            game.new_match(json!({ "pile": 0 }), 2, 1),
            Err(LudoError::Illegal(_))
        ));
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p ludo-core --release`
Expected: compilation error (`info`, `options` absents) — c'est l'échec attendu.

- [ ] **Step 3: Implement**

`crates/ludo-core/src/game.rs`, après `GameMeta` :

```rust
/// A setting the lobby offers for a game. `key` is the name of a field of the
/// game's `Config`; `kind` gives the values it takes and its default, which
/// must be the field's value in `Config::default()` (checked by conformance).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct GameOption {
    pub key: &'static str,
    pub kind: OptionKind,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum OptionKind {
    Toggle { default: bool },
    /// `values` are the serialized names of an enum's variants.
    Choice {
        values: &'static [&'static str],
        default: &'static str,
    },
    Number { min: u32, max: u32, default: u32 },
}
```

Dans le trait `Game`, après `visible_pending` :

```rust
    /// The settings the lobby offers. Default: none.
    fn options() -> Vec<GameOption> {
        Vec::new()
    }

    /// Refuses a config the game cannot be played with (a value out of the
    /// bounds `options` announces). Called by `Match::new`, so by every new
    /// match and every replayed log.
    fn check_config(_config: &Self::Config, _players: u8) -> Result<(), Illegal> {
        Ok(())
    }
```

`crates/ludo-core/src/log.rs`, dans `Match::new`, juste après la vérification du nombre de joueurs :

```rust
        G::check_config(&config, players)?;
```

`crates/ludo-core/src/any.rs` : importer `GameOption` et `serde::Serialize` est déjà importé ; ajouter avant `pub trait AnyGame` :

```rust
/// What the lobby needs to know of a game: its metadata and its options.
#[derive(Clone, Debug, Serialize)]
pub struct GameInfo {
    #[serde(flatten)]
    pub meta: GameMeta,
    pub options: Vec<GameOption>,
}
```

Dans `pub trait AnyGame`, après `fn meta(&self) -> GameMeta;` :

```rust
    fn options(&self) -> Vec<GameOption>;

    fn info(&self) -> GameInfo {
        GameInfo {
            meta: self.meta(),
            options: self.options(),
        }
    }
```

Dans `impl<G: Game + Determinize + Heuristics> AnyGame for GameEntry<G>` :

```rust
    fn options(&self) -> Vec<GameOption> {
        G::options()
    }
```

`crates/ludo-core/src/lib.rs` :

```rust
pub use any::{find, AnyGame, AnyMatch, GameEntry, GameInfo, LudoError};
pub use game::{
    Determinize, Game, GameMeta, GameOption, Heuristics, Illegal, OptionKind, Outcome, Pending,
    Seat, Viewer,
};
```

`crates/ludo-core/src/testgames.rs` : importer `GameOption, OptionKind` depuis `crate`, puis dans `impl Game for Nim`, après `meta()` :

```rust
    fn options() -> Vec<GameOption> {
        vec![GameOption {
            key: "pile",
            kind: OptionKind::Number {
                min: 1,
                max: 30,
                default: 10,
            },
        }]
    }

    fn check_config(config: &NimConfig, _players: u8) -> Result<(), Illegal> {
        if (1..=30).contains(&config.pile) {
            Ok(())
        } else {
            Err(Illegal::new("Nim se joue avec un tas de 1 à 30."))
        }
    }
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p ludo-core --release`
Expected: PASS, dont les trois nouveaux tests.

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --release
git add crates/ludo-core
git commit -m "core: options de partie et validation de la config

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: Conformance des options

**Files:**
- Modify: `crates/ludo-core/src/conformance.rs` (`check_options`, appelée au début de `check`)
- Modify: `crates/ludo-core/src/testgames.rs` (jeu de test `BadOptions`)

**Interfaces:**
- Consumes: `Game::options`, `Game::check_config`, `OptionKind` (Task 1).
- Produces: `pub fn check_options<G: Game>(players: u8)` dans `ludo_core::conformance`. Tous les jeux qui appellent déjà `check` en profitent sans changement.

- [ ] **Step 1: Write the failing tests**

`crates/ludo-core/src/testgames.rs`, après `BadClock` et son `impl Determinize` :

```rust
/// Nim announcing a default pile that `NimConfig::default()` does not have.
pub struct BadOptions;

impl Game for BadOptions {
    type State = NimState;
    type Action = u8;
    type View = NimState;
    type Config = NimConfig;

    fn meta() -> GameMeta {
        GameMeta {
            id: "badoptions",
            ..Nim::meta()
        }
    }
    fn options() -> Vec<GameOption> {
        vec![GameOption {
            key: "pile",
            kind: OptionKind::Number {
                min: 1,
                max: 30,
                default: 7,
            },
        }]
    }
    fn new(config: &NimConfig, players: u8, seed: u64) -> NimState {
        Nim::new(config, players, seed)
    }
    fn pending(s: &NimState) -> Pending {
        Nim::pending(s)
    }
    fn legal_actions(s: &NimState, seat: Seat) -> Vec<u8> {
        Nim::legal_actions(s, seat)
    }
    fn apply(s: &mut NimState, seat: Seat, a: &u8) -> Result<(), Illegal> {
        Nim::apply(s, seat, a)
    }
    fn view(s: &NimState, viewer: Viewer) -> NimState {
        Nim::view(s, viewer)
    }
    fn outcome(s: &NimState) -> Option<Outcome> {
        Nim::outcome(s)
    }
}

impl Determinize for BadOptions {
    fn determinize<R: Rng + ?Sized>(s: &NimState, _observer: Seat, _rng: &mut R) -> NimState {
        s.clone()
    }
}
```

`crates/ludo-core/src/conformance.rs`, module `tests` :

```rust
    #[test]
    #[should_panic(expected = "option pile")]
    fn wrong_option_defaults_are_caught() {
        check::<crate::testgames::BadOptions>(&NimConfig::default(), 2, 1, 5);
    }

    #[test]
    fn nim_options_conform() {
        super::check_options::<Nim>(2);
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p ludo-core --release conformance`
Expected: FAIL — `check_options` absent (compilation), puis, une fois déclarée vide, `wrong_option_defaults_are_caught` ne panique pas.

- [ ] **Step 3: Implement**

`crates/ludo-core/src/conformance.rs` : ajouter `GameOption`, `OptionKind` aux imports depuis `crate`, `serde_json::json` aux imports, puis :

```rust
/// The options the lobby offers agree with the game's `Config`: each one
/// names a field whose `Config::default()` value is the announced default,
/// every offered value is a valid config, and a number just outside its
/// bounds is refused by `check_config`.
pub fn check_options<G: Game>(players: u8) {
    let default = json(&G::Config::default());
    for GameOption { key, kind } in G::options() {
        let field = default
            .get(key)
            .unwrap_or_else(|| panic!("option {key} : aucun champ de ce nom dans Config"));
        let (announced, offered, outside) = match kind {
            OptionKind::Toggle { default } => (json!(default), vec![json!(false), json!(true)], vec![]),
            OptionKind::Choice { values, default } => (
                json!(default),
                values.iter().map(|v| json!(v)).collect(),
                vec![],
            ),
            OptionKind::Number { min, max, default } => (
                json!(default),
                vec![json!(min), json!(max)],
                [min.checked_sub(1), max.checked_add(1)]
                    .into_iter()
                    .flatten()
                    .map(|v| json!(v))
                    .collect(),
            ),
        };
        assert_eq!(
            field, &announced,
            "option {key} : le défaut annoncé diffère de Config::default()"
        );
        let with = |value: &Value| {
            let mut config = default.clone();
            config[key] = value.clone();
            serde_json::from_value::<G::Config>(config)
        };
        for value in &offered {
            let config = with(value)
                .unwrap_or_else(|e| panic!("option {key} : la valeur {value} n'est pas une config : {e}"));
            assert!(
                G::check_config(&config, players).is_ok(),
                "option {key} : la valeur proposée {value} est refusée"
            );
        }
        for value in &outside {
            // A value the field's type cannot hold is refused already.
            if let Ok(config) = with(value) {
                assert!(
                    G::check_config(&config, players).is_err(),
                    "option {key} : la valeur hors bornes {value} est acceptée"
                );
            }
        }
    }
}
```

Au début de `pub fn check`, avant la boucle :

```rust
    check_options::<G>(players);
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test --workspace --release`
Expected: PASS, toute la suite (les conformances de tous les jeux passent par `check_options`, sans option pour l'instant hors Nim).

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
git add crates/ludo-core
git commit -m "core: la conformance vérifie les options annoncées

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: `listGames()` avec options, `ludo arena --config`

**Files:**
- Modify: `crates/ludo-wasm/src/lib.rs:52-60` (`list_games`)
- Modify: `crates/ludo-cli/src/main.rs` (USAGE, analyse des arguments d'`arena`, `arena` reçoit la config)
- Modify: `CLAUDE.md` (« Commandes » : exemple `--config`)

**Interfaces:**
- Consumes: `AnyGame::info()` (Task 1).
- Produces: `listGames()` renvoie un tableau de `GameInfo` (champ `options` en plus) ; `ludo arena <jeu> <parties> <A> <B> [joueurs] [--config '<json>']`.

- [ ] **Step 1: Write the failing test**

`crates/ludo-cli/src/main.rs`, ajouter en fin de fichier :

```rust
#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::arena_extras;

    #[test]
    fn arena_reads_players_and_config_in_any_order() {
        assert_eq!(arena_extras(&[]).unwrap(), (2, serde_json::Value::Null));
        assert_eq!(arena_extras(&["4"]).unwrap(), (4, serde_json::Value::Null));
        assert_eq!(
            arena_extras(&["--config", r#"{"hands":3}"#, "3"]).unwrap(),
            (3, json!({ "hands": 3 }))
        );
        assert!(arena_extras(&["--config"]).is_err());
        assert!(arena_extras(&["--config", "{"]).is_err());
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p ludo-cli --release`
Expected: FAIL — `arena_extras` introuvable.

- [ ] **Step 3: Implement**

`crates/ludo-wasm/src/lib.rs`, `list_games` :

```rust
#[wasm_bindgen(js_name = listGames)]
pub fn list_games() -> String {
    text(
        &ludo_registry::GAMES
            .iter()
            .map(|g| g.info())
            .collect::<Vec<_>>(),
    )
}
```

`crates/ludo-cli/src/main.rs` :

- première ligne de doc et `USAGE` : `ludo arena <jeu> <parties> <easy|medium|hard> <easy|medium|hard> [joueurs] [--config '<json>']` ;
- bras `arena` :

```rust
        ["arena", game, games, a, b, rest @ ..] => {
            let (players, config) = arena_extras(rest)?;
            arena(game, parse(games)?, level(a)?, level(b)?, players, &config)
        }
```

- nouvelle fonction :

```rust
/// `[joueurs] [--config '<json>']`, in any order; 2 players and the default
/// config when absent.
fn arena_extras(rest: &[&str]) -> Result<(u8, Value), String> {
    let mut players = 2;
    let mut config = Value::Null;
    let mut args = rest.iter();
    while let Some(&arg) = args.next() {
        if arg == "--config" {
            let text = args
                .next()
                .ok_or_else(|| format!("--config attend un objet JSON\n{USAGE}"))?;
            config = serde_json::from_str(text).map_err(|e| format!("config invalide : {e}"))?;
        } else {
            players = parse(arg)?;
        }
    }
    Ok((players, config))
}
```

- `fn arena(id: &str, games: u32, a: Level, b: Level, players: u8, config: &Value)` ; dans `score`, `game.new_match(config.clone(), players, u64::from(g))`.

`CLAUDE.md`, section « Commandes », après la ligne Ghost Town :

```markdown
- `… arena <jeu> … [joueurs] --config '<json>'` : même mesure avec des options, par exemple `cargo run --release -p ludo-cli -- arena pairs-ghost-town 200 hard easy --config '{"hands":2}'`.
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test --workspace --release`, puis `cargo run --release -p ludo-cli -- arena pairs-ghost-town 4 easy easy --config '{"hands":2}'`
Expected: PASS ; l'arène affiche un score sans erreur.

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
git add crates/ludo-wasm crates/ludo-cli CLAUDE.md
git commit -m "wasm, cli: listGames avec options, arena --config

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: Ghost Town, premier jeu à options

**Files:**
- Modify: `games/pairs/src/ghost_town/mod.rs:24-33` (`Config`), `:410-440` (`options`, `check_config`, `new`)
- Modify: `games/pairs/tests/ghost_town_rules.rs:60-67` et `:323`
- Modify: `games/pairs/tests/conformance.rs:32-39`
- Modify: `games/pairs/RULES.md` (section Ghost Town : la durée se règle dans le lobby, 1 à 15 mains)

**Interfaces:**
- Consumes: `GameOption`, `OptionKind`, `Game::check_config` (Task 1), `check_options` via `check` (Task 2).
- Produces: `pairs::ghost_town::Config { pub hands: u8 }` (défaut `DEFAULT_HANDS` = 5) ; option `{"key":"hands","kind":{"type":"number","min":1,"max":15,"default":5}}`.

- [ ] **Step 1: Write the failing tests**

`games/pairs/tests/ghost_town_rules.rs`, remplacer `the_number_of_hands_comes_from_the_config_within_bounds` par :

```rust
#[test]
fn the_number_of_hands_comes_from_the_config() {
    let hands = |h| GhostTown::new(&Config { hands: h }, 2, 1).hands_total;
    assert_eq!(Config::default().hands, 5);
    assert_eq!(hands(2), 2);
    assert_eq!(hands(15), 15);
}

#[test]
fn a_config_out_of_bounds_is_refused() {
    use ludo_core::Match;
    for hands in [0, 16] {
        let e = Match::<GhostTown>::new(Config { hands }, 2, 1).err().expect("refused");
        assert_eq!(e.0, "Ghost Town se joue en 1 à 15 mains.");
    }
}

#[test]
fn an_old_log_with_null_hands_replays_with_five() {
    let config: Config = serde_json::from_value(serde_json::json!({ "hands": null })).unwrap();
    assert_eq!(config.hands, 5);
    let config: Config = serde_json::from_value(serde_json::json!({})).unwrap();
    assert_eq!(config.hands, 5);
}
```

(Si `serde_json` ou `ludo_core` manquent aux `[dev-dependencies]` de `games/pairs/Cargo.toml`, les y ajouter avec la même version que dans `[dependencies]`.)

Ligne 323 : `Config { hands: Some(2) }` → `Config { hands: 2 }`.

`games/pairs/tests/conformance.rs`, fonction `ghost_town_conforms_at_every_table_size` :

```rust
    check::<GhostTown>(&Config { hands: 1 }, 3, 100, 17);
```

et supprimer la ligne `check::<GhostTown>(&Config { hands: Some(255) }, 2, 5, 18);` (une config hors bornes est désormais refusée : c'est `a_config_out_of_bounds_is_refused` qui le couvre).

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p pairs --release ghost_town`
Expected: FAIL (compilation : `hands` est encore un `Option<u8>`).

- [ ] **Step 3: Implement**

`games/pairs/src/ghost_town/mod.rs` : importer `GameOption, OptionKind` depuis `ludo_core` et `serde::Deserializer`, puis remplacer `Config` :

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    /// Hands in a game, 1 to `MAX_HANDS`.
    #[serde(default = "default_hands", deserialize_with = "hands_or_default")]
    pub hands: u8,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hands: DEFAULT_HANDS,
        }
    }
}

fn default_hands() -> u8 {
    DEFAULT_HANDS
}

/// Logs written before the lobby offered options hold `null`: the default.
fn hands_or_default<'de, D: Deserializer<'de>>(d: D) -> Result<u8, D::Error> {
    Ok(Option::<u8>::deserialize(d)?.unwrap_or(DEFAULT_HANDS))
}
```

Dans `impl Game for GhostTown`, après `meta()` :

```rust
    fn options() -> Vec<GameOption> {
        vec![GameOption {
            key: "hands",
            kind: OptionKind::Number {
                min: 1,
                max: u32::from(MAX_HANDS),
                default: u32::from(DEFAULT_HANDS),
            },
        }]
    }

    fn check_config(config: &Config, _players: u8) -> Result<(), Illegal> {
        if (1..=MAX_HANDS).contains(&config.hands) {
            Ok(())
        } else {
            Err(Illegal::new(format!(
                "Ghost Town se joue en 1 à {MAX_HANDS} mains."
            )))
        }
    }
```

Dans `new` :

```rust
        // `check_config` keeps it in bounds; clamp anyway, `new` cannot fail.
        let hands_total = config.hands.clamp(1, MAX_HANDS);
```

`games/pairs/RULES.md`, section Ghost Town, point ❓ sur le nombre de mains : ajouter « Réglable dans le lobby, de 1 à 15 mains (5 par défaut). »

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test --workspace --release`
Expected: PASS, dont `ghost_town_conforms_at_every_table_size` (qui passe aussi par `check_options`).

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
git add games/pairs
git commit -m "ghost-town: nombre de mains réglable (option de partie)

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: Formulaire `GameOptions` et libellés

**Files:**
- Modify: `web/src/engine/types.ts` (`OptionKind`, `GameOption`, `GameConfig`, `GameMeta.options`)
- Create: `web/src/games/options.ts` (libellés, config par défaut, résumé)
- Create: `web/src/shell/GameOptions.tsx`
- Create: `web/src/shell/__tests__/gameOptions.test.tsx`
- Modify: `web/src/shell/__tests__/home.test.tsx:7-9` (`options: []` dans `meta`)

**Interfaces:**
- Consumes: JSON de `listGames()` (Task 3).
- Produces:
  ```ts
  export type OptionKind =
    | { type: "toggle"; default: boolean }
    | { type: "choice"; values: string[]; default: string }
    | { type: "number"; min: number; max: number; default: number };
  export interface GameOption { key: string; kind: OptionKind }
  export type GameConfig = Record<string, boolean | string | number>;
  // GameMeta gains: options: GameOption[]
  // web/src/games/options.ts
  export function defaultConfig(meta: GameMeta): GameConfig | null;
  export function optionSummary(meta: GameMeta, config: GameConfig): string[];
  // web/src/shell/GameOptions.tsx
  export function GameOptions(props: { meta: GameMeta; config: GameConfig | null; onChange?(config: GameConfig): void }): JSX.Element | null;
  ```

- [ ] **Step 1: Write the failing tests**

`web/src/shell/__tests__/gameOptions.test.tsx` :

```tsx
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import type { GameMeta } from "../../engine/types";
import { defaultConfig, optionSummary } from "../../games/options";
import { GameOptions } from "../GameOptions";

const GHOST: GameMeta = {
  id: "pairs-ghost-town",
  name: "Ghost Town",
  minPlayers: 2,
  maxPlayers: 6,
  hiddenInfo: true,
  clock: false,
  options: [{ key: "hands", kind: { type: "number", min: 1, max: 15, default: 5 } }],
};

const MIXED: GameMeta = {
  ...GHOST,
  id: "test-mixed",
  options: [
    { key: "fast", kind: { type: "toggle", default: false } },
    { key: "mode", kind: { type: "choice", values: ["Off", "On"], default: "Off" } },
  ],
};

describe("game options", () => {
  it("builds the default config from the engine's options", () => {
    expect(defaultConfig(GHOST)).toEqual({ hands: 5 });
    expect(defaultConfig({ ...GHOST, options: [] })).toBeNull();
  });

  it("summarizes a config with the game's labels", () => {
    expect(optionSummary(GHOST, { hands: 3 })).toEqual(["Mains : 3"]);
    expect(optionSummary(MIXED, { fast: true, mode: "On" })).toEqual(["fast : oui", "mode : On"]);
  });

  it("offers every value of a number option, the current one selected", () => {
    const html = renderToStaticMarkup(<GameOptions meta={GHOST} config={{ hands: 3 }} onChange={() => {}} />);
    expect(html).toContain("Mains");
    expect(html.match(/<option /g)?.length).toBe(15);
    expect(html).toContain('<option value="3" selected="">3</option>');
  });

  it("renders toggles and choices", () => {
    const html = renderToStaticMarkup(<GameOptions meta={MIXED} config={{ fast: true, mode: "On" }} onChange={() => {}} />);
    expect(html).toContain('type="checkbox"');
    expect(html).toContain("checked");
    expect(html).toContain('<option value="On" selected="">On</option>');
  });

  it("is read-only without onChange", () => {
    const html = renderToStaticMarkup(<GameOptions meta={GHOST} config={{ hands: 3 }} />);
    expect(html).not.toContain("<select");
    expect(html).toContain("Mains : 3");
  });

  it("renders_nothing_without_options", () => {
    expect(renderToStaticMarkup(<GameOptions meta={{ ...GHOST, options: [] }} config={null} onChange={() => {}} />)).toBe("");
    expect(renderToStaticMarkup(<GameOptions meta={GHOST} config={null} />)).toBe("");
  });
});
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd web && pnpm test -- gameOptions`
Expected: FAIL (modules `../../games/options` et `../GameOptions` introuvables).

- [ ] **Step 3: Implement**

`web/src/engine/types.ts`, avant `GameMeta` :

```ts
/** A setting the lobby offers (`ludo_core::OptionKind`). */
export type OptionKind =
  | { type: "toggle"; default: boolean }
  | { type: "choice"; values: string[]; default: string }
  | { type: "number"; min: number; max: number; default: number };

export interface GameOption {
  /** A field of the game's `Config`. */
  key: string;
  kind: OptionKind;
}

/** A game's `Config`, as sent to `newMatch`. */
export type GameConfig = Record<string, boolean | string | number>;
```

et dans `GameMeta` : `options: GameOption[];`.

`web/src/games/options.ts` :

```ts
import type { GameConfig, GameMeta } from "../engine/types";

/** Lobby labels by `<game id>.<key>`, and `<game id>.<key>.<value>` for choices. Missing: the key itself. */
const LABELS: Record<string, string> = {
  "pairs-ghost-town.hands": "Mains",
};

export function optionLabel(gameId: string, key: string): string {
  return LABELS[`${gameId}.${key}`] ?? key;
}

export function valueLabel(gameId: string, key: string, value: string): string {
  return LABELS[`${gameId}.${key}.${value}`] ?? value;
}

/** The engine's defaults, or null for a game without options. */
export function defaultConfig(meta: GameMeta): GameConfig | null {
  if (meta.options.length === 0) return null;
  return Object.fromEntries(meta.options.map((o) => [o.key, o.kind.default]));
}

/** One "Label : valeur" line per option, for whoever cannot change them. */
export function optionSummary(meta: GameMeta, config: GameConfig): string[] {
  return meta.options.map(({ key }) => {
    const value = config[key];
    const shown = typeof value === "boolean" ? (value ? "oui" : "non") : valueLabel(meta.id, key, String(value));
    return `${optionLabel(meta.id, key)} : ${shown}`;
  });
}
```

`web/src/shell/GameOptions.tsx` :

```tsx
import type { GameConfig, GameMeta, GameOption } from "../engine/types";
import { optionLabel, optionSummary, valueLabel } from "../games/options";

interface Props {
  meta: GameMeta;
  config: GameConfig | null;
  /** Absent: the options are only shown (guest, table). */
  onChange?(config: GameConfig): void;
}

/** A game's options, from what the engine announces: editable in the lobby, a summary elsewhere. */
export function GameOptions({ meta, config, onChange }: Props) {
  if (meta.options.length === 0 || config === null) return null;
  if (!onChange) {
    return (
      <ul className="options-summary">
        {optionSummary(meta, config).map((line) => (
          <li key={line}>{line}</li>
        ))}
      </ul>
    );
  }
  const set = (key: string, value: boolean | string | number) => onChange({ ...config, [key]: value });
  return (
    <div className="options">
      {meta.options.map((option) => (
        <div className="field" key={option.key}>
          <label htmlFor={`option-${option.key}`}>{optionLabel(meta.id, option.key)}</label>
          <OptionInput gameId={meta.id} option={option} value={config[option.key]} set={(v) => set(option.key, v)} />
        </div>
      ))}
    </div>
  );
}

function OptionInput({
  gameId,
  option,
  value,
  set,
}: {
  gameId: string;
  option: GameOption;
  value: boolean | string | number | undefined;
  set(value: boolean | string | number): void;
}) {
  const id = `option-${option.key}`;
  const kind = option.kind;
  switch (kind.type) {
    case "toggle":
      return <input id={id} type="checkbox" checked={value === true} onChange={(e) => set(e.target.checked)} />;
    case "choice":
      return (
        <select id={id} value={String(value)} onChange={(e) => set(e.target.value)}>
          {kind.values.map((v) => (
            <option key={v} value={v}>
              {valueLabel(gameId, option.key, v)}
            </option>
          ))}
        </select>
      );
    case "number":
      // Every value in bounds is listed: nothing to validate here.
      return (
        <select id={id} value={String(value)} onChange={(e) => set(Number(e.target.value))}>
          {Array.from({ length: kind.max - kind.min + 1 }, (_, k) => kind.min + k).map((n) => (
            <option key={n} value={n}>
              {n}
            </option>
          ))}
        </select>
      );
  }
}
```

`web/src/shell/__tests__/home.test.tsx`, helper `meta` : ajouter `options: []` à l'objet renvoyé.

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cd web && pnpm test && pnpm type-check`
Expected: PASS. `type-check` signale tout autre littéral `GameMeta` sans `options` : y ajouter `options: []`.

- [ ] **Step 5: Commit**

```bash
git add web/src/engine/types.ts web/src/games/options.ts web/src/shell/GameOptions.tsx web/src/shell/__tests__
git commit -m "web: formulaire d'options de partie construit depuis le moteur

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 6: La config de la partie, du lobby à la table

**Files:**
- Modify: `web/src/session/session.ts` (`SessionState.config`)
- Modify: `web/src/session/local.ts` (config dans l'état)
- Modify: `web/src/session/host.ts` (config : constructeur, `start`, `restart`, message `lobby`, sauvegarde, reprise)
- Modify: `web/src/session/guest.ts` (lit la config du message `lobby`)
- Modify: `web/src/session/protocol.ts` (`PROTOCOL_VERSION = 3`, `lobby.config`)
- Modify: `web/src/session/storage.ts` (`SavedHost.config?`)
- Modify: `web/src/shell/Lobby.tsx` (panneau « Options », config transmise à `onLocal` / `onHost`)
- Modify: `web/src/App.tsx` (passe la config aux sessions)
- Modify: `web/src/shell/TableScreen.tsx` (résumé : salle d'attente de l'invité, de l'hôte, et table)
- Modify: `web/src/shell/HostLobby.tsx` (résumé en lecture seule)
- Test: `web/src/session/__tests__/host.test.ts`, `web/src/session/__tests__/guest.test.ts`
- Create: `web/src/session/__tests__/storage.test.ts`

**Interfaces:**
- Consumes: `GameConfig`, `GameOptions`, `defaultConfig` (Task 5).
- Produces:
  ```ts
  // SessionState gains: config: GameConfig | null  (INITIAL: null)
  new LocalSession(gameId: string, seats: TableSeat[], config: GameConfig | null = null)
  new HostSession(gameId: string, hostName: string, others: HostChoice[], clock: ClockSettings | null, config: GameConfig | null, saved?: SavedHost)
  // HostMessage lobby: { type: "lobby"; gameId: string; seats: SeatInfo[]; started: boolean; config: GameConfig | null }
  // Lobby props: onLocal(seats, config), onHost?(choices, clock, config)
  ```

- [ ] **Step 1: Write the failing tests**

`web/src/session/__tests__/host.test.ts` : dans le test existant, `new HostSession("yavalath", "Ana", [{ kind: "open" }, { kind: "open" }], null)` devient `new HostSession("yavalath", "Ana", [{ kind: "open" }, { kind: "open" }], null, null)`. Ajouter :

```ts
  it("tells guests the game's options", () => {
    peers.length = 0;
    const host = new HostSession("pairs-ghost-town", "Ana", [{ kind: "open" }], null, { hands: 3 });
    peers[0].emit("open");
    const conn = new FakeConn();
    conn.open = true;
    peers[0].emit("connection", conn);
    conn.emit("data", { type: "hello", protocol: PROTOCOL_VERSION, name: "Bo" });
    expect(host.getState().config).toEqual({ hands: 3 });
    expect(conn.sent).toContainEqual(expect.objectContaining({ type: "lobby", config: { hands: 3 } }));
    host.dispose();
  });

```

`HostSession.resume` construit une `Table`, qui crée un vrai Worker (absent sous vitest) : la lecture de la config sauvée passe donc par un helper pur de `storage.ts`, testé dans un nouveau fichier `web/src/session/__tests__/storage.test.ts` :

```ts
import { describe, expect, it } from "vitest";
import { savedConfig, type SavedHost } from "../storage";

const SAVE: SavedHost = { code: "ABCDEF", gameId: "yavalath", seats: [], log: null, clock: null, banks: [] };

describe("savedConfig", () => {
  it("resumes_a_save_without_config", () => {
    expect(savedConfig(SAVE)).toBeNull();
    expect(savedConfig({ ...SAVE, config: null })).toBeNull();
    expect(savedConfig({ ...SAVE, config: { hands: 3 } })).toEqual({ hands: 3 });
  });
});
```

`web/src/session/__tests__/guest.test.ts`, dans le `describe` :

```ts
  it("shows the options the host chose", () => {
    const { session, conn } = connected();
    conn.emit("data", { type: "lobby", gameId: "pairs-ghost-town", seats: [], started: false, config: { hands: 3 } });
    expect(session.getState().config).toEqual({ hands: 3 });
    conn.emit("data", { type: "lobby", gameId: "yavalath", seats: [], started: false, config: null });
    expect(session.getState().config).toBeNull();
    session.dispose();
  });
```

(Si `FakeConn.emit("data", …)` n'est pas le chemin qu'utilise `guest.test.ts` pour livrer un message de l'hôte, suivre celui qu'il utilise.)

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd web && pnpm test -- session`
Expected: FAIL (`config` absent de l'état, mauvais nombre d'arguments).

- [ ] **Step 3: Implement**

`web/src/session/session.ts` : importer `GameConfig` ; ajouter à `SessionState`

```ts
  /** The game's options (null: a game without options). */
  config: GameConfig | null;
```

et `config: null,` dans `INITIAL`.

`web/src/session/local.ts` : `private config: GameConfig | null = null` (type au lieu de `unknown`), et `config,` dans le `this.set({...})` du constructeur.

`web/src/session/protocol.ts` : `PROTOCOL_VERSION = 3` ; importer `GameConfig` ; le membre `lobby` devient

```ts
  | { type: "lobby"; gameId: string; seats: SeatInfo[]; started: boolean; config: GameConfig | null }
```

`web/src/session/storage.ts`, `SavedHost` :

```ts
  /** Absent in saves made before game options. */
  config?: GameConfig | null;
```

et, après l'interface :

```ts
/** The options of a saved game; saves made before game options have none. */
export function savedConfig(saved: SavedHost): GameConfig | null {
  return saved.config ?? null;
}
```

`web/src/session/host.ts` :
- constructeur : nouveau paramètre `private config: GameConfig | null,` entre `clock` et `saved?` ; dans les deux branches, ajouter `config` au premier `this.set({...})` (`config: this.config`) ;
- `static resume(saved)` : `new HostSession(saved.gameId, "", [], saved.clock, savedConfig(saved), saved)` (import de `savedConfig` depuis `./storage`) ;
- `start()` et `restart()` : `this.table.start(randomSeed(), this.config)` ;
- `broadcastLobby()` : ajouter `config: this.config,` au message ;
- `persist()` : ajouter `config: this.config,` à l'objet sauvé.

`web/src/session/guest.ts`, cas `"lobby"` : ajouter `config: msg.config ?? null,` au `this.set({...})`.

`web/src/shell/Lobby.tsx` :
- importer `GameConfig`, `defaultConfig`, `GameOptions` ;
- props : `onLocal(seats: TableSeat[], config: GameConfig | null): void;` et `onHost?(choices: SeatChoice[], clock: ClockSettings | null, config: GameConfig | null): void;` ;
- état : `const [config, setConfig] = useState(() => defaultConfig(game));` ;
- entre le panneau « Sièges » et le panneau « Lancer » :

```tsx
        {config && (
          <section className="panel">
            <h3>Options</h3>
            <GameOptions meta={game} config={config} onChange={setConfig} />
          </section>
        )}
```

- appels : `onLocal(localSeats(), config)` et `onHost(others, …clock…, config)`.

`web/src/App.tsx`, cas `"lobby"` :

```tsx
          onLocal={(seats, config) => open(new LocalSession(screen.game.id, seats, config))}
          onHost={(choices, clock, config) =>
            open(new HostSession(screen.game.id, name, choices.map(hostChoice), clock, config))
          }
```

`web/src/shell/TableScreen.tsx` :
- importer `GameOptions` ;
- `GuestLobby` reçoit `meta` : `<GuestLobby state={state} meta={meta} />`, et affiche avant le `<p className="hint">` : `{meta && <GameOptions meta={meta} config={state.config} />}` ;
- `HostLobby` reçoit aussi `meta` (`<HostLobby session={session} state={state} meta={meta} />`) et affiche dans son panneau « Partie en ligne », avant les boutons : `{meta && <GameOptions meta={meta} config={state.config} />}` ;
- `Playing` reçoit `meta` et affiche, juste après `<RulesReminder … />` : `{meta && <GameOptions meta={meta} config={state.config} />}`.

`web/src/shell/HostLobby.tsx` : props `{ session, state, meta }: { session: HostSession; state: SessionState; meta: GameMeta | undefined }`, import de `GameMeta` et `GameOptions`.

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cd web && pnpm test && pnpm type-check`
Expected: PASS.

- [ ] **Step 5: Vérifier dans le navigateur**

Run: `./scripts/build-wasm.sh dev && cd web && pnpm build && pnpm preview`, puis, avec `playwright-core` et le Chromium de `~/.cache/ms-playwright` : ouvrir Ghost Town → le lobby montre « Options » avec « Mains » à 5 → choisir 2 → « Jouer sur cet appareil » → la table affiche « Mains : 2 » et la partie se termine après 2 mains. Ouvrir Yavalath → aucun panneau « Options ».
Expected: comme décrit ; aucune erreur dans la console.

- [ ] **Step 6: Commit**

```bash
git add web/src
git commit -m "web: options réglées dans le lobby, transmises aux invités et à la table

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

## Après le plan

Fusionner `options-de-partie` sur `main` quand CI et Pages sont verts, puis mettre à jour la spec (§9 « Ajustements faits au plan 1 », si l'implémentation a dévié) et rédiger le plan 2 (`games/cards`, Président).

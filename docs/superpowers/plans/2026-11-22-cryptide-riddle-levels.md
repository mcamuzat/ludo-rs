# Énigmes de Cryptide en quatre niveaux — plan de mise en œuvre

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal :** le générateur d'énigmes de Cryptide prend un niveau (Échauffement, Indice, Facile, Énigme) et l'écran d'entraînement le propose.

**Architecture :** tout se décide dans `games/cryptide/src/riddle.rs` : chaque niveau change la condition d'arrêt de la partie d'IA (ou s'en passe, à l'échauffement) et remplit l'explication de la solution. `crates/ludo-wasm` passe le niveau, `web/src/games/cryptide/Riddle.tsx` affiche l'énoncé du niveau et vérifie le clic.

**Tech Stack :** Rust (crate `cryptide`, `serde`), wasm-bindgen, React + Vitest.

**Spec :** `docs/superpowers/specs/2026-11-22-cryptide-riddle-levels-design.md`

## Global Constraints

- Le moteur possède toute la logique : l'écran ne calcule ni réponse ni explication.
- Les textes de l'écran Cryptide restent en français dans `Riddle.tsx` (jeu pas encore traduit) ; seuls les libellés de l'accueil passent par `web/src/i18n/fr.ts` et `en.ts`.
- Changer `Riddle` impose de mettre à jour `web/src/games/cryptide/types.ts` dans le même commit.
- Mélanges et tirages : `ChaCha8Rng`, jamais de `usize` tiré.
- Niveau 2 : seuls les jetons du joueur visé comptent (`knowledge::possible`).
- Niveaux en accès libre ; niveau par défaut de l'écran : Échauffement.
- Avant chaque commit : `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`.
- `RULES.md` de Cryptide, section « Énigmes (entraînement) », mise à jour avec le moteur.

## Review Focus

1. Niveau 3 introuvable (les parties finissent avant que chaque indice soit unique) : le générateur doit encore rendre une énigme pour 3, 4 et 5 joueurs dans les deux modes, en moins de 5 s → test `every_level_comes_quickly` (tâche 3), repli à 2 indices si besoin.
2. Énigme à 5 joueurs en mode avancé au niveau 4 : l'énumération des combinaisons ne doit ni geler l'écran ni gonfler la réponse → plafond et `complete` (tâche 4).
3. Mode avancé au niveau 2 : un indice et son contraire ont des libellés proches ; le choix compare l'indice entier (`clue` et `negated`) → test web (tâche 6).
4. Changer de niveau puis cliquer « Nouvelle énigme » : l'ancienne réponse et les essais ratés ne doivent pas rester → test web (tâche 6).
5. Échauffement : aucune case n'a de cube, toutes sont cliquables ; une case fausse est barrée comme aujourd'hui → test web (tâche 6).

---

### Tâche 1 : type `Riddle` à niveaux, Échauffement

**Files :**
- Modify : `games/cryptide/src/riddle.rs`, `games/cryptide/src/lib.rs` (réexporter les nouveaux types)
- Test : `games/cryptide/tests/riddle.rs`

**Interfaces :**
- Produces :
  ```rust
  #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
  pub enum RiddleLevel { Warmup, Hint, Easy, Full }

  pub struct Riddle {
      pub level: RiddleLevel,
      pub players: u8, pub mode: Mode, pub squares: Vec<Square>,
      pub tokens: Vec<Tokens>, pub log: Vec<Event>,
      /// Siège visé : celui dont on donne l'indice (Easy, Full), celui dont on
      /// cherche l'indice (Hint), Seat(0) à l'échauffement.
      pub solver: Seat,
      pub hint: Hint,              // = hints[solver]
      pub habitat: Cell,
      pub hints: Vec<Hint>,
      /// Indices montrés au joueur, par siège.
      pub given: Vec<Option<Hint>>,
      /// Par siège, les cases que garde son indice (solution de l'échauffement).
      pub hint_cells: Vec<Vec<Cell>>,
      pub deduction: Vec<Vec<Hint>>,   // vide par siège hors Easy/Full
      pub mode_hints: Vec<Hint>,
  }
  pub fn riddle(level: RiddleLevel, mode: Mode, players: u8, seed: u64) -> Option<Riddle>;
  ```
  La tâche 2 ajoute `eliminations: Vec<Elimination>` (vide hors Hint), la tâche 4 `combinations: Combinations` (`Default` hors Full).

- [ ] **Étape 1 : adapter les tests existants et écrire celui de l'échauffement**

Les trois tests actuels passent `RiddleLevel::Full` en premier argument. Nouveau test :

```rust
#[test]
fn a_warmup_shows_every_hint_and_no_token() {
    for mode in [Mode::Normal, Mode::Advanced] {
        for players in 3u8..=5 {
            let r = riddle(RiddleLevel::Warmup, mode, players, 1).expect("a warmup");
            assert!(r.tokens.iter().all(|t| *t == Tokens::default()));
            assert!(r.log.is_empty());
            assert_eq!(r.given, r.hints.iter().copied().map(Some).collect::<Vec<_>>());
            let board = board_of(&r);
            let table = HintTable::new(&board, mode);
            let all: Vec<usize> = r.hints.iter().map(|h| table.index_of(h).unwrap()).collect();
            assert_eq!(habitat(&table, &all), Some(r.habitat));
            for (p, cells) in r.hint_cells.iter().enumerate() {
                assert_eq!(*cells, r.hints[p].cells(&board).iter().collect::<Vec<_>>());
            }
        }
    }
}
```
Et dans `a_riddle_gives_the_solver_s_hint_and_what_it_knows` (niveau Full) : `given[solver] == Some(hint)`, les autres `None`.

- [ ] **Étape 2 : lancer, vérifier l'échec** — `cargo test --release -p cryptide --test riddle` : ne compile pas (`RiddleLevel` absent).

- [ ] **Étape 3 : implémenter** — l'échauffement appelle `clues::generate(mode, players, seed)` sans jouer, `tokens = vec![Tokens::default(); CELLS]`, `log` vide. Full garde le chemin actuel (`play` + `solver`). `riddle_of` reçoit le niveau et remplit `given` et `hint_cells` (`hint.cells(&board).iter()`). `Hint` et `Easy` peuvent renvoyer `None` jusqu'aux tâches 2 et 3.

- [ ] **Étape 4 : lancer, vérifier le succès** — même commande, tout passe.

- [ ] **Étape 5 : commit** — `Cryptide : niveaux d'énigme, échauffement`.

### Tâche 2 : niveau Indice (deviner l'indice d'un joueur)

**Files :** Modify `games/cryptide/src/riddle.rs`, `games/cryptide/src/lib.rs` ; Test `games/cryptide/tests/riddle.rs`

**Interfaces :**
- Consumes : `knowledge::possible(table, tokens, seat) -> Vec<usize>`, `marks` (rendre `pub(crate)` dans `knowledge.rs`).
- Produces :
  ```rust
  #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
  pub enum Mark { Disc, Cube }
  /// Un indice du mode que les jetons du siège visé écartent.
  #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
  pub struct Elimination { pub hint: Hint, pub cell: Cell, pub mark: Mark }
  ```

- [ ] **Étape 1 : test**

```rust
#[test]
fn a_hint_riddle_leaves_one_hint_to_the_target() {
    for mode in [Mode::Normal, Mode::Advanced] {
        for players in 3u8..=5 {
            for seed in 0..3u64 {
                let r = riddle(RiddleLevel::Hint, mode, players, seed).expect("a hint riddle");
                let board = board_of(&r);
                let table = HintTable::new(&board, mode);
                let target = r.solver;
                assert_eq!(possible(&table, &r.tokens, target), [table.index_of(&r.hint).unwrap()]);
                assert!(r.given.iter().all(Option::is_none));
                assert_eq!(r.eliminations.len(), r.mode_hints.len() - 1);
                for e in &r.eliminations {
                    assert_ne!(e.hint, r.hint);
                    let inside = e.hint.cells(&board).contains(e.cell);
                    match e.mark {
                        Mark::Disc => { assert!(r.tokens[e.cell.index()].discs.contains(&target)); assert!(!inside); }
                        Mark::Cube => { assert_eq!(r.tokens[e.cell.index()].cube, Some(target)); assert!(inside); }
                    }
                }
                assert!(!r.log.iter().any(|e| matches!(e, Event::Won { .. })));
            }
        }
    }
}
```

- [ ] **Étape 2 : lancer, vérifier l'échec** (`riddle` rend `None`).

- [ ] **Étape 3 : implémenter** — `play` prend une condition d'arrêt `fn(&State) -> Option<Stop>` (siège visé + `per_seat` éventuel). Pour `Hint` : hors `Setup`/`Over`, le premier siège depuis `s.turn` dont `possible(...).len() == 1`. Pour chaque autre indice du mode, une élimination : la première case de disque du siège hors de l'indice, sinon la première case de cube dans l'indice (ordre des cases).

- [ ] **Étape 4 : lancer, vérifier le succès.**

- [ ] **Étape 5 : commit** — `Cryptide : énigme « trouve l'indice d'un joueur »`.

### Tâche 3 : niveau Facile

**Files :** Modify `games/cryptide/src/riddle.rs` ; Test `games/cryptide/tests/riddle.rs`

**Interfaces :** Produces `const EASY_MAX: usize` (indices possibles par autre siège au plus, 1 d'abord).

- [ ] **Étape 1 : tests**

```rust
#[test]
fn an_easy_riddle_leaves_each_other_seat_few_hints() {
    for mode in [Mode::Normal, Mode::Advanced] {
        for players in 3u8..=5 {
            for seed in 0..3u64 {
                let r = riddle(RiddleLevel::Easy, mode, players, seed).expect("an easy riddle");
                for (p, list) in r.deduction.iter().enumerate() {
                    if p != r.solver.index() {
                        assert!(!list.is_empty() && list.len() <= EASY_MAX);
                        assert!(list.contains(&r.hints[p]));
                    }
                }
            }
        }
    }
}

#[test]
fn every_level_comes_quickly() {
    for level in [RiddleLevel::Warmup, RiddleLevel::Hint, RiddleLevel::Easy, RiddleLevel::Full] {
        let t = Instant::now();
        riddle(level, Mode::Advanced, 5, 3).expect("a riddle");
        assert!(t.elapsed().as_secs() < 5, "{level:?}: {:?}", t.elapsed());
    }
}
```
`EASY_MAX` est `pub const`, réexporté par `lib.rs`. `every_level_comes_quickly` remplace `a_riddle_comes_quickly` ; `a_riddle_is_pinned_by_its_seed` boucle sur les quatre niveaux.

- [ ] **Étape 2 : lancer, vérifier l'échec.**

- [ ] **Étape 3 : implémenter** — condition : le `solver()` actuel, en plus `per_seat` de chaque autre siège de longueur ≤ `EASY_MAX`.

- [ ] **Étape 4 : lancer avec `EASY_MAX = 1`.** Si `expect("an easy riddle")` échoue ou si `every_level_comes_quickly` dépasse 5 s, passer `EASY_MAX = 2` (repli prévu par la spec) et relancer. Noter dans le message de commit la valeur retenue et combien de graines sur 18 (2 modes × 3 tailles × 3 graines) trouvent une énigme à 1.

- [ ] **Étape 5 : commit** — `Cryptide : énigme facile`.

### Tâche 4 : combinaisons de l'énigme complète

**Files :** Modify `games/cryptide/src/riddle.rs` ; Test `games/cryptide/tests/riddle.rs`

**Interfaces :**
- Produces :
  ```rust
  #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
  pub enum Outcome { Habitat(Cell), Nowhere, Several(u8), Repeated }
  #[derive(Clone, Debug, PartialEq, Eq, Serialize)]
  pub struct Combination { pub hints: Vec<Hint>, pub outcome: Outcome } // par siège, résolveur compris
  #[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
  pub struct Combinations {
      pub working: Vec<Combination>,   // toutes celles qui désignent une case
      pub fallen: Vec<Combination>,    // les FALLEN_SHOWN premières tombées
      pub nowhere: u32, pub several: u32, pub repeated: u32,
      pub complete: bool,
  }
  pub const FALLEN_SHOWN: usize = 12;
  ```

- [ ] **Étape 1 : test**

```rust
#[test]
fn a_full_riddle_explains_its_combinations() {
    for mode in [Mode::Normal, Mode::Advanced] {
        for players in 3u8..=5 {
            for seed in 0..3u64 {
                let r = riddle(RiddleLevel::Full, mode, players, seed).unwrap();
                let c = &r.combinations;
                assert!(c.complete);
                assert!(!c.working.is_empty());
                assert!(c.working.iter().all(|w| w.outcome == Outcome::Habitat(r.habitat)));
                assert!(c.working.iter().any(|w| w.hints == r.hints));
                assert!(c.fallen.len() <= FALLEN_SHOWN);
                assert!(c.fallen.iter().all(|f| !matches!(f.outcome, Outcome::Habitat(_))));
                assert_eq!(c.fallen.len() as u32, (c.nowhere + c.several + c.repeated).min(FALLEN_SHOWN as u32));
            }
        }
    }
}
```

- [ ] **Étape 2 : lancer, vérifier l'échec.**

- [ ] **Étape 3 : implémenter** — produit cartésien, pour chaque autre siège, de `possible(table, tokens, p)` privé de l'indice du résolveur et des indices sans case commune avec lui (même filtre que `Query::walk`), le résolveur fixé. Issue : deux sièges au même indice → `Repeated` ; intersection vide → `Nowhere` ; plusieurs cases → `Several(n)` ; une → `Habitat`. Arrêt après `knowledge::CAP` combinaisons (`complete = false`). Rempli au seul niveau Full.

- [ ] **Étape 4 : lancer, vérifier le succès** (et `every_level_comes_quickly`).

- [ ] **Étape 5 : `RULES.md`** — section « Énigmes (entraînement) » : les quatre niveaux, leur condition d'arrêt, `EASY_MAX` retenu, ce que montre chaque solution. Date [U 2026-11-22].

- [ ] **Étape 6 : commit** — `Cryptide : combinaisons dans la solution de l'énigme`.

### Tâche 5 : pont WASM et types web

**Files :** Modify `crates/ludo-wasm/src/lib.rs:175-185`, `web/src/engine/types.ts:80`, `web/src/engine/clients.ts:49`, `web/src/engine/engine.worker.ts:56`, `web/src/games/cryptide/types.ts:71-84`

**Interfaces :**
- Produces : `cryptideRiddle(level: string, mode: string, players: number, seed: number)` côté wasm et `EngineClient.cryptideRiddle<T>(level, mode, players, seed)` ; types TS `RiddleLevel = "Warmup" | "Hint" | "Easy" | "Full"`, `Mark`, `Elimination`, `Outcome` (`{ Habitat: Cell } | "Nowhere" | { Several: number } | "Repeated"`), `Combination`, `Combinations`, et les nouveaux champs de `Riddle`.

- [ ] **Étape 1 :** la fonction wasm lit le niveau comme le mode (`serde_json::from_value`, erreur « Niveau inconnu : {level}. »).
- [ ] **Étape 2 :** `./scripts/build-wasm.sh`, puis `cd web && pnpm tsc --noEmit` : les seules erreurs sont dans `Riddle.tsx` et le test Cryptide (appel sans niveau, fixture incomplète) — corrigées en tâche 6.
- [ ] **Étape 3 :** commit avec la tâche 6 (le web ne compile pas entre les deux).

### Tâche 6 : écran des niveaux

**Files :** Modify `web/src/games/cryptide/Riddle.tsx`, `web/src/games/cryptide/HexMap.tsx`, `web/src/games/cryptide/cryptide.css`, `web/src/i18n/fr.ts`, `web/src/i18n/en.ts` ; Test `web/src/games/cryptide/__tests__/cryptide.test.tsx`

**Interfaces :**
- Consumes : types de la tâche 5.
- Produces : `HexMap` prend `shaded?: Set<Cell>` (cases hors de l'ensemble estompées, classe `cr-shaded`) ; `RiddleBoard` prend en plus `hintMisses: Hint[]` et `onGuessHint(h: Hint): void`.

Textes (français, dans `Riddle.tsx`) :
- Sélecteur `Niveau` : « 1 · Échauffement », « 2 · L'indice d'un joueur », « 3 · Énigme facile », « 4 · Énigme ».
- Énoncés : Warmup « Voici tous les indices. Trouve la seule case qui les vérifie tous. » ; Hint « Les jetons de {nom} ne laissent qu'un indice possible. Lequel ? » ; Easy/Full : l'énoncé actuel.
- Solution Warmup : la liste des indices, chacun un bouton qui estompe la carte hors de ses cases (`shaded = hint_cells[p]`).
- Solution Hint : l'indice juste, puis « {indice} : écarté par le disque de {nom} en {case} » / « … par le cube de {nom} en {case} » (`place(cell)`).
- Solution Full : « Combinaisons qui marchent » (une ligne par combinaison : les indices par siège → la case), puis « Tombées : {nowhere} sans case commune, {several} qui laissent plusieurs cases, {repeated} où deux joueurs auraient le même indice », suivi des exemples de `fallen`.
- Accueil, `t.home.riddlesHint` : fr « De l'échauffement à l'énigme, en quatre niveaux », en « From warm-up to full riddle, in four levels ».

- [ ] **Étape 1 : tests** (dans `describe("riddles")`, fixture `RIDDLE` complétée : `level: "Full"`, `given`, `hint_cells`, `eliminations: []`, `combinations` avec une combinaison qui marche et une tombée `"Nowhere"`) :
  - `"warmup lists every hint and lets every space be picked"` : niveau Warmup, `given` complet, tokens vides → chaque indice apparaît, toutes les cases ont la classe `playable`.
  - `"a hint riddle hides every hint and offers the mode's hints to pick"` : niveau Hint → aucun « Indice de », un bouton par `mode_hints` ; avec `hintMisses = [h]`, le bouton de `h` est désactivé.
  - `"a hint is matched whole, negation included"` : fixture Advanced où `hint` est négatif ; cliquer son contraire positif appelle `onGuessHint` avec l'indice positif (et `CryptideRiddles` le compte raté — tester la fonction de comparaison `sameHint(a, b)` exportée de `Riddle.tsx`).
  - `"the full solution shows the combinations"` : `solved = "shown"` → « Combinaisons qui marchent » et « 1 sans case commune ».
  - `"a new riddle clears the misses"` : tester le réducteur d'état de l'écran s'il est extrait ; sinon vérifier que `next()` remet `misses`, `hintMisses` et `solved` à zéro (extraire `freshAttempt()` pour le rendre testable).
- [ ] **Étape 2 : lancer** `cd web && pnpm test cryptide` : échec.
- [ ] **Étape 3 : implémenter** — `CryptideRiddles` garde `level` (défaut `"Warmup"`), le passe à `cryptideRiddle` ; au niveau Hint la carte n'a aucune case cliquable et la réponse se choisit dans une liste de boutons groupée comme `HintHelp` (familles), qui ne s'affiche pas en plus.
- [ ] **Étape 4 : lancer** `pnpm test` et `pnpm build` : tout passe.
- [ ] **Étape 5 : vérification navigateur** — `pnpm preview` et Playwright (`playwright-core` + Chromium de `~/.cache/ms-playwright`) : chaque niveau se génère et se résout ; capture de chacun dans le scratchpad.
- [ ] **Étape 6 : commit** (tâches 5 et 6) — `Cryptide : écran des énigmes en quatre niveaux`.

### Tâche 7 : vérification finale

- [ ] `cargo test --workspace --release`, `cargo clippy --workspace --all-targets -- -D warnings`, `cd web && pnpm test && pnpm build` : tout passe. Aucune arène à relancer (l'IA ne change pas).

# Siberian Dice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal :** ajouter Siberian Dice (règle de base, 2 joueurs) à ludo-rs, jouable contre l'IA et en ligne.

**Architecture :** une crate `games/siberian-dice` (règles, vue, IA), sur le modèle de `games/black-pot` (dés tirés par le moteur à partir d'une graine cachée), branchée dans `crates/ludo-registry` et affichée par `web/src/games/siberian-dice/` (plateau SVG hexagonal, sur le modèle de `web/src/games/yavalath/Board.tsx`).

**Tech Stack :** Rust (ludo-core, rand 0.8, rand_chacha, serde), React + TypeScript + Vitest.

**Spec :** `docs/superpowers/specs/2026-10-30-siberian-dice-design.md`

## Global Constraints

- Id du jeu `siberian-dice`, nom affiché « Siberian Dice », famille web `board`, exactement 2 joueurs.
- 12 dés par joueur ; 6 dés parfaits pour gagner ; limite : 30 tours de table (60 coups) sans nouveau record ⇒ **nul**.
- Un dé repris sur le plateau peut être reposé sur sa propre case [U].
- Les lancers viennent d'une graine `ChaCha8Rng` dans l'état, jamais dans une vue ; on tire un `u32` puis `% 6 + 1` (jamais `gen_range` sur `usize`, jamais `SmallRng`).
- `apply` renvoie `Illegal` sans modifier l'état et ne panique jamais.
- Le moteur calcule tout (dés parfaits, compteurs, limite) ; le web n'affiche que la vue.
- Le PDF source (`~/Téléchargements/siberian-dice.pnp.pdf`) n'est jamais copié dans le dépôt.
- Commits avec la config locale du dépôt (`marc.camuzat@gmail.com`), messages en français préfixés `siberian-dice : `, terminés par `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Avant chaque commit : `cargo fmt --all` et `cargo clippy --workspace --all-targets -- -D warnings`.

## Review Focus

1. Un dé repris sur le plateau : pendant la pose, sa case de départ est vide et ne compte plus comme voisine ; la reposer au même endroit est légal → test `a_die_can_go_back_to_the_cell_it_left` (Task 2).
2. Choisir un dé adverse, ou une case occupée, ou `Place` avant `Pick` → `Illegal`, état inchangé → test `illegal_moves_change_nothing` (Task 2).
3. Réserve vide : `Pick(Reserve)` refusé, et `timeout_action` prend un dé posé → tests `an_empty_reserve_cannot_be_picked_from` et `timeout_picks_a_board_die_when_the_reserve_is_empty` (Task 2).
4. Partie finie : plus personne n'est attendu, tout `apply` est refusé, et la vue nomme les gagnants (les deux en cas de nul) → test `after_the_end_nothing_is_pending` (Task 2).
5. Un invité qui reçoit la vue pendant l'étape « poser » voit la valeur lancée et la case libérée allumée → test web `shows_the_rolled_die_and_lights_the_vacated_cell` (Task 4).

---

### Task 1 : crate et géométrie du plateau

**Files :**
- Create : `games/siberian-dice/Cargo.toml` (copie de celui de `games/black-pot`, `name = "siberian-dice"`, `description = "Siberian Dice rules for ludo-rs"`)
- Create : `games/siberian-dice/src/lib.rs` (pour l'instant `pub mod board;`)
- Create : `games/siberian-dice/src/board.rs`
- Modify : `Cargo.toml` racine (membre `games/siberian-dice`, dépendance `siberian-dice = { path = "games/siberian-dice" }`)

**Interfaces :**
- Produces (dans `board.rs`) :
  - `pub type CellId = u8;`
  - `pub struct Cell { pub a: u8, pub b: u8 }` (`Clone, Copy, Debug, PartialEq, Eq`)
  - `pub fn cells() -> &'static [Cell]` : les 37 cases `a, b ∈ 1..=7`, `|a − b| ≤ 3`, triées par `(a, b)` ; l'index est le `CellId`.
  - `pub fn at(ab: u8) -> CellId` : la case de numéro `ab` de la page 3 (44 → le centre) ; panique si `ab` n'existe pas (tests seulement).
  - `pub fn label(id: CellId) -> u8` : `10 * a + b`.
  - `pub fn neighbors(id: CellId) -> &'static [CellId]` : parmi `(a−1,b−1) (a+1,b+1) (a,b−1) (a+1,b) (a,b+1) (a−1,b)`, celles qui existent. Calculé une fois (`OnceLock`).

- [ ] **Step 1 : tests en bas de `board.rs`**

```rust
#[test] fn thirty_seven_cells() { assert_eq!(cells().len(), 37); }
#[test] fn rows_of_the_printed_board() {
    // [R] page 3 : rangée du haut, du milieu, du bas.
    for ab in [41, 52, 63, 74, 11, 22, 33, 44, 55, 66, 77, 14, 25, 36, 47] { at(ab); }
    assert_eq!(label(at(44)), 44);
}
#[test] fn neighbors_of_center_edge_and_corner() {
    let n = |ab| { let mut v: Vec<u8> = neighbors(at(ab)).iter().map(|&c| label(c)).collect(); v.sort(); v };
    assert_eq!(n(44), vec![33, 34, 43, 45, 54, 55]);
    assert_eq!(n(11), vec![12, 21, 22]);
    assert_eq!(n(41), vec![31, 42, 52]);
    assert_eq!(n(22), vec![11, 12, 21, 23, 32, 33]);
}
#[test] fn adjacency_is_symmetric() { /* pour toute paire, c ∈ neighbors(d) ⇔ d ∈ neighbors(c) */ }
```

- [ ] **Step 2 :** `cargo test -p siberian-dice` → échec (fonctions absentes).
- [ ] **Step 3 :** implémenter `board.rs`.
- [ ] **Step 4 :** `cargo test -p siberian-dice` → PASS.
- [ ] **Step 5 :** commit `siberian-dice : plateau de 37 cases`.

### Task 2 : règles, vue, conformance

**Files :**
- Modify : `games/siberian-dice/src/lib.rs`
- Create : `games/siberian-dice/src/view.rs`, `games/siberian-dice/src/testing.rs`, `games/siberian-dice/src/heuristics.rs` (provisoire : `rollout_action` = action légale au hasard, remplacée en Task 3)
- Create : `games/siberian-dice/RULES.md`, `games/siberian-dice/tests/rules.rs`, `games/siberian-dice/tests/conformance.rs`

**Interfaces :**
- Consumes : `board::{cells, neighbors, at, label, CellId}`.
- Produces (dans `lib.rs`, tous `Serialize, Deserialize`, `Clone, Debug, PartialEq, Eq`) :
  - `pub struct SiberianDice;` ; `pub const DICE: u8 = 12; pub const TARGET: u8 = 6; pub const QUIET_ROUNDS: u32 = 30;`
  - `#[serde(default)] pub struct Config {}` (`Default`)
  - `pub struct Die { pub owner: u8, pub value: u8 }` (`Copy`)
  - `pub enum Source { Reserve, Cell(CellId) }` (`Copy, Hash`)
  - `pub enum Action { Pick(Source), Place(CellId) }` (`Hash`)
  - `pub enum Step { Choose, Place { from: Source, value: u8 } }`
  - `pub struct LastMove { pub seat: u8, pub from: Source, pub to: CellId, pub value: u8 }`
  - `pub enum End { Won { seat: u8 }, BothSix, QuietLimit }` (`Copy`)
  - `pub struct State { pub board: Vec<Option<Die>>, pub reserve: [u8; 2], pub turn: Seat, pub step: Step, pub records: [u8; 2], pub quiet: u32, pub last: Option<LastMove>, pub seed: u64, pub rolls: u32, pub end: Option<End> }`
  - `pub fn is_perfect(board: &[Option<Die>], id: CellId) -> bool`
  - `pub fn perfect_count(board: &[Option<Die>], seat: u8) -> u8`
  - `fn roll(seed: u64, k: u32) -> u8` : même schéma que `black_pot::roll` (graine `seed ^ k * 0x9E37_79B9_7F4A_7C15`, `gen::<u32>() % 6 + 1`).
  - `impl Game` : `meta()` = `{ id: "siberian-dice", name: "Siberian Dice", min_players: 2, max_players: 2, hidden_info: true, clock: true }` ; `check_config` refuse tout sauf 2 (« Siberian Dice se joue à 2. ») ; `outcome` = `Ranked([0,1])`/`[1,0]` pour `Won`, `[0,0]` pour `BothSix` et `QuietLimit`.
  - `impl Determinize` : copie avec `seed = rng.gen()`, comme Black-Pot.
- Produces (dans `testing.rs`) :
  - `pub fn position(dice: &[(u8, u8, u8)], turn: u8) -> State` : `(ab, owner, value)` posés, réserves = `DICE` − dés posés, `step = Choose`, `quiet = 0`, records = comptes de dés parfaits actuels.
  - `pub fn rolled(s: &mut State, from: Source, value: u8)` : simule un `Pick` qui a donné `value` (retire le dé de sa case ou de la réserve, `step = Place { from, value }`).
- Produces (dans `view.rs`, `#[serde(rename_all = "camelCase")]`) : `pub struct View { pub cells: Vec<CellView>, pub reserve: [u8; 2], pub perfect: [u8; 2], pub turn: u8, pub step: Step, pub last: Option<LastMove>, pub quiet_left: u32, pub end: Option<End>, pub winners: Vec<u8> }` ; `pub struct CellView { pub a: u8, pub b: u8, pub die: Option<DieView> }` ; `pub struct DieView { pub owner: u8, pub value: u8, pub perfect: bool }`. `quiet_left` = tours de table restants = `QUIET_ROUNDS − quiet / 2`. `winners` : vide en cours ; `[seat]` pour `Won` ; `[0, 1]` pour un nul.

Règles de `apply` (le reste suit la spec §3) :
- `Pick(Reserve)` exige `reserve[turn] > 0` ; `Pick(Cell(c))` exige un dé de `turn` en `c`. Le dé quitte sa case, `value = roll(seed, rolls)`, `rolls += 1`, `step = Place { from, value }`.
- `Place(c)` exige `step = Place` et `c` vide. Le dé est posé ; puis on compte (spec §3 Fin) : `p = [perfect_count(0), perfect_count(1)]` ; les deux ≥ 6 ⇒ `BothSix` ; une seule ⇒ `Won` ; sinon, si un `p[i] > records[i]` ⇒ records mis à jour, `quiet = 0`, sinon `quiet += 1`, et `quiet ≥ 2 * QUIET_ROUNDS` ⇒ `QuietLimit`. Le tour passe à l'autre siège, `step = Choose`.
- `legal_actions` : à l'étape `Choose`, `Pick(Reserve)` si la réserve n'est pas vide, puis un `Pick(Cell)` par dé posé du joueur ; à l'étape `Place`, un `Place` par case vide.
- `timeout_action` : premier élément de `legal_actions` (donc la réserve d'abord, puis le premier dé posé, puis la première case vide).

- [ ] **Step 1 : tests dans `tests/rules.rs`**

```rust
// Figure d'exemple de [R] : rangée 33-44-55-66 (66 = « X »), 43-54 au-dessus, 45 au-dessous.
const FIGURE: [(u8, u8, u8); 6] = [(43, 1, 2), (54, 0, 3), (33, 1, 1), (44, 0, 5), (55, 1, 4), (45, 0, 6)];

#[test] fn the_example_figure_of_the_rules() {
    let mut s = position(&FIGURE, 1);
    let perfect = |s: &State, ab| is_perfect(&s.board, at(ab));
    assert!(perfect(&s, 44) && perfect(&s, 54));
    assert!(!perfect(&s, 33) && !perfect(&s, 43) && !perfect(&s, 55) && !perfect(&s, 45));
    rolled(&mut s, Source::Cell(at(33)), 1);
    SiberianDice::apply(&mut s, Seat(1), &Action::Place(at(66))).unwrap();
    assert!(!perfect(&s, 44));
    assert!(perfect(&s, 43) && perfect(&s, 55) && perfect(&s, 66) && perfect(&s, 54));
}
```

Autres tests, nommés et avec leurs assertions :
- `a_new_game` : plateau vide, réserves `[12, 12]`, `turn = Seat(0)`, `legal_actions` = `[Pick(Reserve)]`.
- `picking_from_the_reserve_rolls_a_die` : après `Pick(Reserve)`, `reserve[0] = 11`, `step = Place { from: Reserve, value }` avec `value ∈ 1..=6`, 37 `Place` légaux.
- `a_die_can_go_back_to_the_cell_it_left` : dé bleu en 44, `Pick(Cell(44))` → 44 est vide et `Place(44)` est légal ; pendant la pose, un voisin de 44 ne le compte plus (`is_perfect` change en conséquence).
- `an_empty_reserve_cannot_be_picked_from` et `timeout_picks_a_board_die_when_the_reserve_is_empty` (réserve bleue à 0, un dé bleu en 11 → `timeout_action = Pick(Cell(at(11)))`).
- `an_opponents_move_can_win_the_game` : bleu a 5 dés parfaits, et un de ses 1 isolés est voisin d'une case vide ; rouge y pose → `end = Won { seat: 0 }`, `outcome = Ranked([0, 1])`.
- `six_each_at_once_is_a_draw` : une pose qui donne 6 aux deux → `BothSix`, `Ranked([0, 0])`.
- `the_quiet_limit_draws` : `quiet = 2 * QUIET_ROUNDS − 1`, une pose sans nouveau record → `QuietLimit` ; même position avec une pose qui bat un record → `quiet = 0` et la partie continue.
- `illegal_moves_change_nothing` : `Place` à l'étape `Choose`, `Pick` d'un dé adverse, `Pick` hors tour, `Place` sur une case occupée, tout coup après la fin → `Err` et `s` identique à avant (`assert_eq!` sur un clone).
- `after_the_end_nothing_is_pending` : `pending` vide, vue `winners` = `[0]` après `Won { seat: 0 }`, `[0, 1]` après un nul.
- `the_rolls_of_a_seed_are_pinned` : les 8 premiers `roll(seed=1, k)` exposés via une suite de `Pick`/`Place` depuis `new(&Config::default(), 2, 1)` égalent un tableau figé (le relever une fois, puis l'écrire en dur).

`tests/conformance.rs`, sur le modèle de Black-Pot :
- `two_players_conform` : `check_options::<SiberianDice>(2)` et `check::<SiberianDice>(&Config::default(), 2, 20, 52)`.
- `the_view_shows_the_game_but_not_the_seed` : le JSON de la vue ne contient ni `seed` ni `rolls` ; `cells` a 37 entrées ; `reserve` = `[12, 12]` ; `quietLeft` = 30 ; `winners` = `[]`.

- [ ] **Step 2 :** `cargo test -p siberian-dice` → échecs attendus.
- [ ] **Step 3 :** implémenter `lib.rs`, `view.rs`, `testing.rs` et un `heuristics.rs` minimal. Écrire `RULES.md` : chaque règle de la spec §3 citée [R] (PDF rév. 2.0, page 1) ou [U] (2026-10-30), le compte « après la pose » marqué comme choix du site, et le statut de la figure d'exemple (vérifiée par un test).
- [ ] **Step 4 :** `cargo test -p siberian-dice` → PASS.
- [ ] **Step 5 :** commit `siberian-dice : règles et vue`.

### Task 3 : IA, registre, arène

**Files :**
- Modify : `games/siberian-dice/src/heuristics.rs`
- Create : `games/siberian-dice/tests/ai.rs`
- Modify : `crates/ludo-registry/Cargo.toml`, `crates/ludo-registry/src/lib.rs` (`static SIBERIAN_DICE: GameEntry<siberian_dice::SiberianDice>` et l'entrée dans `GAMES`)
- Modify : `CLAUDE.md` (ligne d'arène dans « Commandes »)

**Interfaces :**
- Consumes : `State`, `Action`, `Step`, `perfect_count`, `legal_actions` de Task 2.
- Produces : `pub fn gain(s: &State, seat: Seat, to: CellId) -> i32` (dans `heuristics.rs`, réexportée) = (dés parfaits de `seat` − ceux de l'adversaire) après la pose du dé lancé en `to`, moins le même écart avant le coup. On ne l'appelle qu'à l'étape `Place`.

La conception de l'IA revient au builder (spec §4). Il y a seulement deux contraintes, fixées par les tests :
- `rollout_action` : à l'étape `Place`, on joue une case de `gain` maximal la plupart du temps, et une case au hasard sinon. À l'étape `Choose`, on choisit au hasard, avec une préférence à régler pour la réserve tant qu'elle n'est pas vide.
- `evaluate` : on coupe la simulation après un horizon fixe, et on la note d'après l'écart de dés parfaits (dans [0, 1], la somme des deux valeurs fait 1).

- [ ] **Step 1 : tests**
  - `tests/ai.rs` : `the_rollout_takes_a_winning_cell` : bleu a 5 dés parfaits, et une case vide en rend un 6e parfait avec la valeur lancée. Sur 50 graines de `rollout_action`, cette case est jouée au moins 40 fois.
  - `tests/ai.rs` : `evaluate_favours_the_side_with_more_perfect_dice`.
  - `crates/ludo-registry/src/lib.rs` : `siberian_dice_is_for_two` : `find("siberian-dice").unwrap().player_counts() == vec![2]`.
- [ ] **Step 2 :** `cargo test -p siberian-dice -p ludo-registry` → échecs attendus.
- [ ] **Step 3 :** implémenter les heuristiques et le branchement dans le registre.
- [ ] **Step 4 :** `cargo test --workspace --release` → PASS. Ensuite `cargo run --release -p ludo-cli -- arena siberian-dice 200 hard easy` doit donner **≥ 0.55**. Noter le score et la durée dans `CLAUDE.md`, sur le modèle de la ligne Black-Pot. Si le score est sous 0.55, régler les heuristiques ou `search_actions` sans toucher à l'ISMCTS générique, et mesurer à nouveau.
- [ ] **Step 5 :** commit `siberian-dice : IA et registre`.

### Task 4 : site web

**Files :**
- Create : `web/src/games/siberian-dice/types.ts` : copie TypeScript de `View`, `CellView`, `DieView`, `Step`, `Source`, `Action`, `LastMove`, `End`, sous la forme serde par défaut (`{ Pick: "Reserve" }`, `{ Pick: { Cell: 12 } }`, `{ Place: 12 }`, `"Choose"`, `{ Place: { from, value } }`).
- Create : `web/src/games/siberian-dice/Board.tsx` (`export function SiberianDiceBoard(props: BoardProps)`), `siberian-dice.css`, `rules.ts` (`export const SIBERIAN_DICE_RULES: string[]`, d'accord avec `RULES.md`), `__tests__/siberian-dice.test.tsx`
- Modify : `web/src/games/registry.ts` : `{ id: "siberian-dice", family: "board", Board: SiberianDiceBoard, rules: SIBERIAN_DICE_RULES }`

**Interfaces :**
- Consumes : la vue JSON de Task 2 et `legalActions` (les actions de Task 2) fournis par `BoardProps`.

Affichage (spec §5) :
- Hexagones en pointe. On reprend `center`/`hexPoints` de Yavalath avec les coordonnées axiales `q = a − 4`, `r = b − a`. Cela donne 11 à gauche, 77 à droite, 41 et 74 en haut, 14 et 47 en bas, 43 au-dessus de 33 et 44, comme sur la page 3.
- Les dés sont des carrés arrondis bleus ou rouges, avec les points de leur face. Un dé dont `perfect` est vrai reçoit la classe `perfect` (contour laiton).
- Le panneau de chaque joueur affiche son nom, sa réserve et « parfaits n/6 », avec la classe `active` pour le joueur au trait.
- À l'étape `Choose`, la réserve et les dés qui ont un `Pick` légal sont cliquables.
- À l'étape `Place`, la valeur lancée s'affiche près de la réserve, et chaque case qui a un `Place` légal est allumée (classe `playable`) et cliquable.
- Les cases `from` et `to` de `last` portent la classe `last`.
- Si `end` est défini, un message de fin s'affiche : gagnant, nul (six chacun) ou nul (limite).

- [ ] **Step 1 : tests Vitest**, avec `renderToStaticMarkup` comme dans `black-pot.test.tsx` :
  - `marks_perfect_dice` : avec la figure d'exemple, les dés en 44 et 54 portent la classe `perfect`, pas les autres.
  - `shows_the_rolled_die_and_lights_the_vacated_cell` : vue à l'étape `Place { from: { Cell: id44 }, value: 3 }`, avec 44 vide dans `cells` et `Place(id44)` dans `legalActions` → la valeur 3 est rendue, et la case 44 porte `playable`.
  - `the_reserve_is_clickable_only_when_picking_from_it_is_legal`.
  - `names_the_winner_and_the_draws`.
- [ ] **Step 2 :** `cd web && pnpm test siberian-dice` → échec.
- [ ] **Step 3 :** implémenter.
- [ ] **Step 4 :** `./scripts/build-wasm.sh`, puis `cd web && pnpm test && pnpm build` → PASS. Ensuite, vérifier dans le navigateur (`pnpm preview` + `playwright-core` et le Chromium de `~/.cache/ms-playwright`) : une partie contre l'IA Facile, prise de la réserve, reprise d'un dé, dé parfait mis en évidence. Comparer une capture du plateau avec la page 3 du PDF (orientation).
- [ ] **Step 5 :** commit `siberian-dice : plateau web`.

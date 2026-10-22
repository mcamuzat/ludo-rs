# Bread Basket Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ajouter Bread Basket 3.11 (`pairs-bread-basket`, 2–6 joueurs) à la crate `games/pairs` : moteur, IA, registre et table web.

**Architecture:** Un module `games/pairs/src/bread_basket/` sur le modèle exact de `the_judge/` (`mod.rs`, `cards.rs`, `view.rs`, `determinize.rs`, `heuristics.rs`, `testing.rs`). Le paquet à Oignons a son propre type `Card` (le `Rank` 1–10 du deck Pairs n'est pas touché) et une pioche simple `Vec<Card>` sans remélange : la pioche vide termine la main, `Supply` (qui remélange la défausse) ne convient pas.

**Tech Stack:** Rust (ludo-core `Game`/`Determinize`/`Heuristics`), rand_chacha, React/TypeScript, vitest.

**Spec:** `docs/superpowers/specs/2026-10-13-pairs-design.md` §29 ; procédure « Ajouter un jeu Pairs » du `CLAUDE.md`.

## Global Constraints

- Nom affiché « Bread Basket », id `pairs-bread-basket`, 2 à 6 joueurs, `hidden_info: true`, `clock: false`.
- Paquet : 5 × chaque rang 1–10 + 5 Oignons valant −5 (55 cartes) ; l'Oignon est la carte la plus basse.
- Mains : 10 / 9 / 8 / 7 / 6 cartes pour 2 / 3 / 4 / 5 / 6 joueurs.
- Option `rounds` : `Six` (défaut) ou `PerPlayer`.
- Aucune statistique affichée (ni risque, ni compte de cartes restantes par rang).
- Textes du jeu en français ; commentaires de code en anglais, règles marquées `[B]` (source Bread Basket 3.11) ou `❓` (choix du site), comme `[R]`/`❓` dans The Judge.
- Commits avec l'e-mail du dépôt (`marc.camuzat@gmail.com`, déjà en config locale), un par tâche, suffixe `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.

## Review Focus

1. Pioche vidée par la **pioche forcée** d'un sandwich d'air, ou par la relance d'une file vidée par un sandwich : la main doit s'arrêter aussitôt, sans tour de plus ni panique. → test `deck_runs_out_on_the_air_draw` et `deck_runs_out_when_restarting_the_line` (tâche 2).
2. Un joueur vide sa main **grâce à** un sandwich qui vide aussi la file : la file est relancée, puis le dernier tour des autres commence. → test `going_out_with_a_sandwich_restarts_the_line_then_last_turns` (tâche 2).
3. Sandwich d'air qui laisse le joueur avec une carte alors qu'il venait de poser sa dernière : il n'est **pas** sorti. → test `an_air_sandwich_on_the_last_card_does_not_go_out` (tâche 2).
4. Scores négatifs (Oignons ramassés, Oignons en main) : sérialisés et classés correctement. → test `onions_make_scores_negative_and_rank_correctly` (tâche 2).
5. Option `PerPlayer` à 5 joueurs : exactement 5 manches, donneur qui tourne. → test `per_player_plays_one_deal_each` (tâche 2) et cas de conformance (tâche 3).

---

### Task 1: Cartes et détection des sandwiches

**Files:**
- Create: `games/pairs/src/bread_basket/cards.rs`
- Create: `games/pairs/src/bread_basket/mod.rs` (pour l'instant `pub mod cards;` seulement)
- Modify: `games/pairs/src/lib.rs` (`pub mod bread_basket;`)

**Interfaces:**
- Produces :
  - `pub struct Card(i8)`, `#[serde(try_from = "i8", into = "i8")]`, `Copy + Ord + Hash + Debug`. Valeurs permises : −5 et 1..=10. `Card::ONION`, `Card::new(v: i8) -> Option<Card>`, `Card::value(self) -> i8`, `Card::is_onion(self) -> bool`.
  - `pub fn full_deck() -> Vec<Card>` : 55 cartes, ordre croissant.
  - `pub fn hand_penalty(card: Card) -> i32` : `[B]` rang v → v ; Oignon → 5 (il compte −5 en main).
  - `pub fn filling_points(cards: &[Card]) -> i32` : somme des valeurs (Oignon −5).
  - `pub enum Completion { Nothing, Air, Sandwich { start: usize } }` (`PartialEq, Debug`).
  - `pub fn completes(line: &[Card], card: Card) -> Completion` : en remontant depuis la fin de `line`, s'arrêter à la première carte `>= card` ; égale et c'est la dernière de la file → `Air` ; égale plus tôt → `Sandwich { start: index }` ; plus haute ou file épuisée → `Nothing`.
  - `pub fn hand_size(players: u8) -> usize` : 10, 9, 8, 7, 6 pour 2..=6.

- [ ] **Step 1: Tests unitaires dans `cards.rs`** (`#[cfg(test)]`) :

```rust
#[test] fn the_deck_has_five_of_each_and_five_onions() // len 55, 5 × Card::ONION, 5 × chaque 1..=10
#[test] fn card_rejects_other_values() // Card::new(0), new(11), new(-4) == None ; serde_json "-5" ok, "0" err
#[test] fn completes_finds_simple_and_long_sandwiches() {
    assert_eq!(completes(&c(&[4, 3]), card(4)), Completion::Sandwich { start: 0 });
    assert_eq!(completes(&c(&[2, 9, 8, 3]), card(9)), Completion::Sandwich { start: 1 });
    assert_eq!(completes(&c(&[6, -5, 3]), card(6)), Completion::Sandwich { start: 0 });
}
#[test] fn completes_stops_at_a_higher_card() {
    assert_eq!(completes(&c(&[5, 3, 7]), card(5)), Completion::Nothing); // 5-3-7-5 de la fiche
    assert_eq!(completes(&c(&[1, 2]), card(3)), Completion::Nothing);
}
#[test] fn completes_detects_air_including_onions() {
    assert_eq!(completes(&c(&[3, 8]), card(8)), Completion::Air);
    assert_eq!(completes(&c(&[-5]), card(-5)), Completion::Air);
}
#[test] fn penalties_and_filling() {
    assert_eq!(hand_penalty(card(7)), 7);
    assert_eq!(hand_penalty(Card::ONION), 5);
    assert_eq!(filling_points(&c(&[-5, 3])), -2); // 6-(-5)-3-6 de la fiche
}
#[test] fn hand_sizes() // 2→10 … 6→6
```
(`c`/`card` : petits helpers de test locaux à partir de `i8`.)

- [ ] **Step 2:** `cargo test -p pairs bread_basket::cards` → FAIL (non compilé).
- [ ] **Step 3:** Implémenter `cards.rs` selon les interfaces.
- [ ] **Step 4:** `cargo test -p pairs bread_basket::cards` → PASS.
- [ ] **Step 5:** Commit `bread-basket : cartes et sandwiches`.

### Task 2: Règles, vue et section RULES.md

**Files:**
- Modify: `games/pairs/src/bread_basket/mod.rs`
- Create: `games/pairs/src/bread_basket/view.rs`, `games/pairs/src/bread_basket/testing.rs`
- Modify: `games/pairs/src/lib.rs` (`pub use bread_basket::BreadBasket;`)
- Modify: `games/pairs/RULES.md` (section « Bread Basket (`pairs-bread-basket`, 2 à 6 joueurs) », plan de The Judge : source [B], Mise en place, Tour, Sandwiches, Fin de main, Partie, Choix ❓ numérotés comme la spec §29)
- Test: `games/pairs/tests/bread_basket_rules.rs`

**Interfaces:**
- Consumes : tout `cards.rs` (tâche 1).
- Produces :
  - `pub struct BreadBasket;` `impl Game` avec `type Config = Config`, `Action`, `State`, `View`.
  - `#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)] pub enum Rounds { #[default] Six, PerPlayer }` ; `Rounds::count(self, players: u8) -> u8` (6 ou `players`).
  - `#[derive(Default, …)] #[serde(default)] pub struct Config { pub rounds: Rounds }` ; `options()` : `GameOption { key: "rounds", kind: OptionKind::Choice { values: &["Six", "PerPlayer"], default: "Six" } }`.
  - `pub enum Action { Play { card: Card }, Draw }` (`Hash`, serde).
  - `pub enum Phase { Play, Over }`.
  - `pub enum HandEnd { DeckEmpty, WentOut }`.
  - `#[serde(tag = "type")] pub enum Event` :
    `Dealt { round: u8, dealer: Seat, first: Seat }`,
    `Played { seat: Seat, card: Card }`,
    `Sandwich { seat: Seat, bread: Card, filling: Vec<Card>, points: i32 }`,
    `Air { seat: Seat, card: Card }`,
    `Drew { seat: Seat }`,
    `LineStarted { card: Card }`,
    `WentOut { seat: Seat }`,
    `HandOver { end: HandEnd, gained: Vec<i32>, penalties: Vec<i32>, scores: Vec<i32> }`.
  - `pub struct State` : `players, config, round_no: u8, seed: u64, dealer: Seat, deck: Vec<Card>` (dessus = dernier élément), `discard: Vec<Card>, line: Vec<Card>, hands: Vec<Vec<Card>>, taken: Vec<Vec<Card>>` (garniture de la main en cours), `scores: Vec<i32>, out: Option<Seat>` (qui a vidé sa main), `phase, turn: Seat, events: Vec<Event>`.
  - `testing::table(hands: &[&[i8]], line: &[i8], deck: &[i8]) -> State` : manche 1 sur 6, donneur = dernier siège, tour au siège 0, rien de ramassé, scores à 0.
  - `View` (`camelCase`) : `round_no, rounds, phase, dealer, turn: Option<Seat>, line: Vec<Card>, deck_size: usize, discard: Vec<Card>, hand: Vec<Card>, seats: Vec<SeatView { taken: Vec<Card>, taken_points: i32, hand_size: usize, score: i32, out: bool }>, events`.

Déroulé (une seule fonction de fin de tour, `advance`, appelée après chaque action) :
- Donne `[B]` : remélanger `full_deck()` avec `ChaCha8Rng::seed_from_u64(mix(seed, round_no))` (même `mix` que The Judge), `hand_size(players)` cartes chacun, puis `restart_line`. Le donneur ❓ de la manche 1 est `seed % players` ; ensuite il passe à gauche ; le joueur à sa gauche commence.
- `Play` : retirer la carte de la main, `Played`, puis selon `completes(line, card)` :
  `Sandwich { start }` → la garniture `line[start+1..]` va dans `taken[seat]`, les deux pains dans `discard`, `line.truncate(start)`, événement `Sandwich` ;
  `Air` → les deux cartes en `discard`, `Air`, puis pioche forcée (`Drew`) ❓ avant toute relance de file ; pioche vide → fin de main `DeckEmpty` ;
  `Nothing` → pousser sur la file.
- `Draw` : pioche → main, `Drew`. Pioche vide après coup → fin `DeckEmpty` immédiate (`[B]` la main s'arrête quand la pioche est vide).
- Après l'action (si la main continue) : file vide → `restart_line` (pioche vide → `DeckEmpty`). Puis si `out` est `None` et la main du joueur est vide → `out = Some(seat)`, `WentOut`. Puis tour suivant ; si `out == Some(next)` → fin `WentOut` (chacun des autres a joué une fois ; ❓ une autre sortie pendant ce tour de table ne change rien).
- Fin de main : `gained[i] = filling_points(taken[i])`, `penalties[i] = Σ hand_penalty`, `scores[i] += gained − penalties`, `HandOver`. Dernière manche (`round_no == config.rounds.count(players)`) → `Phase::Over` ; sinon donne suivante.
- `legal_actions` : une `Play` par valeur distincte de la main (triées) + `Draw` ; vide hors de son tour ou en `Over`. Une pioche vide n'existe jamais en cours de main (elle la termine), donc `Draw` est toujours légal.
- `outcome` : `Outcome::Ranked` par scores décroissants, ex æquo au même rang ❓ (copier The Judge).
- `apply` : messages d'erreur « La partie est terminée. », « Ce n'est pas ton tour. », « Tu n'as pas cette carte. » ; `events.clear()` en début d'action.

- [ ] **Step 1: Tests dans `tests/bread_basket_rules.rs`** (via `testing::table`, valeurs `i8`) :

```rust
#[test] fn deal_sizes_and_first_line_card()            // new() à 2..=6 : mains 10..6, file d'une carte, pioche 55 − n·main − 1
#[test] fn a_sandwich_scores_its_filling_and_discards_the_bread()
    // file [4, 3], main J0 [4, 9] ; Play 4 → taken[0] == [3], discard == [4, 4], file vide relancée depuis la pioche
#[test] fn a_sandwich_can_take_only_the_end_of_the_line() // file [8, 2, 9, 5], Play 9 → file == [8], taken == [5]
#[test] fn an_air_sandwich_discards_and_forces_a_draw()   // file [3, 7], Play 7 → discard [7,7], main +1 carte, file [3]
#[test] fn an_air_sandwich_on_the_last_card_does_not_go_out() // main [7] seule ; pas de WentOut, hand_size 1
#[test] fn deck_runs_out_on_the_air_draw()               // pioche vide, Play qui fait l'air → HandOver { end: DeckEmpty }
#[test] fn deck_runs_out_when_restarting_the_line()      // pioche vide, sandwich qui vide la file → DeckEmpty
#[test] fn drawing_the_last_card_ends_the_hand()         // pioche [5], Draw → DeckEmpty, la carte piochée compte en pénalité
#[test] fn going_out_gives_each_other_player_one_turn()  // 3 joueurs, J0 pose sa dernière carte → WentOut ; J1 et J2 jouent ; HandOver { end: WentOut } avant que J0 rejoue
#[test] fn going_out_with_a_sandwich_restarts_the_line_then_last_turns()
#[test] fn onions_make_scores_negative_and_rank_correctly()
    // garniture [-5] et un Oignon en main → score −10 ; outcome classe −10 dernier
#[test] fn hand_penalty_subtracts_ranks_and_onions()     // main [10, -5] en fin de main → penalties == 15
#[test] fn six_rounds_by_default_and_the_deal_rotates()  // partie jouée au hasard jusqu'au bout : 6 Dealt, donneurs successifs +1
#[test] fn per_player_plays_one_deal_each()              // Config { rounds: PerPlayer }, 5 joueurs → 5 Dealt puis Over
#[test] fn illegal_moves_are_refused()                   // carte absente, hors tour, après Over
#[test] fn the_view_hides_other_hands_and_the_deck()     // View de J1 : hand == mains[1], seats[0].hand_size seul, pas d'ordre de pioche
```

- [ ] **Step 2:** `cargo test -p pairs --test bread_basket_rules` → FAIL.
- [ ] **Step 3:** Implémenter `mod.rs`, `view.rs`, `testing.rs` ; écrire la section RULES.md.
- [ ] **Step 4:** `cargo test -p pairs --test bread_basket_rules` puis `cargo clippy -p pairs --all-targets -- -D warnings` → PASS, aucun avertissement.
- [ ] **Step 5:** Commit `bread-basket : règles`.

### Task 3: IA, déterminisation, registre, conformance, arène

**Files:**
- Create: `games/pairs/src/bread_basket/determinize.rs`, `games/pairs/src/bread_basket/heuristics.rs`
- Modify: `crates/ludo-registry/src/lib.rs` (`static PAIRS_BREAD_BASKET: GameEntry<pairs::BreadBasket>` + entrée dans `GAMES`)
- Modify: `games/pairs/tests/conformance.rs`
- Modify: `CLAUDE.md` (ligne d'arène dans « Commandes »), `README.md` (ligne « **Bread Basket** (2–6 joueurs, paquet à Oignons) »)

**Interfaces:**
- Consumes : `State`, `Action`, `completes`, `filling_points`, `hand_penalty`.
- Produces : `impl Determinize for BreadBasket`, `impl Heuristics for BreadBasket` (seul `rollout_action`).

Déterminisation : les mains des autres et la pioche sont redistribuées parmi les cartes que l'observateur ne voit pas (`full_deck()` moins sa main, la file, la défausse et toutes les garnitures), tailles conservées ; `seed` retiré au hasard (`rng.next_u64()`), comme The Judge.

Politique de simulation (`WANDER = 0.1` coup au hasard) : pour chaque `Play` légale, note = garniture gagnée (`filling_points`, 0 pour l'air) + `hand_penalty(card)` (carte qui quitte la main) − la meilleure garniture qu'une carte quelconque permettrait au suivant de ramasser sur la file obtenue (exposition, tous rangs supposés possibles). `Draw` note −(pénalité moyenne d'une carte non vue). Jouer la meilleure note.

- [ ] **Step 1: Tests** (`#[cfg(test)]` des deux fichiers) :

```rust
#[test] fn hands_and_deck_change_but_not_what_the_observer_sees() // vue identique sur 20 graines, main de l'observateur intacte, tailles des mains et de la pioche conservées, pioches différentes
#[test] fn completes_the_best_sandwich()       // file [9, 8, 6], main [9, 6, 2] → Play 9 (garniture 8+6 > 6)
#[test] fn sheds_an_onion_rather_than_feed_a_sandwich() // file [10, 9], main [-5, 8] : 8 donnerait 9+8 à qui a un 10 ; joue -5
#[test] fn the_rollout_action_is_always_legal() // 2..=6 joueurs, 5 parties chacun, jusqu'à Over
```
Conformance :
```rust
#[test]
fn bread_basket_conforms_at_every_table_size() {
    use pairs::bread_basket::{BreadBasket, Config, Rounds};
    for players in 2..=6u8 {
        check::<BreadBasket>(&Config::default(), players, 30, 2000 + u64::from(players));
    }
    check::<BreadBasket>(&Config { rounds: Rounds::PerPlayer }, 5, 30, 2011);
}
```

- [ ] **Step 2:** `cargo test -p pairs bread_basket` et `cargo test -p pairs --test conformance bread_basket` → FAIL puis implémenter.
- [ ] **Step 3:** Implémenter les deux fichiers et l'entrée du registre.
- [ ] **Step 4:** `cargo test -p pairs` + `cargo test -p ludo-registry` + clippy → PASS.
- [ ] **Step 5:** Arène : `cargo run --release -p ludo-cli -- arena pairs-bread-basket 200 hard easy 2` puis `… 200 hard easy 4`. Attendu ≥ 0.55. En dessous, ajuster la politique (poids de l'exposition) avant de continuer ; noter les chiffres mesurés dans `CLAUDE.md` et dans la spec §29 (« Mesuré : … »).
- [ ] **Step 6:** Commit `bread-basket : IA, registre`.

### Task 4: Table web

**Files:**
- Create: `web/src/games/pairs/bread-basket/types.ts`, `web/src/games/pairs/bread-basket/Board.tsx`
- Modify: `web/src/games/pairs/cards.tsx` (Oignon), `web/src/games/pairs/pairs.css` (`.pairs-card.onion`)
- Modify: `web/src/games/pairs/rules.ts` (`BREAD_BASKET_RULES`), `web/src/games/registry.ts` (`{ id: "pairs-bread-basket", family: "pairs", Board: BreadBasketBoard, rules: BREAD_BASKET_RULES }`), `web/src/games/options.ts` (`"pairs-bread-basket.rounds": "Manches"`, `.Six`: `"6"`, `.PerPlayer`: `"Une donne par joueur"`)
- Test: `web/src/games/pairs/__tests__/bread-basket.test.tsx`

**Interfaces:**
- Consumes : la `View`, l'`Action` et l'`Event` sérialisés de la tâche 2 (cartes en nombres, −5 = Oignon ; `{ Play: { card } }` | `"Draw"`).
- Produces : `BreadBasketBoard(props: BoardProps)`, `describeBreadBasketEvent(e, names): string`.

`Card` : quand `rank === -5`, classe `onion` et libellé « −5 » avec le titre « Oignon » ; les autres jeux n'envoient jamais −5. Table : la **file** au centre, de gauche à droite ; pioche (`CardBack` « pioche ») et défausse visible ; par siège : garniture ramassée et ses points, cartes en main (dos), score, « est sorti » si `out`. Main du joueur : une carte cliquable par valeur légale (comme The Judge), bouton « Piocher ». Aucune statistique. Textes des événements, par exemple : « Ana fait un sandwich au 9 : 8, 6 (+14). », « Hugo fait un sandwich d'air de 7 et pioche. », « Cléo n'a plus de cartes : un dernier tour pour les autres. », « Fin de la main (pioche vide) : Ana +14 − 3 … ».

- [ ] **Step 1: Tests** (copier la structure de `the-judge.test.tsx`) : un contrôle par action légale exactement (cartes + « Piocher ») ; une carte −5 s'affiche « −5 » avec la classe `onion` ; `describeBreadBasketEvent` pour `Sandwich`, `Air`, `HandOver` ; spectateur (`me = null`) sans contrôle.
- [ ] **Step 2:** `pnpm -C web test bread-basket` → FAIL.
- [ ] **Step 3:** Implémenter.
- [ ] **Step 4:** `pnpm -C web test`, `pnpm -C web lint`, `pnpm -C web build` (type-check + WASM) → PASS.
- [ ] **Step 5:** Vérification navigateur sans affichage (playwright-core + Chromium de `~/.cache/ms-playwright` sur `pnpm preview`) : lancer une partie contre l'IA à 3, jouer quelques coups, faire une capture.
- [ ] **Step 6:** Commit `web : Bread Basket`.

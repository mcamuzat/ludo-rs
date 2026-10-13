# Pairs, plan 5 : Goblin Poker — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Goblin Poker (3–8 joueurs, jeu à jetons où l'on jette des cartes dans un pot commun redistribué) se joue sur ludo-rs contre l'IA et en ligne, en 5 mains.

**Architecture:** Cinquième module de `games/pairs`, `goblin_poker/`, sous l'id `pairs-goblin-poker`, en suivant la procédure « Ajouter un jeu Pairs » de `CLAUDE.md` et le modèle de `ghost_town/` (jetons, mains cachées, mise en place simultanée). Aucun changement de plateforme ni de `deck.rs`.

**Tech Stack:** Rust stable (rand 0.8, rand_chacha 0.3, serde), React 19 + TypeScript, vitest 3.

**Spec:** `docs/superpowers/specs/2026-10-13-pairs-design.md` §13 (règles et points non tranchés), §2, §3, §6, §7 pour les principes communs. Source : *Pairs Companion Book*, chapitre « Goblin Poker ».

## Global Constraints

- Branche `pairs-goblin-poker`. `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --release` avant chaque commit.
- `apply` rend `Illegal` sans modifier l'état, ne panique jamais. Une vue ne contient ni graine, ni `Supply`, ni main d'un autre, ni carte choisie ou jetée par un autre.
- Mélanges et tirages en `ChaCha8Rng` ; jamais de tirage `usize`.
- La somme des soldes et du pot vaut toujours zéro. Une pile ne contient jamais deux cartes d'un rang.
- Aucune statistique dans la vue ni sur le plateau. Messages `Illegal` et textes du plateau en français ; code et commentaires en anglais.

## Review Focus

1. **Tous se couchent sauf un au milieu d'un tour de table** : le dernier prend le pot sans avoir à agir, les cartes déjà jetées sont défaussées. → `the_last_player_in_takes_the_pot_at_once`.
2. **Élimination en séquence à la redistribution** : s'il ne reste qu'un joueur, il gagne sans recevoir sa carte. → `the_last_survivor_wins_before_catching_a_pair`.
3. **Ordre des piles** : 2-3-6 plus bas que 2-3 ; égalité parfaite départagée par la pioche, carte en paire défaussée et remplacée ; pioche vide → premier siège à égalité. → `an_extra_card_makes_a_stack_lower`, `a_perfect_tie_is_broken_from_the_deck`, `an_empty_deck_leaves_the_first_tied_seat`.
4. **Partage du pot après le cinquième tour** : division entière, reste au pot de la main suivante ; à la dernière main, reste à personne ; la somme reste nulle. → `survivors_split_the_pot_and_odd_chips_stay`.
5. **Action absurde** (carte absente de la main, `Pitch` deux fois dans le même tour de table, coup hors tour, siège hors table, coup après la fin) : `Illegal`, état intact. → `absurd_actions_are_refused`.
6. **Vue** : ni main d'un autre, ni carte jetée par un autre, ni graine. → `a_view_hides_other_hands_and_pitched_cards` et la conformance.
7. **Plateau couché, éliminé, ayant déjà jeté, ou en fin de partie** : rien à cliquer. → tests web.

## Interfaces (toutes les tâches)

```rust
// games/pairs/src/goblin_poker/mod.rs
pub struct GoblinPoker;
pub const ANTE: u32 = 1;
pub const HAND: usize = 6;
pub const DEFAULT_HANDS: u8 = 5;
pub const MAX_HANDS: u8 = 15;
pub struct Config { pub hands: Option<u8> }
pub enum Action { Place { card: Rank }, Pitch { card: Rank }, Fold }
pub enum Phase { Setup, Play, Over }
#[serde(tag = "type")]
pub enum Event {
    Placed { seat: Seat, card: Rank },
    TieBreak { seat: Seat, card: Rank, kept: bool },   // kept: false = paired, discarded and replaced
    Starts { seat: Seat },
    Pitched { seat: Seat },
    Folded { seat: Seat, cost: u32 },
    Caught { seat: Seat, card: Rank, paired: bool },   // paired: pays `card`, out
    HandEnded { winners: Vec<Seat>, share: u32 },      // one winner: the whole pot
}
pub struct State {
    pub players: u8, pub hands_total: u8, pub hand_no: u8, pub round: u8,
    pub supply: Supply, pub seed: u64,
    pub hands: Vec<Vec<Rank>>, pub stacks: Vec<Vec<Rank>>,
    pub placed: Vec<Option<Rank>>,   // Setup
    pub pitched: Vec<Option<Rank>>,  // this round, face down
    pub out: Vec<bool>,
    pub phase: Phase, pub turn: Option<Seat>,
    pub pot: u32, pub chips: Vec<i32>, pub events: Vec<Event>,
}
impl State {
    pub fn active(&self) -> Vec<Seat>;
    pub fn fold_cost(&self) -> Option<u32>;                 // lowest face-up card of the seats still in
    pub fn unlocated_for(&self, observer: Option<Seat>) -> [u8; 10];
    pub fn catch_cost(&self, seat: Seat) -> f64;            // AI only: chips a random unseen card costs `seat`
}
// view.rs — #[serde(rename_all = "camelCase")]
pub struct View { hand_no, hands_total, round, phase, seats: Vec<SeatView>, hand: Vec<Rank>,
    placed: Option<Rank>, pitched: Option<Rank>, turn: Option<Seat>, pot: u32, ante: u32,
    fold_cost: Option<u32>, deck: usize, discard: usize, discard_seen: Vec<Rank>, events }
pub struct SeatView { stack: Vec<Rank>, hand_size: usize, out: bool, pitched: bool, chips: i32 }
// testing.rs
pub fn setup(hands: &[&[u8]], draws: &[u8]) -> State;                       // hand 1, antes paid, nobody has placed
pub fn table(hands: &[&[u8]], stacks: &[&[u8]], draws: &[u8]) -> State;     // hand 1, round 1, Play, seat 0 to move
```

JSON des actions : `"Fold"`, `{"Place":{"card":4}}`, `{"Pitch":{"card":4}}`.

Le mélange des cartes jetées utilise un flux `ChaCha8Rng` propre à la main et au tour de table, tiré de `seed`.

## Tasks

### Task 1: Règles, vue, RULES.md
**Files:** Create `games/pairs/src/goblin_poker/{mod.rs,view.rs,testing.rs}`, `games/pairs/tests/goblin_poker_rules.rs` ; Modify `games/pairs/src/lib.rs`, `games/pairs/RULES.md`.
**Produces:** les interfaces ci-dessus sauf `Determinize`/`Heuristics`/`catch_cost`.
- [ ] Écrire `tests/goblin_poker_rules.rs` et `testing.rs` ; les voir échouer (module absent). Tests : ceux du Review Focus (1–6), plus `a_new_game_antes_and_deals_six_cards`, `the_number_of_hands_comes_from_the_config_within_bounds`, `placed_cards_start_the_stacks_and_the_lowest_starts`, `setup_shows_each_seat_only_itself_pending`, `the_turn_goes_left_from_the_lowest_stack`, `folding_costs_the_lowest_card_in_play`, `pitched_cards_are_dealt_face_up_from_the_lowest_stack`, `a_pair_costs_its_rank_and_puts_the_player_out`, `a_hand_lasts_five_rounds_at_most`, `the_game_ends_after_the_last_hand_ranked_by_chips`, `chips_and_cards_are_conserved` (parties au hasard, 3 à 8 joueurs), `actions_have_the_json_the_board_sends`, `the_deal_of_a_seed_is_pinned`.
- [ ] Écrire `mod.rs`, `view.rs`, la section de `RULES.md` (source [R], ❓ numérotés) ; tous les tests passent.
- [ ] Commit `pairs: Goblin Poker rules, view and RULES.md`.

### Task 2: Déterminisation et conformance
**Files:** Create `games/pairs/src/goblin_poker/determinize.rs` ; Modify `games/pairs/tests/conformance.rs`.
- [ ] Tests (vue de l'observateur inchangée ; mains et cartes jetées des autres redistribuées, tailles conservées ; la main et la carte jetée de l'observateur gardées ; choix des autres oubliés en mise en place) et cas de conformance de 3 à 8 joueurs, plus `hands: Some(1)` et `Some(255)` ; implémentation ; commit `pairs: Goblin Poker determinization and conformance`.

### Task 3: IA et registre
**Files:** Create `games/pairs/src/goblin_poker/heuristics.rs` ; Modify `games/pairs/src/goblin_poker/mod.rs` (`catch_cost`), `crates/ludo-registry/src/lib.rs`, `CLAUDE.md`.
**Politique de rollout :** en `Setup`, au hasard (`search_actions` : la plus petite carte seulement) ; en `Play`, se coucher quand `catch_cost` dépasse le coût de se coucher plus la part du pot (`pot / joueurs en jeu`) ; sinon jeter la carte dont le rang est dans le plus de piles adverses, jamais un rang de sa propre pile s'il y a un autre choix.
- [ ] Tests (jette un rang présent chez les autres et absent de sa pile, ne jette pas un rang de sa pile quand il a le choix, se couche quand le risque dépasse le coût, reste quand se coucher coûte cher, action toujours légale de 3 à 8 joueurs, une seule candidate à la mise en place) ; implémentation ; `ludo arena pairs-goblin-poker 400 hard easy 3` ≥ 0,55, consigné dans `CLAUDE.md` ; commit `pairs: Goblin Poker AI and registry entry`.

### Task 4: Plateau web et docs
**Files:** Create `web/src/games/pairs/goblin-poker/{types.ts,Board.tsx}`, `web/src/games/pairs/__tests__/goblin-poker.test.tsx` ; Modify `web/src/games/registry.ts`, `README.md`.
**Plateau :** par siège, la pile, le nombre de cartes en main, le solde signé, « a jeté » / « hors de la main » ; au centre la main et le tour de table, le pot, pioche et défausse, les défausses visibles ; ma main (cartes cliquables : « Commencer ma pile avec ce 4 », « Jeter ce 4 »), « Se coucher (N jetons) », ma carte jetée, événements en phrases.
- [ ] Test du plateau (chaque contrôle envoie exactement une action légale en mise en place et en jeu ; rien à cliquer hors de son tour, couché, éliminé, spectateur ou en fin de partie ; aucun pourcentage ; chaque événement en phrase) ; implémentation ; `pnpm test && pnpm build` ; une partie à 8 sièges dans un navigateur sans affichage ; commit `web: Goblin Poker board`.

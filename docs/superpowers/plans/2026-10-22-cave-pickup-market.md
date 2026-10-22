# The Cave, Pickup, Market Day Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Ajouter trois jeux Pairs (`pairs-the-cave`, `pairs-pickup`, `pairs-market-day`) avec moteur, IA, registre et table web.

**Architecture:** Un module par jeu dans `games/pairs/src/` sur le modèle de `bread_basket/` (`mod.rs`, `view.rs`, `determinize.rs`, `heuristics.rs`, `testing.rs`). Deck Pairs commun (`deck::{full_deck, Rank}`), pioche en `Vec<Rank>` mélangée par `ChaCha8Rng::seed_from_u64(mix(seed, round))` comme Bread Basket. Coups simultanés de Pickup sur le modèle d'Octopus (`Pending` à plusieurs sièges, `visible_pending` qui cache qui a choisi, `determinize` qui oublie les choix des autres).

**Spec:** `docs/superpowers/specs/2026-10-13-pairs-design.md` §30–32 ; procédure « Ajouter un jeu Pairs » du `CLAUDE.md`.

## Global Constraints

- Noms affichés : « The Cave », « Pickup », « Market Day ». 2–6, 2–6, 2–8 joueurs. `hidden_info: true`, `clock: false`.
- Aucune statistique affichée. Textes français ; commentaires anglais ; règles marquées `[C]` (fiche du jeu) et `❓`.
- Commits avec l'e-mail du dépôt et `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Arène : Difficile ≥ 0.55 contre Facile à 2 et à 4 joueurs (`target/release/ludo arena <id> 200 hard easy <n>`).

## Review Focus

1. The Cave : grotte vide et pioche vide ensemble (fin de manche sans panique) → `an_empty_cave_captures_nothing`.
2. Pickup : égalité de rang entre cartes révélées, ordre « à gauche du dernier à avoir agi » d'un tour à l'autre → `ties_act_from_the_left_of_the_last_actor`.
3. Pickup : knock-out ne touche jamais la pile de celui qui prend → `knockouts_spare_the_taker`.
4. Market Day : Pluie seule + Royal + Chevalier + Bricoleur dans le même tour, dans l'ordre de résolution → `resolution_order_spoiled_royal_rain_knight_tinker`.
5. Market Day à 2–3 joueurs : deux cartes par joueur et par tour, meneur suivant d'après la somme → `two_cards_each_at_three_players`.

---

### Task 1: The Cave — règles
Files: `games/pairs/src/the_cave/{mod,view,testing}.rs`, `lib.rs`, `RULES.md`, `tests/the_cave_rules.rs`.
Produces: `TheCave`, `Config {}`, `Action::Play { card: Rank }`, `Phase { Play, Over }`, `Event { Dealt { round, dealer, first }, Played { seat, card, captured: Vec<Rank> }, Refilled { cards: Vec<Rank> }, RoundOver { points: Vec<u16>, scores: Vec<u16> } }`, `State { players, round_no, seed, dealer, deck: Vec<Rank>, cave: Vec<Rank>, hands, captured: Vec<Vec<Rank>>, scores: Vec<u16>, phase, turn, events }`, `pub fn capture(cave: &[Rank], card: Rank) -> Vec<Rank>` (cartes prises, triées), `pub fn round_points(captured: &[Vec<Rank>]) -> Vec<u16>`, `hand_size(players)`, `testing::table(hands, cave, deck)`.
Tests: `capture_takes_the_closest_lower_rank` (4-7-7-8-9 + 8 → [7,7]), `capture_takes_the_highest_when_nothing_is_lower` (4-5-7-9-10 + 4 → [10] ; 5-5 + 5 → [5,5]), `an_empty_cave_captures_nothing`, `the_cave_refills_to_five_until_the_deck_runs_out`, `majority_scores_one_point_per_card_and_ties_share`, `one_round_per_player_and_the_deal_rotates`, `deal_sizes`, `illegal_moves_are_refused`, `the_view_hides_hands_and_deck`.
Commit `the-cave : règles`.

### Task 2: The Cave — IA, registre, web
Files: `determinize.rs`, `heuristics.rs`, registre, `conformance.rs`, `web/src/games/pairs/the-cave/{types.ts,Board.tsx}`, `__tests__/the-cave.test.tsx`, `rules.ts`, `registry.ts`, README, CLAUDE.md.
Rollout : pour chaque carte, valeur = cartes capturées pondérées (+1 par carte d'un rang où l'on aurait la majorité ou l'égalité, +0.5 sinon) − meilleure capture offerte au suivant ; 10 % au hasard.
Tests : déterminisation (vue inchangée, conservation des 55 cartes), `prefers_a_capture_that_wins_a_majority`, rollout légal 2..=6, conformance 2..=6 ; web : un contrôle par action légale, grotte et captures affichées, événements en mots. Arène 2 et 4.
Commit `the-cave : IA, registre, web`.

### Task 3: Pickup — règles
Files: `games/pairs/src/pickup/{mod,view,testing}.rs`, `RULES.md`, `tests/pickup_rules.rs`.
Produces: `Pickup`, `Length { OneDeal (défaut), Target }` (option `length`, valeurs `"OneDeal"`, `"Target"`), `Config { length }`, `pub fn target(players) -> u16` (150/100/75), `Action { Choose { card }, Take { card } }`, `Phase { Choose, Take, Over }`, `State { …, pool: Vec<Rank>, chosen: Vec<Option<Rank>>, queue: Vec<(Seat, Rank)>, last_actor: Seat, piles: Vec<Vec<Rank>>, scores, … }`, événements `Dealt`, `Revealed { cards: Vec<Option<Rank>> }`, `Took { seat, played, card, knocked: Vec<(Seat, u8)> /* seat, count */, pool_knocked: u8 }`, `Stacked { seat, card }`, `PoolDealt { cards }`, `DealOver { scores }`.
Règle de résolution : après révélation, `queue` = ordre d'action ; chaque action prend automatiquement si les cartes prenables sont toutes du même rang, sinon `Phase::Take` pour ce siège (actions `Take { card }`, une par rang prenable).
Tests : `cards_act_from_lowest_to_highest`, `ties_act_from_the_left_of_the_last_actor`, `a_take_knocks_out_the_rank_in_the_pool_and_other_piles`, `knockouts_spare_the_taker`, `nothing_to_take_stacks_the_card`, `a_choice_of_ranks_asks_the_player`, `the_round_ends_and_a_new_pool_is_dealt`, `one_deal_ends_when_hands_are_empty`, `target_redeals_until_someone_reaches_it`, `nobody_sees_who_has_chosen`, `illegal_moves_are_refused`.
Commit `pickup : règles`.

### Task 4: Pickup — IA, registre, web
Choix secret : estimer, pour chaque carte, la meilleure carte prenable après que les cartes plus basses des autres (inconnues : supposées absentes) ont agi = la plus haute carte de la réserve ≤ sa carte ; valeur = cette carte + cartes éliminées chez le meneur − valeur de la carte jouée × 0.3 (garder les hautes) ; prise : rang le plus haut, à égalité celui qui élimine le plus chez les autres. Tests et web comme la tâche 2 ; vue : la réserve, les piles, sa carte choisie, les cartes révélées du tour. Arène 2 et 4.
Commit `pickup : IA, registre, web`.

### Task 5: Market Day — règles
Files: `games/pairs/src/market_day/{mod,view,testing}.rs`, `RULES.md`, `tests/market_day_rules.rs`.
Produces: `MarketDay`, `Config {}`, `Action { Play { card }, Swap { with: usize /* index dans le tour */ }, Keep }`, `Phase { Play, Tinker, Over }`, `pub fn resolve(trick: &[(Seat, Rank)]) -> Resolution { scoring: Vec<bool>, killed: Vec<usize> }` (sans le Bricoleur), `State { …, trick: Vec<(Seat, Rank)>, owner: Vec<Seat> /* après échange */, leader, … }`, événements `Dealt`, `Played`, `TrickScored { cards: Vec<(Seat, Rank, Fate)>, gained, scores }` avec `Fate { Scored, Spoiled, Killed }`, `Swapped { seat, with: Seat, card: Rank }`, `HandOver { scores }`.
Tests : `a_repeated_rank_scores_only_the_last`, `the_first_bandit_scores`, `a_lone_rain_spoils_all_but_royals`, `two_rains_are_ordinary_sevens`, `royals_always_score`, `knights_kill_the_highest_scoring_card_above_three`, `two_knights_kill_two_cards`, `the_tinker_may_swap_with_a_scoring_card_but_not_a_royal`, `a_spoiled_tinker_does_not_act`, `resolution_order_spoiled_royal_rain_knight_tinker`, `the_top_scorer_leads_the_next_trick`, `two_cards_each_at_three_players`, `max_six_or_players_hands`, `illegal_moves_are_refused`.
Commit `market-day : règles`.

### Task 6: Market Day — IA, registre, web
Rollout : jouer la carte qui marque le plus vu le tour en cours (dernier doublon, Pluie seule si elle rapporte, Chevalier s'il tue une carte adverse), 10 % au hasard ; Bricoleur : échanger avec la plus forte carte permise d'un autre joueur. Tests et web comme la tâche 2 ; vue : le tour en cours avec qui a posé quoi. Arène 2 et 4.
Commit `market-day : IA, registre, web`.

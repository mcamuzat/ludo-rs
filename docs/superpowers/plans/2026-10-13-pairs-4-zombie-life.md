# Pairs, plan 4 : Zombie Life — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Zombie Life (3–6 joueurs, variante Origames de Pairs à Refuges et Horde) se joue sur ludo-rs contre l'IA et en ligne, jusqu'à 21 points.

**Architecture:** Quatrième module de `games/pairs`, `zombie_life/`, sous l'id `pairs-zombie-life`, en suivant la procédure « Ajouter un jeu Pairs » de `CLAUDE.md`. Aucun changement de plateforme ni de `deck.rs`.

**Tech Stack:** Rust stable (rand 0.8, rand_chacha 0.3, serde), React 19 + TypeScript, vitest 3.

**Spec:** `docs/superpowers/specs/2026-10-13-pairs-design.md` §12 (règles et points non tranchés), §2, §3, §6, §7 pour les principes communs. Source : règle Origames en 4 pages, *Pairs – Zombie Life*.

## Décisions

Validées par l'utilisateur (spec §12) : cartes retournées en passant = joueurs encore en jeu sans le passeur, plafonné au Refuge ; un simple nombre de cartes cachées ; Horde défaussée en fin de manche ; pioche mélangée une fois puis remélange des défausses ; égalités (plus petite carte, donneur) tirées sur la graine ; victoire partagée à égalité ; aucune statistique.

Prises en écrivant ce plan (la spec §12 est mise à jour en tâche 4) :

- **La carte piochée et les cartes du donneur sont publiques.** Personne d'autre n'agit entre la pioche et la pose, ni pendant la donne, et toutes ces cartes finissent visibles : les cacher ne change aucune décision. Seule information cachée : l'ordre de la pioche. `determinize` = `supply.redeal_hidden` + nouvelle graine.
- **La donne se fait carte par carte** : `Give { card }` donne une carte au prochain siège non servi (à partir de la gauche du donneur, le donneur en dernier), au lieu d'un `Deal { cards }` à 720 permutations. Quand les cartes restantes sont toutes du même rang, elles sont données sans demander.
- `Config { target: Option<u8> }` : `None` ou 0 → 21.

## Global Constraints

- Branche `pairs-zombie-life`. `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --release` avant chaque commit.
- `apply` rend `Illegal` sans modifier l'état, ne panique jamais. Une vue ne contient ni graine ni `Supply`.
- Mélanges et tirages en `ChaCha8Rng` ; jamais de tirage `usize` (tirer un `u32`).
- Aucune statistique dans la vue ni sur le plateau.
- Messages `Illegal` et textes du plateau en français ; code et commentaires en anglais.

## Review Focus

1. **Passer avec moins de cartes que de joueurs restants** : tout le Refuge est caché, jamais de nombre négatif ; chaque élimination en rend une, sans dépasser le Refuge. → `passing_hides_one_card_per_player_still_in` et `an_elimination_turns_one_card_back_up`.
2. **Dernier joueur en jeu** : `Take` refusé, `Draw` et `Pass` permis ; son élimination ou son `Pass` termine la manche. → `the_last_player_in_cannot_take_from_the_horde`.
3. **Carte piochée présente dans les deux zones** : élimination immédiate, Refuge et carte défaussés, pas de phase `Placing`. → `a_card_in_both_zones_overruns_the_player`.
4. **Manche où tout le monde est éliminé** : personne ne marque, la manche suivante démarre quand même, les 55 cartes sont conservées. → `a_round_everyone_loses_scores_nothing` et `cards_are_conserved`.
5. **Action absurde** (`Take` d'un rang absent de la Horde ou déjà au Refuge, `Place` dans une zone qui a le rang, `Give` d'une carte que le donneur n'a pas, coup hors tour ou hors phase) : `Illegal`, état intact. → `absurd_actions_are_refused`.
6. **Plateau une fois passé, éliminé, ou en fin de partie** : rien à cliquer. → tests web.

## Interfaces (toutes les tâches)

```rust
// games/pairs/src/zombie_life/mod.rs
pub struct ZombieLife;
pub const DEFAULT_TARGET: u8 = 21;
pub struct Config { pub target: Option<u8> }
pub enum Zone { Refuge, Horde }
pub enum Action { Draw, Place { zone: Zone }, Take { card: Rank }, Pass, Give { card: Rank } }
pub enum Phase { Dealing, Play, Placing, Over }
pub enum Status { In, Passed, Out }
#[serde(tag = "type")]
pub enum Event {
    Dealt { seat: Seat, card: Rank },
    Starts { seat: Seat },
    Drew { seat: Seat, card: Rank },
    Placed { seat: Seat, card: Rank, zone: Zone },
    Took { seat: Seat, card: Rank },
    Passed { seat: Seat, hidden: u8 },
    Overrun { seat: Seat, card: Rank },
    Scored { seat: Seat, points: u8 },
    Dealer { seat: Seat },
    Reshuffled,
}
pub struct State {
    pub players: u8, pub target: u8, pub round: u16, pub supply: Supply, pub seed: u64,
    pub refuges: Vec<Vec<Rank>>, pub horde: Vec<Rank>,
    pub status: Vec<Status>, pub hidden: Vec<u8>,
    pub eliminated: Vec<Seat>,      // this round, in order
    pub scores: Vec<u16>,
    pub phase: Phase, pub turn: Option<Seat>,
    pub drawn: Option<Rank>,        // Placing
    pub dealing: Vec<Rank>,         // Dealing: the dealer's cards not yet given
    pub events: Vec<Event>,
}
impl State {
    pub fn active(&self) -> Vec<Seat>;            // Status::In, seat order
    pub fn deal_to(&self) -> Option<Seat>;        // Dealing: next seat served
    pub fn overrun_chance(&self, seat: Seat) -> f64; // for the AI only, never in a view
}
// view.rs — #[serde(rename_all = "camelCase")]
pub struct View { round, target, phase, seats: Vec<SeatView>, horde: Vec<Rank>, turn: Option<Seat>,
    drawn: Option<Rank>, dealing: Vec<Rank>, deal_to: Option<Seat>, deck: usize, discard: usize, events }
pub struct SeatView { face_up: Vec<Rank>, face_down: u8, status: Status, score: u16 }
// testing.rs
pub fn table(refuges: &[&[u8]], horde: &[u8], draws: &[u8]) -> State; // round 1, Play, seat 0 to move
```

JSON des actions : `"Draw"`, `"Pass"`, `{"Place":{"zone":"Refuge"}}`, `{"Take":{"card":4}}`, `{"Give":{"card":4}}`.

## Tasks

### Task 1: Règles, vue, RULES.md
**Files:** Create `games/pairs/src/zombie_life/{mod.rs,view.rs,testing.rs}`, `games/pairs/tests/zombie_life_rules.rs` ; Modify `games/pairs/src/lib.rs`, `games/pairs/RULES.md`.
**Produces:** les interfaces ci-dessus sauf `Determinize`/`Heuristics`.
- [ ] Écrire `tests/zombie_life_rules.rs` et `testing.rs` ; les voir échouer (module absent). Tests : les six du Review Focus (1–5), plus `a_new_game_deals_one_card_each_and_the_lowest_starts`, `a_tie_for_the_lowest_card_is_drawn_from_the_seed`, `a_drawn_card_goes_to_the_refuge_or_the_horde`, `taking_moves_a_card_from_the_horde`, `the_turn_skips_players_who_passed_or_are_out`, `passed_players_score_their_face_up_cards`, `the_lowest_score_deals_the_next_round` (égalité → premier éliminé), `the_dealer_gives_one_card_to_each_seat`, `identical_cards_are_dealt_without_asking`, `the_game_ends_at_the_target_and_ties_are_shared`, `the_target_comes_from_the_config`, `the_view_holds_no_seed_and_no_supply`, `the_deal_of_a_seed_is_pinned`, `random_games_end` (3 à 6 joueurs).
- [ ] Écrire `mod.rs`, `view.rs`, la section de `RULES.md` (sources [Z], ❓ numérotés) ; tous les tests passent.
- [ ] Commit `pairs: Zombie Life rules, view and RULES.md`.

### Task 2: Déterminisation et conformance
**Files:** Create `games/pairs/src/zombie_life/determinize.rs` ; Modify `games/pairs/tests/conformance.rs`.
- [ ] Tests (`the_next_card_changes_but_not_the_view`, en `Play` et en `Dealing`) et cas de conformance de 3 à 6 joueurs, plus `target: Some(1)` et `Some(255)` ; implémentation ; commit `pairs: Zombie Life determinization and conformance`.

### Task 3: IA et registre
**Files:** Create `games/pairs/src/zombie_life/heuristics.rs` ; Modify `crates/ludo-registry/src/lib.rs`, `CLAUDE.md`.
**Politique de rollout :** en `Dealing`, au hasard ; en `Placing`, le Refuge s'il est permis, sinon la Horde ; en `Play`, avec `v` = cartes que rapporterait un `Pass` maintenant et `p = overrun_chance` : prendre dans la Horde le rang dont il reste le moins d'exemplaires non localisés (le plus souvent), sinon piocher si `v == 0` ou `p * (v + 1) < 1`, sinon passer.
- [ ] Tests (ne passe jamais quand passer ne rapporte rien, passe quand le risque est élevé et le gain acquis, pioche quand le risque est nul, action toujours légale de 3 à 6 joueurs) ; implémentation ; `ludo arena pairs-zombie-life 400 hard easy 3` ≥ 0,55, consigné dans `CLAUDE.md` ; commit `pairs: Zombie Life AI and registry entry`.

### Task 4: Plateau web et docs
**Files:** Create `web/src/games/pairs/zombie-life/{types.ts,Board.tsx}`, `web/src/games/pairs/__tests__/zombie-life.test.tsx` ; Modify `web/src/games/registry.ts`, `web/src/games/pairs/pairs.css` (si besoin), `README.md`, spec (§12).
**Plateau :** par siège, cartes visibles, pile « cachées », score « n / 21 », état (passé / submergé) ; au centre la manche, la Horde (cartes cliquables pour `Take`), la carte piochée avec « Dans mon Refuge » / « Dans la Horde », les cartes du donneur (« Donner ce 4 à Bob »), « Piocher », « Passer », pioche et défausse, événements en phrases.
- [ ] Test du plateau (chaque contrôle envoie exactement une action légale en `Play`, `Placing` et `Dealing` ; rien à cliquer hors de son tour, passé, éliminé, spectateur ou en fin de partie ; aucun pourcentage ; chaque événement en phrase) ; implémentation ; `pnpm test && pnpm build` ; une partie à 6 sièges dans un navigateur sans affichage ; commit `web: Zombie Life board`.

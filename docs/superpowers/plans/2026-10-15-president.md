# Cartes traditionnelles et Président (spec 4, plan 2) — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Le Président (4 à 7 joueurs, variantes révolution, interdiction de finir sur un 2, jokers forts ou wild) se joue sur ludo-rs contre l'IA et en ligne, dans une nouvelle famille « Cartes traditionnelles » de l'accueil.

**Architecture:** Nouvelle crate `games/cards`, calquée sur `games/pairs` : le paquet commun dans `deck.rs` (couleurs, valeurs, paquets 52/32/54, donne à graine ChaCha), un module par jeu (`president/` : `mod.rs` règles et `Game`, `combos.rs` coups permis, `view.rs`, `determinize.rs`, `heuristics.rs`, `testing.rs`). Les options passent par le mécanisme du plan 1 (`Game::options`, `check_config`, `GameOptions` du lobby). Côté site, `web/src/games/cards/` porte la carte à jouer commune et le plateau du Président.

**Tech Stack:** Rust stable (rand 0.8, rand_chacha 0.3, serde), React 19 + TypeScript, vitest 3.

**Spec:** `docs/superpowers/specs/2026-10-15-president-design.md` : §3 (règles, points ❓ validés par l'utilisateur), §4 (moteur), §6 (web), §7 (tests), §8 (livraison, plan 2). Sources des règles : [W] Wikipédia « Trou du cul (jeu) », [P] Pagat « President ».

## Global Constraints

- Branche `cards-president`, dans un worktree (une autre session peut travailler sur `main`). `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --release` avant chaque commit Rust ; `cd web && pnpm test && pnpm type-check` avant chaque commit web (le type-check demande le wasm : `./scripts/build-wasm.sh dev`).
- `apply` rend `Illegal` sans modifier l'état et ne panique jamais, quel que soit le siège ou l'action reçus. Une vue ne contient jamais la graine, la main d'un autre, ni les cartes d'un échange auquel le spectateur n'a pas pris part.
- Mélanges en `ChaCha8Rng` (jamais `SmallRng`) ; jamais de tirage `usize`. Une même graine donne la même donne en natif et en wasm.
- Aucune statistique dans la vue ni sur le plateau (ni compte des cartes jouées, ni estimation) : règle de l'utilisateur.
- Le moteur possède toute la logique ; le plateau affiche la vue et envoie une action prise dans `legalActions`.
- Chaque règle de `games/cards/RULES.md` cite [W] ou [P] ; un point non tranché est marqué ❓. Le rappel `web/src/games/cards/president/rules.ts` reste d'accord avec `RULES.md`.
- Messages `Illegal` et textes du site en français ; code et commentaires en anglais.
- Commits avec l'adresse `marc.camuzat@gmail.com` (config locale du dépôt), terminés par `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.

## Review Focus

1. **Un joueur sorti n'est plus jamais sollicité** : le tour saute les sièges sans cartes, même quand un pli continue. → `players_who_are_out_are_skipped` (Task 2).
2. **L'échange reste privé** : ni l'événement `Exchanged`, ni la vue d'un tiers ou d'un spectateur ne montrent les cartes échangées. → `the_exchange_is_private` (Task 2).
3. **Le Président peut rendre une carte qu'il vient de recevoir** (un 2, un joker) : le choix porte sur toute sa main. → `the_president_may_hand_back_what_they_received` (Task 2).
4. **Une révolution ne survit pas à la manche** : la manche suivante (et son échange) reprend l'ordre normal. → `a_new_round_restores_the_normal_order` (Task 3).
5. **Une sélection qui ne forme aucun coup permis** (deux valeurs différentes, une carte de trop) laisse « Jouer » inactif. → test `selectionAction` « mixed ranks » (Task 5).

## Interfaces (toutes les tâches)

```rust
// games/cards/src/deck.rs
pub enum Suit { Spades, Hearts, Diamonds, Clubs }              // Suit::ALL
pub enum Rank { Two, Three, …, King, Ace }                       // Rank::ALL, ordre usuel
pub enum Card { Natural { rank: Rank, suit: Suit }, Joker }      // serde tag "kind"
pub enum Face { Natural { rank: Rank }, Joker }                  // serde tag "kind"
impl Card { pub fn face(self) -> Face; pub fn rank(self) -> Option<Rank> }
pub fn pack_52() -> Vec<Card>; pub fn pack_32() -> Vec<Card>; pub fn pack_54() -> Vec<Card>;
pub fn shuffled(cards: Vec<Card>, seed: u64) -> Vec<Card>;
pub struct Deal { pub hands: Vec<Vec<Card>>, pub set_aside: Vec<Card> }
pub fn deal_evenly(cards: Vec<Card>, players: u8, seed: u64) -> Deal;
pub fn redeal<R: Rng + ?Sized>(hands: &mut [&mut Vec<Card>], rng: &mut R);

// games/cards/src/president/mod.rs
pub struct President;                                // id "cards-president"
pub const DEFAULT_ROUNDS: u8 = 5; pub const MAX_ROUNDS: u8 = 10;
pub enum Jokers { Off, High, Wild }                  // défaut Off
pub struct Config { pub rounds: u8, pub revolution: bool, pub no_finish_on_two: bool, pub jokers: Jokers }
pub enum Action { Play { rank: Option<Rank>, naturals: u8, jokers: u8 }, Pass, Give { faces: Vec<Face> } }
pub enum Title { President, VicePresident, Neutral, Secretary, Scum }
pub fn title(place: usize, players: usize) -> Title;
pub fn standings(scores: &[u32]) -> Vec<u8>;
pub enum Phase { Exchange, Play, Over }
pub struct Trick { pub seat: Seat, pub rank: Option<Rank>, pub cards: Vec<Card> }
pub struct Transfer { pub from: Seat, pub to: Seat, pub cards: Vec<Card> }
pub enum Event { RoundStarted{round,dealer,leader,aside}, Exchanged{from,to,count}, Played{seat,rank,cards},
                 Passed{seat}, TrickClosed{leader}, Revolution{reversed}, Finished{seat,fouled},
                 RoundScored{order,points,scores} }       // serde tag "type"
pub struct State { pub players, pub config, pub seed, pub round_no, pub phase, pub hands, pub set_aside,
                   pub titles, pub reversed, pub trick, pub passes, pub turn, pub out, pub fouled,
                   pub transfers, pub to_give, pub scores, pub events }
// president/combos.rs
pub const JOKERS_ALONE: u8 = 13;
pub fn strength(rank: Rank, reversed: bool) -> u8; pub fn value(rank: Option<Rank>, reversed: bool) -> u8;
pub fn face_value(face: Face) -> u8; pub fn exchange_value(card: Card) -> u8;
pub fn hand_order(card: Card, reversed: bool) -> (u8, Card);
pub fn plays(hand: &[Card], trick: Option<&Trick>, jokers: Jokers, reversed: bool) -> Vec<Action>;
// president/testing.rs
pub fn c(text: &str) -> Card; pub fn cards(text: &str) -> Vec<Card>;
pub fn table(config: Config, hands: &[&str], turn: u8) -> State;
pub fn next_round(config: Config, titles: &[Title], hands: &[&str]) -> State;
```

JSON (site) : carte `{"kind":"Natural","rank":"Seven","suit":"Hearts"}` / `{"kind":"Joker"}` ; action `{"Play":{"rank":"Seven","naturals":2,"jokers":0}}`, `"Pass"`, `{"Give":{"faces":[{"kind":"Natural","rank":"Three"}]}}` ; vue en camelCase (`roundNo`, `handSizes`, `setAside`, `toGive`…).

---

### Task 1: Crate `games/cards` et paquet commun

**Files:**
- Create: `games/cards/Cargo.toml`, `games/cards/src/lib.rs`, `games/cards/src/deck.rs`
- Modify: `Cargo.toml` (membre `games/cards`, dépendance de workspace `cards`)

**Interfaces:**
- Produces: tout le bloc `deck.rs` ci-dessus.

- [ ] **Step 1: Write the failing tests**

`Cargo.toml` (racine) : ajouter `"games/cards"` à `members` et `cards = { path = "games/cards" }` à `[workspace.dependencies]`.

`games/cards/Cargo.toml` :

```toml
[package]
name = "cards"
description = "Classic card games (French-suited pack), for ludo-rs"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
ludo-core.workspace = true
rand.workspace = true
rand_chacha.workspace = true
serde.workspace = true

[dev-dependencies]
serde_json.workspace = true
```

`games/cards/src/lib.rs` :

```rust
//! Classic card games played with a French-suited pack. Rules and their
//! sources: RULES.md.

pub mod deck;
```

`games/cards/src/deck.rs`, d'abord les seuls tests (le module entier est écrit à l'étape 3) :

```rust
#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    use super::*;

    #[test]
    fn packs_hold_the_right_cards() {
        let full = pack_52();
        assert_eq!(full.len(), 52);
        assert_eq!(full.iter().collect::<HashSet<_>>().len(), 52);
        let piquet = pack_32();
        assert_eq!(piquet.len(), 32);
        assert!(piquet.iter().all(|c| c.rank() >= Some(Rank::Seven)));
        let with_jokers = pack_54();
        assert_eq!(with_jokers.len(), 54);
        assert_eq!(with_jokers.iter().filter(|&&c| c == Card::Joker).count(), 2);
    }

    #[test]
    fn an_even_deal_sets_the_rest_aside() {
        let deal = deal_evenly(pack_52(), 5, 1);
        assert!(deal.hands.iter().all(|h| h.len() == 10));
        assert_eq!(deal.set_aside.len(), 2);
        let mut all: Vec<Card> = deal.hands.concat();
        all.extend(&deal.set_aside);
        all.sort();
        let mut pack = pack_52();
        pack.sort();
        assert_eq!(all, pack);
        assert!(deal_evenly(pack_52(), 4, 1).set_aside.is_empty());
    }

    #[test]
    fn the_deal_of_a_seed_is_pinned() {
        let deal = deal_evenly(pack_52(), 4, 42);
        assert_eq!(deal, deal_evenly(pack_52(), 4, 42));
        assert_ne!(deal.hands[0], deal_evenly(pack_52(), 4, 43).hands[0]);
        // Pinned: a change of shuffle would make browser logs unreadable.
        let pinned: Vec<Card> = Vec::new();
        assert_eq!(deal.hands[0][..4].to_vec(), pinned);
    }

    #[test]
    fn redeal_keeps_the_sizes_and_the_cards() {
        let mut a = pack_52()[..5].to_vec();
        let mut b = pack_52()[5..12].to_vec();
        let before: HashSet<Card> = a.iter().chain(&b).copied().collect();
        redeal(&mut [&mut a, &mut b], &mut SmallRng::seed_from_u64(3));
        assert_eq!((a.len(), b.len()), (5, 7));
        assert_eq!(a.iter().chain(&b).copied().collect::<HashSet<_>>(), before);
    }

    #[test]
    fn a_card_shows_its_face() {
        let seven = Card::Natural { rank: Rank::Seven, suit: Suit::Hearts };
        assert_eq!(seven.face(), Face::Natural { rank: Rank::Seven });
        assert_eq!(seven.rank(), Some(Rank::Seven));
        assert_eq!(Card::Joker.face(), Face::Joker);
        assert_eq!(Card::Joker.rank(), None);
        assert_eq!(
            serde_json::to_value(seven).unwrap(),
            serde_json::json!({ "kind": "Natural", "rank": "Seven", "suit": "Hearts" })
        );
        assert_eq!(serde_json::to_value(Card::Joker).unwrap(), serde_json::json!({ "kind": "Joker" }));
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p cards --release`
Expected: FAIL à la compilation (`pack_52`, `deal_evenly`… introuvables).

- [ ] **Step 3: Implement**

En tête de `games/cards/src/deck.rs`, au-dessus du module de tests :

```rust
//! The French-suited pack shared by the classic card games. Each game ranks
//! the cards its own way: this module knows no order of strength.

use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Suit {
    Spades,
    Hearts,
    Diamonds,
    Clubs,
}

impl Suit {
    pub const ALL: [Suit; 4] = [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs];
}

/// In the usual order of a pack, two first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Rank {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl Rank {
    pub const ALL: [Rank; 13] = [
        Rank::Two,
        Rank::Three,
        Rank::Four,
        Rank::Five,
        Rank::Six,
        Rank::Seven,
        Rank::Eight,
        Rank::Nine,
        Rank::Ten,
        Rank::Jack,
        Rank::Queen,
        Rank::King,
        Rank::Ace,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Card {
    Natural { rank: Rank, suit: Suit },
    Joker,
}

/// What a card shows when suits do not matter.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Face {
    Natural { rank: Rank },
    Joker,
}

impl Card {
    pub fn face(self) -> Face {
        match self {
            Card::Natural { rank, .. } => Face::Natural { rank },
            Card::Joker => Face::Joker,
        }
    }

    pub fn rank(self) -> Option<Rank> {
        match self {
            Card::Natural { rank, .. } => Some(rank),
            Card::Joker => None,
        }
    }
}

pub fn pack_52() -> Vec<Card> {
    Suit::ALL
        .iter()
        .flat_map(|&suit| Rank::ALL.iter().map(move |&rank| Card::Natural { rank, suit }))
        .collect()
}

/// The piquet pack: seven to ace.
pub fn pack_32() -> Vec<Card> {
    pack_52()
        .into_iter()
        .filter(|c| c.rank() >= Some(Rank::Seven))
        .collect()
}

pub fn pack_54() -> Vec<Card> {
    let mut pack = pack_52();
    pack.extend([Card::Joker, Card::Joker]);
    pack
}

/// `cards` shuffled from `seed`. ChaCha, not `SmallRng`: the latter is a
/// different generator on wasm32 and on 64-bit targets, and a log recorded
/// in the browser must replay natively.
pub fn shuffled(mut cards: Vec<Card>, seed: u64) -> Vec<Card> {
    cards.shuffle(&mut ChaCha8Rng::seed_from_u64(seed));
    cards
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Deal {
    pub hands: Vec<Vec<Card>>,
    /// The cards left over, shown to everyone and kept out of the round.
    pub set_aside: Vec<Card>,
}

/// Shuffles and deals the same number of cards to each of `players`.
pub fn deal_evenly(cards: Vec<Card>, players: u8, seed: u64) -> Deal {
    let mut cards = shuffled(cards, seed);
    let n = usize::from(players);
    let set_aside = cards.split_off(cards.len() - cards.len() % n);
    let mut hands = vec![Vec::new(); n];
    for (i, card) in cards.into_iter().enumerate() {
        hands[i % n].push(card);
    }
    Deal { hands, set_aside }
}

/// For determinization: the cards of `hands` are shuffled among them, each
/// hand keeping its size.
pub fn redeal<R: Rng + ?Sized>(hands: &mut [&mut Vec<Card>], rng: &mut R) {
    let sizes: Vec<usize> = hands.iter().map(|h| h.len()).collect();
    let mut pool = Vec::new();
    for hand in hands.iter_mut() {
        pool.append(hand);
    }
    pool.shuffle(rng);
    for (hand, size) in hands.iter_mut().zip(sizes) {
        let at = pool.len() - size;
        hand.extend(pool.drain(at..));
    }
}
```

- [ ] **Step 4: Fix the pinned deal**

Run: `cargo test -p cards --release the_deal_of_a_seed_is_pinned`
Expected: FAIL sur la dernière assertion seulement, avec `left: [Natural { rank: …, suit: … }, …]`. Remplacer `let pinned: Vec<Card> = Vec::new();` par ces quatre cartes, écrites `vec![Card::Natural { rank: Rank::…, suit: Suit::… }, …]` (c'est la valeur figée, comme `the_deal_of_a_seed_is_pinned` de Pairs).

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p cards --release`
Expected: PASS (5 tests).

- [ ] **Step 6: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --release
git add Cargo.toml Cargo.lock games/cards
git commit -m "cards: crate des jeux de cartes classiques et paquet commun

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: Président, règles de base

**Files:**
- Create: `games/cards/src/president/mod.rs`, `combos.rs`, `view.rs`, `testing.rs`
- Create: `games/cards/RULES.md`
- Create: `games/cards/tests/president_rules.rs`
- Modify: `games/cards/src/lib.rs` (`pub mod president; pub use president::President;`)

**Interfaces:**
- Consumes: `deck.rs` (Task 1) ; `ludo_core::{Game, GameOption, OptionKind, …}`.
- Produces: tout le bloc `president` ci-dessus, sauf `determinize.rs` et `heuristics.rs` (Task 4). Les variantes révolution et interdiction de finir sur un 2 sont branchées en Task 3 (les champs `reversed` et `fouled` existent déjà ; les modes de jokers sont complets ici, dans `combos.rs`).

- [ ] **Step 1: Write the failing tests**

`games/cards/src/lib.rs` :

```rust
//! Classic card games played with a French-suited pack. Rules and their
//! sources: RULES.md.

pub mod deck;
pub mod president;

pub use president::President;
```

`games/cards/tests/president_rules.rs` :

```rust
use cards::deck::{Face, Rank};
use cards::president::testing::{c, cards, next_round, table};
use cards::president::{
    standings, title, Action, Config, Event, Jokers, Phase, President, State, Title,
};
use ludo_core::{Game, Match, Outcome, Pending, Seat, Viewer};
use Title::{President as Pres, Scum, Secretary, VicePresident as Vice};

fn play(rank: Rank, naturals: u8) -> Action {
    Action::Play { rank: Some(rank), naturals, jokers: 0 }
}

fn wild(rank: Rank, naturals: u8, jokers: u8) -> Action {
    Action::Play { rank: Some(rank), naturals, jokers }
}

fn jokers(n: u8) -> Action {
    Action::Play { rank: None, naturals: 0, jokers: n }
}

fn nat(rank: Rank) -> Face {
    Face::Natural { rank }
}

fn one_round() -> Config {
    Config { rounds: 1, ..Config::default() }
}

fn act(s: &mut State, seat: u8, a: Action) {
    President::apply(s, Seat(seat), &a).unwrap_or_else(|e| panic!("{a:?} par {seat} : {e}"));
}

#[test]
fn everyone_gets_the_same_number_of_cards_and_the_rest_is_shown() {
    let s = President::new(&Config::default(), 4, 1);
    assert!(s.hands.iter().all(|h| h.len() == 13));
    assert!(s.set_aside.is_empty());
    let s = President::new(&Config::default(), 5, 1);
    assert!(s.hands.iter().all(|h| h.len() == 10));
    assert_eq!(s.set_aside.len(), 2);
    assert_eq!(President::view(&s, Viewer::Seat(Seat(3))).set_aside, s.set_aside);
    // [W] Seat 0 deals the first round; the player on its left leads.
    assert_eq!(President::pending(&s), Pending::one(Seat(1)));
    let s = President::new(&Config { jokers: Jokers::Wild, ..Config::default() }, 4, 1);
    assert!(s.hands.iter().all(|h| h.len() == 13));
    assert_eq!(s.set_aside.len(), 2);
}

#[test]
fn a_lead_is_one_to_four_cards_of_a_rank() {
    let s = table(one_round(), &["3s 3h 7d 2c", "4s", "5s", "6s"], 0);
    assert_eq!(
        President::legal_actions(&s, Seat(0)),
        vec![
            play(Rank::Two, 1),
            play(Rank::Three, 1),
            play(Rank::Three, 2),
            play(Rank::Seven, 1)
        ]
    );
    assert!(President::legal_actions(&s, Seat(1)).is_empty());
}

#[test]
fn a_play_is_beaten_by_as_many_cards_of_a_higher_rank() {
    let mut s = table(one_round(), &["7s 7h 9d", "6s 6h 8s 8h Ks 2c 2d", "5s", "4s"], 0);
    act(&mut s, 0, play(Rank::Seven, 2));
    assert_eq!(
        President::legal_actions(&s, Seat(1)),
        vec![play(Rank::Two, 2), play(Rank::Eight, 2), Action::Pass]
    );
}

#[test]
fn passing_does_not_stop_you_playing_later_in_the_trick() {
    let mut s = table(one_round(), &["5s 9s", "6s Ks", "7s Qs", "8s Js"], 0);
    act(&mut s, 0, play(Rank::Five, 1));
    act(&mut s, 1, Action::Pass);
    act(&mut s, 2, play(Rank::Seven, 1));
    act(&mut s, 3, Action::Pass);
    act(&mut s, 0, Action::Pass);
    // [P] Seat 1 passed earlier and may play again.
    act(&mut s, 1, play(Rank::King, 1));
}

#[test]
fn when_everyone_else_passes_the_last_player_leads_anything() {
    let mut s = table(one_round(), &["5s 9s 9h", "6s", "7s", "8s"], 0);
    act(&mut s, 0, play(Rank::Five, 1));
    for seat in 1..=3 {
        act(&mut s, seat, Action::Pass);
    }
    assert!(s.trick.is_none());
    assert!(s.events.contains(&Event::TrickClosed { leader: Seat(0) }));
    assert_eq!(President::pending(&s), Pending::one(Seat(0)));
    assert!(President::legal_actions(&s, Seat(0)).contains(&play(Rank::Nine, 2)));
}

#[test]
fn if_the_last_player_went_out_the_player_on_their_left_leads() {
    let mut s = table(one_round(), &["As", "6s 6h", "7s 7h", "8s 8h"], 0);
    act(&mut s, 0, play(Rank::Ace, 1));
    for seat in 1..=3 {
        act(&mut s, seat, Action::Pass);
    }
    assert_eq!(President::pending(&s), Pending::one(Seat(1)));
}

#[test]
fn players_who_are_out_are_skipped() {
    let mut s = table(one_round(), &["3s", "4s 4h", "5s 5h", "6s 6h"], 0);
    act(&mut s, 0, play(Rank::Three, 1));
    act(&mut s, 1, play(Rank::Four, 1));
    act(&mut s, 2, play(Rank::Five, 1));
    act(&mut s, 3, play(Rank::Six, 1));
    assert_eq!(President::pending(&s), Pending::one(Seat(1)));
    assert!(President::legal_actions(&s, Seat(0)).is_empty());
}

#[test]
fn the_order_of_going_out_gives_titles_and_points() {
    let mut s = table(one_round(), &["3s", "4s", "5s", "6s 7s"], 0);
    act(&mut s, 0, play(Rank::Three, 1));
    act(&mut s, 1, play(Rank::Four, 1));
    act(&mut s, 2, play(Rank::Five, 1));
    // Seat 3 still holds cards: the round is over, it is the Scum.
    assert_eq!(s.phase, Phase::Over);
    assert_eq!(s.titles, vec![Pres, Vice, Secretary, Scum]);
    assert_eq!(s.scores, vec![3, 2, 1, 0]);
    assert_eq!(President::outcome(&s), Some(Outcome::Ranked(vec![0, 1, 2, 3])));
    assert_eq!(
        (0..5).map(|place| title(place, 5)).collect::<Vec<_>>(),
        vec![Pres, Vice, Title::Neutral, Secretary, Scum]
    );
}

#[test]
fn equal_scores_share_a_rank() {
    assert_eq!(standings(&[3, 3, 1, 0]), vec![0, 0, 2, 3]);
}

#[test]
fn the_game_ends_after_the_configured_rounds() {
    let mut s = President::new(&Config { rounds: 2, ..Config::default() }, 4, 5);
    let mut steps = 0;
    while let Some(&seat) = President::pending(&s).0.first() {
        let a = President::timeout_action(&s, seat).expect("a pending seat has a move");
        act(&mut s, seat.0, a);
        steps += 1;
        assert!(steps < 10_000, "the game does not end");
    }
    assert_eq!((s.round_no, s.phase), (2, Phase::Over));
    assert_eq!(s.scores.iter().sum::<u32>(), 2 * (3 + 2 + 1));
}

#[test]
fn the_scum_and_the_secretary_hand_over_their_best_cards() {
    let s = next_round(
        Config::default(),
        &[Scum, Pres, Secretary, Vice],
        &["4s 9s Ks 2h", "3s 3h 5s 6s", "4h 8s Qs Ah", "3d 5h 7s 9h"],
    );
    assert_eq!(s.round_no, 2);
    assert_eq!(s.phase, Phase::Exchange);
    assert_eq!(s.hands[0], cards("4s 9s"));
    assert!(s.hands[1].contains(&c("2h")) && s.hands[1].contains(&c("Ks")));
    assert_eq!(s.hands[2], cards("4h 8s Qs"));
    assert!(s.hands[3].contains(&c("Ah")));
    assert_eq!(President::pending(&s), Pending(vec![Seat(1), Seat(3)]));
    assert_eq!(President::view(&s, Viewer::Seat(Seat(0))).given, cards("2h Ks"));
    assert_eq!(President::view(&s, Viewer::Seat(Seat(3))).received, cards("Ah"));
}

#[test]
fn the_president_and_the_vice_hand_cards_back_then_play_starts() {
    let mut s = next_round(
        Config::default(),
        &[Scum, Pres, Secretary, Vice],
        &["4s 9s Ks 2h", "3s 3h 5s 6s", "4h 8s Qs Ah", "3d 5h 7s 9h"],
    );
    assert!(President::apply(&mut s.clone(), Seat(0), &Action::Pass).is_err());
    act(&mut s, 1, Action::Give { faces: vec![nat(Rank::Three), nat(Rank::Three)] });
    assert_eq!(President::pending(&s), Pending::one(Seat(3)));
    act(&mut s, 3, Action::Give { faces: vec![nat(Rank::Three)] });
    assert_eq!(s.phase, Phase::Play);
    assert_eq!(s.hands[0], cards("4s 9s 3s 3h"));
    assert_eq!(s.hands[2], cards("4h 8s Qs 3d"));
    // [W] The Scum deals; the player on its left leads.
    assert_eq!(President::pending(&s), Pending::one(Seat(1)));
}

#[test]
fn the_president_may_hand_back_what_they_received() {
    let s = next_round(
        Config::default(),
        &[Scum, Pres, Secretary, Vice],
        &["4s 9s Ks 2h", "3s 3h 5s 6s", "4h 8s Qs Ah", "3d 5h 7s 9h"],
    );
    let gifts = President::legal_actions(&s, Seat(1));
    assert!(gifts.contains(&Action::Give { faces: vec![nat(Rank::Two), nat(Rank::King)] }));
    assert!(gifts.iter().all(|g| matches!(g, Action::Give { faces } if faces.len() == 2)));
}

#[test]
fn a_joker_is_the_best_card_to_hand_over() {
    let s = next_round(
        Config { jokers: Jokers::High, ..Config::default() },
        &[Scum, Pres, Secretary, Vice],
        &["Jo 2s 5s", "3s", "4s Ah", "6s"],
    );
    assert_eq!(s.hands[0], cards("5s"));
    assert!(President::legal_actions(&s, Seat(1)).contains(&Action::Give {
        faces: vec![nat(Rank::Two), Face::Joker]
    }));
}

#[test]
fn the_exchange_is_private() {
    let s = next_round(
        Config::default(),
        &[Scum, Pres, Secretary, Vice],
        &["4s 9s Ks 2h", "3s 3h 5s 6s", "4h 8s Qs Ah", "3d 5h 7s 9h"],
    );
    let third = President::view(&s, Viewer::Seat(Seat(2)));
    assert!(third.received.is_empty());
    assert_eq!(third.given, cards("Ah"));
    let watcher = President::view(&s, Viewer::Spectator);
    assert!(watcher.received.is_empty() && watcher.given.is_empty() && watcher.hand.is_empty());
    for e in &third.events {
        if let Event::Exchanged { count, .. } = e {
            assert!(*count <= 2);
        }
    }
    let json = serde_json::to_string(&third).unwrap();
    assert!(!json.contains("seed"));
}

#[test]
fn a_hand_is_shown_weakest_first() {
    let s = table(one_round(), &["2s As 3s", "4s", "5s", "6s"], 0);
    assert_eq!(President::view(&s, Viewer::Seat(Seat(0))).hand, cards("3s As 2s"));
}

#[test]
fn strong_jokers_beat_twos_alone_or_as_a_pair() {
    let cfg = Config { jokers: Jokers::High, ..one_round() };
    let mut s = table(cfg, &["2s 2h 3s", "Jo Jo 7s", "5s", "6s"], 0);
    act(&mut s, 0, play(Rank::Two, 2));
    assert_eq!(President::legal_actions(&s, Seat(1)), vec![jokers(2), Action::Pass]);
}

#[test]
fn a_strong_joker_never_joins_other_cards() {
    let cfg = Config { jokers: Jokers::High, ..one_round() };
    let s = table(cfg, &["3s", "Jo 9s 9h", "5s", "6s"], 1);
    let legal = President::legal_actions(&s, Seat(1));
    assert!(legal.contains(&jokers(1)));
    assert!(!legal.iter().any(|a| matches!(a, Action::Play { rank: Some(_), jokers: 1.., .. })));
}

#[test]
fn a_wild_joker_completes_a_set() {
    let cfg = Config { jokers: Jokers::Wild, ..one_round() };
    let mut s = table(cfg, &["6s 6h 4s", "8s Jo 3s", "5s", "9s"], 0);
    act(&mut s, 0, play(Rank::Six, 2));
    assert_eq!(President::legal_actions(&s, Seat(1)), vec![wild(Rank::Eight, 1, 1), Action::Pass]);
}

#[test]
fn a_wild_set_does_not_beat_an_equal_natural_set() {
    let cfg = Config { jokers: Jokers::Wild, ..one_round() };
    let mut s = table(cfg, &["7s 7h 4s", "7d Jo 3s", "5s", "9s"], 0);
    act(&mut s, 0, play(Rank::Seven, 2));
    assert_eq!(President::legal_actions(&s, Seat(1)), vec![Action::Pass]);
}

#[test]
fn wild_jokers_alone_are_the_strongest_set_of_their_size() {
    let cfg = Config { jokers: Jokers::Wild, ..one_round() };
    let mut s = table(cfg, &["2s 2h 4s", "Jo Jo 5s", "6s", "9s"], 0);
    act(&mut s, 0, play(Rank::Two, 2));
    assert_eq!(President::legal_actions(&s, Seat(1)), vec![jokers(2), Action::Pass]);
}

#[test]
fn absurd_actions_are_refused_and_change_nothing() {
    let s = table(one_round(), &["5s 9s", "6s", "7s", "8s"], 0);
    for (seat, a) in [
        (1, play(Rank::Six, 1)),
        (0, play(Rank::Five, 2)),
        (0, play(Rank::King, 1)),
        (0, Action::Pass),
        (0, Action::Give { faces: vec![] }),
        (0, jokers(1)),
        (9, play(Rank::Five, 1)),
    ] {
        let mut copy = s.clone();
        assert!(President::apply(&mut copy, Seat(seat), &a).is_err(), "{a:?}");
        assert_eq!(copy, s);
    }
    let mut over = table(one_round(), &["3s", "4s", "5s", "6s 7s"], 0);
    act(&mut over, 0, play(Rank::Three, 1));
    act(&mut over, 1, play(Rank::Four, 1));
    act(&mut over, 2, play(Rank::Five, 1));
    assert!(President::apply(&mut over, Seat(3), &play(Rank::Six, 1)).is_err());
}

#[test]
fn a_number_of_rounds_out_of_bounds_is_refused() {
    for rounds in [0, 11] {
        let e = Match::<President>::new(Config { rounds, ..Config::default() }, 4, 1)
            .err()
            .expect("refused");
        assert_eq!(e.0, "Le Président se joue en 1 à 10 manches.");
    }
}

#[test]
fn the_clock_passes_or_plays_the_weakest_cards() {
    let s = table(one_round(), &["9s 3h 3d", "4s", "5s", "6s"], 0);
    assert_eq!(President::timeout_action(&s, Seat(0)), Some(play(Rank::Three, 1)));
    let mut s = s;
    act(&mut s, 0, play(Rank::Three, 1));
    assert_eq!(President::timeout_action(&s, Seat(1)), Some(Action::Pass));
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p cards --release --test president_rules`
Expected: FAIL à la compilation (`cards::president` introuvable).

- [ ] **Step 3: Implement `combos.rs`**

`games/cards/src/president/combos.rs` :

```rust
//! What may be played on what, under the order in force. [W][P]

use super::{Action, Jokers, Trick};
use crate::deck::{Card, Face, Rank};

/// Jokers played alone outrank everything, revolution or not.
pub const JOKERS_ALONE: u8 = 13;

/// [W] 3 lowest, then 4 … king, ace, and 2 highest; a revolution reverses it.
pub fn strength(rank: Rank, reversed: bool) -> u8 {
    let normal = match rank {
        Rank::Two => 12,
        other => other as u8 - 1,
    };
    if reversed {
        12 - normal
    } else {
        normal
    }
}

/// The value of a play of `rank` (`None`: jokers alone).
pub fn value(rank: Option<Rank>, reversed: bool) -> u8 {
    rank.map_or(JOKERS_ALONE, |r| strength(r, reversed))
}

/// For the exchange, always in the normal order: a joker is the best card.
pub fn face_value(face: Face) -> u8 {
    match face {
        Face::Joker => JOKERS_ALONE,
        Face::Natural { rank } => strength(rank, false),
    }
}

pub fn exchange_value(card: Card) -> u8 {
    face_value(card.face())
}

/// Where a card sorts in a hand under the order in force: weakest first.
pub fn hand_order(card: Card, reversed: bool) -> (u8, Card) {
    let v = match card.face() {
        Face::Joker => JOKERS_ALONE,
        Face::Natural { rank } => strength(rank, reversed),
    };
    (v, card)
}

/// Every play from `hand`. [W] A lead is one to four cards of a rank; on a
/// trick, as many cards of a strictly higher value, or a pass. Jokers:
/// [W][P] strong ones play alone or as a pair; [P] wild ones join a rank,
/// four cards at most, or play alone.
pub fn plays(hand: &[Card], trick: Option<&Trick>, jokers: Jokers, reversed: bool) -> Vec<Action> {
    let held_jokers = hand.iter().filter(|&&c| c == Card::Joker).count() as u8;
    let mut plays = Vec::new();
    for rank in Rank::ALL {
        let held = hand.iter().filter(|c| c.rank() == Some(rank)).count() as u8;
        for naturals in 1..=held {
            let extra = if jokers == Jokers::Wild {
                held_jokers.min(4 - naturals)
            } else {
                0
            };
            for j in 0..=extra {
                plays.push(Action::Play {
                    rank: Some(rank),
                    naturals,
                    jokers: j,
                });
            }
        }
    }
    if jokers != Jokers::Off {
        for j in 1..=held_jokers {
            plays.push(Action::Play {
                rank: None,
                naturals: 0,
                jokers: j,
            });
        }
    }
    let Some(trick) = trick else {
        return plays;
    };
    let count = trick.cards.len() as u8;
    let to_beat = value(trick.rank, reversed);
    plays.retain(|p| {
        matches!(p, Action::Play { rank, naturals, jokers }
            if naturals + jokers == count && value(*rank, reversed) > to_beat)
    });
    plays.push(Action::Pass);
    plays
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_two_is_highest_and_a_revolution_reverses_the_order() {
        assert!(strength(Rank::Two, false) > strength(Rank::Ace, false));
        assert!(strength(Rank::Ace, false) > strength(Rank::Three, false));
        assert_eq!(strength(Rank::Three, false), 0);
        assert_eq!(strength(Rank::Three, true), 12);
        assert_eq!(strength(Rank::Two, true), 0);
        assert!(value(None, true) > strength(Rank::Three, true));
    }
}
```

- [ ] **Step 4: Implement `mod.rs`**

`games/cards/src/president/mod.rs` :

```rust
//! Président (Trou du cul): be the first to shed your cards, climbing over
//! the previous play with as many cards of a higher rank. Rules and their
//! sources: RULES.md.

mod combos;
pub mod testing;
mod view;

pub use combos::{exchange_value, face_value, hand_order, plays, strength, value, JOKERS_ALONE};
pub use view::View;

use ludo_core::{Game, GameMeta, GameOption, Illegal, OptionKind, Outcome, Pending, Seat, Viewer};
use serde::{Deserialize, Serialize};

use crate::deck::{deal_evenly, pack_52, pack_54, Card, Face, Rank};

pub struct President;

pub const DEFAULT_ROUNDS: u8 = 5;
pub const MAX_ROUNDS: u8 = 10;

/// No jokers, strong jokers above the 2, or wild jokers (spec 4 §3).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Jokers {
    #[default]
    Off,
    High,
    Wild,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Rounds in a game, 1 to `MAX_ROUNDS`.
    pub rounds: u8,
    /// [W] Four of a kind reverses the order.
    pub revolution: bool,
    /// [W] Going out on a 2 (or a joker) makes you the Scum.
    pub no_finish_on_two: bool,
    pub jokers: Jokers,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rounds: DEFAULT_ROUNDS,
            revolution: false,
            no_finish_on_two: false,
            jokers: Jokers::Off,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// `naturals` cards of `rank` and `jokers` jokers; `rank` is `None` for
    /// jokers alone. Suits do not matter: the engine picks them.
    Play {
        rank: Option<Rank>,
        naturals: u8,
        jokers: u8,
    },
    Pass,
    /// President or Vice-President: the cards handed back, in `Face` order.
    Give { faces: Vec<Face> },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Title {
    President,
    VicePresident,
    Neutral,
    Secretary,
    Scum,
}

/// [W] The title of whoever finished at `place` (0: first out) among
/// `players` (4 or more).
pub fn title(place: usize, players: usize) -> Title {
    match place {
        0 => Title::President,
        1 => Title::VicePresident,
        p if p + 1 == players => Title::Scum,
        p if p + 2 == players => Title::Secretary,
        _ => Title::Neutral,
    }
}

/// Ranks from scores (0: first); equal scores share a rank.
pub fn standings(scores: &[u32]) -> Vec<u8> {
    scores
        .iter()
        .map(|&mine| scores.iter().filter(|&&theirs| theirs > mine).count() as u8)
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Exchange,
    Play,
    Over,
}

/// The play to beat.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trick {
    pub seat: Seat,
    pub rank: Option<Rank>,
    pub cards: Vec<Card>,
}

/// Cards handed over at the start of a round.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transfer {
    pub from: Seat,
    pub to: Seat,
    pub cards: Vec<Card>,
}

/// What happened, for the board to tell.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    RoundStarted {
        round: u8,
        dealer: Seat,
        leader: Seat,
        aside: Vec<Card>,
    },
    /// Cards handed over: which ones, only the two players know.
    Exchanged { from: Seat, to: Seat, count: u8 },
    Played {
        seat: Seat,
        rank: Option<Rank>,
        cards: Vec<Card>,
    },
    Passed { seat: Seat },
    TrickClosed { leader: Seat },
    Revolution { reversed: bool },
    /// Out of cards; `fouled`: on a forbidden card.
    Finished { seat: Seat, fouled: bool },
    /// The round's order (first out first), the points it earned and the
    /// scores after it.
    RoundScored {
        order: Vec<Seat>,
        points: Vec<u32>,
        scores: Vec<u32>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub players: u8,
    pub config: Config,
    /// Every deal comes from it: never in a view.
    pub seed: u64,
    pub round_no: u8,
    pub phase: Phase,
    pub hands: Vec<Vec<Card>>,
    pub set_aside: Vec<Card>,
    /// Titles earned in the previous round (empty during the first).
    pub titles: Vec<Title>,
    /// A revolution is in force.
    pub reversed: bool,
    pub trick: Option<Trick>,
    /// Who passed, in a row, since the last play.
    pub passes: Vec<Seat>,
    pub turn: Option<Seat>,
    /// This round: who went out, in order.
    pub out: Vec<Seat>,
    /// This round: who went out on a forbidden card, in order.
    pub fouled: Vec<Seat>,
    /// This round's exchange.
    pub transfers: Vec<Transfer>,
    /// Exchange: who still has to hand cards back.
    pub to_give: Vec<Seat>,
    pub scores: Vec<u32>,
    /// Since the previous decision.
    pub events: Vec<Event>,
}

impl State {
    fn active(&self) -> Vec<Seat> {
        (0..self.players)
            .map(Seat)
            .filter(|s| !self.hands[s.index()].is_empty())
            .collect()
    }

    /// The next seat to the left of `seat` that still holds cards.
    fn next_active(&self, seat: Seat) -> Seat {
        (1..=self.players)
            .map(|k| Seat((seat.0 + k) % self.players))
            .find(|s| !self.hands[s.index()].is_empty())
            .unwrap_or(seat)
    }

    fn seat_with(&self, title: Title) -> Option<Seat> {
        self.titles
            .iter()
            .position(|&t| t == title)
            .map(|i| Seat(i as u8))
    }

    /// A fresh deal, from the round's own stream of the seed so that a log
    /// replays identically.
    fn start_round(&mut self) {
        let pack = match self.config.jokers {
            Jokers::Off => pack_52(),
            Jokers::High | Jokers::Wild => pack_54(),
        };
        let stream = self.seed ^ u64::from(self.round_no + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        let deal = deal_evenly(pack, self.players, stream);
        self.begin_round(deal.hands, deal.set_aside);
    }

    /// The next round, with these hands. [W] Seat 0 deals the first round,
    /// the Scum the others; the player on the dealer's left leads; from the
    /// second round on, the cards are exchanged first.
    fn begin_round(&mut self, hands: Vec<Vec<Card>>, aside: Vec<Card>) {
        self.round_no += 1;
        self.hands = hands;
        self.set_aside = aside;
        self.reversed = false;
        self.trick = None;
        self.passes.clear();
        self.out.clear();
        self.fouled.clear();
        self.transfers.clear();
        self.to_give.clear();
        let dealer = self.seat_with(Title::Scum).unwrap_or(Seat(0));
        let leader = Seat((dealer.0 + 1) % self.players);
        self.turn = Some(leader);
        self.events.push(Event::RoundStarted {
            round: self.round_no,
            dealer,
            leader,
            aside: self.set_aside.clone(),
        });
        self.phase = Phase::Play;
        let (Some(president), Some(vice), Some(secretary), Some(scum)) = (
            self.seat_with(Title::President),
            self.seat_with(Title::VicePresident),
            self.seat_with(Title::Secretary),
            self.seat_with(Title::Scum),
        ) else {
            return;
        };
        // [W] The Scum hands the President their two best cards, the
        // Secretary the Vice-President their best one.
        self.hand_over(scum, president, 2);
        self.hand_over(secretary, vice, 1);
        self.to_give = vec![president, vice];
        self.phase = Phase::Exchange;
    }

    fn hand_over(&mut self, from: Seat, to: Seat, count: usize) {
        let cards = (0..count)
            .filter_map(|_| take_best(&mut self.hands[from.index()]))
            .collect();
        self.receive(from, to, cards);
    }

    fn receive(&mut self, from: Seat, to: Seat, cards: Vec<Card>) {
        self.hands[to.index()].extend(cards.iter().copied());
        self.events.push(Event::Exchanged {
            from,
            to,
            count: cards.len() as u8,
        });
        self.transfers.push(Transfer { from, to, cards });
    }

    /// [W] The President hands back two cards, the Vice-President one.
    fn gift_size(&self, seat: Seat) -> usize {
        match self.titles.get(seat.index()) {
            Some(Title::President) => 2,
            _ => 1,
        }
    }

    /// Every choice of cards to hand back, by face, from the whole hand
    /// (cards just received included).
    fn gifts(&self, seat: Seat) -> Vec<Action> {
        let mut faces: Vec<Face> = self.hands[seat.index()].iter().map(|c| c.face()).collect();
        faces.sort();
        let mut choices: Vec<Vec<Face>> = Vec::new();
        for (i, &a) in faces.iter().enumerate() {
            if self.gift_size(seat) == 1 {
                choices.push(vec![a]);
            } else {
                for &b in &faces[i + 1..] {
                    choices.push(vec![a, b]);
                }
            }
        }
        choices.sort();
        choices.dedup();
        choices
            .into_iter()
            .map(|faces| Action::Give { faces })
            .collect()
    }

    /// [W] The President gives back to the Scum, the Vice-President to the
    /// Secretary; play starts once both have.
    fn give(&mut self, seat: Seat, faces: &[Face]) {
        let back_to = match self.titles[seat.index()] {
            Title::President => Title::Scum,
            _ => Title::Secretary,
        };
        let to = self.seat_with(back_to).expect("every title is held");
        let cards = faces.iter().map(|&f| self.take(seat, f)).collect();
        self.receive(seat, to, cards);
        self.to_give.retain(|&s| s != seat);
        if self.to_give.is_empty() {
            self.phase = Phase::Play;
        }
    }

    /// Removes a card showing `face` from `seat`'s hand, the first suit in
    /// `Suit` order first.
    fn take(&mut self, seat: Seat, face: Face) -> Card {
        let hand = &mut self.hands[seat.index()];
        let at = hand
            .iter()
            .enumerate()
            .filter(|(_, c)| c.face() == face)
            .min_by_key(|(_, c)| **c)
            .map(|(i, _)| i)
            .expect("a legal action names cards in hand");
        hand.remove(at)
    }

    fn play(&mut self, seat: Seat, rank: Option<Rank>, naturals: u8, jokers: u8) {
        let mut cards = Vec::new();
        if let Some(rank) = rank {
            for _ in 0..naturals {
                cards.push(self.take(seat, Face::Natural { rank }));
            }
        }
        for _ in 0..jokers {
            cards.push(self.take(seat, Face::Joker));
        }
        self.events.push(Event::Played {
            seat,
            rank,
            cards: cards.clone(),
        });
        self.trick = Some(Trick { seat, rank, cards });
        self.passes.clear();
        if self.hands[seat.index()].is_empty() {
            self.out.push(seat);
            self.events.push(Event::Finished {
                seat,
                fouled: false,
            });
        }
        self.after_play(seat);
    }

    fn after_play(&mut self, seat: Seat) {
        if self.active().len() <= 1 {
            self.end_round();
        } else {
            self.turn = Some(self.next_active(seat));
        }
    }

    /// [P] Passing keeps you in the trick. [W] Once everyone else still in
    /// the round has passed in a row, the last to play leads; if they are
    /// out, the player on their left.
    fn pass(&mut self, seat: Seat) {
        self.passes.push(seat);
        self.events.push(Event::Passed { seat });
        let owner = self.trick.as_ref().expect("a pass answers a play").seat;
        let owner_in = !self.hands[owner.index()].is_empty();
        if self.passes.len() + usize::from(owner_in) >= self.active().len() {
            let leader = if owner_in {
                owner
            } else {
                self.next_active(owner)
            };
            self.trick = None;
            self.passes.clear();
            self.events.push(Event::TrickClosed { leader });
            self.turn = Some(leader);
        } else {
            self.turn = Some(self.next_active(seat));
        }
    }

    /// [W] Titles and points: n−1 for the first out, one less per place, 0
    /// for the last. The player still holding cards comes after everyone
    /// out; those out on a forbidden card come after them, the first of them
    /// last (❓5).
    fn end_round(&mut self) {
        let order: Vec<Seat> = self
            .out
            .iter()
            .copied()
            .chain(self.active())
            .chain(self.fouled.iter().rev().copied())
            .collect();
        let n = order.len();
        let mut points = vec![0; n];
        let mut titles = vec![Title::Neutral; n];
        for (place, seat) in order.iter().enumerate() {
            points[seat.index()] = (n - 1 - place) as u32;
            titles[seat.index()] = title(place, n);
        }
        for (score, p) in self.scores.iter_mut().zip(&points) {
            *score += p;
        }
        self.events.push(Event::RoundScored {
            order,
            points,
            scores: self.scores.clone(),
        });
        self.titles = titles;
        self.trick = None;
        self.turn = None;
        if self.round_no >= self.config.rounds.clamp(1, MAX_ROUNDS) {
            self.phase = Phase::Over;
        } else {
            self.start_round();
        }
    }

    /// Orders choices weakest first: value, then jokers spent, then cards.
    pub(crate) fn cost(&self, action: &Action) -> (u32, u8, u8) {
        match action {
            Action::Play {
                rank,
                naturals,
                jokers,
            } => (u32::from(value(*rank, self.reversed)), *jokers, *naturals),
            Action::Give { faces } => (faces.iter().map(|&f| u32::from(face_value(f))).sum(), 0, 0),
            Action::Pass => (0, 0, 0),
        }
    }
}

/// [W] The best card of a hand, for the exchange: jokers first, then 2, ace…
fn take_best(hand: &mut Vec<Card>) -> Option<Card> {
    let at = hand
        .iter()
        .enumerate()
        .max_by_key(|(_, c)| (exchange_value(**c), **c))
        .map(|(i, _)| i)?;
    Some(hand.remove(at))
}

impl Game for President {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "cards-president",
            name: "Président",
            min_players: 4,
            max_players: 7,
            hidden_info: true,
            clock: true,
        }
    }

    fn options() -> Vec<GameOption> {
        vec![
            GameOption {
                key: "rounds",
                kind: OptionKind::Number {
                    min: 1,
                    max: u32::from(MAX_ROUNDS),
                    default: u32::from(DEFAULT_ROUNDS),
                },
            },
            GameOption {
                key: "revolution",
                kind: OptionKind::Toggle { default: false },
            },
            GameOption {
                key: "no_finish_on_two",
                kind: OptionKind::Toggle { default: false },
            },
            GameOption {
                key: "jokers",
                kind: OptionKind::Choice {
                    values: &["Off", "High", "Wild"],
                    default: "Off",
                },
            },
        ]
    }

    fn check_config(config: &Config, _players: u8) -> Result<(), Illegal> {
        if (1..=MAX_ROUNDS).contains(&config.rounds) {
            Ok(())
        } else {
            Err(Illegal::new(format!(
                "Le Président se joue en 1 à {MAX_ROUNDS} manches."
            )))
        }
    }

    fn new(config: &Config, players: u8, seed: u64) -> State {
        let n = usize::from(players);
        let mut s = State {
            players,
            config: config.clone(),
            seed,
            round_no: 0,
            phase: Phase::Play,
            hands: vec![Vec::new(); n],
            set_aside: Vec::new(),
            titles: Vec::new(),
            reversed: false,
            trick: None,
            passes: Vec::new(),
            turn: None,
            out: Vec::new(),
            fouled: Vec::new(),
            transfers: Vec::new(),
            to_give: Vec::new(),
            scores: vec![0; n],
            events: Vec::new(),
        };
        s.start_round();
        s
    }

    fn pending(s: &State) -> Pending {
        match s.phase {
            Phase::Exchange => Pending(s.to_give.clone()),
            Phase::Play => s.turn.map_or_else(Pending::none, Pending::one),
            Phase::Over => Pending::none(),
        }
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        match s.phase {
            Phase::Exchange if s.to_give.contains(&seat) => s.gifts(seat),
            Phase::Play if s.turn == Some(seat) => plays(
                &s.hands[seat.index()],
                s.trick.as_ref(),
                s.config.jokers,
                s.reversed,
            ),
            _ => Vec::new(),
        }
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        if s.phase == Phase::Over {
            return Err(Illegal::new("La partie est terminée."));
        }
        if !Self::pending(s).contains(seat) {
            return Err(Illegal::new("Ce n'est pas ton tour."));
        }
        if !Self::legal_actions(s, seat).contains(action) {
            return Err(Illegal::new("Ce coup n'est pas permis."));
        }
        s.events.clear();
        match action {
            Action::Play {
                rank,
                naturals,
                jokers,
            } => s.play(seat, *rank, *naturals, *jokers),
            Action::Pass => s.pass(seat),
            Action::Give { faces } => s.give(seat, faces),
        }
        Ok(())
    }

    fn view(s: &State, viewer: Viewer) -> View {
        View::of(s, viewer)
    }

    fn outcome(s: &State) -> Option<Outcome> {
        (s.phase == Phase::Over).then(|| Outcome::Ranked(standings(&s.scores)))
    }

    /// Pass when allowed; otherwise lead, or hand back, the weakest cards.
    fn timeout_action(s: &State, seat: Seat) -> Option<Action> {
        let legal = Self::legal_actions(s, seat);
        if legal.contains(&Action::Pass) {
            return Some(Action::Pass);
        }
        legal.into_iter().min_by_key(|a| s.cost(a))
    }
}
```

- [ ] **Step 5: Implement `view.rs` and `testing.rs`**

`games/cards/src/president/view.rs` :

```rust
use ludo_core::{Seat, Viewer};
use serde::Serialize;

use super::{hand_order, Event, Phase, State, Title, Transfer, Trick};
use crate::deck::Card;

/// What one seat sees: its own hand and its part of the exchange, every
/// hand size, the table and the scores. Never another hand, the seed, nor
/// an exchange it took no part in.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub round_no: u8,
    pub rounds_total: u8,
    pub phase: Phase,
    pub reversed: bool,
    pub hand_sizes: Vec<usize>,
    /// The viewer's hand, weakest first under the order in force (empty for
    /// a spectator).
    pub hand: Vec<Card>,
    pub trick: Option<Trick>,
    /// Who passed since the last play.
    pub passes: Vec<Seat>,
    /// Play: whose turn it is.
    pub turn: Option<Seat>,
    /// Titles from the previous round (empty during the first).
    pub titles: Vec<Title>,
    pub scores: Vec<u32>,
    /// This round: who went out, and who went out on a forbidden card.
    pub out: Vec<Seat>,
    pub fouled: Vec<Seat>,
    pub set_aside: Vec<Card>,
    /// This round's exchange, as far as the viewer took part in it.
    pub received: Vec<Card>,
    pub given: Vec<Card>,
    /// Exchange: who still has to hand cards back.
    pub to_give: Vec<Seat>,
    pub events: Vec<Event>,
}

impl View {
    pub fn of(s: &State, viewer: Viewer) -> Self {
        let me = match viewer {
            Viewer::Seat(seat) if seat.index() < usize::from(s.players) => Some(seat),
            _ => None,
        };
        let mut hand = me
            .map(|m| s.hands[m.index()].clone())
            .unwrap_or_default();
        hand.sort_by_key(|&c| hand_order(c, s.reversed));
        let mine = |side: fn(&Transfer) -> Seat| -> Vec<Card> {
            me.map(|m| {
                s.transfers
                    .iter()
                    .filter(|t| side(t) == m)
                    .flat_map(|t| t.cards.iter().copied())
                    .collect()
            })
            .unwrap_or_default()
        };
        Self {
            round_no: s.round_no,
            rounds_total: s.config.rounds,
            phase: s.phase,
            reversed: s.reversed,
            hand_sizes: s.hands.iter().map(Vec::len).collect(),
            hand,
            trick: s.trick.clone(),
            passes: s.passes.clone(),
            turn: if s.phase == Phase::Play { s.turn } else { None },
            titles: s.titles.clone(),
            scores: s.scores.clone(),
            out: s.out.clone(),
            fouled: s.fouled.clone(),
            set_aside: s.set_aside.clone(),
            received: mine(|t| t.to),
            given: mine(|t| t.from),
            to_give: s.to_give.clone(),
            events: s.events.clone(),
        }
    }
}
```

`games/cards/src/president/testing.rs` :

```rust
//! Hand-built tables for tests.

use ludo_core::Seat;

use super::{Config, Phase, State, Title};
use crate::deck::{Card, Rank, Suit};

/// "7h", "Ts", "Qd", "2c"…, and "Jo" for a joker.
pub fn c(text: &str) -> Card {
    if text == "Jo" {
        return Card::Joker;
    }
    let mut chars = text.chars();
    let rank = match chars.next() {
        Some('2') => Rank::Two,
        Some('3') => Rank::Three,
        Some('4') => Rank::Four,
        Some('5') => Rank::Five,
        Some('6') => Rank::Six,
        Some('7') => Rank::Seven,
        Some('8') => Rank::Eight,
        Some('9') => Rank::Nine,
        Some('T') => Rank::Ten,
        Some('J') => Rank::Jack,
        Some('Q') => Rank::Queen,
        Some('K') => Rank::King,
        Some('A') => Rank::Ace,
        other => panic!("rang inconnu {other:?} dans {text}"),
    };
    let suit = match chars.next() {
        Some('s') => Suit::Spades,
        Some('h') => Suit::Hearts,
        Some('d') => Suit::Diamonds,
        Some('c') => Suit::Clubs,
        other => panic!("couleur inconnue {other:?} dans {text}"),
    };
    Card::Natural { rank, suit }
}

pub fn cards(text: &str) -> Vec<Card> {
    text.split_whitespace().map(c).collect()
}

/// Round 1, in play: these hands, `turn` to move, no trick on the table.
pub fn table(config: Config, hands: &[&str], turn: u8) -> State {
    let n = hands.len();
    State {
        players: n as u8,
        config,
        seed: 0,
        round_no: 1,
        phase: Phase::Play,
        hands: hands.iter().map(|h| cards(h)).collect(),
        set_aside: Vec::new(),
        titles: Vec::new(),
        reversed: false,
        trick: None,
        passes: Vec::new(),
        turn: Some(Seat(turn)),
        out: Vec::new(),
        fouled: Vec::new(),
        transfers: Vec::new(),
        to_give: Vec::new(),
        scores: vec![0; n],
        events: Vec::new(),
    }
}

/// Round 2, after a round that gave these titles: these hands are dealt
/// and the exchange begins.
pub fn next_round(config: Config, titles: &[Title], hands: &[&str]) -> State {
    let mut s = table(config, &vec![""; titles.len()], 0);
    s.titles = titles.to_vec();
    s.begin_round(hands.iter().map(|h| cards(h)).collect(), Vec::new());
    s
}
```

- [ ] **Step 6: Write `RULES.md`**

`games/cards/RULES.md` :

```markdown
# Jeux de cartes traditionnels — règles implémentées

Sources :

- **[W]** Wikipédia, « Trou du cul (jeu) » (la page « Président (Jeu de
  cartes) » y redirige), consulté le 2026-10-15.
- **[P]** Pagat.com, « President », https://www.pagat.com/climbing/president.html,
  consulté le 2026-10-15.

Un point que les sources ne tranchent pas est marqué ❓ ; ceux du Président
ont été validés par l'utilisateur le 2026-10-15 (spec 4, §3).

## Président (`cards-president`, 4 à 7 joueurs)

### Matériel et ordre

- [W] Paquet de 52 cartes (54 avec les jokers). Les couleurs ne comptent pas.
- [W] Ordre : 3 < 4 < … < 10 < valet < dame < roi < as < 2.

### Donne

- [W] Chacun reçoit le même nombre de cartes ; les cartes en trop sont
  montrées à tous et écartées pour la manche.
- [W] Première manche : le siège 0 donne ; ensuite, le Trou du cul. Le joueur
  à gauche du donneur entame.
- Places fixes : ce sont les parties « statiques » de [W].

### Jeu

- [W] L'entame pose une à quatre cartes de même valeur. Le suivant passe, ou
  pose autant de cartes d'une valeur strictement supérieure.
- [P] Passer n'empêche pas de jouer à son tour suivant dans le même pli.
- [W] Quand tous les autres joueurs encore en jeu ont passé à la suite, le
  dernier à avoir joué entame le pli suivant ; s'il est sorti, la main passe
  au joueur à sa gauche.
- Le moteur choisit les couleurs des cartes posées ou rendues (pique, cœur,
  carreau, trèfle, dans cet ordre) : elles ne changent rien au jeu.

### Titres et points

- [W] Premier sorti Président, deuxième Vice-Président, avant-dernier
  Secrétaire, dernier Trou du cul ; les autres sont Neutres. Le dernier
  joueur qui a encore des cartes est Trou du cul sans jouer.
- [W] Points : n−1 au Président, un de moins par place, 0 au Trou du cul.
- ❓1 La première manche rapporte des points comme les autres ([W] la
  réserve à fixer les titres).
- Fin de partie : après N manches (option, 5 par défaut, de 1 à 10) ;
  classement aux points, les ex-aequo au même rang.

### Échanges (à partir de la 2ᵉ manche)

- [W] Après la donne, le Trou du cul donne ses deux meilleures cartes au
  Président, qui lui rend deux cartes de son choix ; le Secrétaire donne sa
  meilleure carte au Vice-Président, qui lui rend une carte de son choix ;
  les Neutres n'échangent rien.
- Le Président et le Vice-Président choisissent en même temps, dans toute
  leur main. Pour l'échange, l'ordre normal compte et le joker est la
  meilleure carte.

### Variantes (options, désactivées par défaut)

- **Révolution** [W] : quatre cartes de même valeur inversent l'ordre (le 3
  devient la plus forte, le 2 la plus faible) jusqu'à la fin de la manche ;
  les jokers forts restent au-dessus de tout. ❓2 Un nouveau carré inverse de
  nouveau l'ordre.
- **Interdiction de finir sur un 2** [W] : qui pose sa dernière carte avec un
  ou plusieurs 2, ou un ou plusieurs jokers, est Trou du cul.
  ❓3 Pendant une révolution, la carte interdite est le 3 ; les jokers restent
  interdits. ❓4 La carte interdite se juge selon l'ordre en vigueur quand
  elle est posée, avant la révolution que ce coup déclenche éventuellement.
  ❓5 Plusieurs fautifs se classent derrière tous les autres joueurs, le
  premier fautif tout en bas.
- **Jokers forts** [W][P] : au-dessus du 2 ; seuls ou à deux, jamais avec une
  autre valeur.
- **Jokers wild** [P] : le joker prend la valeur des cartes qu'il accompagne
  (quatre cartes au plus). Seuls, ou à deux, les jokers sont la combinaison
  la plus forte de leur taille. ❓6 Un joker vaut une carte naturelle de même
  valeur : à égalité, il faut monter au-dessus. ❓7 Un carré complété par des
  jokers déclenche la révolution.

### Hors périmètre

Putsch, passage de tour, « 2 utilisé comme atout », « … ou rien »,
changement de places, échanges négociés, parties à 3 joueurs ou à plus de 7.
```

- [ ] **Step 7: Run the tests to verify they pass**

Run: `cargo test -p cards --release`
Expected: PASS, dont les 24 tests de `president_rules` et le test de `combos`.

- [ ] **Step 8: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --release
git add games/cards
git commit -m "cards: Président, règles de base (donne, plis, titres, échanges, jokers)

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: Variantes révolution et interdiction de finir sur un 2

**Files:**
- Modify: `games/cards/src/president/mod.rs` (`State::play`, nouvelle méthode `State::top_rank`)
- Test: `games/cards/tests/president_rules.rs` (ajouts)

**Interfaces:**
- Consumes: `State`, `Config`, `Event::Revolution`, `Event::Finished { fouled }`, `State::fouled` (Task 2).
- Produces: `pub(crate) fn top_rank(&self) -> Rank` (le 2, ou le 3 pendant une révolution), utilisé par l'IA en Task 4.

- [ ] **Step 1: Write the failing tests**

Ajouter à la fin de `games/cards/tests/president_rules.rs` :

```rust
/// Plays the clock's choice until the current round is scored.
fn finish_round(s: &mut State) {
    let round = s.round_no;
    while s.round_no == round && s.phase != Phase::Over {
        let seat = President::pending(s).0[0];
        let a = President::timeout_action(s, seat).expect("a pending seat has a move");
        act(s, seat.0, a);
    }
}

fn revolution() -> Config {
    Config { revolution: true, ..one_round() }
}

#[test]
fn four_of_a_kind_starts_a_revolution_only_with_the_option() {
    let hands = ["7s 7h 7d 7c 9s", "3s 4s 2s", "5s", "6s"];
    let mut off = table(one_round(), &hands, 0);
    act(&mut off, 0, play(Rank::Seven, 4));
    assert!(!off.reversed);
    let mut on = table(revolution(), &hands, 0);
    act(&mut on, 0, play(Rank::Seven, 4));
    assert!(on.reversed);
    assert!(on.events.contains(&Event::Revolution { reversed: true }));
}

#[test]
fn a_revolution_reverses_the_ranks() {
    let mut s = table(revolution(), &["9s 9h 9d 9c 8s", "3s 2s Ks", "5s", "6s"], 0);
    act(&mut s, 0, play(Rank::Nine, 4));
    for seat in 1..=3 {
        act(&mut s, seat, Action::Pass);
    }
    act(&mut s, 0, play(Rank::Eight, 1));
    // The 3 is now the best card, the 2 the worst.
    assert_eq!(President::legal_actions(&s, Seat(1)), vec![play(Rank::Three, 1), Action::Pass]);
    assert_eq!(President::view(&s, Viewer::Seat(Seat(1))).hand, cards("2s Ks 3s"));
}

#[test]
fn a_second_four_of_a_kind_turns_the_ranks_back() {
    let mut s = table(revolution(), &["4s 4h 4d 4c 9s", "5s", "6s", "8s"], 0);
    s.reversed = true;
    act(&mut s, 0, play(Rank::Four, 4));
    assert!(!s.reversed);
    assert!(s.events.contains(&Event::Revolution { reversed: false }));
}

#[test]
fn a_four_of_a_kind_with_wild_jokers_is_a_revolution() {
    let cfg = Config { jokers: Jokers::Wild, ..revolution() };
    let mut s = table(cfg, &["9s 9h 9d Jo 4s", "5s", "6s", "8s"], 0);
    act(&mut s, 0, wild(Rank::Nine, 3, 1));
    assert!(s.reversed);
}

#[test]
fn strong_jokers_stay_on_top_during_a_revolution() {
    let cfg = Config { jokers: Jokers::High, ..revolution() };
    let mut s = table(cfg, &["3s 4s", "Jo 5s", "6s", "8s"], 0);
    s.reversed = true;
    act(&mut s, 0, play(Rank::Three, 1));
    assert_eq!(President::legal_actions(&s, Seat(1)), vec![jokers(1), Action::Pass]);
}

#[test]
fn a_new_round_restores_the_normal_order() {
    let cfg = Config { rounds: 2, ..revolution() };
    let mut s = table(cfg, &["4s 4h 4d 4c", "5s 6s", "7s 8s", "9s Ts"], 0);
    act(&mut s, 0, play(Rank::Four, 4));
    assert!(s.reversed);
    finish_round(&mut s);
    assert_eq!(s.round_no, 2);
    assert!(!s.reversed);
}

fn no_two() -> Config {
    Config { no_finish_on_two: true, ..one_round() }
}

#[test]
fn going_out_on_a_two_makes_you_the_scum() {
    let mut s = table(no_two(), &["2s", "3s 4s", "5s 6s", "7s 8s"], 0);
    act(&mut s, 0, play(Rank::Two, 1));
    assert_eq!(s.fouled, vec![Seat(0)]);
    assert!(s.events.contains(&Event::Finished { seat: Seat(0), fouled: true }));
    finish_round(&mut s);
    assert_eq!(s.titles[0], Scum);
    assert_eq!(s.scores[0], 0);
    let mut free = table(one_round(), &["2s", "3s 4s", "5s 6s", "7s 8s"], 0);
    act(&mut free, 0, play(Rank::Two, 1));
    assert_eq!(free.out, vec![Seat(0)]);
}

#[test]
fn during_a_revolution_the_forbidden_card_is_the_three() {
    let cfg = Config { revolution: true, ..no_two() };
    let mut s = table(cfg.clone(), &["3s", "4s 5s", "6s 7s", "8s 9s"], 0);
    s.reversed = true;
    act(&mut s, 0, play(Rank::Three, 1));
    assert_eq!(s.fouled, vec![Seat(0)]);
    let mut s = table(cfg, &["2s", "4s 5s", "6s 7s", "8s 9s"], 0);
    s.reversed = true;
    act(&mut s, 0, play(Rank::Two, 1));
    assert_eq!(s.out, vec![Seat(0)]);
}

#[test]
fn going_out_on_a_joker_is_forbidden_too() {
    let cfg = Config { jokers: Jokers::High, ..no_two() };
    let mut s = table(cfg, &["Jo", "4s 5s", "6s 7s", "8s 9s"], 0);
    act(&mut s, 0, jokers(1));
    assert_eq!(s.fouled, vec![Seat(0)]);
}

#[test]
fn the_forbidden_card_is_judged_before_its_own_revolution() {
    // ❓4: four 3s going out are judged in the normal order: allowed.
    let cfg = Config { revolution: true, ..no_two() };
    let mut s = table(cfg, &["3s 3h 3d 3c", "4s 5s", "6s 7s", "8s 9s"], 0);
    act(&mut s, 0, play(Rank::Three, 4));
    assert_eq!(s.out, vec![Seat(0)]);
    assert!(s.reversed);
}

#[test]
fn the_first_to_foul_finishes_last() {
    let mut s = table(no_two(), &["2s", "2h", "5s 6s", "7s 8s"], 0);
    act(&mut s, 0, play(Rank::Two, 1));
    for seat in 1..=3 {
        act(&mut s, seat, Action::Pass);
    }
    act(&mut s, 1, play(Rank::Two, 1));
    assert_eq!(s.fouled, vec![Seat(0), Seat(1)]);
    finish_round(&mut s);
    assert_eq!((s.titles[0], s.titles[1]), (Scum, Secretary));
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p cards --release --test president_rules`
Expected: FAIL — `four_of_a_kind_starts_a_revolution_only_with_the_option`, `a_revolution_reverses_the_ranks`, `going_out_on_a_two_makes_you_the_scum` et les autres nouveaux tests échouent (`reversed` reste faux, `fouled` reste vide).

- [ ] **Step 3: Implement**

Dans `games/cards/src/president/mod.rs`, remplacer `fn play` par :

```rust
    fn play(&mut self, seat: Seat, rank: Option<Rank>, naturals: u8, jokers: u8) {
        let mut cards = Vec::new();
        if let Some(rank) = rank {
            for _ in 0..naturals {
                cards.push(self.take(seat, Face::Natural { rank }));
            }
        }
        for _ in 0..jokers {
            cards.push(self.take(seat, Face::Joker));
        }
        self.events.push(Event::Played {
            seat,
            rank,
            cards: cards.clone(),
        });
        // [W] Going out on the top card (❓3: the 3 during a revolution) or a
        // joker; ❓4 judged in the order in force when the cards are laid.
        let top = self.top_rank();
        let fouled = self.config.no_finish_on_two
            && cards
                .iter()
                .any(|&c| c == Card::Joker || c.rank() == Some(top));
        // [W] Four cards of a rank (❓7: jokers included) reverse the order
        // until the end of the round; ❓2 another four turn it back.
        if self.config.revolution && cards.len() == 4 {
            self.reversed = !self.reversed;
            self.events.push(Event::Revolution {
                reversed: self.reversed,
            });
        }
        self.trick = Some(Trick { seat, rank, cards });
        self.passes.clear();
        if self.hands[seat.index()].is_empty() {
            if fouled {
                self.fouled.push(seat);
            } else {
                self.out.push(seat);
            }
            self.events.push(Event::Finished { seat, fouled });
        }
        self.after_play(seat);
    }

    /// [W] The strongest natural rank: the 2, or the 3 during a revolution.
    pub(crate) fn top_rank(&self) -> Rank {
        if self.reversed {
            Rank::Three
        } else {
            Rank::Two
        }
    }
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p cards --release`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --release
git add games/cards
git commit -m "cards: Président, révolution et interdiction de finir sur un 2

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: Déterminisation, IA, conformance et registre

**Files:**
- Create: `games/cards/src/president/determinize.rs`, `games/cards/src/president/heuristics.rs`
- Modify: `games/cards/src/president/mod.rs` (`mod determinize; mod heuristics;`)
- Create: `games/cards/tests/conformance.rs`
- Modify: `crates/ludo-registry/Cargo.toml`, `crates/ludo-registry/src/lib.rs`

**Interfaces:**
- Consumes: `State`, `Transfer`, `State::cost`, `State::top_rank`, `standings`, `deck::redeal` (Tasks 1–3).
- Produces: `impl Determinize for President`, `impl Heuristics for President` ; `cards-president` dans `ludo_registry::GAMES`.

- [ ] **Step 1: Write the failing tests**

`games/cards/src/president/mod.rs`, ajouter sous `mod combos;` :

```rust
mod determinize;
mod heuristics;
```

`games/cards/src/president/determinize.rs`, les tests d'abord :

```rust
#[cfg(test)]
mod tests {
    use ludo_core::{Determinize, Game, Seat, Viewer};
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    use crate::president::testing::{c, next_round};
    use crate::president::{Config, President, Title};

    #[test]
    fn other_hands_change_but_not_what_the_observer_sees() {
        let s = President::new(&Config::default(), 4, 3);
        let view = President::view(&s, Viewer::Seat(Seat(1)));
        let mut hands = Vec::new();
        for seed in 0..20 {
            let d = President::determinize(&s, Seat(1), &mut SmallRng::seed_from_u64(seed));
            assert_eq!(President::view(&d, Viewer::Seat(Seat(1))), view);
            assert_eq!(d.hands[1], s.hands[1]);
            assert!(d.hands.iter().all(|h| h.len() == 13));
            hands.push(d.hands[0].clone());
        }
        hands.dedup();
        assert!(hands.len() > 1, "seat 0 always got the same hand");
    }

    #[test]
    fn cards_handed_over_stay_with_who_received_them() {
        use Title::*;
        let s = next_round(
            Config::default(),
            &[Scum, President, Secretary, VicePresident],
            &["4s 9s Ks 2h", "3s 3h 5s 6s", "4h 8s Qs Ah", "3d 5h 7s 9h"],
        );
        for seed in 0..20 {
            let d = President::determinize(&s, Seat(0), &mut SmallRng::seed_from_u64(seed));
            assert!(d.hands[1].contains(&c("2h")) && d.hands[1].contains(&c("Ks")));
            assert!(d.transfers.iter().all(|t| t.from == Seat(0) || t.to == Seat(0)));
            let d = President::determinize(&s, Seat(2), &mut SmallRng::seed_from_u64(seed));
            assert!(d.hands[3].contains(&c("Ah")));
        }
    }
}
```

`games/cards/src/president/heuristics.rs`, les tests d'abord :

```rust
#[cfg(test)]
mod tests {
    use ludo_core::{Game, Heuristics};
    use rand::rngs::SmallRng;
    use rand::seq::SliceRandom;
    use rand::{Rng, SeedableRng};

    use crate::president::{Config, Jokers, President};

    #[test]
    fn the_rollout_action_is_always_legal() {
        let mut rng = SmallRng::seed_from_u64(31);
        let configs = [
            Config::default(),
            Config { rounds: 2, revolution: true, no_finish_on_two: true, jokers: Jokers::Wild },
            Config { rounds: 2, revolution: true, no_finish_on_two: false, jokers: Jokers::High },
        ];
        for config in &configs {
            for players in 4..=7u8 {
                let mut s = President::new(config, players, rng.gen());
                loop {
                    let pending = President::pending(&s);
                    let Some(&seat) = pending.0.choose(&mut rng) else {
                        break;
                    };
                    let action = President::rollout_action(&s, seat, &mut rng);
                    assert!(President::legal_actions(&s, seat).contains(&action), "{action:?}");
                    President::apply(&mut s, seat, &action).unwrap();
                }
            }
        }
    }

    #[test]
    fn a_simulation_stops_when_its_round_is_scored() {
        let root = President::new(&Config::default(), 4, 2);
        assert_eq!(President::evaluate(&root, &root), None);
        let mut later = root.clone();
        later.round_no += 1;
        later.scores = vec![3, 2, 1, 0];
        assert_eq!(President::evaluate(&root, &later), Some(vec![1.0, 2.0 / 3.0, 1.0 / 3.0, 0.0]));
    }
}
```

`games/cards/tests/conformance.rs` :

```rust
use cards::president::{Config, Jokers, President};
use ludo_core::conformance::check;

#[test]
fn president_conforms_at_every_table_size() {
    for players in 4..=7u8 {
        check::<President>(&Config::default(), players, 20, 100 + u64::from(players));
    }
}

#[test]
fn president_conforms_with_every_variant() {
    let all = Config {
        rounds: 3,
        revolution: true,
        no_finish_on_two: true,
        jokers: Jokers::Wild,
    };
    check::<President>(&all, 4, 20, 7);
    check::<President>(&all, 5, 15, 8);
    let strong = Config {
        jokers: Jokers::High,
        revolution: true,
        ..Config::default()
    };
    check::<President>(&strong, 6, 15, 9);
}
```

Registre — `crates/ludo-registry/Cargo.toml`, `[dependencies]` : `cards.workspace = true`. `crates/ludo-registry/src/lib.rs` : après `static PAIRS_BLACKSTONE…`,

```rust
static CARDS_PRESIDENT: GameEntry<cards::President> = GameEntry::new();
```

et `&CARDS_PRESIDENT,` en dernière entrée de `GAMES`.

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p cards --release`
Expected: FAIL à la compilation (`Determinize` / `Heuristics` non implémentés pour `President`).

- [ ] **Step 3: Implement**

En tête de `games/cards/src/president/determinize.rs` :

```rust
use ludo_core::{Determinize, Seat};
use rand::Rng;

use super::{President, State};
use crate::deck::{redeal, Card};

impl Determinize for President {
    /// The other hands are redrawn among themselves, each keeping its size.
    /// Cards the observer handed over stay with whoever received them while
    /// they are still there; the other seats' exchange is forgotten, and
    /// the seed of future deals is redrawn.
    fn determinize<R: Rng + ?Sized>(s: &State, observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        let o = observer.index();
        let mut pinned: Vec<(usize, Card)> = Vec::new();
        for t in s.transfers.iter().filter(|t| t.from == observer) {
            let hand = &mut d.hands[t.to.index()];
            for card in &t.cards {
                if let Some(at) = hand.iter().position(|c| c == card) {
                    pinned.push((t.to.index(), hand.remove(at)));
                }
            }
        }
        let mut others: Vec<&mut Vec<Card>> = d
            .hands
            .iter_mut()
            .enumerate()
            .filter(|(i, _)| *i != o)
            .map(|(_, h)| h)
            .collect();
        redeal(&mut others, rng);
        for (i, card) in pinned {
            d.hands[i].push(card);
        }
        d.transfers.retain(|t| t.from == observer || t.to == observer);
        d.seed = rng.gen();
        d
    }
}
```

En tête de `games/cards/src/president/heuristics.rs` :

```rust
use ludo_core::{Game, Heuristics, Outcome, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use super::{standings, Action, Phase, President, State};

impl State {
    /// A play that spends a joker or the strongest natural rank.
    fn spends_top_card(&self, play: &Action) -> bool {
        matches!(play, Action::Play { rank, jokers, .. }
            if *jokers > 0 || *rank == Some(self.top_rank()))
    }
}

impl Heuristics for President {
    /// From what `seat` sees only. Hand back the weakest cards; lead the
    /// whole set of the weakest rank; on a trick, climb as little as
    /// possible, but pass half the time rather than spend a top card.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        let legal = Self::legal_actions(s, seat);
        let weakest = legal
            .iter()
            .filter(|a| **a != Action::Pass)
            .min_by_key(|a| s.cost(a))
            .cloned();
        let chosen = match s.phase {
            Phase::Exchange => weakest,
            Phase::Play => match weakest.clone() {
                None => Some(Action::Pass),
                Some(Action::Play { rank, .. }) if s.trick.is_none() => legal
                    .iter()
                    .filter(|a| matches!(a, Action::Play { rank: r, jokers: 0, .. } if *r == rank))
                    .max_by_key(|a| s.cost(a))
                    .cloned()
                    .or(weakest),
                Some(play) if s.spends_top_card(&play) && rng.gen_bool(0.5) => Some(Action::Pass),
                Some(play) => Some(play),
            },
            Phase::Over => None,
        };
        chosen
            .or_else(|| legal.choose(rng).cloned())
            .unwrap_or(Action::Pass)
    }

    /// A simulation stops once the round it started in is scored: the next
    /// deal is chance.
    fn evaluate(root: &State, sim: &State) -> Option<Vec<f64>> {
        (sim.round_no > root.round_no).then(|| Outcome::Ranked(standings(&sim.scores)).rewards())
    }
}
```

(Sur un pli, `weakest` est un coup qui bat le pli ; « passer la moitié du temps » ne s'applique qu'à ce cas, puisque l'entame est traitée par le bras précédent.)

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test --workspace --release`
Expected: PASS, dont `president_conforms_*`, `ids_are_unique_and_every_game_starts` (registre) et les tests de `determinize.rs` / `heuristics.rs`.

Puis : `cargo run --release -p ludo-cli -- arena cards-president 4 easy easy` et `cargo run --release -p ludo-cli -- arena cards-president 4 easy easy 5 --config '{"jokers":"Wild","revolution":true}'`
Expected: une ligne `cards-president à … score A = …` chacune, sans erreur.

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
git add games/cards crates/ludo-registry Cargo.lock
git commit -m "cards: Président, déterminisation, IA, conformance et registre

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: Le site : famille « Cartes traditionnelles » et plateau du Président

**Files:**
- Create: `web/src/games/cards/types.ts`, `web/src/games/cards/PlayingCard.tsx`, `web/src/games/cards/cards.css`
- Create: `web/src/games/cards/president/types.ts`, `web/src/games/cards/president/Board.tsx`, `web/src/games/cards/president/rules.ts`
- Create: `web/src/games/cards/__tests__/president.test.tsx`
- Modify: `web/src/games/registry.ts` (famille `"cards"`, entrée `cards-president`)
- Modify: `web/src/games/options.ts` (libellés du Président)
- Modify: `web/src/shell/Home.tsx:20-23` (étagère), `web/src/shell/Emblem.tsx` (dessin de la famille)
- Modify: `web/src/shell/__tests__/home.test.tsx` (étagère)

**Interfaces:**
- Consumes: JSON de la vue et des actions (bloc « JSON (site) » ci-dessus) ; `BoardProps`, `Table`, `CardBack` (`web/src/games/pairs/cards.tsx`).
- Produces: `PlayingCard`, `cardName` (réutilisables par les futurs jeux classiques) ; `PresidentBoard`, `selectionAction`, `describePresidentEvent`, `PRESIDENT_RULES`.

- [ ] **Step 1: Write the failing tests**

`web/src/games/cards/__tests__/president.test.tsx` :

```tsx
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { PresidentBoard, describePresidentEvent, selectionAction } from "../president/Board";
import type { PresidentAction, PresidentView } from "../president/types";
import type { Card, Rank, Suit } from "../types";

const N = (rank: Rank, suit: Suit = "Spades"): Card => ({ kind: "Natural", rank, suit });
const JOKER: Card = { kind: "Joker" };
const NAMES = ["Ana", "Hugo", "Cléo", "Malo"];

const PLAY: PresidentView = {
  roundNo: 2,
  roundsTotal: 5,
  phase: "Play",
  reversed: false,
  handSizes: [3, 5, 4, 0],
  hand: [N("Seven"), N("Seven", "Hearts"), JOKER],
  trick: { seat: 1, rank: "Six", cards: [N("Six"), N("Six", "Hearts")] },
  passes: [2],
  turn: 0,
  titles: ["President", "Scum", "Secretary", "VicePresident"],
  scores: [3, 0, 1, 2],
  out: [3],
  fouled: [],
  setAside: [],
  received: [N("Two", "Clubs"), N("King")],
  given: [N("Three"), N("Four")],
  toGive: [],
  events: [
    { type: "Played", seat: 1, rank: "Six", cards: [N("Six"), N("Six", "Hearts")] },
    { type: "Passed", seat: 2 },
  ],
};

const PLAY_LEGAL: PresidentAction[] = [
  { Play: { rank: "Seven", naturals: 2, jokers: 0 } },
  { Play: { rank: "Seven", naturals: 1, jokers: 1 } },
  "Pass",
];

const EXCHANGE: PresidentView = { ...PLAY, phase: "Exchange", trick: null, passes: [], turn: null, out: [], toGive: [0, 3] };
const GIVE: PresidentAction = {
  Give: { faces: [{ kind: "Natural", rank: "Three" }, { kind: "Natural", rank: "Five" }] },
};

function board(view: PresidentView, legal: PresidentAction[], me: number | null): string {
  return renderToStaticMarkup(
    <PresidentBoard view={view} legalActions={legal} pending={[]} me={me} seatNames={NAMES} submit={() => {}} />,
  );
}

describe("selectionAction", () => {
  it("finds the play made of exactly the selected cards", () => {
    expect(selectionAction([N("Seven"), N("Seven", "Hearts")], PLAY_LEGAL)).toEqual(PLAY_LEGAL[0]);
    expect(selectionAction([N("Seven"), JOKER], PLAY_LEGAL)).toEqual(PLAY_LEGAL[1]);
    expect(selectionAction([N("Seven")], PLAY_LEGAL)).toBeNull();
    expect(selectionAction([], PLAY_LEGAL)).toBeNull();
  });

  it("finds nothing for mixed ranks or a card too many", () => {
    expect(selectionAction([N("Seven"), N("Eight")], PLAY_LEGAL)).toBeNull();
    expect(selectionAction([N("Seven"), N("Seven", "Hearts"), JOKER], PLAY_LEGAL)).toBeNull();
  });

  it("matches jokers alone, and handed-back cards in any order", () => {
    const pair: PresidentAction = { Play: { rank: null, naturals: 0, jokers: 2 } };
    expect(selectionAction([JOKER, JOKER], [pair])).toEqual(pair);
    expect(selectionAction([N("Five", "Hearts"), N("Three", "Clubs")], [GIVE])).toEqual(GIVE);
    expect(selectionAction([N("Five")], [GIVE])).toBeNull();
  });
});

describe("PresidentBoard", () => {
  it("shows the trick, my hand, the titles, and Jouer off until a play is selected", () => {
    const html = board(PLAY, PLAY_LEGAL, 0);
    expect(html).toContain("Trou du cul");
    expect(html).toContain("Président");
    expect(html).toContain('disabled="">Jouer</button>');
    expect(html).toContain("Passer");
    expect(html).toContain("reçu 2 de trèfle, roi de pique");
    expect(html).toContain('aria-label="7 de cœur"');
    expect(html).not.toContain("Révolution");
  });

  it("shows the revolution", () => {
    expect(board({ ...PLAY, reversed: true }, PLAY_LEGAL, 0)).toContain("Révolution");
  });

  it("asks the President to hand cards back", () => {
    const html = board(EXCHANGE, [GIVE], 0);
    expect(html).toContain("Rendre");
    expect(html).toContain("Rends 2 cartes");
    expect(html).not.toContain("Passer");
  });

  it("shows no hand to a spectator", () => {
    const html = board({ ...PLAY, hand: [] }, [], null);
    expect(html).not.toContain("aria-pressed");
    expect(html).not.toContain(">Jouer<");
  });
});

describe("describePresidentEvent", () => {
  it("tells what happened", () => {
    expect(describePresidentEvent({ type: "Passed", seat: 2 }, NAMES)).toBe("Cléo passe.");
    expect(describePresidentEvent({ type: "Exchanged", from: 1, to: 0, count: 2 }, NAMES)).toBe(
      "Hugo donne 2 cartes à Ana.",
    );
    expect(describePresidentEvent({ type: "Revolution", reversed: true }, NAMES)).toContain("Révolution");
    expect(describePresidentEvent({ type: "Finished", seat: 3, fouled: true }, NAMES)).toContain("interdite");
    expect(describePresidentEvent({ type: "Played", seat: 0, rank: null, cards: [JOKER] }, NAMES)).toBe(
      "Ana joue joker.",
    );
  });
});
```

`web/src/shell/__tests__/home.test.tsx` : ajouter `meta("cards-president", "Président"),` à `CATALOG`, et dans le `describe` :

```tsx
  it("shelves the classic card games after the Pairs deck", () => {
    const html = home();
    expect(html).toContain("Cartes traditionnelles");
    expect(html.indexOf("Deck Pairs")).toBeLessThan(html.indexOf("Cartes traditionnelles"));
    expect(html.indexOf("Cartes traditionnelles")).toBeLessThan(html.indexOf("Président"));
  });
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd web && pnpm test`
Expected: FAIL — `president.test.tsx` ne trouve pas `../president/Board` ; le test d'accueil ne trouve pas « Cartes traditionnelles ».

- [ ] **Step 3: Implement the shared card**

`web/src/games/cards/types.ts` :

```ts
// Mirrors games/cards/src/deck.rs (Card, Face).

export type Rank =
  | "Two"
  | "Three"
  | "Four"
  | "Five"
  | "Six"
  | "Seven"
  | "Eight"
  | "Nine"
  | "Ten"
  | "Jack"
  | "Queen"
  | "King"
  | "Ace";

export type Suit = "Spades" | "Hearts" | "Diamonds" | "Clubs";

export type Card = { kind: "Natural"; rank: Rank; suit: Suit } | { kind: "Joker" };

/** What a card shows when suits do not matter. */
export type Face = { kind: "Natural"; rank: Rank } | { kind: "Joker" };
```

`web/src/games/cards/PlayingCard.tsx` :

```tsx
import type { Card, Rank, Suit } from "./types";
import "./cards.css";

/** French indices: V(alet), D(ame), R(oi). */
const INDEX: Record<Rank, string> = {
  Two: "2",
  Three: "3",
  Four: "4",
  Five: "5",
  Six: "6",
  Seven: "7",
  Eight: "8",
  Nine: "9",
  Ten: "10",
  Jack: "V",
  Queen: "D",
  King: "R",
  Ace: "A",
};

const RANK_NAME: Record<Rank, string> = {
  ...INDEX,
  Jack: "valet",
  Queen: "dame",
  King: "roi",
  Ace: "as",
};

const SYMBOL: Record<Suit, string> = { Spades: "♠", Hearts: "♥", Diamonds: "♦", Clubs: "♣" };
const SUIT_NAME: Record<Suit, string> = { Spades: "pique", Hearts: "cœur", Diamonds: "carreau", Clubs: "trèfle" };

/** "7 de cœur", "dame de pique", "joker". */
export function cardName(card: Card): string {
  return card.kind === "Joker" ? "joker" : `${RANK_NAME[card.rank]} de ${SUIT_NAME[card.suit]}`;
}

/** A playing card: a toggle button when it can be picked, plain otherwise. */
export function PlayingCard({
  card,
  small,
  selected,
  onClick,
}: {
  card: Card;
  small?: boolean;
  selected?: boolean;
  onClick?: () => void;
}) {
  const red = card.kind === "Natural" && (card.suit === "Hearts" || card.suit === "Diamonds");
  const cls = `playing-card${red ? " red" : ""}${card.kind === "Joker" ? " joker" : ""}${small ? " small" : ""}${
    selected ? " selected" : ""
  }`;
  const face =
    card.kind === "Joker" ? (
      <span>Joker</span>
    ) : (
      <>
        <span>{INDEX[card.rank]}</span>
        <span className="suit">{SYMBOL[card.suit]}</span>
      </>
    );
  if (!onClick) {
    return (
      <span className={cls} title={cardName(card)}>
        {face}
      </span>
    );
  }
  return (
    <button
      type="button"
      className={`${cls} playable`}
      aria-pressed={selected ?? false}
      aria-label={cardName(card)}
      onClick={onClick}
    >
      {face}
    </button>
  );
}
```

`web/src/games/cards/cards.css` :

```css
.playing-card { display: inline-flex; flex-direction: column; align-items: center; justify-content: center; gap: 1px; width: 40px; height: 58px; padding: 0; border: 1px solid var(--face-line); border-radius: 6px; background: var(--face); color: #1f2421; font: 800 1rem/1 var(--f-display); box-shadow: 0 1px 2px rgb(0 0 0 / 25%); }
.playing-card .suit { font-size: 1.15rem; }
.playing-card.red { color: #b3261e; }
.playing-card.joker { color: #6b3fa0; font-size: 0.7rem; }
.playing-card.small { width: 28px; height: 40px; font-size: 0.8rem; }
.playing-card.small .suit { font-size: 0.85rem; }
.playing-card.playable { cursor: pointer; transition: transform 0.12s; }
.playing-card.playable:hover { transform: translateY(-4px); }
.playing-card.selected, .playing-card.selected:hover { transform: translateY(-10px); outline: 2px solid var(--brass); outline-offset: 1px; }
.president-hand { display: flex; flex-wrap: wrap; gap: 4px; justify-content: center; padding-top: 10px; }
.president-trick { display: flex; gap: 4px; justify-content: center; align-items: center; min-height: 60px; }
.president-banner { margin: 0; font-weight: 700; color: var(--brass); }
```

- [ ] **Step 4: Implement the President board**

`web/src/games/cards/president/types.ts` :

```ts
// Mirrors games/cards/src/president (View, Action, Event).
import type { Card, Face, Rank } from "../types";

export type Title = "President" | "VicePresident" | "Neutral" | "Secretary" | "Scum";

export interface Trick {
  seat: number;
  /** null: jokers alone. */
  rank: Rank | null;
  cards: Card[];
}

export type PresidentEvent =
  | { type: "RoundStarted"; round: number; dealer: number; leader: number; aside: Card[] }
  | { type: "Exchanged"; from: number; to: number; count: number }
  | { type: "Played"; seat: number; rank: Rank | null; cards: Card[] }
  | { type: "Passed"; seat: number }
  | { type: "TrickClosed"; leader: number }
  | { type: "Revolution"; reversed: boolean }
  | { type: "Finished"; seat: number; fouled: boolean }
  | { type: "RoundScored"; order: number[]; points: number[]; scores: number[] };

export interface PresidentView {
  roundNo: number;
  roundsTotal: number;
  phase: "Exchange" | "Play" | "Over";
  reversed: boolean;
  handSizes: number[];
  /** My hand, weakest first. */
  hand: Card[];
  trick: Trick | null;
  passes: number[];
  turn: number | null;
  /** From the previous round (empty during the first). */
  titles: Title[];
  scores: number[];
  out: number[];
  fouled: number[];
  setAside: Card[];
  /** This round's exchange, as far as I took part in it. */
  received: Card[];
  given: Card[];
  toGive: number[];
  events: PresidentEvent[];
}

export type PresidentAction =
  | { Play: { rank: Rank | null; naturals: number; jokers: number } }
  | "Pass"
  | { Give: { faces: Face[] } };
```

`web/src/games/cards/president/rules.ts` :

```ts
// Short reminder shown at the table. The full rules and their sources:
// games/cards/RULES.md (keep both in step).

export const PRESIDENT_RULES = [
  "Ordre : 3, 4, … 10, valet, dame, roi, as, 2 (la plus forte). Les couleurs ne comptent pas.",
  "L'entame pose une à quatre cartes de même valeur. Ensuite, pose autant de cartes d'une valeur plus forte, ou passe : passer ne t'empêche pas de rejouer dans le même pli.",
  "Quand tous les autres ont passé, le dernier à avoir joué ouvre un nouveau pli.",
  "Premier sans cartes : Président, puis Vice-président… Secrétaire, et le dernier Trou du cul. Points : n−1 au Président, 0 au Trou du cul.",
  "Dès la 2ᵉ manche, le Trou du cul donne ses 2 meilleures cartes au Président, qui lui en rend 2 ; le Secrétaire donne sa meilleure au Vice-président, qui lui en rend 1.",
  "Options : révolution (un carré inverse l'ordre jusqu'à la fin de la manche), pas de fin sur un 2 (ni sur un 3 pendant une révolution, ni sur un joker), jokers forts (au-dessus du 2) ou wild (ils remplacent une valeur).",
];
```

`web/src/games/cards/president/Board.tsx` :

```tsx
import { useEffect, useState } from "react";
import type { BoardProps } from "../../registry";
import { CardBack, Table } from "../../pairs/cards";
import { PlayingCard, cardName } from "../PlayingCard";
import type { Card, Face } from "../types";
import type { PresidentAction, PresidentEvent, PresidentView, Title } from "./types";

export const TITLE_LABEL: Record<Title, string> = {
  President: "Président",
  VicePresident: "Vice-président",
  Neutral: "Neutre",
  Secretary: "Secrétaire",
  Scum: "Trou du cul",
};

function faceKey(face: Face): string {
  return face.kind === "Joker" ? "Joker" : face.rank;
}

function cardFaceKey(card: Card): string {
  return card.kind === "Joker" ? "Joker" : card.rank;
}

function sameFaces(faces: Face[], selection: Card[]): boolean {
  const a = faces.map(faceKey).sort().join(",");
  const b = selection.map(cardFaceKey).sort().join(",");
  return a === b;
}

/** The legal action the selected cards make: a play of exactly those cards, or those cards handed back. */
export function selectionAction(selection: Card[], legal: PresidentAction[]): PresidentAction | null {
  if (selection.length === 0) return null;
  const naturals = selection.filter((c) => c.kind === "Natural");
  const jokers = selection.length - naturals.length;
  return (
    legal.find((a) => {
      if (a === "Pass") return false;
      if ("Give" in a) return sameFaces(a.Give.faces, selection);
      const play = a.Play;
      return (
        play.naturals === naturals.length &&
        play.jokers === jokers &&
        naturals.every((c) => c.kind === "Natural" && c.rank === play.rank)
      );
    }) ?? null
  );
}

function plural(count: number, word: string): string {
  return `${count} ${word}${count > 1 ? "s" : ""}`;
}

export function describePresidentEvent(e: PresidentEvent, names: string[]): string {
  switch (e.type) {
    case "RoundStarted": {
      const aside = e.aside.length > 0 ? ` Écartées : ${e.aside.map(cardName).join(", ")}.` : "";
      return `Manche ${e.round} : ${names[e.dealer]} distribue, ${names[e.leader]} commence.${aside}`;
    }
    case "Exchanged":
      return `${names[e.from]} donne ${plural(e.count, "carte")} à ${names[e.to]}.`;
    case "Played":
      return `${names[e.seat]} joue ${e.cards.map(cardName).join(", ")}.`;
    case "Passed":
      return `${names[e.seat]} passe.`;
    case "TrickClosed":
      return `Tout le monde passe : ${names[e.leader]} ouvre un nouveau pli.`;
    case "Revolution":
      return e.reversed ? "Révolution : l'ordre des cartes est inversé." : "Contre-révolution : l'ordre normal revient.";
    case "Finished":
      return e.fouled
        ? `${names[e.seat]} finit sur une carte interdite : Trou du cul.`
        : `${names[e.seat]} n'a plus de cartes.`;
    case "RoundScored":
      return `Fin de manche : ${e.order.map((seat) => `${names[seat]} +${e.points[seat]}`).join(", ")}.`;
  }
}

export function PresidentBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as PresidentView;
  const legal = legalActions as PresidentAction[];
  const [picked, setPicked] = useState<number[]>([]);
  // Every update starts a new decision: nothing selected.
  useEffect(() => setPicked([]), [view]);

  const selection = picked.flatMap((i) => (v.hand[i] ? [v.hand[i]] : []));
  const action = selectionAction(selection, legal);
  const gift = legal.find((a): a is { Give: { faces: Face[] } } => a !== "Pass" && "Give" in a);
  const playing = legal.some((a) => a === "Pass" || "Play" in a);
  const toggle = (i: number) => setPicked((p) => (p.includes(i) ? p.filter((x) => x !== i) : [...p, i]));
  const send = (a: PresidentAction) => {
    setPicked([]);
    submit(a);
  };

  const seats = v.handSizes.map((size, i) => {
    const fouled = v.fouled.includes(i);
    const out = fouled || v.out.includes(i);
    return (
      <section key={i} className={`pairs-seat${v.turn === i || v.toGive.includes(i) ? " active" : ""}`}>
        <h3>
          <span>{seatNames[i]}</span>
          <span>{plural(v.scores[i], "point")}</span>
        </h3>
        {v.titles.length > 0 && <p className="hint">{TITLE_LABEL[v.titles[i]]}</p>}
        {out ? (
          <p className="hint">{fouled ? "Sorti sur une carte interdite" : "Sorti"}</p>
        ) : (
          <CardBack count={size} label="en main" />
        )}
        {v.passes.includes(i) && <p className="hint">Passe</p>}
      </section>
    );
  });

  const center = (
    <>
      <p className="pairs-round">
        Manche {v.roundNo} / {v.roundsTotal}
      </p>
      {v.reversed && <p className="president-banner">Révolution : le 3 est la plus forte, le 2 la plus faible.</p>}
      <div className="president-trick">
        {v.trick ? (
          <>
            {v.trick.cards.map((card, k) => (
              <PlayingCard key={k} card={card} />
            ))}
            <span className="hint">{seatNames[v.trick.seat]}</span>
          </>
        ) : (
          <span className="hint">Table vide</span>
        )}
      </div>
      {me !== null && v.hand.length > 0 && (
        <div className="president-hand">
          {v.hand.map((card, i) => (
            <PlayingCard
              key={i}
              card={card}
              selected={picked.includes(i)}
              onClick={gift || playing ? () => toggle(i) : undefined}
            />
          ))}
        </div>
      )}
      {(gift || playing) && (
        <div className="row">
          <button className="primary" disabled={!action} onClick={() => action && send(action)}>
            {gift ? "Rendre" : "Jouer"}
          </button>
          {legal.includes("Pass") && <button onClick={() => send("Pass")}>Passer</button>}
        </div>
      )}
      {/* Template strings: one text node each, so the hint reads as one string. */}
      {gift && <p className="hint">{`Rends ${plural(gift.Give.faces.length, "carte")} de ton choix.`}</p>}
      {playing && !v.trick && <p className="hint">À toi d'ouvrir : une à quatre cartes de même valeur.</p>}
      {playing && v.trick && (
        <p className="hint">{`Pose ${plural(v.trick.cards.length, "carte")} de valeur plus forte, ou passe.`}</p>
      )}
      {v.phase === "Exchange" && !gift && <p className="hint">Échange des cartes en cours…</p>}
      {me !== null && (v.received.length > 0 || v.given.length > 0) && (
        <p className="hint">
          {`Échange :${v.received.length > 0 ? ` reçu ${v.received.map(cardName).join(", ")}` : ""}${
            v.received.length > 0 && v.given.length > 0 ? " ;" : ""
          }${v.given.length > 0 ? ` donné ${v.given.map(cardName).join(", ")}` : ""}.`}
        </p>
      )}
      {v.setAside.length > 0 && (
        <div className="pairs-seen">
          <span className="hint">Écartées :</span>
          {v.setAside.map((card, k) => (
            <PlayingCard key={k} card={card} small />
          ))}
        </div>
      )}
      <ul className="pairs-events">
        {v.events.map((e, i) => (
          <li key={i}>{describePresidentEvent(e, seatNames)}</li>
        ))}
      </ul>
    </>
  );

  return <Table seats={seats} first={me ?? 0} center={center} />;
}
```

- [ ] **Step 5: Register the family and the game**

`web/src/games/registry.ts` :
- importer `import { PresidentBoard } from "./cards/president/Board";` et `import { PRESIDENT_RULES } from "./cards/president/rules";` ;
- `export type GameFamily = "board" | "tiles" | "pairs" | "cards";` ;
- dernière entrée de la liste : `{ id: "cards-president", family: "cards", Board: PresidentBoard, rules: PRESIDENT_RULES },`.

`web/src/games/options.ts`, dans `LABELS` :

```ts
  "cards-president.rounds": "Manches",
  "cards-president.revolution": "Révolution",
  "cards-president.no_finish_on_two": "Pas de fin sur un 2",
  "cards-president.jokers": "Jokers",
  "cards-president.jokers.Off": "Sans",
  "cards-president.jokers.High": "Forts",
  "cards-president.jokers.Wild": "Wild (remplacent une valeur)",
```

`web/src/shell/Home.tsx`, `SHELVES` :

```tsx
const SHELVES: { title: string; families: GameFamily[] }[] = [
  { title: "Plateau et tuiles", families: ["board", "tiles"] },
  { title: "Deck Pairs", families: ["pairs"] },
  { title: "Cartes traditionnelles", families: ["cards"] },
];
```

`web/src/shell/Emblem.tsx` : déplacer `const initial = name.replace(/^L['’]/, "").charAt(0).toUpperCase();` en tête de `Emblem`, et ajouter avant le dessin Pairs :

```tsx
  if (family === "cards") {
    return (
      <svg viewBox="0 0 44 44" aria-hidden="true">
        <rect width="44" height="44" rx="10" style={{ fill: "var(--accent-soft)" }} />
        <rect x="12" y="7" width="20" height="30" rx="3" style={{ fill: "var(--face)", stroke: "var(--accent)" }} strokeWidth="1.5" />
        <text x="22" y="22" fontWeight="800" fontSize="11" fill="#b3261e" style={{ fontFamily: "var(--f-display)" }} textAnchor="middle">
          {initial}
        </text>
        <text x="22" y="33" fontSize="9" fill="#b3261e" textAnchor="middle">
          ♥
        </text>
      </svg>
    );
  }
```

et mettre à jour son commentaire : « one drawing per family, the game's initial on cards ».

- [ ] **Step 6: Run the tests to verify they pass**

Run: `./scripts/build-wasm.sh dev && cd web && pnpm test && pnpm type-check`
Expected: PASS (les nouveaux tests du Président et de l'accueil compris), type-check propre.

- [ ] **Step 7: Commit**

```bash
git add web/src
git commit -m "web: famille Cartes traditionnelles et plateau du Président

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 6: Mesure de l'IA, documentation et vérification dans le navigateur

**Files:**
- Modify: `CLAUDE.md` (« Commandes » : ligne d'arène ; nouvelle section « Ajouter un jeu de cartes classique »)
- Modify: `README.md` (liste des jeux)
- Modify: `docs/superpowers/specs/2026-10-15-president-design.md` (§9 « Ajustements faits au plan 2 », statut)

**Interfaces:**
- Consumes: tout ce qui précède.

- [ ] **Step 1: Mesurer l'IA**

Run: `cargo run --release -p ludo-cli -- arena cards-president 200 hard easy 4`, puis `cargo run --release -p ludo-cli -- arena cards-president 200 hard easy 4 --config '{"revolution":true,"jokers":"Wild"}'`
Expected: score A ≥ 0.55 dans les deux cas (spec §7 : « le niveau difficile bat nettement le facile »). Noter les deux valeurs. En dessous de 0.55, s'arrêter et consigner une décision avant d'aller plus loin (piste : `search_actions` qui limite les coups des grosses mains, spec §4).

- [ ] **Step 2: Documenter**

`CLAUDE.md`, section « Commandes », après la ligne Blackstone (valeurs mesurées à l'étape 1) :

```markdown
- `cargo run --release -p ludo-cli -- arena cards-president 200 hard easy 4` : Difficile doit marquer ≥ 0.55 contre Facile (<valeur> mesuré ; <valeur> avec `--config '{"revolution":true,"jokers":"Wild"}'`).
```

`CLAUDE.md`, après la section « Ajouter un jeu Pairs » :

```markdown
## Ajouter un jeu de cartes classique
Comme « Ajouter un jeu Pairs », dans `games/cards` :
1. `games/cards/src/<jeu>/` : `mod.rs`, `view.rs`, `determinize.rs`, `heuristics.rs`, `testing.rs` ; `pub mod <jeu>;` et la réexportation dans `games/cards/src/lib.rs`. Le paquet vient de `deck.rs` (`Card`, `Face`, `pack_52`/`pack_32`/`pack_54`, `deal_evenly`, `redeal`) : l'étendre là plutôt que de le recopier. L'ordre des valeurs appartient au jeu.
2. Une section dans `games/cards/RULES.md` ; `tests/<jeu>_rules.rs` ; un cas dans `tests/conformance.rs` par nombre de joueurs et par variante.
3. Les options passent par `Game::options` et `check_config`, avec leurs libellés dans `web/src/games/options.ts`.
4. `crates/ludo-registry/src/lib.rs` : un `static` et une entrée dans `GAMES`.
5. `web/src/games/cards/<jeu>/` : `types.ts`, `Board.tsx` (cartes : `../PlayingCard`), `rules.ts`, un test dans `__tests__/`, une entrée `family: "cards"` dans `web/src/games/registry.ts`.
6. `ludo-cli arena cards-<jeu> 200 hard easy`, et la ligne correspondante dans « Commandes ».
```

`README.md`, à la fin de la liste « Jeux : » (remplacer le point final de la dernière ligne par « ; ») :

```markdown
- **Président** (4–7 joueurs, cartes traditionnelles, règles d'après Wikipédia et Pagat ; révolution, fin sur un 2 et jokers en options).
```

Spec : passer « Statut » à « plan 2 livré » et ajouter une section :

```markdown
## 9. Ajustements faits au plan 2

- ❓4 (nouveau, à valider) : la carte interdite se juge selon l'ordre en vigueur quand elle est posée, avant la révolution que ce coup déclenche.
- Le plateau sélectionne des cartes et n'active « Jouer » que si la sélection forme un coup permis ; comme une action désigne une valeur, des naturels et des jokers, une sélection correspond toujours à un seul coup : la liste de choix prévue au §6 pour les jokers wild n'est pas nécessaire.
- L'IA arrête ses simulations à la fin de la manche en cours (`evaluate`) : la donne suivante relève du hasard.
- Mesures de l'arène : voir `CLAUDE.md`.
```

(et y ajouter toute décision consignée pendant l'exécution).

- [ ] **Step 3: Vérifier dans le navigateur**

Run: `./scripts/build-wasm.sh && cd web && pnpm build && pnpm preview --port 4318 --strictPort` (en arrière-plan), puis un script `playwright-core` (Chromium de `~/.cache/ms-playwright`) qui :
1. ouvre l'accueil et vérifie l'étagère « Cartes traditionnelles » avec « Président » ;
2. ouvre le Président : le lobby montre « Options » avec « Manches » à 5, « Révolution », « Pas de fin sur un 2 » et « Jokers » à « Sans » ;
3. choisit 1 manche, coche « Révolution », lance « Jouer sur cet appareil » ;
4. joue la partie jusqu'au bout : à chaque décision, clique la première carte jouable de la main, puis « Jouer » s'il est actif, sinon « Passer » (et, à l'échange, deux cartes puis « Rendre ») ;
5. vérifie que l'écran de fin « Partie terminée » apparaît et que la console n'a pas d'erreur (hors `favicon.ico`).
Expected: comme décrit.

- [ ] **Step 4: Commit**

```bash
git add CLAUDE.md README.md docs/superpowers/specs/2026-10-15-president-design.md
git commit -m "docs: Président (arène, procédure jeu de cartes, README, ajustements de la spec)

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

## Après le plan

Relecture finale de la branche, fusion sur `main`, puis le cycle i18n (spec à écrire) ou un autre jeu de cartes classique.

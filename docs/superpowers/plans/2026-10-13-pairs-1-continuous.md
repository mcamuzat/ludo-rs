# Pairs, plan 1 : deck commun et Continuous Pairs — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Continuous Pairs (2–8 joueurs) se joue sur ludo-rs contre l'IA et en ligne, sur un matériel de cartes que les prochains jeux Pairs réutiliseront.

**Architecture:** Une crate `games/pairs` : `deck.rs` (rang, deck de 55 cartes, pioche/défausse avec remélange) et un module par jeu ; `continuous/` implémente `Game`, `Determinize`, `Heuristics` sous l'id `pairs-continuous`. Côté web, `web/src/games/pairs/` porte les composants de cartes partagés et un plateau par jeu. La plateforme (`ludo-core`, lobby, P2P, ISMCTS) n'est pas modifiée.

**Tech Stack:** Rust stable (rand 0.8 `SmallRng`, serde), React 19 + TypeScript + Vite 7, vitest 3.

**Spec:** `docs/superpowers/specs/2026-10-13-pairs-design.md` (§1–4, §6–8). Le plan 2 couvrira §5 (L'Auberge des Pirates).

## Global Constraints

- Branche `pairs`. Avant chaque commit : `cargo fmt --all` et `cargo clippy --workspace --all-targets -- -D warnings`.
- Tests Rust : `cargo test --workspace --release`.
- `games/pairs` reste natif : jamais de `wasm-bindgen`, `js-sys`, `web-sys`.
- `apply` rend `Illegal` **sans modifier l'état** et ne panique jamais sur une entrée.
- Une vue ne contient jamais la graine, l'ordre de la pioche ni les cartes brûlées.
- Le web affiche et envoie des actions : toute valeur dérivée (risque de paire, plus petite carte, cartes non situées) vient de la `View`.
- Messages `Illegal` et textes d'interface en français ; commentaires de code en anglais, comme le reste du dépôt.
- Les livrets (PDF) ne sont pas versionnés ; `RULES.md` reformule.
- `web/vite.config.ts` n'est pas modifié (il autorise déjà `.ngrok-free.dev` et `.ngrok-free.app`).
- Id du jeu : `pairs-continuous`. Nom affiché : `Continuous Pairs`. `hidden_info: true`, `clock: false`.
- Cibles : 2 joueurs 31, 3 → 21, 4 → 16, 5 → 13, 6 à 8 → 11. Cartes brûlées à chaque mélange : 5.

### Écarts assumés par rapport à la spec (consignés dans la spec à la tâche 5)

- `Supply::draw()` ne prend pas de `rng` : la graine vit dans le `Supply` (un remélange doit être rejouable depuis le `MatchLog`).
- Pas de `Heuristics::evaluate` : une simulation jusqu'au bout est courte, et `evaluate` est interrogée à chaque pas simulé.
- `pair_in`, la défausse face cachée de mains et la redistribution des mains (`redeal_unseen` complet) arrivent au plan 2, avec le premier jeu qui en a besoin. Ici : `Supply::redeal_hidden`.

## Review Focus

1. **Un abandon qui prend la dernière carte d'un autre joueur** laisse deux piles vides : chacun doit recevoir sa carte automatique dans l'ordre du tour, sans double tour ni blocage. → test `folding_on_an_opponents_last_card_refills_both` (tâche 2).
2. **Remélange d'une défausse de 5 cartes ou moins** : brûler 5 cartes ne doit pas vider la pioche qu'on vient de refaire. → test `a_small_discard_still_yields_a_card` (tâche 1).
3. **Config venue du réseau avec `target: 0`** : la partie ne doit pas naître terminée. → test `a_zero_target_falls_back_to_the_booklet` (tâche 2).
4. **Action désérialisée absurde** (siège 7 à une table de 2, rang 11) : `Illegal` ou erreur serde, jamais de panique d'index. → tests `folding_from_a_seat_outside_the_table_is_refused` (tâche 2) et `ranks_outside_1_to_10_do_not_deserialize` (tâche 1).
5. **Vue de fin de partie dans le plateau** (`turn: null`, aucune action légale) : le plateau s'affiche, nomme le perdant et ne propose rien. → test `shows the loser and offers nothing once the game is over` (tâche 5).

---

### Task 1: Crate `games/pairs` et matériel commun (`deck.rs`)

**Files:**
- Create: `games/pairs/Cargo.toml`, `games/pairs/src/lib.rs`, `games/pairs/src/deck.rs`
- Modify: `Cargo.toml` (racine)

**Interfaces:**
- Produces (dans `pairs::deck`) :
  - `pub const DECK_SIZE: usize = 55;`
  - `pub struct Rank` — `Rank::new(u8) -> Option<Rank>`, `value(self) -> u8`, `index(self) -> usize` (= valeur − 1), `Rank::all() -> impl Iterator<Item = Rank>` ; `Copy + Ord + Hash` ; JSON = nombre 1–10.
  - `pub fn full_deck() -> Vec<Rank>`, `has_rank(&[Rank], Rank) -> bool`, `lowest(&[Rank]) -> Option<Rank>`, `count_by_rank(&[Rank]) -> [u8; 10]`.
  - `pub struct Supply` — `new(cards, burn: u8, seed: u64)`, `from_parts(deck, seen, unseen, burn, seed)` (le dessus de la pioche est le **dernier** élément), `can_draw()`, `draw() -> Option<Rank>`, `discard(impl IntoIterator<Item = Rank>)`, `deck_len()`, `discard_len()`, `seen() -> &[Rank]`, `shuffles() -> u32`, `unlocated() -> [u8; 10]`, `redeal_hidden(&mut self, rng)`.

- [ ] **Step 1: Déclarer la crate**

`games/pairs/Cargo.toml` :

```toml
[package]
name = "pairs"
description = "Games played with the Pairs deck, for ludo-rs"
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

`Cargo.toml` (racine) : ajouter `"games/pairs"` à `members` et, sous `[workspace.dependencies]`, la ligne `pairs = { path = "games/pairs" }`.

`games/pairs/src/lib.rs` :

```rust
//! Games played with the Pairs deck. Rules and their sources: RULES.md.

pub mod deck;
```

- [ ] **Step 2: Écrire les tests (ils échouent à la compilation)**

`games/pairs/src/deck.rs`, en bas du fichier :

```rust
#[cfg(test)]
mod tests {
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    use super::*;

    fn ranks(values: &[u8]) -> Vec<Rank> {
        values.iter().map(|&v| Rank::new(v).unwrap()).collect()
    }

    #[test]
    fn the_deck_has_n_cards_of_rank_n() {
        let deck = full_deck();
        assert_eq!(deck.len(), DECK_SIZE);
        let counts = count_by_rank(&deck);
        assert_eq!(counts, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    }

    #[test]
    fn ranks_outside_1_to_10_do_not_deserialize() {
        assert!(Rank::new(0).is_none());
        assert!(Rank::new(11).is_none());
        assert!(serde_json::from_str::<Rank>("0").is_err());
        assert!(serde_json::from_str::<Rank>("11").is_err());
        assert_eq!(serde_json::from_str::<Rank>("7").unwrap(), Rank::new(7).unwrap());
        assert_eq!(serde_json::to_string(&Rank::new(7).unwrap()).unwrap(), "7");
    }

    #[test]
    fn helpers_read_a_stack() {
        let stack = ranks(&[9, 3, 6]);
        assert!(has_rank(&stack, Rank::new(3).unwrap()));
        assert!(!has_rank(&stack, Rank::new(4).unwrap()));
        assert_eq!(lowest(&stack), Rank::new(3));
        assert_eq!(lowest(&[]), None);
    }

    #[test]
    fn a_new_supply_burns_five_cards() {
        let s = Supply::new(full_deck(), 5, 1);
        assert_eq!(s.deck_len(), 50);
        assert_eq!(s.discard_len(), 5);
        assert!(s.seen().is_empty());
        assert_eq!(s.shuffles(), 1);
    }

    #[test]
    fn the_same_seed_gives_the_same_order() {
        let mut a = Supply::new(full_deck(), 5, 42);
        let mut b = Supply::new(full_deck(), 5, 42);
        let mut c = Supply::new(full_deck(), 5, 43);
        let (da, db, dc): (Vec<_>, Vec<_>, Vec<_>) = (
            (0..50).map(|_| a.draw()).collect(),
            (0..50).map(|_| b.draw()).collect(),
            (0..50).map(|_| c.draw()).collect(),
        );
        assert_eq!(da, db);
        assert_ne!(da, dc);
    }

    #[test]
    fn an_empty_deck_is_rebuilt_from_the_discards() {
        let mut s = Supply::new(full_deck(), 5, 7);
        let drawn: Vec<Rank> = (0..50).map(|_| s.draw().unwrap()).collect();
        s.discard(drawn);
        // 55 cards in the discard: reshuffle, burn 5, draw 1.
        assert!(s.draw().is_some());
        assert_eq!(s.shuffles(), 2);
        assert_eq!(s.deck_len(), 49);
        assert_eq!(s.discard_len(), 5);
    }

    #[test]
    fn a_small_discard_still_yields_a_card() {
        let mut s = Supply::from_parts(Vec::new(), ranks(&[7, 7]), Vec::new(), 5, 1);
        assert_eq!(s.draw(), Rank::new(7));
        assert_eq!(s.deck_len() + s.discard_len(), 1);
    }

    #[test]
    fn nothing_left_means_no_card() {
        let mut s = Supply::new(Vec::new(), 5, 1);
        assert!(!s.can_draw());
        assert_eq!(s.draw(), None);
    }

    #[test]
    fn from_parts_draws_the_last_card_first() {
        let mut s = Supply::from_parts(ranks(&[2, 5, 9]), Vec::new(), Vec::new(), 0, 0);
        assert_eq!(s.draw(), Rank::new(9));
        assert_eq!(s.draw(), Rank::new(5));
    }

    #[test]
    fn unlocated_counts_the_deck_and_the_burned_cards_only() {
        let mut s = Supply::new(full_deck(), 5, 3);
        assert_eq!(s.unlocated(), [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let card = s.draw().unwrap();
        s.discard([card]);
        let mut expected = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        expected[card.index()] -= 1;
        assert_eq!(s.unlocated(), expected);
        assert_eq!(s.seen(), &[card]);
    }

    #[test]
    fn redealing_keeps_what_everyone_knows() {
        let mut s = Supply::new(full_deck(), 5, 3);
        let card = s.draw().unwrap();
        s.discard([card]);
        let before = s.clone();
        let mut moved = false;
        for seed in 0..10 {
            let mut d = before.clone();
            d.redeal_hidden(&mut SmallRng::seed_from_u64(seed));
            assert_eq!(d.deck_len(), before.deck_len());
            assert_eq!(d.discard_len(), before.discard_len());
            assert_eq!(d.seen(), before.seen());
            assert_eq!(d.unlocated(), before.unlocated());
            assert_eq!(d.shuffles(), before.shuffles());
            moved |= d != before;
        }
        assert!(moved, "the hidden cards never moved");
    }
}
```

- [ ] **Step 3: Vérifier que ça échoue**

Run: `cargo test -p pairs --release`
Expected: erreurs de compilation (`cannot find type Rank`, `Supply`, …).

- [ ] **Step 4: Écrire `deck.rs`** (au-dessus du module de tests)

```rust
//! The Pairs deck: one 1, two 2s, … ten 10s (55 cards), and the draw and
//! discard piles every Pairs game shares.

use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::{Rng, RngCore, SeedableRng};
use serde::{Deserialize, Serialize};

pub const DECK_SIZE: usize = 55;

/// A card. Cards of one rank are interchangeable: there are no suits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct Rank(u8);

impl Rank {
    pub const MIN: u8 = 1;
    pub const MAX: u8 = 10;

    pub fn new(value: u8) -> Option<Self> {
        (Self::MIN..=Self::MAX)
            .contains(&value)
            .then_some(Self(value))
    }

    pub fn value(self) -> u8 {
        self.0
    }

    /// Position in a per-rank table: 0 for the 1, 9 for the 10.
    pub fn index(self) -> usize {
        usize::from(self.0 - Self::MIN)
    }

    pub fn all() -> impl Iterator<Item = Rank> {
        (Self::MIN..=Self::MAX).map(Rank)
    }
}

impl TryFrom<u8> for Rank {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, String> {
        Rank::new(value).ok_or_else(|| format!("rang hors de 1–10 : {value}"))
    }
}

impl From<Rank> for u8 {
    fn from(rank: Rank) -> u8 {
        rank.0
    }
}

/// The 55 cards, in rank order.
pub fn full_deck() -> Vec<Rank> {
    Rank::all()
        .flat_map(|r| std::iter::repeat_n(r, usize::from(r.value())))
        .collect()
}

pub fn has_rank(cards: &[Rank], rank: Rank) -> bool {
    cards.contains(&rank)
}

pub fn lowest(cards: &[Rank]) -> Option<Rank> {
    cards.iter().copied().min()
}

/// How many cards of each rank (see `Rank::index`).
pub fn count_by_rank(cards: &[Rank]) -> [u8; 10] {
    let mut counts = [0u8; 10];
    for card in cards {
        counts[card.index()] += 1;
    }
    counts
}

/// The draw pile and the discard pile. An empty draw pile is rebuilt from the
/// discards; `burn` cards then go face down to the discards.
///
/// The seed lives here so that a reshuffle replays identically from a match
/// log. Never put a `Supply` in a view.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Supply {
    /// Draw pile; the top is the last element.
    deck: Vec<Rank>,
    /// Discards everyone has seen.
    seen: Vec<Rank>,
    /// Discards nobody has seen (burned cards).
    unseen: Vec<Rank>,
    burn: u8,
    seed: u64,
    shuffles: u32,
}

impl Supply {
    /// Shuffles `cards` and burns.
    pub fn new(cards: Vec<Rank>, burn: u8, seed: u64) -> Self {
        let mut supply = Self::from_parts(Vec::new(), Vec::new(), cards, burn, seed);
        supply.restock();
        supply
    }

    /// Piles as given, without shuffling. The top of `deck` is its last element.
    pub fn from_parts(
        deck: Vec<Rank>,
        seen: Vec<Rank>,
        unseen: Vec<Rank>,
        burn: u8,
        seed: u64,
    ) -> Self {
        Self {
            deck,
            seen,
            unseen,
            burn,
            seed,
            shuffles: 0,
        }
    }

    fn restock(&mut self) {
        self.deck.append(&mut self.seen);
        self.deck.append(&mut self.unseen);
        // One independent stream per shuffle.
        let stream = self.seed ^ u64::from(self.shuffles).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        self.deck.shuffle(&mut SmallRng::seed_from_u64(stream));
        self.shuffles += 1;
        // ❓ Never burn the last card: a reshuffle always leaves one to draw.
        let burned = usize::from(self.burn).min(self.deck.len().saturating_sub(1));
        let keep = self.deck.len() - burned;
        self.unseen.extend(self.deck.drain(keep..));
    }

    pub fn can_draw(&self) -> bool {
        !(self.deck.is_empty() && self.seen.is_empty() && self.unseen.is_empty())
    }

    /// The top card. `None` only when both piles are empty.
    pub fn draw(&mut self) -> Option<Rank> {
        if self.deck.is_empty() {
            if !self.can_draw() {
                return None;
            }
            self.restock();
        }
        self.deck.pop()
    }

    /// Face up: everyone has seen these cards.
    pub fn discard(&mut self, cards: impl IntoIterator<Item = Rank>) {
        self.seen.extend(cards);
    }

    pub fn deck_len(&self) -> usize {
        self.deck.len()
    }

    pub fn discard_len(&self) -> usize {
        self.seen.len() + self.unseen.len()
    }

    pub fn seen(&self) -> &[Rank] {
        &self.seen
    }

    /// How many times the draw pile has been built (1 after `new`).
    pub fn shuffles(&self) -> u32 {
        self.shuffles
    }

    /// Per rank, the cards nobody can place: the draw pile and the burned cards.
    pub fn unlocated(&self) -> [u8; 10] {
        let mut counts = count_by_rank(&self.deck);
        for card in &self.unseen {
            counts[card.index()] += 1;
        }
        counts
    }

    /// For determinization: redraws the order of the draw pile, which cards
    /// are burned, and the seed of future shuffles. Pile sizes and the seen
    /// discards are unchanged.
    pub fn redeal_hidden<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        let burned = self.unseen.len();
        self.deck.append(&mut self.unseen);
        self.deck.shuffle(rng);
        let keep = self.deck.len() - burned;
        self.unseen.extend(self.deck.drain(keep..));
        self.seed = rng.next_u64();
    }
}
```

- [ ] **Step 5: Vérifier que ça passe**

Run: `cargo test -p pairs --release`
Expected: 11 tests passent.

- [ ] **Step 6: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
git add Cargo.toml Cargo.lock games/pairs
git commit -m "pairs: the 55-card deck and its draw and discard piles"
```

---

### Task 2: Règles de Continuous Pairs

**Files:**
- Create: `games/pairs/src/continuous/mod.rs`, `games/pairs/src/continuous/view.rs`, `games/pairs/src/continuous/testing.rs`, `games/pairs/tests/continuous_rules.rs`, `games/pairs/RULES.md`
- Modify: `games/pairs/src/lib.rs`

**Interfaces:**
- Consumes: tout `pairs::deck` (tâche 1).
- Produces (dans `pairs::continuous`) :
  - `pub struct ContinuousPairs;` implémentant `ludo_core::Game` (pas encore `Determinize`).
  - `pub struct Config { pub target: Option<u8> }`, `pub const BURN: u8 = 5`, `pub fn default_target(players: u8) -> u8`.
  - `pub enum Action { Hit, Fold { seat: Seat, card: Rank } }` — JSON `"Hit"` et `{"Fold":{"seat":1,"card":3}}`.
  - `pub enum Event { Dealt{seat,card}, Replaced{seat,card}, Paired{seat,card}, Folded{seat,from,card}, Reshuffled }` — JSON étiqueté par `type`.
  - `pub struct State { players, target, supply, stacks, scored, turn, loser, events }` (champs publics) avec `State::deal(players, target, supply) -> State`, `score(seat) -> u8`, `in_play() -> impl Iterator<Item = (Seat, Rank)>`, `lowest_in_play() -> Option<Rank>`, `pair_odds(seat) -> u16` (pour mille), `expected_hit_cost(seat) -> f64`.
  - `pub struct View`, `pub struct SeatView` (JSON camelCase).
  - `pub mod testing` : `r(u8) -> Rank`, `ranks(&[u8]) -> Vec<Rank>`, `supply(draws: &[u8]) -> Supply`, `table(stacks: &[&[u8]], draws: &[u8]) -> State`, `table_with_rest(stacks: &[&[u8]]) -> State`.
- `pairs::ContinuousPairs` est réexporté à la racine de la crate.

- [ ] **Step 1: Écrire `RULES.md`**

`games/pairs/RULES.md` :

```markdown
# Jeux du deck Pairs — règles implémentées

Le deck : 55 cartes sans couleur, une de rang 1, deux de rang 2, … dix de
rang 10. Une *paire* : deux cartes de même rang dans une même pile.

Les points marqués ❓ ne sont pas tranchés par la source : ce sont nos choix.

## Continuous Pairs (`pairs-continuous`, 2 à 8 joueurs)

Source : **[R]** *Pairs Companion Book, PDF Edition* (James Ernest, Hip
Pocket Games, 2018), chapitres « How to Play Pairs », « Breaking a Tie for
Low » et « Continuous Pairs ».

### Mise en place [R]
- On mélange et on brûle 5 cartes (face cachée, à la défausse). On en brûle 5
  à chaque nouveau mélange.
- Chaque joueur reçoit une carte visible. La plus petite commence.
- Égalité pour la plus petite : chaque joueur à égalité reçoit une carte de
  plus ; on compare ces nouvelles cartes entre eux, et on recommence tant
  qu'il reste une égalité. Une carte de départage qui ferait une paire est
  défaussée et remplacée. Les cartes de départage restent dans la pile.

### Tour [R]
- Le joueur prend une carte (*hit*) ou se couche (*fold*). Le tour passe à
  gauche.
- Prendre une carte : si elle fait une paire avec sa pile, il marque le rang
  de la paire, garde une des deux cartes comme carte de score, et le reste de
  **sa** pile est défaussé. Les autres piles ne bougent pas.
- Se coucher : il prend n'importe quelle carte en jeu (dans sa pile ou celle
  d'un autre), la garde comme carte de score et marque son rang ; **sa** pile
  est défaussée.
- Un joueur sans carte ne choisit pas : il reçoit une carte, et c'est son
  tour.
- Les cartes de score ne retournent pas dans la pioche.
- Pioche vide : on mélange la défausse (pas les piles, pas les cartes de
  score), on brûle 5 cartes et on continue.

### Fin [R]
Le premier joueur qui atteint le score cible perd ; il n'y a pas de gagnant
unique. Cible : 31 à 2 joueurs, 21 à 3, 16 à 4, 13 à 5, 11 de 6 à 8.

### Choix sur les points non tranchés
1. ❓ Classement : le perdant est dernier, tous les autres premiers ex æquo.
2. ❓ Au remélange, on ne brûle jamais la dernière carte : s'il y a 5 cartes
   ou moins à mélanger, on en garde une à piocher.
3. ❓ S'il ne reste aucune carte à piocher (pioche et défausse vides),
   prendre une carte n'est pas proposé, et un joueur sans carte passe son
   tour.
4. ❓ Si la pioche s'épuise pendant le départage initial, le premier des
   joueurs encore à égalité commence.
```

- [ ] **Step 2: Écrire les aides de test**

`games/pairs/src/continuous/testing.rs` :

```rust
//! Hand-built tables for tests (of this crate and of its users).

use ludo_core::Seat;

use super::{default_target, State, BURN};
use crate::deck::{full_deck, Rank, Supply};

pub fn r(value: u8) -> Rank {
    Rank::new(value).expect("a rank from 1 to 10")
}

pub fn ranks(values: &[u8]) -> Vec<Rank> {
    values.iter().map(|&v| r(v)).collect()
}

/// A supply whose next draws are `draws`, in that order; empty discard.
pub fn supply(draws: &[u8]) -> Supply {
    let mut deck = ranks(draws);
    deck.reverse();
    Supply::from_parts(deck, Vec::new(), Vec::new(), BURN, 0)
}

/// A game in progress: these stacks, seat 0 to move, no score yet, and
/// `draws` as the next cards of the draw pile.
pub fn table(stacks: &[&[u8]], draws: &[u8]) -> State {
    let players = stacks.len() as u8;
    State {
        players,
        target: default_target(players),
        supply: supply(draws),
        stacks: stacks.iter().map(|s| ranks(s)).collect(),
        scored: vec![Vec::new(); stacks.len()],
        turn: Some(Seat(0)),
        loser: None,
        events: Vec::new(),
    }
}

/// Like `table`, with every other card of the deck in the draw pile.
pub fn table_with_rest(stacks: &[&[u8]]) -> State {
    let mut rest = full_deck();
    for card in stacks.iter().flat_map(|s| ranks(s)) {
        let at = rest
            .iter()
            .position(|&c| c == card)
            .expect("more cards of a rank than the deck holds");
        rest.remove(at);
    }
    let draws: Vec<u8> = rest.iter().map(|c| c.value()).collect();
    table(stacks, &draws)
}
```

- [ ] **Step 3: Écrire les tests de règles (ils échouent à la compilation)**

`games/pairs/tests/continuous_rules.rs` :

```rust
use ludo_core::{Game, Outcome, Pending, Seat, Viewer};
use pairs::continuous::testing::{r, ranks, supply, table, table_with_rest};
use pairs::continuous::{default_target, Action, Config, ContinuousPairs, Event, State, BURN};
use pairs::deck::{Supply, DECK_SIZE};
use serde_json::json;

fn hit(s: &mut State, seat: u8) {
    ContinuousPairs::apply(s, Seat(seat), &Action::Hit).unwrap();
}

fn fold(s: &mut State, seat: u8, from: u8, card: u8) {
    let action = Action::Fold {
        seat: Seat(from),
        card: r(card),
    };
    ContinuousPairs::apply(s, Seat(seat), &action).unwrap();
}

#[test]
fn targets_follow_the_booklet() {
    let targets: Vec<u8> = (2..=8).map(default_target).collect();
    assert_eq!(targets, vec![31, 21, 16, 13, 11, 11, 11]);
}

#[test]
fn a_new_game_deals_one_card_each_and_keeps_55_cards() {
    for players in 2..=8u8 {
        for seed in 0..20 {
            let s = ContinuousPairs::new(&Config::default(), players, seed);
            assert!(s.stacks.iter().all(|stack| !stack.is_empty()));
            let in_play: usize = s.stacks.iter().map(Vec::len).sum();
            assert_eq!(
                in_play + s.supply.deck_len() + s.supply.discard_len(),
                DECK_SIZE
            );
            assert_eq!(s.target, default_target(players));
            assert_eq!(ContinuousPairs::pending(&s).0.len(), 1);
        }
    }
}

#[test]
fn a_zero_target_falls_back_to_the_booklet() {
    let s = ContinuousPairs::new(&Config { target: Some(0) }, 2, 1);
    assert_eq!(s.target, 31);
    assert!(!ContinuousPairs::pending(&s).is_empty());
}

#[test]
fn a_custom_target_is_kept() {
    let s = ContinuousPairs::new(&Config { target: Some(9) }, 3, 1);
    assert_eq!(s.target, 9);
}

#[test]
fn the_lowest_card_starts() {
    let s = State::deal(3, 21, supply(&[9, 4, 7]));
    assert_eq!(s.turn, Some(Seat(1)));
    assert_eq!(s.stacks, vec![ranks(&[9]), ranks(&[4]), ranks(&[7])]);
}

#[test]
fn the_booklet_tie_break_example() {
    // A 10, B 4, C 4, D 10, E 7; then B 8 and C 9: B starts.
    let s = State::deal(5, 13, supply(&[10, 4, 4, 10, 7, 8, 9]));
    assert_eq!(s.turn, Some(Seat(1)));
    assert_eq!(s.stacks[1], ranks(&[4, 8]));
    assert_eq!(s.stacks[2], ranks(&[4, 9]));
    assert_eq!(s.stacks[0], ranks(&[10]));
}

#[test]
fn a_tie_break_card_that_pairs_is_replaced() {
    // A 4, B 4, C 9; A draws a 4 (pair: replaced by a 7), B draws an 8.
    let s = State::deal(3, 21, supply(&[4, 4, 9, 4, 7, 8]));
    assert_eq!(s.stacks[0], ranks(&[4, 7]));
    assert_eq!(s.stacks[1], ranks(&[4, 8]));
    assert_eq!(s.turn, Some(Seat(0)));
    assert_eq!(s.supply.seen(), &[r(4)]);
    assert!(s.events.contains(&Event::Replaced {
        seat: Seat(0),
        card: r(4)
    }));
}

#[test]
fn a_second_tie_deals_a_third_card() {
    // A 3, B 3; then A 6, B 6; then A 9, B 2: B starts.
    let s = State::deal(2, 31, supply(&[3, 3, 6, 6, 9, 2]));
    assert_eq!(s.turn, Some(Seat(1)));
    assert_eq!(s.stacks[1], ranks(&[3, 6, 2]));
}

#[test]
fn a_tie_break_that_runs_out_of_cards_still_starts() {
    let s = State::deal(2, 31, supply(&[3, 3]));
    assert_eq!(s.turn, Some(Seat(0)));
}

#[test]
fn a_hit_without_a_pair_grows_the_stack_and_passes_left() {
    let mut s = table(&[&[8], &[5]], &[3]);
    hit(&mut s, 0);
    assert_eq!(s.stacks[0], ranks(&[8, 3]));
    assert_eq!(s.turn, Some(Seat(1)));
    assert_eq!(
        s.events,
        vec![Event::Dealt {
            seat: Seat(0),
            card: r(3)
        }]
    );
}

#[test]
fn a_pair_scores_its_rank_and_clears_only_that_stack() {
    let mut s = table(&[&[8, 3], &[5]], &[8]);
    hit(&mut s, 0);
    assert!(s.stacks[0].is_empty());
    assert_eq!(s.scored[0], ranks(&[8]));
    assert_eq!(s.score(Seat(0)), 8);
    assert_eq!(s.supply.seen(), ranks(&[8, 3]).as_slice());
    assert_eq!(s.stacks[1], ranks(&[5]));
    assert_eq!(s.turn, Some(Seat(1)));
    assert_eq!(
        s.events,
        vec![Event::Paired {
            seat: Seat(0),
            card: r(8)
        }]
    );
}

#[test]
fn folding_takes_any_card_in_play() {
    let mut s = table(&[&[9, 6], &[2, 7]], &[]);
    fold(&mut s, 0, 1, 2);
    assert_eq!(s.stacks[1], ranks(&[7]));
    assert!(s.stacks[0].is_empty());
    assert_eq!(s.scored[0], ranks(&[2]));
    assert_eq!(s.supply.seen(), ranks(&[9, 6]).as_slice());
    assert_eq!(s.turn, Some(Seat(1)));
}

#[test]
fn folding_on_your_own_card_discards_the_rest() {
    let mut s = table(&[&[9, 4], &[7]], &[]);
    fold(&mut s, 0, 0, 4);
    assert!(s.stacks[0].is_empty());
    assert_eq!(s.scored[0], ranks(&[4]));
    assert_eq!(s.supply.seen(), ranks(&[9]).as_slice());
}

#[test]
fn every_card_in_play_is_a_legal_fold() {
    let s = table(&[&[9, 4], &[7]], &[1]);
    let legal = ContinuousPairs::legal_actions(&s, Seat(0));
    assert_eq!(legal.len(), 4);
    assert!(legal.contains(&Action::Hit));
    for (seat, card) in [(0, 9), (0, 4), (1, 7)] {
        assert!(legal.contains(&Action::Fold {
            seat: Seat(seat),
            card: r(card)
        }));
    }
    assert!(ContinuousPairs::legal_actions(&s, Seat(1)).is_empty());
}

#[test]
fn a_player_without_cards_hits_automatically() {
    let mut s = table(&[&[], &[5]], &[4, 9]);
    s.turn = Some(Seat(1));
    hit(&mut s, 1);
    // Seat 0 had no card: it received one, and that was its turn.
    assert_eq!(s.stacks, vec![ranks(&[9]), ranks(&[5, 4])]);
    assert_eq!(s.turn, Some(Seat(1)));
    assert_eq!(
        s.events,
        vec![
            Event::Dealt {
                seat: Seat(1),
                card: r(4)
            },
            Event::Dealt {
                seat: Seat(0),
                card: r(9)
            },
        ]
    );
}

#[test]
fn folding_on_an_opponents_last_card_refills_both() {
    let mut s = table(&[&[6], &[3]], &[7, 8]);
    fold(&mut s, 0, 1, 3);
    // Both stacks were empty: seat 1 then seat 0 got a card, then seat 1 decides.
    assert_eq!(s.stacks, vec![ranks(&[8]), ranks(&[7])]);
    assert_eq!(s.turn, Some(Seat(1)));
}

#[test]
fn reaching_the_target_ends_the_game_with_one_loser() {
    let mut s = table(&[&[5, 2], &[9], &[4]], &[5]);
    s.target = 5;
    hit(&mut s, 0);
    assert_eq!(s.loser, Some(Seat(0)));
    assert_eq!(ContinuousPairs::pending(&s), Pending::none());
    assert_eq!(
        ContinuousPairs::outcome(&s),
        Some(Outcome::Ranked(vec![1, 0, 0]))
    );
    assert!(ContinuousPairs::legal_actions(&s, Seat(1)).is_empty());
    assert!(ContinuousPairs::apply(&mut s, Seat(1), &Action::Hit).is_err());
}

#[test]
fn scores_add_up_across_scoring_cards() {
    let mut s = table(&[&[3], &[9]], &[]);
    s.scored[0] = ranks(&[8, 2]);
    assert_eq!(s.score(Seat(0)), 10);
    assert_eq!(ContinuousPairs::outcome(&s), None);
}

#[test]
fn a_refused_action_leaves_the_state_alone() {
    let s = table(&[&[9, 4], &[7]], &[1]);
    let refused = [
        (
            Seat(0),
            Action::Fold {
                seat: Seat(1),
                card: r(4),
            },
        ),
        (Seat(1), Action::Hit),
    ];
    for (seat, action) in refused {
        let mut copy = s.clone();
        assert!(ContinuousPairs::apply(&mut copy, seat, &action).is_err());
        assert_eq!(copy, s);
    }
}

#[test]
fn folding_from_a_seat_outside_the_table_is_refused() {
    let s = table(&[&[9, 4], &[7]], &[1]);
    let mut copy = s.clone();
    let action = Action::Fold {
        seat: Seat(7),
        card: r(4),
    };
    assert!(ContinuousPairs::apply(&mut copy, Seat(0), &action).is_err());
    assert_eq!(copy, s);
}

#[test]
fn no_card_to_draw_means_no_hit() {
    let s = table(&[&[5], &[9]], &[]);
    let legal = ContinuousPairs::legal_actions(&s, Seat(0));
    assert!(!legal.contains(&Action::Hit));
    assert_eq!(legal.len(), 2);
    let mut copy = s.clone();
    assert!(ContinuousPairs::apply(&mut copy, Seat(0), &Action::Hit).is_err());
    assert_eq!(copy, s);
}

#[test]
fn an_empty_deck_is_reshuffled_from_the_discards() {
    let mut s = table(&[&[5], &[9]], &[]);
    s.supply = Supply::from_parts(Vec::new(), ranks(&[2, 2, 3, 3, 4, 4, 4]), Vec::new(), BURN, 3);
    hit(&mut s, 0);
    assert_eq!(s.events[0], Event::Reshuffled);
    assert_eq!(s.stacks[0].len(), 2);
    assert_eq!(s.supply.deck_len(), 1);
    assert_eq!(s.supply.discard_len(), 5);
}

#[test]
fn every_viewer_sees_the_same_table() {
    let s = table_with_rest(&[&[10], &[9]]);
    let a = ContinuousPairs::view(&s, Viewer::Seat(Seat(0)));
    assert_eq!(a, ContinuousPairs::view(&s, Viewer::Seat(Seat(1))));
    assert_eq!(a, ContinuousPairs::view(&s, Viewer::Spectator));
    assert_eq!(a.turn, Some(Seat(0)));
    assert_eq!(a.lowest, Some(r(9)));
    assert_eq!(a.deck, 53);
    assert_eq!(a.unlocated, vec![1, 2, 3, 4, 5, 6, 7, 8, 8, 9]);
    // Nine 10s among 53 cards nobody can place.
    assert_eq!(a.seats[0].pair_odds, 169);
    assert_eq!(a.seats[1].pair_odds, 150);
}

#[test]
fn the_json_the_web_relies_on() {
    assert_eq!(serde_json::to_value(Action::Hit).unwrap(), json!("Hit"));
    let fold = Action::Fold {
        seat: Seat(1),
        card: r(3),
    };
    assert_eq!(
        serde_json::to_value(&fold).unwrap(),
        json!({"Fold": {"seat": 1, "card": 3}})
    );
    assert!(serde_json::from_value::<Action>(json!({"Fold": {"seat": 1, "card": 11}})).is_err());
    assert_eq!(
        serde_json::to_value(Event::Folded {
            seat: Seat(0),
            from: Seat(1),
            card: r(3)
        })
        .unwrap(),
        json!({"type": "Folded", "seat": 0, "from": 1, "card": 3})
    );
    assert_eq!(
        serde_json::to_value(Event::Reshuffled).unwrap(),
        json!({"type": "Reshuffled"})
    );
    let s = table(&[&[8], &[5]], &[3]);
    let view = serde_json::to_value(ContinuousPairs::view(&s, Viewer::Spectator)).unwrap();
    assert_eq!(view["seats"][0]["pairOdds"], json!(0));
    assert_eq!(view["seats"][0]["stack"], json!([8]));
    assert_eq!(view["target"], json!(31));
    assert!(view.get("supply").is_none());
}
```

- [ ] **Step 4: Vérifier que ça échoue**

Run: `cargo test -p pairs --release`
Expected: erreurs de compilation (`could not find continuous in pairs`).

- [ ] **Step 5: Écrire le moteur**

`games/pairs/src/lib.rs` devient :

```rust
//! Games played with the Pairs deck. Rules and their sources: RULES.md.

pub mod continuous;
pub mod deck;

pub use continuous::ContinuousPairs;
```

`games/pairs/src/continuous/mod.rs` :

```rust
//! Continuous Pairs (James Ernest and Paul Peterson). Rules and their
//! sources: RULES.md.

pub mod testing;
mod view;

pub use view::{SeatView, View};

use ludo_core::{Game, GameMeta, Illegal, Outcome, Pending, Seat, Viewer};
use serde::{Deserialize, Serialize};

use crate::deck::{full_deck, has_rank, Rank, Supply, DECK_SIZE};

pub struct ContinuousPairs;

/// [R] Cards burned after every shuffle.
pub const BURN: u8 = 5;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// The losing score. `None` (or 0): the booklet's, from the player count.
    pub target: Option<u8>,
}

/// [R] 60 / players + 1, kept at 11 from 6 players up.
pub fn default_target(players: u8) -> u8 {
    match players {
        2 => 31,
        3 => 21,
        4 => 16,
        5 => 13,
        _ => 11,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Take a card.
    Hit,
    /// Take `card` from `seat`'s stack as a scoring card.
    Fold { seat: Seat, card: Rank },
}

/// What happened, for the board to show.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    /// `seat` received `card` without pairing.
    Dealt { seat: Seat, card: Rank },
    /// A tie-break card that paired: discarded and replaced.
    Replaced { seat: Seat, card: Rank },
    /// `seat` paired `card` and scored it.
    Paired { seat: Seat, card: Rank },
    /// `seat` folded, taking `card` from `from`.
    Folded { seat: Seat, from: Seat, card: Rank },
    /// The discards became the draw pile.
    Reshuffled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub players: u8,
    pub target: u8,
    pub supply: Supply,
    /// Face-up cards of each seat. A stack never holds two cards of a rank.
    pub stacks: Vec<Vec<Rank>>,
    /// Scoring cards of each seat; they never return to the supply.
    pub scored: Vec<Vec<Rank>>,
    /// Seat to move; `None` once the game is over.
    pub turn: Option<Seat>,
    pub loser: Option<Seat>,
    /// Since the previous decision (or since the deal).
    pub events: Vec<Event>,
}

impl State {
    /// [R] One card each; the lowest card starts, ties broken by more cards.
    pub fn deal(players: u8, target: u8, supply: Supply) -> State {
        let n = usize::from(players);
        let mut s = State {
            players,
            target,
            supply,
            stacks: vec![Vec::new(); n],
            scored: vec![Vec::new(); n],
            turn: None,
            loser: None,
            events: Vec::new(),
        };
        let mut drawn: Vec<(Seat, Rank)> = Vec::new();
        for seat in (0..players).map(Seat) {
            if let Some(card) = s.draw_unpaired(seat) {
                drawn.push((seat, card));
            }
        }
        let mut tied = lowest_of(&drawn);
        // [R] Tied players get one more card each, compared among themselves.
        while tied.len() > 1 {
            let mut round: Vec<(Seat, Rank)> = Vec::new();
            for &seat in &tied {
                match s.draw_unpaired(seat) {
                    Some(card) => round.push((seat, card)),
                    None => break,
                }
            }
            // ❓4: out of cards, the first of the tied players starts.
            if round.len() < tied.len() {
                break;
            }
            tied = lowest_of(&round);
        }
        s.turn = Some(tied.first().copied().unwrap_or(Seat(0)));
        s
    }

    pub fn score(&self, seat: Seat) -> u8 {
        self.scored[seat.index()].iter().map(|c| c.value()).sum()
    }

    /// Every face-up card with the seat holding it.
    pub fn in_play(&self) -> impl Iterator<Item = (Seat, Rank)> + '_ {
        self.stacks
            .iter()
            .enumerate()
            .flat_map(|(i, stack)| stack.iter().map(move |&card| (Seat(i as u8), card)))
    }

    /// The cheapest fold.
    pub fn lowest_in_play(&self) -> Option<Rank> {
        self.in_play().map(|(_, card)| card).min()
    }

    /// Chance, per thousand, that the next card pairs `seat`'s stack, from
    /// what everyone knows: the cards nobody can place.
    pub fn pair_odds(&self, seat: Seat) -> u16 {
        let counts = self.supply.unlocated();
        let total: u32 = counts.iter().map(|&c| u32::from(c)).sum();
        if total == 0 {
            return 0;
        }
        let matching: u32 = self.stacks[seat.index()]
            .iter()
            .map(|card| u32::from(counts[card.index()]))
            .sum();
        (matching * 1000 / total) as u16
    }

    /// Points `seat` expects to score by taking a card.
    pub fn expected_hit_cost(&self, seat: Seat) -> f64 {
        let counts = self.supply.unlocated();
        let total: u32 = counts.iter().map(|&c| u32::from(c)).sum();
        if total == 0 {
            return 0.0;
        }
        let weighted: u32 = self.stacks[seat.index()]
            .iter()
            .map(|card| u32::from(counts[card.index()]) * u32::from(card.value()))
            .sum();
        f64::from(weighted) / f64::from(total)
    }

    fn draw(&mut self) -> Option<Rank> {
        let before = self.supply.shuffles();
        let card = self.supply.draw();
        if self.supply.shuffles() != before {
            self.events.push(Event::Reshuffled);
        }
        card
    }

    /// [R] A card for `seat` that does not pair its stack: a pairing card is
    /// discarded and replaced. `None` when the supply cannot provide one.
    fn draw_unpaired(&mut self, seat: Seat) -> Option<Rank> {
        for _ in 0..DECK_SIZE {
            let card = self.draw()?;
            if has_rank(&self.stacks[seat.index()], card) {
                self.supply.discard([card]);
                self.events.push(Event::Replaced { seat, card });
            } else {
                self.stacks[seat.index()].push(card);
                self.events.push(Event::Dealt { seat, card });
                return Some(card);
            }
        }
        None
    }

    fn hit(&mut self, seat: Seat, card: Rank) {
        let i = seat.index();
        if has_rank(&self.stacks[i], card) {
            // [R] Score the pair's rank and keep one of its cards; only this
            // player's stack is discarded.
            let stack = std::mem::take(&mut self.stacks[i]);
            self.supply.discard(stack);
            self.scored[i].push(card);
            self.events.push(Event::Paired { seat, card });
        } else {
            self.stacks[i].push(card);
            self.events.push(Event::Dealt { seat, card });
        }
    }

    fn fold(&mut self, seat: Seat, from: Seat, at: usize) {
        let card = self.stacks[from.index()].remove(at);
        self.scored[seat.index()].push(card);
        let stack = std::mem::take(&mut self.stacks[seat.index()]);
        self.supply.discard(stack);
        self.events.push(Event::Folded { seat, from, card });
    }

    /// After `seat` scored or took a card: the game ends, or the turn moves.
    fn finish_turn(&mut self, seat: Seat) {
        if self.score(seat) >= self.target {
            // [R] The first player to reach the target loses.
            self.loser = Some(seat);
            self.turn = None;
        } else {
            self.advance(seat);
        }
    }

    /// The turn passes left. [R] A player with no cards automatically hits,
    /// and that is their turn.
    fn advance(&mut self, from: Seat) {
        let mut seat = from;
        // Each empty stack passed gets a card, so a full lap finds a player
        // with cards. Every stack empty with nothing to draw would put all 55
        // cards among the scoring cards, long after someone reached the target.
        for _ in 0..2 * usize::from(self.players) {
            seat = Seat((seat.0 + 1) % self.players);
            if !self.stacks[seat.index()].is_empty() {
                break;
            }
            // ❓3: with nothing to draw, a player without cards is skipped.
            if let Some(card) = self.draw() {
                self.stacks[seat.index()].push(card);
                self.events.push(Event::Dealt { seat, card });
            }
        }
        self.turn = Some(seat);
    }
}

fn lowest_of(cards: &[(Seat, Rank)]) -> Vec<Seat> {
    let low = cards.iter().map(|&(_, card)| card).min();
    cards
        .iter()
        .filter(|&&(_, card)| Some(card) == low)
        .map(|&(seat, _)| seat)
        .collect()
}

impl Game for ContinuousPairs {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "pairs-continuous",
            name: "Continuous Pairs",
            min_players: 2,
            max_players: 8,
            hidden_info: true,
            clock: false,
        }
    }

    fn new(config: &Config, players: u8, seed: u64) -> State {
        let target = config
            .target
            .filter(|&t| t > 0)
            .unwrap_or_else(|| default_target(players));
        State::deal(players, target, Supply::new(full_deck(), BURN, seed))
    }

    fn pending(s: &State) -> Pending {
        match s.turn {
            Some(seat) => Pending::one(seat),
            None => Pending::none(),
        }
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        if s.turn != Some(seat) {
            return Vec::new();
        }
        let mut actions = Vec::new();
        if s.supply.can_draw() {
            actions.push(Action::Hit);
        }
        // [R] Continuous Pairs: a fold may take any card in play.
        actions.extend(
            s.in_play()
                .map(|(from, card)| Action::Fold { seat: from, card }),
        );
        actions
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        let Some(turn) = s.turn else {
            return Err(Illegal::new("La partie est terminée."));
        };
        if turn != seat {
            return Err(Illegal::new("Ce n'est pas ton tour."));
        }
        match *action {
            Action::Hit => {
                if !s.supply.can_draw() {
                    return Err(Illegal::new("Il n'y a plus de carte à piocher."));
                }
                s.events.clear();
                if let Some(card) = s.draw() {
                    s.hit(seat, card);
                }
            }
            Action::Fold { seat: from, card } => {
                let at = s
                    .stacks
                    .get(from.index())
                    .and_then(|stack| stack.iter().position(|&c| c == card))
                    .ok_or_else(|| Illegal::new("Cette carte n'est pas en jeu."))?;
                s.events.clear();
                s.fold(seat, from, at);
            }
        }
        s.finish_turn(seat);
        Ok(())
    }

    fn view(s: &State, _viewer: Viewer) -> View {
        View::of(s)
    }

    fn outcome(s: &State) -> Option<Outcome> {
        // ❓1: the loser is last, everyone else tied first.
        let loser = s.loser?;
        let mut ranks = vec![0u8; usize::from(s.players)];
        ranks[loser.index()] = 1;
        Some(Outcome::Ranked(ranks))
    }
}
```

`games/pairs/src/continuous/view.rs` :

```rust
use ludo_core::Seat;
use serde::Serialize;

use super::{Event, State};
use crate::deck::Rank;

/// Stacks and scores are public and the draw pile is hidden from everyone:
/// every viewer gets the same view.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub seats: Vec<SeatView>,
    /// The losing score.
    pub target: u8,
    /// Seat to move; `None` once the game is over.
    pub turn: Option<Seat>,
    pub loser: Option<Seat>,
    /// Cards in the draw pile.
    pub deck: usize,
    /// Cards in the discard pile, burned cards included.
    pub discard: usize,
    /// Per rank (index = rank − 1), the cards nobody can place: the draw
    /// pile and the burned cards.
    pub unlocated: Vec<u8>,
    /// The lowest card in play: what the cheapest fold costs.
    pub lowest: Option<Rank>,
    /// What happened since the previous decision.
    pub events: Vec<Event>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatView {
    pub stack: Vec<Rank>,
    pub scored: Vec<Rank>,
    pub score: u8,
    /// Chance, per thousand, that the next card pairs this stack.
    pub pair_odds: u16,
}

impl View {
    pub fn of(s: &State) -> Self {
        Self {
            seats: (0..s.players)
                .map(Seat)
                .map(|seat| SeatView {
                    stack: s.stacks[seat.index()].clone(),
                    scored: s.scored[seat.index()].clone(),
                    score: s.score(seat),
                    pair_odds: s.pair_odds(seat),
                })
                .collect(),
            target: s.target,
            turn: s.turn,
            loser: s.loser,
            deck: s.supply.deck_len(),
            discard: s.supply.discard_len(),
            unlocated: s.supply.unlocated().to_vec(),
            lowest: s.lowest_in_play(),
            events: s.events.clone(),
        }
    }
}
```

- [ ] **Step 6: Vérifier que ça passe**

Run: `cargo test -p pairs --release`
Expected: les 11 tests de `deck` et les 24 tests de `continuous_rules` passent.

- [ ] **Step 7: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
git add games/pairs
git commit -m "pairs: Continuous Pairs rules, view and RULES.md"
```

---

### Task 3: Déterminisation et conformance

**Files:**
- Create: `games/pairs/src/continuous/determinize.rs`, `games/pairs/tests/conformance.rs`
- Modify: `games/pairs/src/continuous/mod.rs`

**Interfaces:**
- Consumes: `Supply::redeal_hidden` (tâche 1), `ContinuousPairs`, `State`, `Config` (tâche 2).
- Produces: `impl Determinize for ContinuousPairs`. (L'inscription au registre attend la tâche 4 : `GameEntry<G>` exige `G: Heuristics`.)

- [ ] **Step 1: Écrire les tests**

`games/pairs/tests/conformance.rs` :

```rust
use ludo_core::conformance::check;
use pairs::continuous::{Config, ContinuousPairs};

#[test]
fn continuous_pairs_conforms_at_every_table_size() {
    for players in 2..=8u8 {
        check::<ContinuousPairs>(&Config::default(), players, 60, u64::from(players));
    }
}

#[test]
fn continuous_pairs_conforms_with_a_short_target() {
    check::<ContinuousPairs>(&Config { target: Some(6) }, 4, 100, 99);
}
```

`games/pairs/src/continuous/determinize.rs` :

```rust
use ludo_core::{Determinize, Seat};
use rand::Rng;

use super::{ContinuousPairs, State};

impl Determinize for ContinuousPairs {
    /// Nobody sees the draw pile or the burned cards, whoever observes:
    /// their order is redrawn, everything else is public.
    fn determinize<R: Rng + ?Sized>(s: &State, _observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        d.supply.redeal_hidden(rng);
        d
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Determinize, Game, Seat, Viewer};
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    use crate::continuous::{Action, Config, ContinuousPairs};

    #[test]
    fn the_next_card_changes_but_not_the_view() {
        let s = ContinuousPairs::new(&Config::default(), 3, 5);
        let seat = ContinuousPairs::pending(&s).0[0];
        let view = ContinuousPairs::view(&s, Viewer::Seat(seat));
        let mut next_cards = Vec::new();
        for seed in 0..20 {
            let mut d = ContinuousPairs::determinize(&s, seat, &mut SmallRng::seed_from_u64(seed));
            assert_eq!(ContinuousPairs::view(&d, Viewer::Seat(seat)), view);
            ContinuousPairs::apply(&mut d, seat, &Action::Hit).unwrap();
            next_cards.push(d.events.clone());
        }
        next_cards.dedup();
        assert!(next_cards.len() > 1, "every determinization drew the same card");
    }

    #[test]
    fn two_determinizations_reshuffle_differently() {
        let s = ContinuousPairs::new(&Config::default(), 2, 5);
        let a = ContinuousPairs::determinize(&s, Seat(0), &mut SmallRng::seed_from_u64(1));
        let b = ContinuousPairs::determinize(&s, Seat(0), &mut SmallRng::seed_from_u64(2));
        assert_ne!(a.supply, b.supply);
    }
}
```

Dans `games/pairs/src/continuous/mod.rs`, ajouter `mod determinize;` au-dessus de `pub mod testing;`.

- [ ] **Step 2: Lancer**

Run: `cargo test -p pairs --release`
Expected: tout passe (la conformance vérifie : aller-retour serde de l'état, `pending` vide ⇔ `outcome`, action hors tour refusée sans modifier l'état, vue indépendante des données cachées, rejeu du journal). Un échec ici désigne un vrai défaut du moteur de la tâche 2 : le corriger dans `mod.rs`, pas dans le test.

- [ ] **Step 3: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
git add games/pairs
git commit -m "pairs: Continuous Pairs determinization and conformance"
```

---

### Task 4: IA de Continuous Pairs

**Files:**
- Create: `games/pairs/src/continuous/heuristics.rs`
- Modify: `games/pairs/src/continuous/mod.rs`, `crates/ludo-registry/Cargo.toml`, `crates/ludo-registry/src/lib.rs`, `CLAUDE.md`

**Interfaces:**
- Consumes: `State::expected_hit_cost`, `State::lowest_in_play`, `State::in_play`, `State::score`, `Supply::can_draw`, `testing::{table, table_with_rest, r}`.
- Produces: `impl Heuristics for ContinuousPairs` (`rollout_action`, `search_actions` ; `evaluate` et `budget` par défaut) ; l'entrée `pairs-continuous` dans `ludo_registry::GAMES`.

- [ ] **Step 1: Écrire les tests**

`games/pairs/src/continuous/heuristics.rs`, en bas :

```rust
#[cfg(test)]
mod tests {
    use ludo_core::{Game, Heuristics, Seat};
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    use crate::continuous::testing::{r, table, table_with_rest};
    use crate::continuous::{Action, ContinuousPairs};

    fn rollout(s: &crate::continuous::State, seed: u64) -> Action {
        ContinuousPairs::rollout_action(s, Seat(0), &mut SmallRng::seed_from_u64(seed))
    }

    #[test]
    fn folds_cheaply_rather_than_risk_a_big_stack() {
        // About 4.3 points at risk on a hit; folding on the 1 costs 1.
        let s = table_with_rest(&[&[10, 9, 8], &[1]]);
        for seed in 0..30 {
            assert_eq!(
                rollout(&s, seed),
                Action::Fold {
                    seat: Seat(1),
                    card: r(1)
                }
            );
        }
    }

    #[test]
    fn hits_when_folding_costs_more_than_the_risk() {
        // About 1.7 points at risk; the cheapest fold costs 9.
        let s = table_with_rest(&[&[10], &[9]]);
        for seed in 0..30 {
            assert_eq!(rollout(&s, seed), Action::Hit);
        }
    }

    #[test]
    fn never_folds_into_the_target() {
        let mut s = table_with_rest(&[&[10, 9, 8], &[2]]);
        s.scored[0] = vec![r(10), r(10), r(9)];
        assert_eq!(s.score(Seat(0)) + 2, s.target);
        for seed in 0..30 {
            assert_eq!(rollout(&s, seed), Action::Hit);
        }
    }

    #[test]
    fn folds_when_there_is_nothing_to_draw() {
        let s = table(&[&[10], &[9]], &[]);
        for seed in 0..30 {
            assert_eq!(
                rollout(&s, seed),
                Action::Fold {
                    seat: Seat(1),
                    card: r(9)
                }
            );
        }
    }

    #[test]
    fn the_rollout_action_is_always_legal() {
        use crate::continuous::Config;
        use rand::Rng;
        let mut rng = SmallRng::seed_from_u64(11);
        for players in 2..=8u8 {
            let mut s = ContinuousPairs::new(&Config::default(), players, rng.gen());
            while let Some(&seat) = ContinuousPairs::pending(&s).0.first() {
                let action = ContinuousPairs::rollout_action(&s, seat, &mut rng);
                assert!(ContinuousPairs::legal_actions(&s, seat).contains(&action));
                ContinuousPairs::apply(&mut s, seat, &action).unwrap();
            }
        }
    }

    #[test]
    fn the_search_only_weighs_the_two_lowest_ranks() {
        let s = table_with_rest(&[&[10, 7], &[3, 9], &[3, 5]]);
        let actions = ContinuousPairs::search_actions(&s, Seat(0));
        let fold = |seat: u8, card: u8| Action::Fold {
            seat: Seat(seat),
            card: r(card),
        };
        assert_eq!(actions.len(), 4);
        for wanted in [Action::Hit, fold(1, 3), fold(2, 3), fold(2, 5)] {
            assert!(actions.contains(&wanted), "{wanted:?}");
        }
        let legal = ContinuousPairs::legal_actions(&s, Seat(0));
        assert!(actions.iter().all(|a| legal.contains(a)));
    }

    #[test]
    fn the_search_keeps_a_fold_when_it_is_the_only_move() {
        let s = table(&[&[10], &[9]], &[]);
        assert!(!ContinuousPairs::search_actions(&s, Seat(0)).is_empty());
    }
}
```

Dans `mod.rs`, ajouter `mod heuristics;` sous `mod determinize;`.

- [ ] **Step 2: Vérifier que ça échoue**

Run: `cargo test -p pairs --release heuristics`
Expected: erreur de compilation (`Heuristics` n'est pas implémenté pour `ContinuousPairs`).

- [ ] **Step 3: Écrire la politique** (au-dessus des tests)

```rust
use ludo_core::{Game, Heuristics, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use super::{Action, ContinuousPairs, State};
use crate::deck::Rank;

/// Below this gap (in points) between folding and hitting, either is played.
const CLOSE: f64 = 0.5;

/// The folds on the lowest card in play, from whoever holds one.
fn cheapest_folds(s: &State) -> Vec<Action> {
    let Some(low) = s.lowest_in_play() else {
        return Vec::new();
    };
    s.in_play()
        .filter(|&(_, card)| card == low)
        .map(|(seat, card)| Action::Fold { seat, card })
        .collect()
}

impl Heuristics for ContinuousPairs {
    /// Fold when the sure cost of the lowest card is below the points a hit
    /// puts at risk; never fold into the target while a card can be drawn.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        let Some(fold) = cheapest_folds(s).choose(rng).cloned() else {
            return Action::Hit;
        };
        if !s.supply.can_draw() {
            return fold;
        }
        let Action::Fold { card, .. } = fold else {
            return Action::Hit;
        };
        if s.score(seat) + card.value() >= s.target {
            return Action::Hit;
        }
        let margin = s.expected_hit_cost(seat) - f64::from(card.value());
        if margin > CLOSE || (margin > -CLOSE && rng.gen_bool(0.5)) {
            fold
        } else {
            Action::Hit
        }
    }

    /// A hit, and the folds on the two lowest ranks in play: paying more than
    /// that to fold is never worth searching. Stacks are public, so this
    /// depends only on what `seat` sees.
    fn search_actions(s: &State, seat: Seat) -> Vec<Action> {
        let mut in_play: Vec<Rank> = s.in_play().map(|(_, card)| card).collect();
        in_play.sort_unstable();
        in_play.dedup();
        let legal = Self::legal_actions(s, seat);
        let Some(&limit) = in_play.get(1).or(in_play.first()) else {
            return legal;
        };
        legal
            .into_iter()
            .filter(|action| match action {
                Action::Hit => true,
                Action::Fold { card, .. } => *card <= limit,
            })
            .collect()
    }
}
```

- [ ] **Step 4: Inscrire le jeu au registre**

`crates/ludo-registry/Cargo.toml`, sous `[dependencies]` : ajouter `pairs.workspace = true`.

`crates/ludo-registry/src/lib.rs` : ajouter le `static` et l'entrée.

```rust
static YAVALATH: GameEntry<yavalath::Yavalath> = GameEntry::new();
static RIICHI: GameEntry<riichi::Riichi> = GameEntry::new();
static PAIRS_CONTINUOUS: GameEntry<pairs::ContinuousPairs> = GameEntry::new();

pub static GAMES: &[&dyn AnyGame] = &[&YAVALATH, &RIICHI, &PAIRS_CONTINUOUS];
```

- [ ] **Step 5: Vérifier que tout passe**

Run: `cargo test --workspace --release`
Expected: tout passe, dont les 7 tests de `heuristics` et `ids_are_unique_and_every_game_starts` du registre (qui démarre une partie de 2 à 8 joueurs).

- [ ] **Step 6: Mesurer l'IA**

Run: `cargo run --release -p ludo-cli -- arena pairs-continuous 400 hard easy`
puis `cargo run --release -p ludo-cli -- arena pairs-continuous 400 hard easy 4`
Expected: le score de Difficile (premier niveau) est ≥ 0,55 à 2 joueurs.

Si le score à 2 joueurs est sous 0,55 : essayer, en remesurant à chaque fois sur 400 parties, (a) `CLOSE` à 0.25 puis 1.0, (b) `search_actions` limité au seul rang le plus bas (`in_play.first()`). Garder le réglage qui marque le plus. Si aucun n'atteint 0,55, garder le meilleur : ce jeu a un plafond de hasard, et le chiffre mesuré devient le seuil consigné au step suivant.

- [ ] **Step 7: Consigner dans `CLAUDE.md`**

Dans la section `## Commandes`, sous la ligne `ludo-cli -- arena yavalath …`, ajouter (en remplaçant les deux nombres par ceux mesurés au step 6) :

```markdown
- `cargo run --release -p ludo-cli -- arena pairs-continuous 400 hard easy` : Difficile doit marquer ≥ 0.55 contre Facile (<mesure à 2 joueurs> mesuré ; <mesure à 4 joueurs> à 4 joueurs, où le hasard pèse plus).
```

Dans la section `## Principes`, ajouter :

```markdown
- **Pairs** : `games/pairs` porte tous les jeux du deck Pairs, un module par jeu, le matériel commun dans `deck.rs`. La graine des remélanges vit dans le `Supply` : il ne va jamais dans une vue, et `determinize` appelle `redeal_hidden`.
```

- [ ] **Step 8: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
git add games/pairs CLAUDE.md Cargo.lock crates/ludo-registry
git commit -m "pairs: Continuous Pairs AI (fold-or-hit policy, narrowed search)"
```

---

### Task 5: Cartes partagées et plateau web

**Files:**
- Create: `web/src/games/pairs/cards.tsx`, `web/src/games/pairs/pairs.css`, `web/src/games/pairs/continuous/types.ts`, `web/src/games/pairs/continuous/Board.tsx`, `web/src/games/pairs/__tests__/continuous.test.tsx`
- Modify: `web/src/games/registry.ts`, `README.md`, `NOTICE`, `docs/superpowers/specs/2026-10-13-pairs-design.md`

**Interfaces:**
- Consumes: le JSON du moteur, figé par le test `the_json_the_web_relies_on` (tâche 2) ; `BoardProps` de `web/src/games/registry.ts` (`view`, `legalActions`, `pending`, `me`, `seatNames`, `submit`).
- Produces: `Card`, `CardBack`, `Table` (réutilisés par le plan 2) ; `ContinuousBoard` ; `foldTargets`, `canHit`, `describeEvent`.

- [ ] **Step 1: Types miroir du moteur**

`web/src/games/pairs/continuous/types.ts` :

```ts
// Mirrors games/pairs/src/continuous (View, Action, Event).

/** 1 to 10. */
export type Rank = number;

export type ContinuousEvent =
  | { type: "Dealt"; seat: number; card: Rank }
  | { type: "Replaced"; seat: number; card: Rank }
  | { type: "Paired"; seat: number; card: Rank }
  | { type: "Folded"; seat: number; from: number; card: Rank }
  | { type: "Reshuffled" };

export interface ContinuousSeatView {
  stack: Rank[];
  scored: Rank[];
  score: number;
  /** Per thousand. */
  pairOdds: number;
}

export interface ContinuousView {
  seats: ContinuousSeatView[];
  target: number;
  turn: number | null;
  loser: number | null;
  deck: number;
  discard: number;
  /** Index = rank − 1. */
  unlocated: number[];
  lowest: Rank | null;
  events: ContinuousEvent[];
}

export type ContinuousAction = "Hit" | { Fold: { seat: number; card: Rank } };
```

- [ ] **Step 2: Écrire le test du plateau (il échoue)**

`web/src/games/pairs/__tests__/continuous.test.tsx` :

```tsx
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { ContinuousBoard, canHit, describeEvent, foldTargets } from "../continuous/Board";
import type { ContinuousAction, ContinuousView } from "../continuous/types";

const NAMES = ["Ana", "Bob", "Cléo"];

const VIEW: ContinuousView = {
  seats: [
    { stack: [10, 7], scored: [3], score: 3, pairOdds: 320 },
    { stack: [4], scored: [], score: 0, pairOdds: 75 },
    { stack: [], scored: [8, 2], score: 10, pairOdds: 0 },
  ],
  target: 21,
  turn: 0,
  loser: null,
  deck: 31,
  discard: 12,
  unlocated: [1, 1, 2, 3, 4, 5, 5, 6, 8, 8],
  lowest: 4,
  events: [
    { type: "Paired", seat: 2, card: 8 },
    { type: "Reshuffled" },
  ],
};

const LEGAL: ContinuousAction[] = [
  "Hit",
  { Fold: { seat: 0, card: 10 } },
  { Fold: { seat: 0, card: 7 } },
  { Fold: { seat: 1, card: 4 } },
];

function render(view: ContinuousView, legal: ContinuousAction[], me: number | null) {
  return renderToStaticMarkup(
    <ContinuousBoard
      view={view}
      legalActions={legal}
      pending={view.turn === null ? [] : [view.turn]}
      me={me}
      seatNames={NAMES}
      submit={() => {}}
    />,
  );
}

describe("Continuous Pairs board", () => {
  it("reads the moves from the legal actions and nowhere else", () => {
    expect(canHit(LEGAL)).toBe(true);
    expect(canHit([{ Fold: { seat: 1, card: 4 } }])).toBe(false);
    expect(foldTargets(LEGAL)).toEqual([
      { seat: 0, card: 10 },
      { seat: 0, card: 7 },
      { seat: 1, card: 4 },
    ]);
    expect(foldTargets([])).toEqual([]);
  });

  it("offers a hit and one button per card that can be taken", () => {
    const html = render(VIEW, LEGAL, 0);
    expect(html).toContain("Prendre une carte");
    expect(html.match(/<button[^>]*pairs-card/g)).toHaveLength(3);
    expect(html).toContain("3 / 21");
    expect(html).toContain("Risque de paire : 32 %");
    expect(html).toContain("Se coucher coûte au moins 4");
  });

  it("offers nothing to a player who is not to move", () => {
    const html = render(VIEW, [], 1);
    expect(html).not.toContain("<button");
    expect(html).toContain("Ana");
  });

  it("shows the loser and offers nothing once the game is over", () => {
    const over: ContinuousView = { ...VIEW, turn: null, loser: 2, events: [] };
    const html = render(over, [], 0);
    expect(html).toContain("Cléo a perdu");
    expect(html).not.toContain("<button");
  });

  it("seats eight players", () => {
    const names = ["A", "B", "C", "D", "E", "F", "G", "H"];
    const eight: ContinuousView = {
      ...VIEW,
      seats: names.map(() => ({ stack: [5], scored: [], score: 0, pairOdds: 80 })),
      target: 11,
    };
    const html = renderToStaticMarkup(
      <ContinuousBoard view={eight} legalActions={[]} pending={[0]} me={null} seatNames={names} submit={() => {}} />,
    );
    expect(html.match(/class="pairs-seat/g)).toHaveLength(8);
  });

  it("tells what happened in words", () => {
    expect(describeEvent({ type: "Dealt", seat: 1, card: 6 }, NAMES)).toBe("Bob reçoit un 6.");
    expect(describeEvent({ type: "Paired", seat: 2, card: 8 }, NAMES)).toBe("Cléo fait une paire de 8 : 8 points.");
    expect(describeEvent({ type: "Folded", seat: 0, from: 1, card: 4 }, NAMES)).toBe(
      "Ana se couche et prend le 4 de Bob : 4 points.",
    );
    expect(describeEvent({ type: "Folded", seat: 0, from: 0, card: 7 }, NAMES)).toBe(
      "Ana se couche et garde son 7 : 7 points.",
    );
    expect(describeEvent({ type: "Replaced", seat: 0, card: 4 }, NAMES)).toBe(
      "Départage : le 4 d'Ana ferait une paire, il est remplacé.",
    );
    expect(describeEvent({ type: "Reshuffled" }, NAMES)).toBe("La défausse est remélangée, 5 cartes sont brûlées.");
  });
});
```

Run: `cd web && pnpm test`
Expected: FAIL (`Cannot find module '../continuous/Board'`).

- [ ] **Step 3: Composants de cartes partagés**

`web/src/games/pairs/cards.tsx` :

```tsx
import type { CSSProperties, ReactNode } from "react";
import "./pairs.css";

/** A face-up card: a button when it can be chosen, plain otherwise. */
export function Card({
  rank,
  onClick,
  title,
  small,
}: {
  rank: number;
  onClick?: () => void;
  title?: string;
  small?: boolean;
}) {
  const cls = `pairs-card r${rank}${small ? " small" : ""}`;
  if (!onClick) return <span className={cls}>{rank}</span>;
  return (
    <button type="button" className={`${cls} playable`} onClick={onClick} title={title} aria-label={title}>
      {rank}
    </button>
  );
}

/** A face-down pile with its size. */
export function CardBack({ count, label }: { count: number; label: string }) {
  return (
    <span className="pairs-pile">
      <span className="pairs-card back">{count}</span>
      {label}
    </span>
  );
}

/**
 * Seats around a centre. `first` is the seat drawn at the bottom (the
 * viewer's). On narrow screens the CSS turns the ring into a list.
 */
export function Table({ seats, first, center }: { seats: ReactNode[]; first: number; center: ReactNode }) {
  const n = seats.length;
  return (
    <div className="pairs-table">
      <div className="pairs-center">{center}</div>
      {seats.map((seat, i) => {
        const angle = Math.PI / 2 + (2 * Math.PI * ((i - first + n) % n)) / n;
        const style = {
          "--x": `${50 + 41 * Math.cos(angle)}%`,
          "--y": `${50 + 40 * Math.sin(angle)}%`,
        } as CSSProperties;
        return (
          <div key={i} className="pairs-place" style={style}>
            {seat}
          </div>
        );
      })}
    </div>
  );
}
```

`web/src/games/pairs/pairs.css` :

```css
.pairs-card {
  display: inline-flex; align-items: center; justify-content: center;
  width: 38px; height: 54px; margin: 0 2px; padding: 0;
  border: 1px solid var(--line); border-radius: 6px;
  background: var(--card); color: var(--rank, inherit);
  font: 700 1.25rem/1 system-ui, sans-serif;
  box-shadow: inset 0 -5px 0 var(--rank, var(--line));
}
.pairs-card.small { width: 24px; height: 34px; font-size: 0.85rem; box-shadow: inset 0 -3px 0 var(--rank, var(--line)); }
.pairs-card.back { background: repeating-linear-gradient(45deg, var(--line), var(--line) 4px, var(--card) 4px, var(--card) 8px); color: inherit; box-shadow: none; }
.pairs-card.playable { cursor: pointer; outline: 2px solid var(--accent); outline-offset: 1px; }
.pairs-card.playable:hover { transform: translateY(-3px); }
.pairs-card.r1 { --rank: hsl(100 45% 40%); }
.pairs-card.r2 { --rank: hsl(25 80% 50%); }
.pairs-card.r3 { --rank: hsl(350 70% 48%); }
.pairs-card.r4 { --rank: hsl(225 60% 52%); }
.pairs-card.r5 { --rank: hsl(280 45% 50%); }
.pairs-card.r6 { --rank: hsl(35 85% 45%); }
.pairs-card.r7 { --rank: hsl(20 35% 42%); }
.pairs-card.r8 { --rank: hsl(52 80% 38%); }
.pairs-card.r9 { --rank: hsl(5 75% 50%); }
.pairs-card.r10 { --rank: hsl(175 55% 34%); }

.pairs-pile { display: inline-flex; flex-direction: column; align-items: center; gap: 2px; font-size: 0.8rem; color: var(--muted); }

.pairs-table { position: relative; width: min(100%, 860px); aspect-ratio: 4 / 3; margin: 0 auto; }
.pairs-center {
  position: absolute; left: 50%; top: 50%; transform: translate(-50%, -50%);
  width: 46%; display: flex; flex-direction: column; align-items: center; gap: 8px; text-align: center;
}
.pairs-place { position: absolute; left: var(--x); top: var(--y); transform: translate(-50%, -50%); width: 24%; min-width: 150px; }

.pairs-seat { background: var(--card); border: 1px solid var(--line); border-radius: 10px; padding: 6px 8px; }
.pairs-seat.active { border-color: var(--accent); box-shadow: 0 0 0 2px var(--accent); }
.pairs-seat.lost { border-color: var(--danger); }
.pairs-seat h3 { margin: 0 0 4px; font-size: 0.95rem; display: flex; justify-content: space-between; gap: 8px; }
.pairs-seat .stack { min-height: 56px; display: flex; flex-wrap: wrap; row-gap: 4px; }
.pairs-seat .scored { margin-top: 4px; min-height: 36px; }
.pairs-seat .odds { margin: 2px 0 0; font-size: 0.8rem; color: var(--muted); }
.pairs-piles { display: flex; gap: 16px; justify-content: center; }
.pairs-unlocated { display: flex; gap: 6px; flex-wrap: wrap; justify-content: center; font-size: 0.75rem; color: var(--muted); }
.pairs-unlocated span { display: inline-flex; flex-direction: column; align-items: center; }
.pairs-events { list-style: none; padding: 0; margin: 0; font-size: 0.85rem; }

@media (max-width: 720px) {
  .pairs-table { aspect-ratio: auto; display: flex; flex-direction: column; gap: 8px; }
  .pairs-center, .pairs-place { position: static; transform: none; width: auto; }
}
```

- [ ] **Step 4: Le plateau**

`web/src/games/pairs/continuous/Board.tsx` :

```tsx
import type { BoardProps } from "../../registry";
import { Card, CardBack, Table } from "../cards";
import type { ContinuousAction, ContinuousEvent, ContinuousView, Rank } from "./types";

export function canHit(legal: ContinuousAction[]): boolean {
  return legal.includes("Hit");
}

/** The cards the player may take by folding, as the engine lists them. */
export function foldTargets(legal: ContinuousAction[]): { seat: number; card: Rank }[] {
  return legal.flatMap((a) => (typeof a === "object" ? [a.Fold] : []));
}

/** « d'Ana », « de Bob ». */
function of(name: string): string {
  return /^[aeiouyhàâéèêëîïôùûœ]/i.test(name) ? `d'${name}` : `de ${name}`;
}

export function describeEvent(e: ContinuousEvent, names: string[]): string {
  switch (e.type) {
    case "Dealt":
      return `${names[e.seat]} reçoit un ${e.card}.`;
    case "Replaced":
      return `Départage : le ${e.card} ${of(names[e.seat])} ferait une paire, il est remplacé.`;
    case "Paired":
      return `${names[e.seat]} fait une paire de ${e.card} : ${e.card} points.`;
    case "Folded":
      return e.seat === e.from
        ? `${names[e.seat]} se couche et garde son ${e.card} : ${e.card} points.`
        : `${names[e.seat]} se couche et prend le ${e.card} ${of(names[e.from])} : ${e.card} points.`;
    case "Reshuffled":
      return "La défausse est remélangée, 5 cartes sont brûlées.";
  }
}

export function ContinuousBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as ContinuousView;
  const legal = legalActions as ContinuousAction[];
  const folds = foldTargets(legal);

  const seats = v.seats.map((seat, i) => {
    const cls = `pairs-seat${v.turn === i ? " active" : ""}${v.loser === i ? " lost" : ""}`;
    return (
      <section key={i} className={cls}>
        <h3>
          <span>{seatNames[i]}</span>
          <span>
            {seat.score} / {v.target}
          </span>
        </h3>
        <div className="stack">
          {seat.stack.map((card, k) => {
            const takeable = folds.some((f) => f.seat === i && f.card === card);
            return (
              <Card
                key={k}
                rank={card}
                title={takeable ? `Se coucher en prenant ce ${card} (${card} points)` : undefined}
                onClick={
                  takeable ? () => submit({ Fold: { seat: i, card } } satisfies ContinuousAction) : undefined
                }
              />
            );
          })}
        </div>
        {seat.stack.length > 0 && <p className="odds">Risque de paire : {Math.round(seat.pairOdds / 10)} %</p>}
        <div className="scored">
          {seat.scored.map((card, k) => (
            <Card key={k} rank={card} small />
          ))}
        </div>
      </section>
    );
  });

  const center = (
    <>
      <div className="pairs-piles">
        <CardBack count={v.deck} label="Pioche" />
        <CardBack count={v.discard} label="Défausse" />
      </div>
      {v.loser !== null && <p className="hint">{seatNames[v.loser]} a perdu.</p>}
      {canHit(legal) && (
        <button type="button" onClick={() => submit("Hit" satisfies ContinuousAction)}>
          Prendre une carte
        </button>
      )}
      {folds.length > 0 && v.lowest !== null && (
        <p className="hint">Se coucher coûte au moins {v.lowest} : clique sur la carte à prendre.</p>
      )}
      <div className="pairs-unlocated" title="Cartes que personne ne peut situer : pioche et cartes brûlées">
        {v.unlocated.map((count, i) => (
          <span key={i}>
            <Card rank={i + 1} small />×{count}
          </span>
        ))}
      </div>
      <ul className="pairs-events">
        {v.events.map((e, i) => (
          <li key={i}>{describeEvent(e, seatNames)}</li>
        ))}
      </ul>
    </>
  );

  return <Table seats={seats} first={me ?? 0} center={center} />;
}
```

`web/src/games/registry.ts` : importer et inscrire le plateau.

```ts
import { ContinuousBoard } from "./pairs/continuous/Board";
```

```ts
export const GAME_UIS: GameUi[] = [
  { id: "yavalath", Board: YavalathBoard },
  { id: "riichi", Board: RiichiBoard },
  { id: "pairs-continuous", Board: ContinuousBoard },
];
```

- [ ] **Step 5: Vérifier le web**

Run: `./scripts/build-wasm.sh dev && cd web && pnpm test && pnpm build`
Expected: les 6 tests de `continuous.test.tsx` passent avec les tests existants ; `tsc --noEmit` et `vite build` sans erreur.

- [ ] **Step 6: Jouer une partie dans le navigateur**

Run: `cd web && pnpm dev`, ouvrir l'URL affichée.
Vérifier, contre des IA puis à une table de 8 sièges :
- « Continuous Pairs » figure dans la liste des jeux, de 2 à 8 joueurs ;
- le premier joueur est celui de la plus petite carte ;
- « Prendre une carte » et le clic sur une carte (se coucher) jouent le coup, les scores montent, le journal dit ce que les IA ont fait ;
- la partie s'arrête quand un joueur atteint la cible et le nomme perdant ;
- à 8 joueurs les panneaux ne se chevauchent pas (sinon ajuster `width`/`min-width` de `.pairs-place` dans `pairs.css`).

Noter dans le message de commit tout défaut d'affichage constaté et non corrigé.

- [ ] **Step 7: README, NOTICE, spec**

`README.md`, dans la liste des jeux, après la ligne Mahjong Riichi :

```markdown
- **Continuous Pairs** (2–8 joueurs, James Ernest et Paul Peterson, deck Pairs).
```

`NOTICE`, à la fin :

```
Pairs est un jeu de James Ernest et Paul Peterson (Hip Pocket Games /
Cheapass Games). Les règles implémentées dans games/pairs sont reformulées
d'après le « Pairs Companion Book » ; ni ses textes ni les illustrations des
éditions du jeu ne sont inclus.
```

`docs/superpowers/specs/2026-10-13-pairs-design.md` : ajouter à la fin une section.

```markdown
## 9. Ajustements faits au plan 1

- `Supply::draw()` ne prend pas de `rng` : la graine des remélanges est dans
  le `Supply`, pour qu'un remélange se rejoue à l'identique depuis le
  `MatchLog`. `determinize` la retire au hasard (`redeal_hidden`).
- Au remélange, on ne brûle jamais la dernière carte.
- Continuous Pairs ne fournit pas `Heuristics::evaluate` : les simulations
  vont jusqu'au bout, elles sont courtes.
- `pair_in`, la défausse face cachée d'une main et la redistribution des
  mains arrivent avec l'Auberge (plan 2).
```

- [ ] **Step 8: Commit**

```bash
cd .. && cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --release
git add web/src/games README.md NOTICE docs/superpowers/specs/2026-10-13-pairs-design.md
git commit -m "web: Continuous Pairs board and the shared Pairs cards"
```

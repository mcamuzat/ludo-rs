# Pairs, plan 2 : L'Auberge des Pirates — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** L'Auberge des Pirates (Regent multijoueur sans jetons, 2–8 joueurs, en plusieurs manches) se joue sur ludo-rs contre l'IA et en ligne.

**Architecture:** Un second module de `games/pairs`, `auberge/`, implémente `Game`, `Determinize`, `Heuristics` sous l'id `pairs-auberge`, sur le `Supply` du plan 1 étendu aux mains cachées. Le plateau web réutilise `cards.tsx`. La plateforme n'est pas modifiée.

**Tech Stack:** Rust stable (rand 0.8, rand_chacha 0.3, serde), React 19 + TypeScript + Vite 7, vitest 3.

**Spec:** `docs/superpowers/specs/2026-10-13-pairs-design.md` (§5, §6–8, et §9 pour les ajustements).

## Global Constraints

- Branche `auberge`. Avant chaque commit : `cargo fmt --all` et `cargo clippy --workspace --all-targets -- -D warnings`. Tests : `cargo test --workspace --release`.
- `games/pairs` reste natif. `apply` rend `Illegal` sans modifier l'état et ne panique jamais.
- Une vue ne contient jamais la graine, la pioche, la main d'un autre siège, ni la carte qu'un autre a choisie pendant la mise en place.
- Tout mélange du moteur utilise `ChaCha8Rng`, jamais `SmallRng`.
- Le web affiche et envoie : risque estimé, paire en main, cartes non situées viennent de la `View`.
- Messages `Illegal` et interface en français ; commentaires de code en anglais.
- Id `pairs-auberge`, nom `L'Auberge des Pirates`, 2–8 joueurs, `hidden_info: true`, `clock: false`.
- Main de départ : 3 cartes. Manches : 5 par défaut, plafonnées à 15. Aucune carte brûlée.

### Écarts assumés par rapport à la spec (consignés dans la spec à la tâche 5)

- `Config { rounds: Option<u8> }` (et non `u8`) : `None` ou 0 = 5 manches, comme la config de Continuous Pairs.
- Une manche bloquée (plus de pioche, le joueur au tour sans action) ne rapporte de point à personne, au lieu d'être partagée : les points restent des entiers.
- Pendant la mise en place, un siège ne sait pas qui a déjà posé (`visible_pending` ne montre que soi) : la déterminisation peut ainsi refaire le choix des autres.
- Le résultat d'une accusation est public, mais ni la main montrée ni le rang de la paire.

## Review Focus

1. **La main montrée par une accusation** ne doit apparaître que chez l'accusateur et l'accusé, jamais chez un tiers ni un spectateur. → `the_shown_hand_reaches_only_the_accuser_and_the_accused` (tâche 2).
2. **Manche bloquée** (pioche et défausse vides, aucune attaque ni accusation possible) : la partie continue. → `a_stuck_round_is_won_by_nobody` (tâche 2).
3. **Cible hors table ou soi-même** (`Dare { target: 9 }`, s'accuser soi-même, viser un éliminé) : `Illegal`, pas de panique. → `absurd_targets_are_refused` (tâche 2).
4. **Remélange après des éliminations** : ce qu'un accusateur a vu n'est plus localisable, la connaissance doit être oubliée. → `a_reshuffle_forgets_what_was_seen` (tâche 2).
5. **Plateau pendant la mise en place une fois ma carte posée, et en fin de partie** : rien à cliquer. → tests web (tâche 5).

---

### Task 1: Matériel commun pour les mains cachées

**Files:**
- Create: `games/pairs/src/testing.rs`
- Modify: `games/pairs/src/deck.rs`, `games/pairs/src/lib.rs`, `games/pairs/src/continuous/testing.rs`

**Interfaces:**
- Produces : `deck::pair_in(&[Rank]) -> Option<Rank>` (le plus haut rang présent deux fois) ; `Supply::discard_unseen(impl IntoIterator<Item = Rank>)` ; `Supply::redeal_hidden_with(&mut self, keep: &[Rank], hands: &mut [&mut Vec<Rank>], rng)` ; `pairs::testing::{r, ranks}` (réexportés par `continuous::testing`).

- [ ] **Step 1: Tests** — ajouter dans le module `tests` de `games/pairs/src/deck.rs`, avant sa dernière accolade :

```rust
    #[test]
    fn pair_in_finds_the_highest_pair() {
        assert_eq!(pair_in(&ranks(&[3, 7, 3, 9, 9])), Rank::new(9));
        assert_eq!(pair_in(&ranks(&[3, 7, 9])), None);
        assert_eq!(pair_in(&[]), None);
    }

    #[test]
    fn face_down_discards_stay_unlocated() {
        let mut s = Supply::from_parts(ranks(&[2]), Vec::new(), Vec::new(), 0, 0);
        s.discard_unseen(ranks(&[7, 7]));
        assert!(s.seen().is_empty());
        assert_eq!(s.discard_len(), 2);
        assert_eq!(s.unlocated()[6], 2);
    }

    #[test]
    fn redealing_with_hands_keeps_sizes_and_cards() {
        let before = Supply::from_parts(ranks(&[1, 2, 3, 4]), ranks(&[9]), ranks(&[10, 10, 5]), 0, 0);
        let mut moved = false;
        for seed in 0..20 {
            let mut s = before.clone();
            let mut a = ranks(&[6, 7]);
            let mut b = ranks(&[8]);
            s.redeal_hidden_with(
                &ranks(&[10, 10]),
                &mut [&mut a, &mut b],
                &mut SmallRng::seed_from_u64(seed),
            );
            assert_eq!((a.len(), b.len()), (2, 1));
            assert_eq!(s.deck_len(), 4);
            assert_eq!(s.discard_len(), 4);
            assert_eq!(s.seen(), before.seen());
            // The two 10s the observer saw are still face down in the discards.
            assert_eq!(s.unseen.iter().filter(|c| c.value() == 10).count(), 2);
            assert!(!a.iter().chain(&b).any(|c| c.value() == 10));
            let mut all: Vec<Rank> = s.deck.iter().chain(&s.unseen).chain(&a).chain(&b).copied().collect();
            all.sort_unstable();
            assert_eq!(all, ranks(&[1, 2, 3, 4, 5, 6, 7, 8, 10, 10]));
            moved |= a != ranks(&[6, 7]);
        }
        assert!(moved, "the hands never changed");
    }
```

Run: `cargo test -p pairs --release --lib deck` — Expected: erreurs de compilation (`pair_in`, `discard_unseen`, `redeal_hidden_with` introuvables).

- [ ] **Step 2: Implémenter**

`games/pairs/src/testing.rs` :

```rust
//! Shorthands for tests (of this crate and of its users).

use crate::deck::Rank;

pub fn r(value: u8) -> Rank {
    Rank::new(value).expect("a rank from 1 to 10")
}

pub fn ranks(values: &[u8]) -> Vec<Rank> {
    values.iter().map(|&v| r(v)).collect()
}
```

`games/pairs/src/lib.rs` : ajouter `pub mod testing;` après `pub mod deck;`.

`games/pairs/src/continuous/testing.rs` : supprimer les deux fonctions `r` et `ranks` et les remplacer par `pub use crate::testing::{r, ranks};` (retirer `Rank` de l'import `crate::deck` s'il n'est plus utilisé).

`games/pairs/src/deck.rs` — après `count_by_rank` :

```rust
/// The highest rank present at least twice.
pub fn pair_in(cards: &[Rank]) -> Option<Rank> {
    let counts = count_by_rank(cards);
    Rank::all().filter(|r| counts[r.index()] >= 2).max()
}
```

Dans `impl Supply`, après `discard` :

```rust
    /// Face down: nobody at the table has seen these cards (a player who
    /// has is the game's business, see `redeal_hidden_with`).
    pub fn discard_unseen(&mut self, cards: impl IntoIterator<Item = Rank>) {
        self.unseen.extend(cards);
    }
```

Remplacer `redeal_hidden` par :

```rust
    /// For determinization in a game without hidden hands.
    pub fn redeal_hidden<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        self.redeal_hidden_with(&[], &mut [], rng);
    }

    /// For determinization: everything an observer cannot place is redrawn —
    /// the order of the draw pile, the face-down discards, and the `hands`
    /// given (the other players'), each keeping its size. `keep` lists
    /// face-down discards the observer has seen: they stay where they are.
    /// The seed of future shuffles is redrawn too.
    pub fn redeal_hidden_with<R: Rng + ?Sized>(
        &mut self,
        keep: &[Rank],
        hands: &mut [&mut Vec<Rank>],
        rng: &mut R,
    ) {
        let mut kept = Vec::new();
        for &card in keep {
            if let Some(at) = self.unseen.iter().position(|&c| c == card) {
                kept.push(self.unseen.swap_remove(at));
            }
        }
        let face_down = self.unseen.len();
        self.deck.append(&mut self.unseen);
        let sizes: Vec<usize> = hands.iter().map(|hand| hand.len()).collect();
        for hand in hands.iter_mut() {
            self.deck.append(hand);
        }
        self.deck.shuffle(rng);
        for (hand, size) in hands.iter_mut().zip(sizes) {
            let at = self.deck.len() - size;
            hand.extend(self.deck.drain(at..));
        }
        let at = self.deck.len() - face_down;
        self.unseen.extend(self.deck.drain(at..));
        self.unseen.append(&mut kept);
        self.seed = rng.next_u64();
    }
```

- [ ] **Step 3: Vérifier** — Run: `cargo test -p pairs --release` — Expected: tout passe (les tests du plan 1 inchangés, plus 3).

- [ ] **Step 4: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
git add games/pairs && git commit -m "pairs: hidden hands in the shared supply (pair_in, face-down discards, redeal with hands)"
```

---

### Task 2: Règles de L'Auberge des Pirates

**Files:**
- Create: `games/pairs/src/auberge/mod.rs`, `games/pairs/src/auberge/view.rs`, `games/pairs/src/auberge/testing.rs`, `games/pairs/tests/auberge_rules.rs`
- Modify: `games/pairs/src/lib.rs`, `games/pairs/RULES.md`

**Interfaces:**
- Consumes : `deck::{full_deck, has_rank, pair_in, Rank, Supply}`, `testing::{r, ranks}`.
- Produces (dans `pairs::auberge`) : `Auberge` (`Game`), `Config { rounds: Option<u8> }`, `HAND`, `DEFAULT_ROUNDS`, `MAX_ROUNDS`, `Action { Place{card}, Draw, Attack{target,card}, Dare{target}, Accuse{target} }`, `Phase { Setup, Play, Over }`, `Event`, `State` (champs publics ; `alive()`, `own_pair(seat)`, `unlocated_for(Option<Seat>)`, `pair_chance(Option<Seat>, Seat) -> f64`, `dare_risk(Option<Seat>, Seat) -> f64`, `draw_risk(Seat) -> f64`), `View`, `SeatView`, `testing::{table, setup}`. `pairs::Auberge` réexporté.

- [ ] **Step 1: `RULES.md`** — ajouter à la fin de `games/pairs/RULES.md` :

```markdown

## L'Auberge des Pirates (`pairs-auberge`, 2 à 8 joueurs)

Source : **[A]** la règle de *Pairs – L'Auberge des Pirates* (Origames,
2017), adaptation de Regent (James Ernest et Paul Peterson).

### But [A]
Être le dernier joueur en jeu. On est éliminé si on fait une paire en
défiant, si on est accusé à raison d'avoir une paire, ou si on accuse à tort.

### Mise en place [A]
- Chaque joueur reçoit 3 cartes en main.
- Tous choisissent en même temps une carte de leur main et la posent visible
  devant eux. La plus petite commence.

### Tour [A]
Une action au choix :
- **Attaquer** : jouer une carte de sa main devant un autre joueur, sans
  faire de paire avec les cartes qu'il a devant lui. C'est à lui de jouer.
- **Défier** : poser la première carte de la pioche, visible, devant un
  autre joueur. Sans paire avec ses cartes visibles : c'est à lui de jouer.
  Avec une paire : on est éliminé, la carte est défaussée, et c'est au joueur
  à sa gauche de jouer.
- **Accuser** un joueur : il montre sa main à l'accusateur seul. S'il a une
  paire entre sa main et les cartes devant lui (ou dans sa main), il est
  éliminé et c'est au joueur à gauche de l'accusateur. Sinon l'accusateur
  est éliminé et c'est à l'accusé de jouer.
- **Piocher** : prendre une carte en main sans la montrer. Une paire ainsi
  faite ne fait pas perdre tant qu'on n'est pas accusé. C'est au joueur à
  gauche.

### Choix sur les points non tranchés
1. ❓ Égalité pour commencer : tirage au sort entre les joueurs à égalité
   (la règle dit « le plus âgé »).
2. ❓ Une partie se joue en plusieurs manches (5 par défaut). Le vainqueur
   d'une manche marque 1 point ; classement final aux points, égalités
   conservées.
3. ❓ Un joueur éliminé défausse ses cartes visibles face visible et sa main
   face cachée.
4. ❓ On ne peut pas accuser un joueur qui n'a aucune carte en main : il ne
   peut pas avoir de paire.
5. ❓ Pioche vide : la défausse est remélangée. Si pioche et défausse sont
   vides, piocher et défier ne sont pas proposés ; si le joueur au tour n'a
   alors aucune action, la manche s'arrête sans vainqueur.
6. ❓ Le résultat d'une accusation est connu de tous ; la main montrée et le
   rang de la paire, de l'accusateur seul.
7. ❓ « À gauche » saute les joueurs éliminés.
```

- [ ] **Step 2: Aides de test** — `games/pairs/src/auberge/testing.rs` :

```rust
//! Hand-built tables for tests.

use ludo_core::Seat;

use super::{Phase, State, DEFAULT_ROUNDS};
use crate::deck::Supply;
use crate::testing::ranks;

fn supply(draws: &[u8]) -> Supply {
    let mut deck = ranks(draws);
    deck.reverse();
    Supply::from_parts(deck, Vec::new(), Vec::new(), 0, 0)
}

fn blank(hands: &[&[u8]], draws: &[u8]) -> State {
    let n = hands.len();
    State {
        players: n as u8,
        rounds: DEFAULT_ROUNDS,
        round: 1,
        supply: supply(draws),
        seed: 0,
        hands: hands.iter().map(|h| ranks(h)).collect(),
        fronts: vec![Vec::new(); n],
        placed: vec![None; n],
        out: vec![false; n],
        known: vec![Vec::new(); n],
        phase: Phase::Setup,
        turn: None,
        wins: vec![0; n],
        events: Vec::new(),
    }
}

/// Round 1 before anyone has placed a card; `draws` are the next cards of
/// the draw pile, in order.
pub fn setup(hands: &[&[u8]], draws: &[u8]) -> State {
    blank(hands, draws)
}

/// Round 1 in play: these hands and face-up cards, seat 0 to move.
pub fn table(hands: &[&[u8]], fronts: &[&[u8]], draws: &[u8]) -> State {
    let mut s = blank(hands, draws);
    s.fronts = fronts.iter().map(|f| ranks(f)).collect();
    s.phase = Phase::Play;
    s.turn = Some(Seat(0));
    s
}
```

- [ ] **Step 3: Tests de règles** — `games/pairs/tests/auberge_rules.rs` :

```rust
use ludo_core::{Game, Outcome, Pending, Seat, Viewer};
use pairs::auberge::testing::{setup, table};
use pairs::auberge::{Action, Auberge, Config, Event, Phase, State};
use pairs::deck::{Supply, DECK_SIZE};
use pairs::testing::{r, ranks};
use serde_json::json;

fn act(s: &mut State, seat: u8, action: Action) {
    Auberge::apply(s, Seat(seat), &action).unwrap_or_else(|e| panic!("{action:?} : {e}"));
}

fn place(card: u8) -> Action {
    Action::Place { card: r(card) }
}

fn attack(target: u8, card: u8) -> Action {
    Action::Attack {
        target: Seat(target),
        card: r(card),
    }
}

fn dare(target: u8) -> Action {
    Action::Dare {
        target: Seat(target),
    }
}

fn accuse(target: u8) -> Action {
    Action::Accuse {
        target: Seat(target),
    }
}

fn refused(s: &State, seat: u8, action: Action) {
    let mut copy = s.clone();
    assert!(
        Auberge::apply(&mut copy, Seat(seat), &action).is_err(),
        "{action:?} accepted"
    );
    assert_eq!(&copy, s);
    assert!(!Auberge::legal_actions(s, Seat(seat)).contains(&action));
}

#[test]
fn a_new_game_deals_three_cards_and_waits_for_everyone() {
    for players in 2..=8u8 {
        for seed in 0..10 {
            let s = Auberge::new(&Config::default(), players, seed);
            assert_eq!(s.phase, Phase::Setup);
            assert!(s.hands.iter().all(|h| h.len() == 3));
            assert_eq!(s.supply.deck_len(), DECK_SIZE - 3 * usize::from(players));
            assert_eq!(Auberge::pending(&s).0.len(), usize::from(players));
            assert_eq!((s.round, s.rounds), (1, 5));
        }
    }
}

#[test]
fn rounds_come_from_the_config_within_bounds() {
    let rounds = |r| Auberge::new(&Config { rounds: r }, 2, 1).rounds;
    assert_eq!(rounds(None), 5);
    assert_eq!(rounds(Some(0)), 5);
    assert_eq!(rounds(Some(3)), 3);
    assert_eq!(rounds(Some(255)), 15);
}

#[test]
fn a_placed_card_stays_secret_until_everyone_has_placed() {
    let mut s = setup(&[&[5, 6, 7], &[2, 8, 9], &[4, 4, 10]], &[1]);
    act(&mut s, 0, place(5));
    assert_eq!(Auberge::pending(&s), Pending(vec![Seat(1), Seat(2)]));
    let other = Auberge::view(&s, Viewer::Seat(Seat(1)));
    assert!(other.seats[0].front.is_empty());
    assert_eq!(other.seats[0].hand_size, 3);
    assert_eq!(other.placed, None);
    assert_eq!(Auberge::view(&s, Viewer::Seat(Seat(0))).placed, Some(r(5)));
    // Nobody learns who has already chosen.
    assert_eq!(
        Auberge::visible_pending(&s, Viewer::Seat(Seat(0))),
        Pending::none()
    );
    assert_eq!(
        Auberge::visible_pending(&s, Viewer::Seat(Seat(1))),
        Pending::one(Seat(1))
    );
    assert_eq!(
        Auberge::visible_pending(&s, Viewer::Spectator),
        Pending::none()
    );
    refused(&s, 0, place(6));
}

#[test]
fn the_lowest_placed_card_starts() {
    let mut s = setup(&[&[5, 6, 7], &[2, 8, 9], &[4, 4, 10]], &[1]);
    act(&mut s, 0, place(5));
    act(&mut s, 2, place(4));
    act(&mut s, 1, place(2));
    assert_eq!(s.phase, Phase::Play);
    assert_eq!(s.turn, Some(Seat(1)));
    assert_eq!(s.fronts, vec![ranks(&[5]), ranks(&[2]), ranks(&[4])]);
    assert_eq!(s.hands, vec![ranks(&[6, 7]), ranks(&[8, 9]), ranks(&[4, 10])]);
    assert!(s.events.contains(&Event::Starts { seat: Seat(1) }));
    assert_eq!(Auberge::pending(&s), Pending::one(Seat(1)));
}

#[test]
fn a_tie_for_lowest_is_drawn_by_lot() {
    let mut starters = Vec::new();
    for seed in 0..40 {
        let mut s = setup(&[&[3, 6, 7], &[3, 8, 9], &[5, 5, 10]], &[1]);
        s.seed = seed;
        act(&mut s, 0, place(3));
        act(&mut s, 1, place(3));
        act(&mut s, 2, place(5));
        starters.push(s.turn.unwrap());
    }
    assert!(starters.contains(&Seat(0)) && starters.contains(&Seat(1)));
    assert!(!starters.contains(&Seat(2)));
}

#[test]
fn placing_a_card_you_do_not_hold_is_refused() {
    let s = setup(&[&[5, 6, 7], &[2, 8, 9]], &[1]);
    refused(&s, 0, place(9));
    refused(&s, 0, Action::Draw);
}

#[test]
fn drawing_is_secret_and_passes_left() {
    let mut s = table(&[&[5, 6], &[8]], &[&[2], &[3]], &[9]);
    act(&mut s, 0, Action::Draw);
    assert_eq!(s.hands[0], ranks(&[5, 6, 9]));
    assert_eq!(s.turn, Some(Seat(1)));
    assert_eq!(s.events, vec![Event::Drew { seat: Seat(0) }]);
}

#[test]
fn an_attack_gives_a_hand_card_and_the_turn_to_the_target() {
    let mut s = table(&[&[5, 6], &[8], &[7]], &[&[2], &[3], &[4]], &[9]);
    act(&mut s, 0, attack(2, 5));
    assert_eq!(s.hands[0], ranks(&[6]));
    assert_eq!(s.fronts[2], ranks(&[4, 5]));
    assert_eq!(s.turn, Some(Seat(2)));
}

#[test]
fn an_attack_may_not_make_a_visible_pair() {
    let s = table(&[&[5, 6], &[8]], &[&[2], &[5]], &[9]);
    refused(&s, 0, attack(1, 5));
    refused(&s, 0, attack(1, 9));
    assert!(Auberge::legal_actions(&s, Seat(0)).contains(&attack(1, 6)));
}

#[test]
fn a_dare_that_does_not_pair_gives_the_turn_to_the_target() {
    let mut s = table(&[&[5], &[8], &[7]], &[&[2], &[3], &[4]], &[9]);
    act(&mut s, 0, dare(2));
    assert_eq!(s.fronts[2], ranks(&[4, 9]));
    assert_eq!(s.turn, Some(Seat(2)));
    assert_eq!(
        s.events,
        vec![Event::Dared {
            seat: Seat(0),
            target: Seat(2),
            card: r(9),
            paired: false
        }]
    );
}

#[test]
fn a_dare_that_pairs_eliminates_the_darer() {
    let mut s = table(&[&[5], &[8], &[7]], &[&[2], &[3], &[4]], &[3]);
    act(&mut s, 0, dare(1));
    assert!(s.out[0]);
    assert_eq!(s.fronts[1], ranks(&[3]));
    assert_eq!(s.turn, Some(Seat(1)));
    // The dared card and the darer's face-up cards are seen; the hand is not.
    assert_eq!(s.supply.seen(), ranks(&[3, 2]).as_slice());
    assert_eq!(s.supply.discard_len(), 3);
    assert!(s.hands[0].is_empty() && s.fronts[0].is_empty());
    assert!(s.events.contains(&Event::Out { seat: Seat(0) }));
}

#[test]
fn a_right_accusation_eliminates_the_accused() {
    // A pair between the hand and the face-up cards.
    let mut s = table(&[&[5], &[8, 3], &[7]], &[&[2], &[3], &[4]], &[9]);
    act(&mut s, 0, accuse(1));
    assert!(s.out[1] && !s.out[0]);
    // Left of the accuser, skipping the eliminated player.
    assert_eq!(s.turn, Some(Seat(2)));
    assert_eq!(s.known[0], ranks(&[8, 3]));
    // A pair inside the hand counts too.
    let mut s = table(&[&[5], &[6, 6], &[7]], &[&[2], &[3], &[4]], &[9]);
    act(&mut s, 0, accuse(1));
    assert!(s.out[1]);
}

#[test]
fn a_wrong_accusation_eliminates_the_accuser() {
    let mut s = table(&[&[5], &[8, 9], &[7]], &[&[2], &[3], &[4]], &[9]);
    act(&mut s, 0, accuse(1));
    assert!(s.out[0] && !s.out[1]);
    assert_eq!(s.turn, Some(Seat(1)));
    assert_eq!(s.hands[1], ranks(&[8, 9]));
}

#[test]
fn the_shown_hand_reaches_only_the_accuser_and_the_accused() {
    let mut s = table(&[&[5], &[8, 9], &[7]], &[&[2], &[3], &[4]], &[9]);
    act(&mut s, 0, accuse(1));
    let shown = |viewer| {
        Auberge::view(&s, viewer)
            .events
            .into_iter()
            .find_map(|e| match e {
                Event::Accused { hand, guilty, .. } => Some((hand, guilty)),
                _ => None,
            })
            .expect("an accusation event")
    };
    assert_eq!(shown(Viewer::Seat(Seat(0))), (ranks(&[8, 9]), false));
    assert_eq!(shown(Viewer::Seat(Seat(1))), (ranks(&[8, 9]), false));
    assert_eq!(shown(Viewer::Seat(Seat(2))), (Vec::new(), false));
    assert_eq!(shown(Viewer::Spectator), (Vec::new(), false));
}

#[test]
fn a_player_without_hand_cannot_be_accused() {
    let s = table(&[&[5], &[], &[7]], &[&[2], &[3], &[4]], &[9]);
    refused(&s, 0, accuse(1));
    assert!(Auberge::legal_actions(&s, Seat(0)).contains(&accuse(2)));
}

#[test]
fn absurd_targets_are_refused() {
    let mut s = table(&[&[5], &[8], &[7]], &[&[2], &[3], &[4]], &[9]);
    refused(&s, 0, dare(9));
    refused(&s, 0, dare(0));
    refused(&s, 0, accuse(0));
    refused(&s, 0, attack(9, 5));
    refused(&s, 1, Action::Draw);
    refused(&s, 9, Action::Draw);
    s.out[2] = true;
    refused(&s, 0, dare(2));
    refused(&s, 0, accuse(2));
}

#[test]
fn the_last_player_standing_wins_the_round_and_a_new_one_is_dealt() {
    let mut s = table(&[&[5], &[6, 6]], &[&[2], &[3]], &[9]);
    act(&mut s, 0, accuse(1));
    assert_eq!(s.wins, vec![1, 0]);
    assert_eq!(s.round, 2);
    assert_eq!(s.phase, Phase::Setup);
    assert!(s.hands.iter().all(|h| h.len() == 3));
    assert!(s.fronts.iter().all(Vec::is_empty));
    assert_eq!(s.out, vec![false, false]);
    assert!(s.known.iter().all(Vec::is_empty));
    assert_eq!(s.supply.deck_len(), DECK_SIZE - 6);
    assert!(s.events.contains(&Event::RoundWon {
        seat: Some(Seat(0))
    }));
    assert_eq!(Auberge::pending(&s).0.len(), 2);
}

#[test]
fn the_game_ends_after_the_last_round() {
    let mut s = table(&[&[5], &[6, 6]], &[&[2], &[3]], &[9]);
    s.rounds = 1;
    act(&mut s, 0, accuse(1));
    assert_eq!(s.phase, Phase::Over);
    assert_eq!(Auberge::pending(&s), Pending::none());
    assert_eq!(Auberge::outcome(&s), Some(Outcome::Ranked(vec![0, 1])));
    refused(&s, 0, Action::Draw);
    refused(&s, 1, place(3));
}

#[test]
fn the_final_ranking_follows_the_rounds_won_and_keeps_ties() {
    let mut s = table(&[&[5], &[8], &[7], &[9]], &[&[], &[], &[], &[]], &[]);
    s.phase = Phase::Over;
    s.turn = None;
    s.wins = vec![2, 2, 0, 1];
    assert_eq!(Auberge::outcome(&s), Some(Outcome::Ranked(vec![0, 0, 3, 2])));
    s.phase = Phase::Play;
    s.turn = Some(Seat(0));
    assert_eq!(Auberge::outcome(&s), None);
}

#[test]
fn a_stuck_round_is_won_by_nobody() {
    // No cards to draw; after this attack seat 0 can neither attack (its 5
    // would pair seat 1's) nor accuse (seat 1 has no hand).
    let mut s = table(&[&[5], &[9]], &[&[], &[5]], &[]);
    s.turn = Some(Seat(1));
    act(&mut s, 1, attack(0, 9));
    assert_eq!(s.wins, vec![0, 0]);
    assert_eq!(s.round, 2);
    assert_eq!(s.phase, Phase::Setup);
    assert!(s.events.contains(&Event::RoundWon { seat: None }));
}

#[test]
fn without_cards_to_draw_only_attacks_and_accusations_remain() {
    let s = table(&[&[5], &[9]], &[&[], &[]], &[]);
    let legal = Auberge::legal_actions(&s, Seat(0));
    assert_eq!(legal, vec![attack(1, 5), accuse(1)]);
}

#[test]
fn a_reshuffle_forgets_what_was_seen() {
    let mut s = table(&[&[5], &[8], &[7]], &[&[], &[], &[]], &[]);
    s.supply = Supply::from_parts(Vec::new(), ranks(&[2]), ranks(&[4]), 0, 1);
    s.known[0] = ranks(&[4]);
    act(&mut s, 0, Action::Draw);
    assert_eq!(s.events[0], Event::Reshuffled);
    assert!(s.known[0].is_empty());
    assert_eq!(s.hands[0].len(), 2);
    assert_eq!(s.supply.deck_len(), 1);
}

#[test]
fn the_view_tells_a_player_about_their_own_pair_only() {
    let s = table(&[&[7, 3], &[8]], &[&[7], &[2]], &[9]);
    let mine = Auberge::view(&s, Viewer::Seat(Seat(0)));
    assert_eq!(mine.own_pair, Some(r(7)));
    assert_eq!(mine.hand, ranks(&[7, 3]));
    assert_eq!(mine.seats[0].pair_odds, 0);
    assert!(mine.seats[1].pair_odds > 0);
    let theirs = Auberge::view(&s, Viewer::Seat(Seat(1)));
    assert_eq!(theirs.own_pair, None);
    assert_eq!(theirs.hand, ranks(&[8]));
    assert_eq!(theirs.seats[0].hand_size, 2);
    let watcher = Auberge::view(&s, Viewer::Spectator);
    assert!(watcher.hand.is_empty());
    assert_eq!(watcher.own_pair, None);
}

#[test]
fn unlocated_counts_what_the_viewer_cannot_place() {
    let mut s = table(&[&[7, 3], &[8]], &[&[7], &[2]], &[9]);
    s.known[0] = ranks(&[10]);
    // Seat 0 places: both face-up cards, its hand, and the 10 it saw.
    assert_eq!(
        s.unlocated_for(Some(Seat(0))),
        [1, 1, 2, 4, 5, 6, 5, 8, 9, 9]
    );
    assert_eq!(s.unlocated_for(None), [1, 1, 3, 4, 5, 6, 6, 8, 9, 10]);
}

#[test]
fn the_json_the_web_relies_on() {
    assert_eq!(serde_json::to_value(Action::Draw).unwrap(), json!("Draw"));
    assert_eq!(
        serde_json::to_value(place(3)).unwrap(),
        json!({"Place": {"card": 3}})
    );
    assert_eq!(
        serde_json::to_value(attack(1, 7)).unwrap(),
        json!({"Attack": {"target": 1, "card": 7}})
    );
    assert_eq!(
        serde_json::to_value(dare(2)).unwrap(),
        json!({"Dare": {"target": 2}})
    );
    assert_eq!(
        serde_json::to_value(accuse(2)).unwrap(),
        json!({"Accuse": {"target": 2}})
    );
    assert_eq!(
        serde_json::to_value(Event::Accused {
            seat: Seat(0),
            target: Seat(1),
            guilty: true,
            hand: ranks(&[6, 6])
        })
        .unwrap(),
        json!({"type": "Accused", "seat": 0, "target": 1, "guilty": true, "hand": [6, 6]})
    );
    assert_eq!(
        serde_json::to_value(Event::RoundWon { seat: None }).unwrap(),
        json!({"type": "RoundWon", "seat": null})
    );
    let s = table(&[&[7, 3], &[8]], &[&[7], &[2]], &[9]);
    let view = serde_json::to_value(Auberge::view(&s, Viewer::Seat(Seat(0)))).unwrap();
    assert_eq!(view["phase"], json!("Play"));
    assert_eq!(view["ownPair"], json!(7));
    assert_eq!(view["seats"][1]["handSize"], json!(1));
    assert_eq!(view["seats"][1]["wins"], json!(0));
    assert!(view["seats"][1]["pairOdds"].is_number());
    assert_eq!(view["discardSeen"], json!([]));
    assert_eq!(view["rounds"], json!(5));
    for hidden in ["supply", "hands", "known", "seed"] {
        assert!(view.get(hidden).is_none(), "{hidden} leaked");
    }
}
```

Run: `cargo test -p pairs --release` — Expected: erreur de compilation (`could not find auberge in pairs`).

- [ ] **Step 4: Moteur**

`games/pairs/src/lib.rs` devient :

```rust
//! Games played with the Pairs deck. Rules and their sources: RULES.md.

pub mod auberge;
pub mod continuous;
pub mod deck;
pub mod testing;

pub use auberge::Auberge;
pub use continuous::ContinuousPairs;
```

`games/pairs/src/auberge/mod.rs` :

```rust
//! L'Auberge des Pirates: multiplayer Regent without chips (Origames
//! edition of a James Ernest and Paul Peterson game). Rules and their
//! sources: RULES.md.

pub mod testing;
mod view;

pub use view::{SeatView, View};

use ludo_core::{Game, GameMeta, Illegal, Outcome, Pending, Seat, Viewer};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::deck::{full_deck, has_rank, pair_in, Rank, Supply};

pub struct Auberge;

/// [A] Cards dealt to each player at the start of a round.
pub const HAND: usize = 3;
/// ❓2
pub const DEFAULT_ROUNDS: u8 = 5;
pub const MAX_ROUNDS: u8 = 15;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Rounds in a game, capped at `MAX_ROUNDS`. `None` (or 0): 5.
    pub rounds: Option<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Setup: the hand card to put face up.
    Place { card: Rank },
    /// Take a card in hand, unseen.
    Draw,
    /// Play a hand card face up in front of `target`.
    Attack { target: Seat, card: Rank },
    /// Put the top card of the draw pile face up in front of `target`.
    Dare { target: Seat },
    /// Claim `target` holds a pair.
    Accuse { target: Seat },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    /// Everyone chooses a card to place, at the same time.
    Setup,
    Play,
    Over,
}

/// What happened, for the board to show.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    /// Setup is over: `seat` had chosen `card`.
    Placed { seat: Seat, card: Rank },
    Starts { seat: Seat },
    Drew { seat: Seat },
    Attacked { seat: Seat, target: Seat, card: Rank },
    Dared {
        seat: Seat,
        target: Seat,
        card: Rank,
        /// The card paired `target`'s face-up cards: `seat` is out.
        paired: bool,
    },
    Accused {
        seat: Seat,
        target: Seat,
        guilty: bool,
        /// The hand shown. Views empty it for everyone but `seat` and `target`.
        hand: Vec<Rank>,
    },
    Out { seat: Seat },
    /// `None`: the round was stuck and nobody won it.
    RoundWon { seat: Option<Seat> },
    Reshuffled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub players: u8,
    pub rounds: u8,
    /// The current round, from 1.
    pub round: u8,
    pub supply: Supply,
    /// For each round's deal and tie-break. Never in a view.
    pub seed: u64,
    pub hands: Vec<Vec<Rank>>,
    /// Face-up cards in front of each seat. Never two of a rank.
    pub fronts: Vec<Vec<Rank>>,
    /// Setup: the card each seat has chosen, still in its hand.
    pub placed: Vec<Option<Rank>>,
    /// Eliminated from the current round.
    pub out: Vec<bool>,
    /// Face-down discards each seat has seen (a hand shown by a right
    /// accusation, its own discarded hand). Forgotten at a reshuffle.
    pub known: Vec<Vec<Rank>>,
    pub phase: Phase,
    /// Seat to move during play.
    pub turn: Option<Seat>,
    /// Rounds won.
    pub wins: Vec<u8>,
    /// Since the previous decision in play, or since the last reveal.
    pub events: Vec<Event>,
}

/// One independent stream per round.
fn mix(seed: u64, round: u8) -> u64 {
    seed ^ u64::from(round).wrapping_mul(0xD6E8_FEB8_6659_FD93)
}

const TIE_STREAM: u64 = 0x5851_F42D_4C95_7F2D;

impl State {
    fn seats(&self) -> impl Iterator<Item = Seat> {
        (0..self.players).map(Seat)
    }

    /// Seats still in the round, in seat order.
    pub fn alive(&self) -> Vec<Seat> {
        self.seats().filter(|s| !self.out[s.index()]).collect()
    }

    /// ❓7: the next seat still in the round.
    fn left_of(&self, seat: Seat) -> Seat {
        (1..=self.players)
            .map(|k| Seat((seat.0 + k) % self.players))
            .find(|s| !self.out[s.index()])
            .unwrap_or(seat)
    }

    /// The pair `seat` holds, between its hand and its face-up cards.
    pub fn own_pair(&self, seat: Seat) -> Option<Rank> {
        let mut cards = self.hands[seat.index()].clone();
        cards.extend(&self.fronts[seat.index()]);
        pair_in(&cards)
    }

    /// Per rank, the cards `observer` cannot place: not face up, not in the
    /// seen discards, not in its hand, not among the discards it has seen.
    pub fn unlocated_for(&self, observer: Option<Seat>) -> [u8; 10] {
        let mut counts = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let own = observer
            .into_iter()
            .flat_map(|o| self.hands[o.index()].iter().chain(&self.known[o.index()]));
        for card in self
            .fronts
            .iter()
            .flatten()
            .chain(self.supply.seen())
            .chain(own)
        {
            counts[card.index()] = counts[card.index()].saturating_sub(1);
        }
        counts
    }

    /// The share of the cards `observer` cannot place that would pair
    /// `cards`.
    fn share(&self, observer: Option<Seat>, cards: &[Rank]) -> f64 {
        let pool = self.unlocated_for(observer);
        let total: u32 = pool.iter().map(|&c| u32::from(c)).sum();
        if total == 0 {
            return 0.0;
        }
        let mut ranks = cards.to_vec();
        ranks.sort_unstable();
        ranks.dedup();
        let matching: u32 = ranks.iter().map(|c| u32::from(pool[c.index()])).sum();
        f64::from(matching) / f64::from(total)
    }

    /// Estimated chance that `target` holds a pair, from what `observer`
    /// sees: its hidden cards taken as drawn at random from the cards
    /// `observer` cannot place.
    pub fn pair_chance(&self, observer: Option<Seat>, target: Seat) -> f64 {
        let hidden = self.hands[target.index()].len() as f64;
        if hidden == 0.0 {
            return 0.0;
        }
        let pool = self.unlocated_for(observer);
        let total: f64 = pool.iter().map(|&c| f64::from(c)).sum();
        if total < 2.0 {
            return 0.0;
        }
        let with_front = self.share(observer, &self.fronts[target.index()]);
        let alike: f64 = pool
            .iter()
            .map(|&c| f64::from(c) * (f64::from(c) - 1.0).max(0.0))
            .sum::<f64>()
            / (total * (total - 1.0));
        let clean = (1.0 - with_front).powf(hidden) * (1.0 - alike).powf(hidden * (hidden - 1.0) / 2.0);
        1.0 - clean
    }

    /// Chance that a dare on `target` pairs its face-up cards, eliminating
    /// the darer.
    pub fn dare_risk(&self, observer: Option<Seat>, target: Seat) -> f64 {
        self.share(observer, &self.fronts[target.index()])
    }

    /// Chance that the card `seat` draws gives it a pair.
    pub fn draw_risk(&self, seat: Seat) -> f64 {
        let mut cards = self.hands[seat.index()].clone();
        cards.extend(&self.fronts[seat.index()]);
        self.share(Some(seat), &cards)
    }

    fn draw(&mut self) -> Option<Rank> {
        let before = self.supply.shuffles();
        let card = self.supply.draw();
        if self.supply.shuffles() != before {
            // The discards are in the draw pile again: nobody knows where
            // the cards they saw went.
            self.events.push(Event::Reshuffled);
            self.known.iter_mut().for_each(Vec::clear);
        }
        card
    }

    /// [A] Three cards each, then everyone places one. Leaves `events` alone.
    fn start_round(&mut self) {
        let n = usize::from(self.players);
        self.round += 1;
        self.supply = Supply::new(full_deck(), 0, mix(self.seed, self.round));
        self.hands = vec![Vec::new(); n];
        for _ in 0..HAND {
            for i in 0..n {
                if let Some(card) = self.supply.draw() {
                    self.hands[i].push(card);
                }
            }
        }
        self.fronts = vec![Vec::new(); n];
        self.placed = vec![None; n];
        self.out = vec![false; n];
        self.known = vec![Vec::new(); n];
        self.phase = Phase::Setup;
        self.turn = None;
    }

    /// [A] Everyone has chosen: the cards are turned up and the lowest starts.
    fn reveal(&mut self) {
        self.events.clear();
        let mut shown: Vec<(Seat, Rank)> = Vec::new();
        for seat in self.seats().collect::<Vec<_>>() {
            let i = seat.index();
            let Some(card) = self.placed[i].take() else {
                continue;
            };
            let Some(at) = self.hands[i].iter().position(|&c| c == card) else {
                continue;
            };
            self.hands[i].remove(at);
            self.fronts[i].push(card);
            self.events.push(Event::Placed { seat, card });
            shown.push((seat, card));
        }
        let low = shown.iter().map(|&(_, card)| card).min();
        let tied: Vec<Seat> = shown
            .iter()
            .filter(|&&(_, card)| Some(card) == low)
            .map(|&(seat, _)| seat)
            .collect();
        // ❓1: a tie is drawn by lot.
        let first = match tied.as_slice() {
            [] => Seat(0),
            [only] => *only,
            _ => {
                let mut rng = ChaCha8Rng::seed_from_u64(mix(self.seed, self.round) ^ TIE_STREAM);
                tied[rng.gen_range(0..tied.len())]
            }
        };
        self.events.push(Event::Starts { seat: first });
        self.phase = Phase::Play;
        self.pass_turn(first);
    }

    /// ❓3: face-up cards are discarded face up, the hand face down.
    fn eliminate(&mut self, seat: Seat) {
        let i = seat.index();
        self.out[i] = true;
        let front = std::mem::take(&mut self.fronts[i]);
        self.supply.discard(front);
        let hand = std::mem::take(&mut self.hands[i]);
        self.known[i].extend(&hand);
        self.supply.discard_unseen(hand);
        self.events.push(Event::Out { seat });
    }

    /// `next` moves, unless the round is over.
    fn pass_turn(&mut self, next: Seat) {
        if let [winner] = self.alive().as_slice() {
            // [A] The last player standing wins.
            return self.end_round(Some(*winner));
        }
        self.turn = Some(next);
        if self.play_actions(next).is_empty() {
            // ❓5: nothing to draw and nothing to play.
            self.end_round(None);
        }
    }

    fn end_round(&mut self, winner: Option<Seat>) {
        if let Some(seat) = winner {
            self.wins[seat.index()] += 1;
        }
        self.events.push(Event::RoundWon { seat: winner });
        if self.round >= self.rounds {
            self.phase = Phase::Over;
            self.turn = None;
        } else {
            self.start_round();
        }
    }

    /// What `seat` may do on its turn.
    fn play_actions(&self, seat: Seat) -> Vec<Action> {
        let others: Vec<Seat> = self.alive().into_iter().filter(|&s| s != seat).collect();
        let mut actions = Vec::new();
        // ❓5: with nothing to draw, drawing and daring are off.
        if self.supply.can_draw() {
            actions.push(Action::Draw);
            actions.extend(others.iter().map(|&target| Action::Dare { target }));
        }
        let mut cards = self.hands[seat.index()].clone();
        cards.sort_unstable();
        cards.dedup();
        for &target in &others {
            for &card in &cards {
                // [A] An attack may not make a visible pair.
                if !has_rank(&self.fronts[target.index()], card) {
                    actions.push(Action::Attack { target, card });
                }
            }
        }
        // ❓4: a player without hand cards cannot hold a pair.
        actions.extend(
            others
                .iter()
                .filter(|t| !self.hands[t.index()].is_empty())
                .map(|&target| Action::Accuse { target }),
        );
        actions
    }

    fn play(&mut self, seat: Seat, action: &Action) {
        self.events.clear();
        let next = match *action {
            // Not a play action: `apply` routes it elsewhere.
            Action::Place { .. } => seat,
            Action::Draw => {
                if let Some(card) = self.draw() {
                    self.hands[seat.index()].push(card);
                }
                self.events.push(Event::Drew { seat });
                self.left_of(seat)
            }
            Action::Attack { target, card } => {
                let hand = &mut self.hands[seat.index()];
                if let Some(at) = hand.iter().position(|&c| c == card) {
                    hand.remove(at);
                    self.fronts[target.index()].push(card);
                }
                self.events.push(Event::Attacked { seat, target, card });
                target
            }
            Action::Dare { target } => match self.draw() {
                Some(card) => {
                    let paired = has_rank(&self.fronts[target.index()], card);
                    self.events.push(Event::Dared {
                        seat,
                        target,
                        card,
                        paired,
                    });
                    if paired {
                        // [A] The darer is out and the card is discarded.
                        self.supply.discard([card]);
                        self.eliminate(seat);
                        self.left_of(seat)
                    } else {
                        self.fronts[target.index()].push(card);
                        target
                    }
                }
                None => seat,
            },
            Action::Accuse { target } => {
                let hand = self.hands[target.index()].clone();
                let guilty = self.own_pair(target).is_some();
                self.events.push(Event::Accused {
                    seat,
                    target,
                    guilty,
                    hand: hand.clone(),
                });
                if guilty {
                    // The accuser has seen the hand that now goes face down.
                    self.known[seat.index()].extend(&hand);
                    self.eliminate(target);
                    self.left_of(seat)
                } else {
                    self.eliminate(seat);
                    target
                }
            }
        };
        self.pass_turn(next);
    }
}

impl Game for Auberge {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "pairs-auberge",
            name: "L'Auberge des Pirates",
            min_players: 2,
            max_players: 8,
            hidden_info: true,
            clock: false,
        }
    }

    fn new(config: &Config, players: u8, seed: u64) -> State {
        let n = usize::from(players);
        let rounds = config
            .rounds
            .filter(|&r| r > 0)
            .map_or(DEFAULT_ROUNDS, |r| r.min(MAX_ROUNDS));
        let mut s = State {
            players,
            rounds,
            round: 0,
            supply: Supply::new(Vec::new(), 0, seed),
            seed,
            hands: Vec::new(),
            fronts: Vec::new(),
            placed: Vec::new(),
            out: Vec::new(),
            known: Vec::new(),
            phase: Phase::Setup,
            turn: None,
            wins: vec![0; n],
            events: Vec::new(),
        };
        s.start_round();
        s
    }

    fn pending(s: &State) -> Pending {
        match s.phase {
            Phase::Setup => Pending(s.seats().filter(|x| s.placed[x.index()].is_none()).collect()),
            Phase::Play => s.turn.map_or_else(Pending::none, Pending::one),
            Phase::Over => Pending::none(),
        }
    }

    /// During setup nobody learns who has already chosen: a seat only sees
    /// itself pending.
    fn visible_pending(s: &State, viewer: Viewer) -> Pending {
        match (s.phase, viewer) {
            (Phase::Setup, Viewer::Seat(seat)) => match s.placed.get(seat.index()) {
                Some(None) => Pending::one(seat),
                _ => Pending::none(),
            },
            (Phase::Setup, Viewer::Spectator) => Pending::none(),
            _ => Self::pending(s),
        }
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        if seat.index() >= usize::from(s.players) {
            return Vec::new();
        }
        match s.phase {
            Phase::Setup if s.placed[seat.index()].is_none() => {
                let mut cards = s.hands[seat.index()].clone();
                cards.sort_unstable();
                cards.dedup();
                cards.into_iter().map(|card| Action::Place { card }).collect()
            }
            Phase::Play if s.turn == Some(seat) => s.play_actions(seat),
            _ => Vec::new(),
        }
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        if s.phase == Phase::Over {
            return Err(Illegal::new("La partie est terminée."));
        }
        let legal = Self::legal_actions(s, seat);
        if legal.is_empty() {
            return Err(Illegal::new(match s.phase {
                Phase::Setup => "Tu as déjà posé ta carte.",
                _ => "Ce n'est pas ton tour.",
            }));
        }
        if !legal.contains(action) {
            return Err(Illegal::new("Ce coup n'est pas permis."));
        }
        match *action {
            Action::Place { card } => {
                s.placed[seat.index()] = Some(card);
                if s.placed.iter().all(Option::is_some) {
                    s.reveal();
                }
            }
            _ => s.play(seat, action),
        }
        Ok(())
    }

    fn view(s: &State, viewer: Viewer) -> View {
        View::of(s, viewer)
    }

    fn outcome(s: &State) -> Option<Outcome> {
        if s.phase != Phase::Over {
            return None;
        }
        // ❓2: ranked by rounds won; equal scores stay tied.
        let ranks = s
            .wins
            .iter()
            .map(|&mine| s.wins.iter().filter(|&&theirs| theirs > mine).count() as u8)
            .collect();
        Some(Outcome::Ranked(ranks))
    }
}
```

`games/pairs/src/auberge/view.rs` :

```rust
use ludo_core::{Seat, Viewer};
use serde::Serialize;

use super::{Event, Phase, State};
use crate::deck::Rank;

/// What one seat sees: its own hand, everyone's face-up cards and hand
/// sizes. Never another hand, the draw pile, nor a card another seat has
/// chosen during setup.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub round: u8,
    pub rounds: u8,
    pub phase: Phase,
    pub seats: Vec<SeatView>,
    /// The viewer's hand (empty for a spectator).
    pub hand: Vec<Rank>,
    /// Setup: the card the viewer has chosen, not yet revealed.
    pub placed: Option<Rank>,
    /// The pair the viewer holds, if any: an accusation would eliminate it.
    pub own_pair: Option<Rank>,
    /// Seat to move during play.
    pub turn: Option<Seat>,
    pub deck: usize,
    /// Cards in the discard pile, face down ones included.
    pub discard: usize,
    /// The discards everyone has seen.
    pub discard_seen: Vec<Rank>,
    /// Per rank (index = rank − 1), the cards the viewer cannot place.
    pub unlocated: Vec<u8>,
    pub events: Vec<Event>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatView {
    pub front: Vec<Rank>,
    pub hand_size: usize,
    /// Eliminated from the current round.
    pub out: bool,
    /// Rounds won.
    pub wins: u8,
    /// The viewer's estimate, per thousand, that this seat holds a pair
    /// (0 for the viewer itself and for an eliminated seat).
    pub pair_odds: u16,
}

impl View {
    pub fn of(s: &State, viewer: Viewer) -> Self {
        let me = match viewer {
            Viewer::Seat(seat) if seat.index() < usize::from(s.players) => Some(seat),
            _ => None,
        };
        let seats = (0..s.players)
            .map(Seat)
            .map(|seat| {
                let i = seat.index();
                let hidden = Some(seat) == me || s.out[i];
                SeatView {
                    front: s.fronts[i].clone(),
                    hand_size: s.hands[i].len(),
                    out: s.out[i],
                    wins: s.wins[i],
                    pair_odds: if hidden {
                        0
                    } else {
                        (s.pair_chance(me, seat) * 1000.0).round() as u16
                    },
                }
            })
            .collect();
        let events = s
            .events
            .iter()
            .cloned()
            .map(|event| match event {
                // ❓6: the hand is shown to the accuser alone (the accused
                // knows its own).
                Event::Accused {
                    seat,
                    target,
                    guilty,
                    hand,
                } => Event::Accused {
                    seat,
                    target,
                    guilty,
                    hand: if me == Some(seat) || me == Some(target) {
                        hand
                    } else {
                        Vec::new()
                    },
                },
                other => other,
            })
            .collect();
        Self {
            round: s.round,
            rounds: s.rounds,
            phase: s.phase,
            seats,
            hand: me.map(|m| s.hands[m.index()].clone()).unwrap_or_default(),
            placed: me.and_then(|m| s.placed[m.index()]),
            own_pair: me.and_then(|m| s.own_pair(m)),
            turn: s.turn,
            deck: s.supply.deck_len(),
            discard: s.supply.discard_len(),
            discard_seen: s.supply.seen().to_vec(),
            unlocated: s.unlocated_for(me).to_vec(),
            events,
        }
    }
}
```

- [ ] **Step 5: Vérifier** — Run: `cargo test -p pairs --release` — Expected: les 26 tests de `auberge_rules` passent avec tout le reste.

- [ ] **Step 6: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
git add games/pairs && git commit -m "pairs: L'Auberge des Pirates rules, view and RULES.md"
```

---

### Task 3: Déterminisation et conformance

**Files:**
- Create: `games/pairs/src/auberge/determinize.rs`
- Modify: `games/pairs/src/auberge/mod.rs`, `games/pairs/tests/conformance.rs`

**Interfaces:**
- Consumes : `Supply::redeal_hidden_with`, `State`.
- Produces : `impl Determinize for Auberge`.

- [ ] **Step 1: Tests et implémentation**

Ajouter à `games/pairs/tests/conformance.rs` :

```rust
#[test]
fn auberge_conforms_at_every_table_size() {
    use pairs::auberge::{Auberge, Config};
    for players in 2..=8u8 {
        check::<Auberge>(&Config::default(), players, 40, 100 + u64::from(players));
    }
    check::<Auberge>(&Config { rounds: Some(1) }, 3, 100, 7);
    check::<Auberge>(&Config { rounds: Some(255) }, 2, 10, 8);
}
```

`games/pairs/src/auberge/determinize.rs` :

```rust
use ludo_core::{Determinize, Seat};
use rand::Rng;

use super::{Auberge, Phase, State};
use crate::deck::Rank;

impl Determinize for Auberge {
    /// The other hands, the draw pile and the face-down discards are redrawn
    /// among the cards `observer` cannot place; the face-down discards it
    /// has seen stay. During setup the other seats' choices are forgotten:
    /// they will choose again from their new hands.
    fn determinize<R: Rng + ?Sized>(s: &State, observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        let o = observer.index();
        let keep: Vec<Rank> = d.known.get(o).cloned().unwrap_or_default();
        {
            let mut others: Vec<&mut Vec<Rank>> = d
                .hands
                .iter_mut()
                .enumerate()
                .filter(|(i, _)| *i != o)
                .map(|(_, hand)| hand)
                .collect();
            d.supply.redeal_hidden_with(&keep, &mut others, rng);
        }
        if d.phase == Phase::Setup {
            for (i, choice) in d.placed.iter_mut().enumerate() {
                if i != o {
                    *choice = None;
                }
            }
        }
        d.seed = rng.next_u64();
        d
    }
}

#[cfg(test)]
mod tests {
    use ludo_core::{Determinize, Game, Seat, Viewer};
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    use crate::auberge::testing::{setup, table};
    use crate::auberge::{Action, Auberge, Config};
    use crate::testing::{r, ranks};

    #[test]
    fn other_hands_change_but_not_what_the_observer_sees() {
        let s = Auberge::new(&Config::default(), 4, 3);
        let view = Auberge::view(&s, Viewer::Seat(Seat(1)));
        let mut hands = Vec::new();
        for seed in 0..20 {
            let d = Auberge::determinize(&s, Seat(1), &mut SmallRng::seed_from_u64(seed));
            assert_eq!(Auberge::view(&d, Viewer::Seat(Seat(1))), view);
            assert_eq!(d.hands[1], s.hands[1]);
            assert!(d.hands.iter().all(|h| h.len() == 3));
            hands.push(d.hands[0].clone());
        }
        hands.dedup();
        assert!(hands.len() > 1, "seat 0 always got the same hand");
    }

    #[test]
    fn a_hand_the_observer_saw_discarded_is_not_dealt_again() {
        // Seat 0 rightly accused seat 1 and saw its two 10s go face down.
        let mut s = table(&[&[5], &[10, 10], &[7]], &[&[2], &[3], &[4]], &[1, 6, 8, 9]);
        Auberge::apply(&mut s, Seat(0), &Action::Accuse { target: Seat(1) }).unwrap();
        assert_eq!(s.known[0], ranks(&[10, 10]));
        for seed in 0..30 {
            let d = Auberge::determinize(&s, Seat(0), &mut SmallRng::seed_from_u64(seed));
            assert!(!d.hands[2].contains(&r(10)), "seed {seed}");
            assert_eq!(d.supply.discard_len(), s.supply.discard_len());
        }
        // Seat 2 did not see them: for it they may be anywhere.
        let dealt = (0..60).any(|seed| {
            let d = Auberge::determinize(&s, Seat(2), &mut SmallRng::seed_from_u64(seed));
            d.hands[0].contains(&r(10))
        });
        assert!(dealt);
    }

    #[test]
    fn during_setup_the_others_choose_again() {
        let mut s = setup(&[&[5, 6, 7], &[2, 8, 9], &[4, 4, 10]], &[1, 3]);
        for (seat, card) in [(0, 5), (1, 2)] {
            Auberge::apply(&mut s, Seat(seat), &Action::Place { card: r(card) }).unwrap();
        }
        let d = Auberge::determinize(&s, Seat(0), &mut SmallRng::seed_from_u64(1));
        assert_eq!(d.placed, vec![Some(r(5)), None, None]);
        assert_eq!(
            Auberge::visible_pending(&d, Viewer::Seat(Seat(0))),
            Auberge::visible_pending(&s, Viewer::Seat(Seat(0)))
        );
    }
}
```

Dans `games/pairs/src/auberge/mod.rs`, ajouter `mod determinize;` avant `pub mod testing;`.

- [ ] **Step 2: Vérifier** — Run: `cargo test -p pairs --release` — Expected: tout passe. Un échec de conformance désigne un défaut du moteur de la tâche 2 : le corriger là.

- [ ] **Step 3: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
git add games/pairs && git commit -m "pairs: L'Auberge des Pirates determinization and conformance"
```

---

### Task 4: IA et registre

**Files:**
- Create: `games/pairs/src/auberge/heuristics.rs`
- Modify: `games/pairs/src/auberge/mod.rs`, `crates/ludo-registry/src/lib.rs`, `CLAUDE.md`

**Interfaces:**
- Consumes : `State::{pair_chance, dare_risk, draw_risk, own_pair, alive}`, `testing::table`.
- Produces : `impl Heuristics for Auberge` ; l'entrée `pairs-auberge` du registre.

- [ ] **Step 1: Tests** — `games/pairs/src/auberge/heuristics.rs`, en bas :

```rust
#[cfg(test)]
mod tests {
    use ludo_core::{Game, Heuristics, Seat};
    use rand::rngs::SmallRng;
    use rand::{Rng, SeedableRng};

    use crate::auberge::testing::{setup, table};
    use crate::auberge::{Action, Auberge, Config, State};
    use crate::testing::r;

    fn rollout(s: &State, seat: u8, seed: u64) -> Action {
        Auberge::rollout_action(s, Seat(seat), &mut SmallRng::seed_from_u64(seed))
    }

    #[test]
    fn accuses_a_player_who_very_likely_holds_a_pair() {
        // Eight hidden cards behind three face-up ones.
        let s = table(
            &[&[5], &[1, 2, 2, 3, 3, 4, 4, 6]],
            &[&[7], &[10, 9, 8]],
            &[5, 5, 6],
        );
        assert!(s.pair_chance(Some(Seat(0)), Seat(1)) > 0.9);
        for seed in 0..30 {
            assert_eq!(rollout(&s, 0, seed), Action::Accuse { target: Seat(1) });
        }
    }

    #[test]
    fn gets_rid_of_its_own_pair_by_attacking_with_it() {
        let s = table(&[&[7, 7, 3], &[4]], &[&[2], &[9]], &[5, 5, 6, 8, 8, 10]);
        for seed in 0..30 {
            assert_eq!(
                rollout(&s, 0, seed),
                Action::Attack {
                    target: Seat(1),
                    card: r(7)
                }
            );
        }
    }

    #[test]
    fn does_not_accuse_on_a_slim_chance() {
        let s = table(&[&[], &[4]], &[&[2], &[9]], &[5, 5, 6, 8, 8, 10]);
        for seed in 0..30 {
            assert_ne!(rollout(&s, 0, seed), Action::Accuse { target: Seat(1) });
        }
    }

    #[test]
    fn places_a_card_of_its_pair_during_setup() {
        let s = setup(&[&[9, 4, 9], &[2, 8, 3]], &[1]);
        for seed in 0..20 {
            assert_eq!(rollout(&s, 0, seed), Action::Place { card: r(9) });
        }
    }

    #[test]
    fn the_rollout_action_is_always_legal() {
        let mut rng = SmallRng::seed_from_u64(21);
        for players in 2..=8u8 {
            for _ in 0..5 {
                let mut s = Auberge::new(&Config::default(), players, rng.gen());
                while let Some(&seat) = Auberge::pending(&s).0.first() {
                    let action = Auberge::rollout_action(&s, seat, &mut rng);
                    assert!(
                        Auberge::legal_actions(&s, seat).contains(&action),
                        "{action:?}"
                    );
                    Auberge::apply(&mut s, seat, &action).unwrap();
                }
            }
        }
    }
}
```

Dans `mod.rs`, ajouter `mod heuristics;` après `mod determinize;`. Run: `cargo test -p pairs --release --lib auberge::heuristics` — Expected: erreur de compilation (`rollout_action` introuvable).

- [ ] **Step 2: Politique** — au-dessus des tests :

```rust
use ludo_core::{Game, Heuristics, Seat};
use rand::seq::SliceRandom;
use rand::Rng;

use super::{Action, Auberge, Phase, State};
use crate::deck::pair_in;

/// Accuse above this estimated chance of a pair.
const ACCUSE: f64 = 0.5;
/// Dare only below this chance of pairing the target's face-up cards.
const DARE: f64 = 0.2;
/// How often a hand card is played on someone rather than kept.
const ATTACK: f64 = 0.6;

/// The opponent `seat` thinks most likely to hold a pair, among those it
/// may accuse.
fn prime_suspect(s: &State, seat: Seat, legal: &[Action]) -> Option<(Seat, f64)> {
    legal
        .iter()
        .filter_map(|action| match action {
            Action::Accuse { target } => Some((*target, s.pair_chance(Some(seat), *target))),
            _ => None,
        })
        .max_by(|a, b| a.1.total_cmp(&b.1))
}

impl Heuristics for Auberge {
    /// From what `seat` sees only. Setup: place a card of its own pair.
    /// Play: accuse a likely pair; else break its own pair by attacking with
    /// it; else mostly attack to empty its hand (no hand, no accusation);
    /// else dare a safe target, or draw.
    fn rollout_action<R: Rng + ?Sized>(s: &State, seat: Seat, rng: &mut R) -> Action {
        let legal = Self::legal_actions(s, seat);
        let any = |rng: &mut R| legal.choose(rng).cloned().unwrap_or(Action::Draw);
        if s.phase == Phase::Setup {
            return match pair_in(&s.hands[seat.index()]) {
                Some(card) => Action::Place { card },
                None => any(rng),
            };
        }
        let suspect = prime_suspect(s, seat, &legal);
        if let Some((target, chance)) = suspect {
            if chance >= ACCUSE {
                return Action::Accuse { target };
            }
        }
        let attacks: Vec<&Action> = legal
            .iter()
            .filter(|a| matches!(a, Action::Attack { .. }))
            .collect();
        if let Some(pair) = s.own_pair(seat) {
            let with_pair: Vec<&&Action> = attacks
                .iter()
                .filter(|a| matches!(a, Action::Attack { card, .. } if *card == pair))
                .collect();
            if let Some(&&attack) = with_pair.choose(rng) {
                return attack.clone();
            }
        }
        if !attacks.is_empty() && rng.gen_bool(ATTACK) {
            if let Some(&attack) = attacks.choose(rng) {
                return attack.clone();
            }
        }
        let safest = legal
            .iter()
            .filter_map(|action| match action {
                Action::Dare { target } => Some((*target, s.dare_risk(Some(seat), *target))),
                _ => None,
            })
            .min_by(|a, b| a.1.total_cmp(&b.1));
        if let Some((target, risk)) = safest {
            if risk <= DARE && (risk <= s.draw_risk(seat) || rng.gen_bool(0.5)) {
                return Action::Dare { target };
            }
        }
        if legal.contains(&Action::Draw) {
            return Action::Draw;
        }
        match suspect {
            Some((target, _)) if attacks.is_empty() => Action::Accuse { target },
            _ => any(rng),
        }
    }
}
```

- [ ] **Step 3: Registre** — `crates/ludo-registry/src/lib.rs` :

```rust
static PAIRS_CONTINUOUS: GameEntry<pairs::ContinuousPairs> = GameEntry::new();
static PAIRS_AUBERGE: GameEntry<pairs::Auberge> = GameEntry::new();

pub static GAMES: &[&dyn AnyGame] = &[&YAVALATH, &RIICHI, &PAIRS_CONTINUOUS, &PAIRS_AUBERGE];
```

- [ ] **Step 4: Vérifier** — Run: `cargo test --workspace --release` — Expected: tout passe.

- [ ] **Step 5: Mesurer** — Run: `cargo run -q --release -p ludo-cli -- arena pairs-auberge 400 hard easy 2>&1 | tr '\r' '\n' | tail -1`, puis la même avec `4` en dernier argument. Expected: ≥ 0,55 à 2 joueurs. Sinon essayer, en remesurant : `ACCUSE` à 0.4 puis 0.6, `ATTACK` à 0.4 puis 0.8 ; garder le meilleur et consigner le chiffre.

- [ ] **Step 6: `CLAUDE.md`** — sous la ligne `arena pairs-continuous`, ajouter la ligne `arena pairs-auberge` avec les deux chiffres mesurés ; après la section « Ajouter un jeu », ajouter :

```markdown
## Ajouter un jeu Pairs
Comme « Ajouter un jeu », mais sans nouvelle crate :
1. `games/pairs/src/<jeu>/` : `mod.rs` (règles, `Game`), `view.rs`, `determinize.rs`, `heuristics.rs`, `testing.rs` ; `pub mod <jeu>;` et la réexportation dans `games/pairs/src/lib.rs`. Le matériel vient de `deck.rs` (`Rank`, `Supply`, `pair_in`…) : l'étendre là plutôt que de le recopier.
2. Une section dans `games/pairs/RULES.md` ; `tests/<jeu>_rules.rs` ; un cas dans `tests/conformance.rs` pour chaque nombre de joueurs.
3. `crates/ludo-registry/src/lib.rs` : un `static` et une entrée dans `GAMES`.
4. `web/src/games/pairs/<jeu>/types.ts` + `Board.tsx` (cartes et table : `../cards.tsx`), un test dans `__tests__/`, une entrée dans `web/src/games/registry.ts`.
5. `ludo-cli arena pairs-<jeu> 400 hard easy`, et la ligne correspondante dans « Commandes ».
```

- [ ] **Step 7: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
git add games/pairs crates/ludo-registry CLAUDE.md && git commit -m "pairs: L'Auberge des Pirates AI and registry entry"
```

---

### Task 5: Plateau web

**Files:**
- Create: `web/src/games/pairs/auberge/types.ts`, `web/src/games/pairs/auberge/Board.tsx`, `web/src/games/pairs/__tests__/auberge.test.tsx`
- Modify: `web/src/games/pairs/pairs.css`, `web/src/games/registry.ts`, `README.md`, `NOTICE`, `docs/superpowers/specs/2026-10-13-pairs-design.md`

**Interfaces:**
- Consumes : le JSON figé par `the_json_the_web_relies_on` (tâche 2) ; `Card`, `CardBack`, `Table` de `../cards` ; `BoardProps`.
- Produces : `AubergeBoard`, `describeAubergeEvent`.

- [ ] **Step 1: Types** — `web/src/games/pairs/auberge/types.ts` :

```ts
// Mirrors games/pairs/src/auberge (View, Action, Event).

/** 1 to 10. */
export type Rank = number;

export type AubergeEvent =
  | { type: "Placed"; seat: number; card: Rank }
  | { type: "Starts"; seat: number }
  | { type: "Drew"; seat: number }
  | { type: "Attacked"; seat: number; target: number; card: Rank }
  | { type: "Dared"; seat: number; target: number; card: Rank; paired: boolean }
  /** `hand` is empty unless the viewer is the accuser or the accused. */
  | { type: "Accused"; seat: number; target: number; guilty: boolean; hand: Rank[] }
  | { type: "Out"; seat: number }
  | { type: "RoundWon"; seat: number | null }
  | { type: "Reshuffled" };

export interface AubergeSeatView {
  front: Rank[];
  handSize: number;
  out: boolean;
  wins: number;
  /** Per thousand; 0 for the viewer and for an eliminated seat. */
  pairOdds: number;
}

export interface AubergeView {
  round: number;
  rounds: number;
  phase: "Setup" | "Play" | "Over";
  seats: AubergeSeatView[];
  hand: Rank[];
  placed: Rank | null;
  ownPair: Rank | null;
  turn: number | null;
  deck: number;
  discard: number;
  discardSeen: Rank[];
  /** Index = rank − 1. */
  unlocated: number[];
  events: AubergeEvent[];
}

export type AubergeAction =
  | "Draw"
  | { Place: { card: Rank } }
  | { Attack: { target: number; card: Rank } }
  | { Dare: { target: number } }
  | { Accuse: { target: number } };
```

- [ ] **Step 2: Test du plateau** — `web/src/games/pairs/__tests__/auberge.test.tsx` :

```tsx
import { isValidElement, type ReactElement, type ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { AubergeBoard, describeAubergeEvent } from "../auberge/Board";
import type { AubergeAction, AubergeView } from "../auberge/types";

type AnyElement = ReactElement<Record<string, unknown>>;

/** Every host element of a tree, resolving the function components (none uses hooks). */
function hostElements(node: ReactNode, out: AnyElement[] = []): AnyElement[] {
  if (Array.isArray(node)) {
    node.forEach((child) => hostElements(child, out));
    return out;
  }
  if (!isValidElement(node)) return out;
  const el = node as AnyElement;
  if (typeof el.type === "function") {
    return hostElements((el.type as (props: unknown) => ReactNode)(el.props), out);
  }
  out.push(el);
  return hostElements(el.props.children as ReactNode, out);
}

const NAMES = ["Ana", "Bob", "Cléo"];

const PLAY: AubergeView = {
  round: 2,
  rounds: 5,
  phase: "Play",
  seats: [
    { front: [5], handSize: 2, out: false, wins: 1, pairOdds: 0 },
    { front: [2, 9], handSize: 3, out: false, wins: 0, pairOdds: 412 },
    { front: [], handSize: 0, out: true, wins: 0, pairOdds: 0 },
  ],
  hand: [7, 7],
  placed: null,
  ownPair: 7,
  turn: 0,
  deck: 30,
  discard: 4,
  discardSeen: [4, 8],
  unlocated: [1, 1, 3, 3, 4, 6, 5, 7, 8, 10],
  events: [{ type: "Out", seat: 2 }],
};

const PLAY_LEGAL: AubergeAction[] = [
  "Draw",
  { Dare: { target: 1 } },
  { Attack: { target: 1, card: 7 } },
  { Accuse: { target: 1 } },
];

const SETUP: AubergeView = {
  ...PLAY,
  round: 1,
  phase: "Setup",
  seats: NAMES.map(() => ({ front: [], handSize: 3, out: false, wins: 0, pairOdds: 120 })),
  hand: [3, 8, 8],
  ownPair: 8,
  turn: null,
  events: [],
};

function board(view: AubergeView, legal: AubergeAction[], me: number | null, submit: (a: unknown) => void = () => {}) {
  return (
    <AubergeBoard
      view={view}
      legalActions={legal}
      pending={view.turn === null ? [] : [view.turn]}
      me={me}
      seatNames={NAMES}
      submit={submit}
    />
  );
}

function clickEverything(view: AubergeView, legal: AubergeAction[]): unknown[] {
  const sent: unknown[] = [];
  const controls = hostElements(board(view, legal, 0, (a) => sent.push(a))).filter(
    (el) => typeof el.props.onClick === "function",
  );
  controls.forEach((el) => (el.props.onClick as () => void)());
  return sent;
}

describe("Auberge des Pirates board", () => {
  it("submits exactly the legal actions during play, one per control", () => {
    const sent = clickEverything(PLAY, PLAY_LEGAL);
    expect(sent).toHaveLength(PLAY_LEGAL.length);
    for (const action of PLAY_LEGAL) expect(sent).toContainEqual(action);
  });

  it("submits exactly the legal placements during setup", () => {
    const legal: AubergeAction[] = [{ Place: { card: 3 } }, { Place: { card: 8 } }];
    const sent = clickEverything(SETUP, legal);
    expect(sent).toHaveLength(2);
    for (const action of legal) expect(sent).toContainEqual(action);
  });

  it("shows the table, the hand and the warning about a held pair", () => {
    const html = renderToStaticMarkup(board(PLAY, PLAY_LEGAL, 0));
    expect(html).toContain("Manche 2 / 5");
    expect(html).toContain("Tu as une paire de 7");
    expect(html).toContain("Paire estimée : 41 %");
    expect(html).toContain("Piocher");
    expect(html).toContain("Accuser");
    expect(html).toContain("éliminé");
    expect(html.match(/class="pairs-seat/g)).toHaveLength(3);
  });

  it("offers nothing once my card is placed", () => {
    const placed: AubergeView = { ...SETUP, placed: 8 };
    const html = renderToStaticMarkup(board(placed, [], 0));
    expect(html).not.toContain("<button");
    expect(html).toContain("En attente des autres joueurs");
  });

  it("offers nothing to a spectator or once the game is over", () => {
    expect(renderToStaticMarkup(board({ ...PLAY, hand: [], ownPair: null }, [], null))).not.toContain("<button");
    const over: AubergeView = { ...PLAY, phase: "Over", turn: null, events: [{ type: "RoundWon", seat: 0 }] };
    const html = renderToStaticMarkup(board(over, [], 0));
    expect(html).not.toContain("<button");
    expect(html).toContain("Partie terminée");
  });

  it("tells what happened in words", () => {
    const say = (e: Parameters<typeof describeAubergeEvent>[0]) => describeAubergeEvent(e, NAMES);
    expect(say({ type: "Placed", seat: 1, card: 4 })).toBe("Bob pose un 4.");
    expect(say({ type: "Starts", seat: 1 })).toBe("Bob commence.");
    expect(say({ type: "Drew", seat: 0 })).toBe("Ana pioche.");
    expect(say({ type: "Attacked", seat: 0, target: 1, card: 7 })).toBe("Ana attaque Bob avec un 7.");
    expect(say({ type: "Dared", seat: 0, target: 1, card: 6, paired: false })).toBe("Ana défie Bob : un 6, pas de paire.");
    expect(say({ type: "Dared", seat: 0, target: 1, card: 9, paired: true })).toBe(
      "Ana défie Bob : un 9, qui fait une paire.",
    );
    expect(say({ type: "Accused", seat: 0, target: 1, guilty: true, hand: [] })).toBe(
      "Ana accuse Bob, à raison.",
    );
    expect(say({ type: "Accused", seat: 0, target: 1, guilty: false, hand: [3, 8] })).toBe(
      "Ana accuse Bob, à tort. Main montrée : 3, 8.",
    );
    expect(say({ type: "Out", seat: 2 })).toBe("Cléo est éliminé·e.");
    expect(say({ type: "RoundWon", seat: 0 })).toBe("Ana gagne la manche.");
    expect(say({ type: "RoundWon", seat: null })).toBe("Manche bloquée : personne ne la gagne.");
    expect(say({ type: "Reshuffled" })).toBe("La défausse est remélangée.");
  });
});
```

Run: `cd web && pnpm test` — Expected: FAIL (`Cannot find module '../auberge/Board'`).

- [ ] **Step 3: Plateau** — `web/src/games/pairs/auberge/Board.tsx` :

```tsx
import type { BoardProps } from "../../registry";
import { Card, CardBack, Table } from "../cards";
import type { AubergeAction, AubergeEvent, AubergeView } from "./types";

export function describeAubergeEvent(e: AubergeEvent, names: string[]): string {
  switch (e.type) {
    case "Placed":
      return `${names[e.seat]} pose un ${e.card}.`;
    case "Starts":
      return `${names[e.seat]} commence.`;
    case "Drew":
      return `${names[e.seat]} pioche.`;
    case "Attacked":
      return `${names[e.seat]} attaque ${names[e.target]} avec un ${e.card}.`;
    case "Dared":
      return `${names[e.seat]} défie ${names[e.target]} : un ${e.card}, ${e.paired ? "qui fait une paire" : "pas de paire"}.`;
    case "Accused": {
      const shown = e.hand.length > 0 ? ` Main montrée : ${e.hand.join(", ")}.` : "";
      return `${names[e.seat]} accuse ${names[e.target]}, ${e.guilty ? "à raison" : "à tort"}.${shown}`;
    }
    case "Out":
      return `${names[e.seat]} est éliminé·e.`;
    case "RoundWon":
      return e.seat === null ? "Manche bloquée : personne ne la gagne." : `${names[e.seat]} gagne la manche.`;
    case "Reshuffled":
      return "La défausse est remélangée.";
  }
}

export function AubergeBoard({ view, legalActions, me, seatNames, submit }: BoardProps) {
  const v = view as AubergeView;
  const legal = legalActions as AubergeAction[];
  const places = legal.flatMap((a) => (typeof a === "object" && "Place" in a ? [a.Place.card] : []));
  const attacks = legal.flatMap((a) => (typeof a === "object" && "Attack" in a ? [a.Attack] : []));

  const seats = v.seats.map((seat, i) => {
    const cls = `pairs-seat${v.turn === i ? " active" : ""}${seat.out ? " lost" : ""}`;
    const dare = legal.find((a) => typeof a === "object" && "Dare" in a && a.Dare.target === i);
    const accuse = legal.find((a) => typeof a === "object" && "Accuse" in a && a.Accuse.target === i);
    const mine = attacks.filter((a) => a.target === i);
    return (
      <section key={i} className={cls}>
        <h3>
          <span>
            {seatNames[i]}
            {seat.out ? " (éliminé)" : ""}
          </span>
          <span title="Manches gagnées">★ {seat.wins}</span>
        </h3>
        <div className="stack">
          {seat.front.map((card, k) => (
            <Card key={k} rank={card} />
          ))}
        </div>
        {!seat.out && <CardBack count={seat.handSize} label="en main" />}
        {i !== me && !seat.out && seat.handSize > 0 && (
          <p className="odds">Paire estimée : {Math.round(seat.pairOdds / 10)} %</p>
        )}
        {(dare || accuse || mine.length > 0) && (
          <div className="pairs-moves">
            {dare && (
              <button type="button" onClick={() => submit(dare)}>
                Défier
              </button>
            )}
            {accuse && (
              <button type="button" onClick={() => submit(accuse)}>
                Accuser
              </button>
            )}
            {mine.map((a) => (
              <Card
                key={a.card}
                rank={a.card}
                small
                title={`Attaquer ${seatNames[i]} avec un ${a.card}`}
                onClick={() => submit({ Attack: a } satisfies AubergeAction)}
              />
            ))}
          </div>
        )}
      </section>
    );
  });

  const center = (
    <>
      <p className="pairs-round">
        Manche {v.round} / {v.rounds}
      </p>
      <div className="pairs-piles">
        <CardBack count={v.deck} label="Pioche" />
        <CardBack count={v.discard} label="Défausse" />
      </div>
      {v.discardSeen.length > 0 && (
        <div className="pairs-seen" title="Cartes défaussées face visible">
          {v.discardSeen.map((card, k) => (
            <Card key={k} rank={card} small />
          ))}
        </div>
      )}
      {v.phase === "Over" && <p className="hint">Partie terminée.</p>}
      {legal.includes("Draw") && (
        <button type="button" onClick={() => submit("Draw" satisfies AubergeAction)}>
          Piocher
        </button>
      )}
      {me !== null && (
        <div className="pairs-hand">
          <span>Ta main :</span>
          {v.hand.map((card, k) => {
            // One button per rank the engine accepts, on its first card.
            const placeable = places.includes(card) && v.hand.indexOf(card) === k;
            return (
              <Card
                key={k}
                rank={card}
                title={placeable ? `Poser ce ${card}` : undefined}
                onClick={placeable ? () => submit({ Place: { card } } satisfies AubergeAction) : undefined}
              />
            );
          })}
        </div>
      )}
      {v.phase === "Setup" && places.length > 0 && <p className="hint">Choisis la carte à poser devant toi.</p>}
      {v.phase === "Setup" && me !== null && places.length === 0 && (
        <p className="hint">En attente des autres joueurs…</p>
      )}
      {v.ownPair !== null && (
        <p className="hint warn">Tu as une paire de {v.ownPair} : une accusation t&apos;éliminerait.</p>
      )}
      <div className="pairs-unlocated" title="Cartes que tu ne peux pas situer">
        {v.unlocated.map((count, i) => (
          <span key={i}>
            <Card rank={i + 1} small />×{count}
          </span>
        ))}
      </div>
      <ul className="pairs-events">
        {v.events.map((e, i) => (
          <li key={i}>{describeAubergeEvent(e, seatNames)}</li>
        ))}
      </ul>
    </>
  );

  return <Table seats={seats} first={me ?? 0} center={center} />;
}
```

Ajouter à la fin de `web/src/games/pairs/pairs.css`, avant le bloc `@media` :

```css
.pairs-round { margin: 0; font-weight: 600; }
.pairs-moves { display: flex; flex-wrap: wrap; gap: 4px; align-items: center; margin-top: 4px; }
.pairs-moves button { padding: 2px 8px; font-size: 0.85rem; }
.pairs-hand { display: flex; flex-wrap: wrap; align-items: center; justify-content: center; gap: 2px; }
.pairs-seen { display: flex; flex-wrap: wrap; justify-content: center; }
.hint.warn { color: var(--danger); font-weight: 600; }
```

`web/src/games/registry.ts` : `import { AubergeBoard } from "./pairs/auberge/Board";` et l'entrée `{ id: "pairs-auberge", Board: AubergeBoard },` après celle de `pairs-continuous`.

- [ ] **Step 4: Vérifier** — Run: `./scripts/build-wasm.sh dev && cd web && pnpm test && pnpm build` — Expected: les 6 tests de `auberge.test.tsx` passent avec les autres ; build sans erreur.

- [ ] **Step 5: Navigateur** — servir `pnpm preview`, jouer une partie de 4 puis 8 sièges contre des IA : la mise en place demande une carte ; piocher, attaquer, défier, accuser jouent le coup ; une manche se termine et la suivante est distribuée ; la partie s'arrête après 5 manches ; pas de chevauchement à 8. Noter dans le commit tout défaut non corrigé.

- [ ] **Step 6: Docs** — `README.md` : ajouter `- **L'Auberge des Pirates** (2–8 joueurs, Regent multijoueur, deck Pairs).` à la liste des jeux. `NOTICE` : ajouter que la règle de l'Auberge est reformulée d'après l'édition Origames. Spec §9 : ajouter les quatre écarts de l'en-tête de ce plan.

- [ ] **Step 7: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --release
git add web/src/games README.md NOTICE docs/superpowers/specs/2026-10-13-pairs-design.md
git commit -m "web: L'Auberge des Pirates board"
```

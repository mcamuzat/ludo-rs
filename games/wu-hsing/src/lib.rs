//! Wu Hsing (Sid Sackson, « Domino Bead Game ») : 2 à 4 joueurs posent des
//! dominos de valeurs 1 à 5 en prolongeant des cycles de couleurs. Règles et
//! source : RULES.md, où [R] est la règle française.

pub mod board;
pub mod deck;
mod view;

pub use board::{valid_cycle, Board, Dir, Placed, Placement};
pub use deck::{deal, is_double, values, Deal, DOMINOES};
pub use view::{HandDomino, LastMove, LegalMove, PlacedView, View};

use ludo_core::{Determinize, Game, GameMeta, Illegal, Outcome, Pending, Seat, Viewer};
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

pub struct WuHsing;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    Place(Placement),
    /// [R] Only when no domino can be laid.
    Pass,
}

/// The end of the game: [R] 5 per double and 25 per other domino left in
/// hand come off each score.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finish {
    pub penalties: Vec<i32>,
    pub totals: Vec<i32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub board: Board,
    /// Sorted.
    pub hands: Vec<Vec<u8>>,
    /// [R] At three, the domino set aside face down.
    pub discarded: Option<u8>,
    pub scores: Vec<i32>,
    pub first: Seat,
    /// `None` once the game is over.
    pub turn: Option<Seat>,
    /// Passes in a row.
    pub passes: u8,
    /// A hand is empty: the round ends before the first player again.
    pub last_round: bool,
    pub last: Option<(Seat, Action, u32)>,
    pub finish: Option<Finish>,
}

impl State {
    fn players(&self) -> u8 {
        self.hands.len() as u8
    }

    /// The hand with one domino per pair of values (the lower id): two
    /// identical dominoes make the same moves.
    fn distinct_hand(&self, seat: Seat) -> Vec<u8> {
        let mut out: Vec<u8> = Vec::new();
        for &d in &self.hands[seat.index()] {
            if !out.iter().any(|&o| values(o) == values(d)) {
                out.push(d);
            }
        }
        out
    }

    /// The legal actions of `seat` with their points: its placements, or
    /// `Pass` alone when it has none. Empty when `seat` is not to play.
    pub fn moves(&self, seat: Seat) -> Vec<(Action, u32)> {
        if self.turn != Some(seat) {
            return Vec::new();
        }
        let moves: Vec<(Action, u32)> = self
            .board
            .moves(&self.distinct_hand(seat))
            .into_iter()
            .map(|(p, points)| (Action::Place(p), points))
            .collect();
        if moves.is_empty() {
            vec![(Action::Pass, 0)]
        } else {
            moves
        }
    }

    fn check(&self, seat: Seat, action: &Action) -> Result<u32, Illegal> {
        match self.turn {
            None => return Err(Illegal::new("La partie est terminée.")),
            Some(t) if t != seat => return Err(Illegal::new("Ce n'est pas ton tour.")),
            _ => {}
        }
        match action {
            Action::Pass => {
                if self.moves(seat).iter().any(|(a, _)| *a != Action::Pass) {
                    Err(Illegal::new(
                        "Un coup est possible : tu ne peux pas passer.",
                    ))
                } else {
                    Ok(0)
                }
            }
            Action::Place(p) => {
                let hand = &self.hands[seat.index()];
                if !hand.contains(&p.domino) {
                    return Err(Illegal::new("Tu n'as pas ce domino."));
                }
                if hand
                    .iter()
                    .any(|&d| d < p.domino && values(d) == values(p.domino))
                {
                    return Err(Illegal::new(
                        "Joue le premier de tes deux dominos identiques.",
                    ));
                }
                if p.flipped && is_double(p.domino) {
                    return Err(Illegal::new("Un double ne se retourne pas."));
                }
                self.board.evaluate(p)
            }
        }
    }

    fn end(&mut self) {
        let penalties: Vec<i32> = self
            .hands
            .iter()
            .map(|h| h.iter().map(|&d| if is_double(d) { 5 } else { 25 }).sum())
            .collect();
        let totals = self
            .scores
            .iter()
            .zip(&penalties)
            .map(|(s, p)| s - p)
            .collect();
        self.finish = Some(Finish { penalties, totals });
        self.turn = None;
    }
}

impl Game for WuHsing {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "wu-hsing",
            name: "Wu Hsing",
            min_players: 2,
            max_players: 4,
            hidden_info: true,
            clock: true,
        }
    }

    fn new(_config: &Config, players: u8, seed: u64) -> State {
        // ChaCha, not SmallRng: a browser log must replay natively.
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let d = deal(players, &mut rng);
        // [R] The first player is drawn at random.
        let first = Seat((rng.gen::<u32>() % u32::from(players)) as u8);
        State {
            board: Board::new(d.starters),
            hands: d.hands,
            discarded: d.discarded,
            scores: vec![0; usize::from(players)],
            first,
            turn: Some(first),
            passes: 0,
            last_round: false,
            last: None,
            finish: None,
        }
    }

    fn pending(s: &State) -> Pending {
        s.turn.map_or_else(Pending::none, Pending::one)
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        s.moves(seat).into_iter().map(|(a, _)| a).collect()
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        let points = s.check(seat, action)?;
        match action {
            Action::Place(p) => {
                let hand = &mut s.hands[seat.index()];
                hand.retain(|&d| d != p.domino);
                if hand.is_empty() {
                    s.last_round = true;
                }
                s.board.place(*p, seat.0);
                s.scores[seat.index()] += points as i32;
                s.passes = 0;
            }
            Action::Pass => s.passes += 1,
        }
        s.last = Some((seat, *action, points));
        // [R] Once a hand is empty, the players after it play on until all
        // have had as many turns; [R] the game also ends when all have passed
        // in a row.
        let next = Seat((seat.0 + 1) % s.players());
        if (s.last_round && next == s.first) || s.passes >= s.players() {
            s.end();
        } else {
            s.turn = Some(next);
        }
        Ok(())
    }

    fn view(s: &State, viewer: Viewer) -> View {
        View::of(s, viewer)
    }

    /// [R] The highest final score wins; equal scores share the rank.
    fn outcome(s: &State) -> Option<Outcome> {
        let f = s.finish.as_ref()?;
        let ranks = f
            .totals
            .iter()
            .map(|t| f.totals.iter().filter(|o| *o > t).count() as u8)
            .collect();
        Some(Outcome::Ranked(ranks))
    }

    /// The placement that scores most, or a pass.
    fn timeout_action(s: &State, seat: Seat) -> Option<Action> {
        s.moves(seat)
            .into_iter()
            .rev()
            .max_by_key(|&(_, points)| points)
            .map(|(a, _)| a)
    }
}

impl Determinize for WuHsing {
    /// The other hands and the domino set aside are dealt again among
    /// themselves, sizes kept.
    fn determinize<R: Rng + ?Sized>(s: &State, observer: Seat, rng: &mut R) -> State {
        let mut d = s.clone();
        let mut hidden: Vec<u8> = d
            .hands
            .iter()
            .enumerate()
            .filter(|&(i, _)| i != observer.index())
            .flat_map(|(_, h)| h.iter().copied())
            .chain(d.discarded)
            .collect();
        hidden.shuffle(rng);
        for (i, hand) in d.hands.iter_mut().enumerate() {
            if i == observer.index() {
                continue;
            }
            let n = hand.len();
            *hand = hidden.split_off(hidden.len() - n);
            hand.sort_unstable();
        }
        d.discarded = hidden.pop();
        d
    }
}

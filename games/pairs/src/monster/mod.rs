//! Monster (James Ernest): the best hand must defeat the Monster, a hand
//! made of everyone's discards. Rules and their sources: RULES.md.

mod determinize;
mod heuristics;
pub mod testing;
mod view;

pub use view::{SeatView, View};

use ludo_core::{Game, GameMeta, Illegal, Outcome, Pending, Seat, Viewer};
use serde::{Deserialize, Serialize};

use crate::deck::{full_deck, pair_in, Rank, Supply};

pub struct Monster;

/// [R] Six cards each.
pub const HAND_SIZE: usize = 6;
/// [R] Up to three discards.
pub const MAX_DISCARDS: usize = 3;
/// [R] A Champion beaten by the Monster pays 5 chips.
pub const PENALTY: u32 = 5;
/// ❓1
pub const DEFAULT_GAMES: u8 = 3;
pub const MAX_GAMES: u8 = 15;

/// [R] 2 chips each at 4 or 5 players, 1 from 6 up.
pub fn ante(players: u8) -> u32 {
    if players <= 5 {
        2
    } else {
        1
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Games (pots won) in a match, capped at `MAX_GAMES`. `None` (or 0): 3.
    pub games: Option<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Discard these cards (0 to 3, lowest first) to the Monster, keeping
    /// a hand without a pair.
    Discard {
        cards: Vec<Rank>,
    },
    Fold,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Act,
    Over,
}

/// [R] How a hand ranks: highest card first, then the next; a card beats
/// nothing. Higher is better. The Monster counts each rank once.
pub fn strength(cards: &[Rank]) -> Vec<Rank> {
    let mut cards = cards.to_vec();
    cards.sort_unstable_by(|a, b| b.cmp(a));
    cards
}

pub fn monster_strength(discards: &[Rank]) -> Vec<Rank> {
    let mut cards = strength(discards);
    cards.dedup();
    cards
}

/// What happened, for the board to show.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    /// A new game: everyone antes; the button.
    GameStarted {
        game: u8,
        button: Seat,
        ante: u32,
    },
    /// A new round among `seats`, six cards each.
    Dealt {
        round: u8,
        seats: Vec<Seat>,
    },
    Discarded {
        seat: Seat,
        count: u8,
    },
    Folded {
        seat: Seat,
    },
    /// The kept hands (`None`: folded or not in the round), the Monster,
    /// the Champions and whether they beat it.
    Showdown {
        hands: Vec<Option<Vec<Rank>>>,
        monster: Vec<Rank>,
        champions: Vec<Seat>,
        won: bool,
    },
    /// Beaten Champions pay 5 each.
    Penalty {
        seats: Vec<Seat>,
    },
    /// `seats` take the pot, `share` each.
    PotWon {
        seats: Vec<Seat>,
        share: u32,
    },
    /// Nobody is left: the players of the round play it again.
    Replay,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub players: u8,
    pub games_total: u8,
    pub game_no: u8,
    pub round_no: u8,
    /// Rounds dealt so far: one shuffle each.
    pub deals: u32,
    /// For each deal. Never in a view.
    pub seed: u64,
    /// Its draw pile holds the cards not dealt.
    pub supply: Supply,
    pub hands: Vec<Vec<Rank>>,
    /// This round's discards of each seat, face down: the Monster's hand.
    pub discards: Vec<Vec<Rank>>,
    /// Cards of the seats that folded this round, out of play face down.
    pub folded: Vec<Vec<Rank>>,
    /// Still in this game.
    pub in_game: Vec<bool>,
    /// Dealt into this round.
    pub in_round: Vec<bool>,
    /// Has acted this round.
    pub acted: Vec<bool>,
    pub button: Seat,
    pub turn: Option<Seat>,
    pub phase: Phase,
    pub pot: u32,
    /// Each seat's balance since the start; with the pot, they sum to zero.
    pub chips: Vec<i32>,
    /// Since the previous decision.
    pub events: Vec<Event>,
}

/// One independent stream per deal.
fn mix(seed: u64, deal: u32) -> u64 {
    seed ^ u64::from(deal).wrapping_mul(0xD6E8_FEB8_6659_FD93)
}

impl State {
    fn seats(&self) -> impl Iterator<Item = Seat> {
        (0..self.players).map(Seat)
    }

    fn pay(&mut self, seat: Seat, chips: u32) {
        self.chips[seat.index()] -= chips as i32;
        self.pot += chips;
    }

    /// The Monster's hand this round.
    pub fn monster(&self) -> Vec<Rank> {
        self.discards.iter().flatten().copied().collect()
    }

    /// [R] Everyone antes, back in the game; the button has moved.
    fn start_game(&mut self) {
        self.game_no += 1;
        self.round_no = 0;
        let ante = ante(self.players);
        for seat in self.seats().collect::<Vec<_>>() {
            self.pay(seat, ante);
        }
        self.in_game = vec![true; usize::from(self.players)];
        self.events.push(Event::GameStarted {
            game: self.game_no,
            button: self.button,
            ante,
        });
        let seats: Vec<Seat> = self.seats().collect();
        self.start_round(&seats);
    }

    /// [R] Six cards to each player still in; the player left of the
    /// button acts first.
    fn start_round(&mut self, seats: &[Seat]) {
        self.round_no += 1;
        self.deals += 1;
        self.supply = Supply::new(full_deck(), 0, mix(self.seed, self.deals));
        let n = usize::from(self.players);
        self.hands = vec![Vec::new(); n];
        self.discards = vec![Vec::new(); n];
        self.folded = vec![Vec::new(); n];
        self.in_round = vec![false; n];
        self.acted = vec![false; n];
        for &seat in seats {
            self.in_round[seat.index()] = true;
            self.hands[seat.index()] = (0..HAND_SIZE).filter_map(|_| self.supply.draw()).collect();
        }
        self.events.push(Event::Dealt {
            round: self.round_no,
            seats: seats.to_vec(),
        });
        self.turn = self.next_to_act(self.button);
    }

    fn next_to_act(&self, after: Seat) -> Option<Seat> {
        (1..=self.players)
            .map(|k| Seat((after.0 + k) % self.players))
            .find(|s| self.in_round[s.index()] && !self.acted[s.index()])
    }

    fn after_act(&mut self, seat: Seat) {
        self.acted[seat.index()] = true;
        match self.next_to_act(seat) {
            Some(next) => self.turn = Some(next),
            None => self.showdown(),
        }
    }

    fn standing(&self) -> Vec<Seat> {
        self.seats()
            .filter(|s| self.in_round[s.index()] && self.folded[s.index()].is_empty())
            .collect()
    }

    /// [R] The best hand is the Champion and must beat the Monster, which
    /// wins ties: a win takes the pot; a loss costs 5 chips and the game.
    /// Folded players are out too. One player left wins; nobody left (all
    /// folded, or the last player lost) replays the round.
    fn showdown(&mut self) {
        self.turn = None;
        let round: Vec<Seat> = self.seats().filter(|s| self.in_round[s.index()]).collect();
        let standing = self.standing();
        let monster = monster_strength(&self.monster());
        let best = standing
            .iter()
            .map(|s| strength(&self.hands[s.index()]))
            .max();
        let champions: Vec<Seat> = standing
            .iter()
            .copied()
            .filter(|s| Some(strength(&self.hands[s.index()])) == best)
            .collect();
        let won = best.as_ref().is_some_and(|b| *b > monster);
        if !standing.is_empty() {
            self.events.push(Event::Showdown {
                hands: self
                    .seats()
                    .map(|s| standing.contains(&s).then(|| self.hands[s.index()].clone()))
                    .collect(),
                monster: monster.clone(),
                champions: champions.clone(),
                won,
            });
        }
        if won {
            self.win(&champions);
            return;
        }
        if !champions.is_empty() {
            for &c in &champions {
                self.pay(c, PENALTY);
            }
            self.events.push(Event::Penalty {
                seats: champions.clone(),
            });
        }
        // [R] Folded players and beaten Champions are out.
        let survivors: Vec<Seat> = standing
            .iter()
            .copied()
            .filter(|s| !champions.contains(s))
            .collect();
        for s in self.seats().collect::<Vec<_>>() {
            self.in_game[s.index()] = survivors.contains(&s);
        }
        match survivors[..] {
            [] => {
                // [R] Nobody left: the same players play again.
                for s in &round {
                    self.in_game[s.index()] = true;
                }
                self.events.push(Event::Replay);
                self.start_round(&round);
            }
            [last] => self.win(&[last]),
            _ => self.start_round(&survivors),
        }
    }

    /// The winners split the pot (❓2: odd chips stay for the next game),
    /// the button moves, a new game starts.
    fn win(&mut self, winners: &[Seat]) {
        let share = self.pot / winners.len().max(1) as u32;
        for &w in winners {
            self.chips[w.index()] += share as i32;
            self.pot -= share;
        }
        self.events.push(Event::PotWon {
            seats: winners.to_vec(),
            share,
        });
        if self.game_no >= self.games_total {
            self.phase = Phase::Over;
            return;
        }
        // [R] The button moves after a full game.
        self.button = Seat((self.button.0 + 1) % self.players);
        self.start_game();
    }

    /// Every discard of 0 to 3 cards that leaves no pair, as sorted ranks.
    pub fn discard_options(hand: &[Rank]) -> Vec<Vec<Rank>> {
        let mut options: Vec<Vec<Rank>> = Vec::new();
        let n = hand.len();
        for mask in 0u32..(1 << n) {
            if mask.count_ones() as usize > MAX_DISCARDS {
                continue;
            }
            let (out, kept): (Vec<usize>, Vec<usize>) = (0..n).partition(|&i| mask & (1 << i) != 0);
            let kept: Vec<Rank> = kept.iter().map(|&i| hand[i]).collect();
            if pair_in(&kept).is_some() {
                continue;
            }
            let mut out: Vec<Rank> = out.iter().map(|&i| hand[i]).collect();
            out.sort_unstable();
            if !options.contains(&out) {
                options.push(out);
            }
        }
        options.sort();
        options
    }
}

impl Game for Monster {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "pairs-monster",
            name: "Monster",
            min_players: 4,
            max_players: 8,
            hidden_info: true,
            clock: false,
        }
    }

    fn new(config: &Config, players: u8, seed: u64) -> State {
        let games_total = config
            .games
            .filter(|&g| g > 0)
            .map_or(DEFAULT_GAMES, |g| g.min(MAX_GAMES));
        let n = usize::from(players);
        let mut s = State {
            players,
            games_total,
            game_no: 0,
            round_no: 0,
            deals: 0,
            seed,
            supply: Supply::new(Vec::new(), 0, seed),
            hands: vec![Vec::new(); n],
            discards: vec![Vec::new(); n],
            folded: vec![Vec::new(); n],
            in_game: vec![true; n],
            in_round: vec![false; n],
            acted: vec![false; n],
            // ❓3: seat 0 holds the button first; seat 1 acts first.
            button: Seat(0),
            turn: None,
            phase: Phase::Act,
            pot: 0,
            chips: vec![0; n],
            events: Vec::new(),
        };
        s.start_game();
        s
    }

    fn pending(s: &State) -> Pending {
        match s.phase {
            Phase::Act => s.turn.map_or_else(Pending::none, Pending::one),
            Phase::Over => Pending::none(),
        }
    }

    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        if s.phase != Phase::Act || s.turn != Some(seat) {
            return Vec::new();
        }
        State::discard_options(&s.hands[seat.index()])
            .into_iter()
            .map(|cards| Action::Discard { cards })
            .chain(std::iter::once(Action::Fold))
            .collect()
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        if s.phase == Phase::Over {
            return Err(Illegal::new("La partie est terminée."));
        }
        if s.turn != Some(seat) {
            return Err(Illegal::new("Ce n'est pas ton tour."));
        }
        if !Self::legal_actions(s, seat).contains(action) {
            return Err(Illegal::new(
                "Écarte au plus 3 cartes, et garde une main sans paire.",
            ));
        }
        s.events.clear();
        let i = seat.index();
        match action {
            Action::Discard { cards } => {
                for card in cards {
                    if let Some(at) = s.hands[i].iter().position(|c| c == card) {
                        s.hands[i].remove(at);
                    }
                }
                s.discards[i] = cards.clone();
                s.events.push(Event::Discarded {
                    seat,
                    count: cards.len() as u8,
                });
            }
            Action::Fold => {
                // [R] Folded cards go out of play, not to the Monster.
                s.folded[i] = std::mem::take(&mut s.hands[i]);
                s.events.push(Event::Folded { seat });
            }
        }
        s.after_act(seat);
        Ok(())
    }

    fn view(s: &State, viewer: Viewer) -> View {
        View::of(s, viewer)
    }

    fn outcome(s: &State) -> Option<Outcome> {
        if s.phase != Phase::Over {
            return None;
        }
        let ranks = s
            .chips
            .iter()
            .map(|&mine| s.chips.iter().filter(|&&theirs| theirs > mine).count() as u8)
            .collect();
        Some(Outcome::Ranked(ranks))
    }
}

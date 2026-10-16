//! Pennywise (James Ernest, Cheapass Games), sources and rules in RULES.md:
//! play a coin into the pot, take back the most change below it; the game
//! ends as soon as a player has no coin left.

mod purse;
mod view;

pub use purse::{Character, Purse, Stack, COINS};
pub use view::View;

use ludo_core::{Game, GameMeta, GameOption, Illegal, OptionKind, Outcome, Pending, Seat, Viewer};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// [C][U] Everyone starts with this character's purse.
    pub character: Character,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Put `coin` into the pot and take back `take`, largest coin first.
    Play { coin: u8, take: Vec<u8> },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LastMove {
    pub seat: Seat,
    pub coin: u8,
    pub take: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub players: u8,
    pub config: Config,
    pub purses: Vec<Purse>,
    pub pot: Purse,
    /// Whose turn it is (the seat that just played once the game is over).
    pub current: Seat,
    pub last: Option<LastMove>,
    /// The seat left without coins: the game is over.
    pub ruined: Option<Seat>,
}

impl State {
    /// Each seat's money, in cents.
    pub fn totals(&self) -> Vec<u32> {
        self.purses.iter().map(Purse::total).collect()
    }

    /// [J] Ranks by money left, richest first; equal money ties. The ruined
    /// seat has 0¢ and every other seat at least a coin, so it comes last.
    pub fn ranks(&self) -> Vec<u8> {
        let totals = self.totals();
        totals
            .iter()
            .map(|&mine| {
                let richer = totals.iter().filter(|&&t| t > mine).count();
                u8::try_from(richer).expect("at most 6 seats")
            })
            .collect()
    }
}

pub struct Pennywise;

impl Game for Pennywise {
    type State = State;
    type Action = Action;
    type View = View;
    type Config = Config;

    fn meta() -> GameMeta {
        GameMeta {
            id: "pennywise",
            name: "Pennywise",
            min_players: 2,
            max_players: 6,
            hidden_info: false,
            clock: true,
        }
    }

    fn options() -> Vec<GameOption> {
        vec![GameOption {
            key: "character",
            kind: OptionKind::Choice {
                values: &["Cody", "Darlene", "Sugar", "Taylor"],
                default: "Cody",
            },
        }]
    }

    fn new(config: &Config, players: u8, seed: u64) -> State {
        // [J] alternates the first player between games; a single game draws it.
        let first = ChaCha8Rng::seed_from_u64(seed).gen::<u32>() % u32::from(players);
        State {
            players,
            config: config.clone(),
            purses: vec![config.character.purse(); usize::from(players)],
            pot: Purse::default(),
            current: Seat(u8::try_from(first).expect("first < players")),
            last: None,
            ruined: None,
        }
    }

    fn pending(s: &State) -> Pending {
        if s.ruined.is_some() {
            Pending::none()
        } else {
            Pending::one(s.current)
        }
    }

    /// [J][U] For each coin value held: every assortment of the pot worth the
    /// most below that coin (taking the maximum is compulsory).
    fn legal_actions(s: &State, seat: Seat) -> Vec<Action> {
        if s.ruined.is_some() || seat != s.current {
            return Vec::new();
        }
        let purse = &s.purses[seat.index()];
        COINS
            .iter()
            .filter(|&&coin| purse.count(coin) > 0)
            .flat_map(|&coin| {
                // The coin played cannot come back: it is not below itself.
                s.pot
                    .max_takes(coin)
                    .into_iter()
                    .map(move |take| Action::Play { coin, take })
            })
            .collect()
    }

    fn apply(s: &mut State, seat: Seat, action: &Action) -> Result<(), Illegal> {
        if s.ruined.is_some() {
            return Err(Illegal::new("La partie est finie."));
        }
        if seat != s.current {
            return Err(Illegal::new("Ce n'est pas ton tour."));
        }
        if !Self::legal_actions(s, seat).contains(action) {
            return Err(Illegal::new(
                "Pose une pièce que tu as et reprends le plus de monnaie possible en dessous.",
            ));
        }
        let Action::Play { coin, take } = action;
        let purse = &mut s.purses[seat.index()];
        purse.remove(*coin);
        s.pot.add(*coin);
        for &c in take {
            s.pot.remove(c);
            purse.add(c);
        }
        // [J] A player who loses their last coin ends the game.
        if purse.is_empty() {
            s.ruined = Some(seat);
        } else {
            s.current = Seat((seat.0 + 1) % s.players);
        }
        s.last = Some(LastMove {
            seat,
            coin: *coin,
            take: take.clone(),
        });
        Ok(())
    }

    fn view(s: &State, _viewer: Viewer) -> View {
        View::of(s)
    }

    fn outcome(s: &State) -> Option<Outcome> {
        s.ruined.map(|_| Outcome::Ranked(s.ranks()))
    }

    fn timeout_action(s: &State, seat: Seat) -> Option<Action> {
        Self::legal_actions(s, seat).into_iter().next()
    }
}

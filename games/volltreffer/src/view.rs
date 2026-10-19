use ludo_core::{Game, Seat, Viewer};
use serde::Serialize;

use crate::{
    Action, Card, HireEvent, Phase, Publication, RoundLine, Shown, State, TakenTrick, Tribe,
    Volltreffer, COSTS,
};

/// A place of the display.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Place {
    pub card: Card,
    pub cost: u32,
    /// The viewer may hire it now.
    pub hireable: bool,
}

/// What everyone knows of one seat.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HandInfo {
    pub score: u32,
    pub cards: u8,
    /// The round so far.
    pub line: RoundLine,
    /// [R] Shown in phase 2, visible until the tricks start.
    pub publication: Option<Publication>,
    pub published: bool,
}

/// What one seat sees: its own hand, the display, the track, the hires of
/// the last table round, the tricks. Never another hand, the deck nor the
/// seed; no count of cards gone (the user wants no statistics shown).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub me: Option<Seat>,
    /// By tribe, then value (empty for a spectator).
    pub hand: Vec<Card>,
    pub seats: Vec<HandInfo>,
    pub phase: Phase,
    /// From 1, for display.
    pub round: u32,
    pub blessing: Card,
    pub trump: Tribe,
    pub display: Vec<Place>,
    pub deck: u8,
    pub bargaining: Seat,
    pub current: Option<Seat>,
    /// [U] The hires of the last table round, as seen at the table; older
    /// ones are for memory.
    pub recent_hires: Vec<HireEvent>,
    pub preemptive: Option<Seat>,
    pub trick: Vec<(Seat, Card)>,
    pub last_trick: Option<TakenTrick>,
    pub last_round: Option<Vec<RoundLine>>,
    pub sanctuary: (u32, u32),
    /// Final ranks, once the game is over.
    pub ranks: Option<Vec<u8>>,
    /// Phase 2: the points of each legal publication, in `legal` order.
    pub publication_points: Vec<u32>,
    /// The viewer's legal actions (empty when it is not its turn).
    pub legal: Vec<Action>,
}

impl State {
    /// The hires everyone has just seen: the last table round of them,
    /// while the hands are being formed.
    pub fn recent_hires(&self) -> &[HireEvent] {
        match self.phase {
            Phase::Hire | Phase::Publish => {
                let from = self.hires.len().saturating_sub(self.n());
                &self.hires[from..]
            }
            Phase::Tricks | Phase::Over => &[],
        }
    }

    /// Publications are seen until the tricks start.
    pub fn visible_publication(&self, seat: usize) -> Option<&Publication> {
        match self.phase {
            Phase::Publish => self.publications[seat].as_ref(),
            _ => None,
        }
    }
}

impl View {
    pub fn of(s: &State, viewer: Viewer) -> Self {
        let me = match viewer {
            Viewer::Seat(seat) if seat.index() < s.n() => Some(seat),
            _ => None,
        };
        let legal = me
            .map(|m| Volltreffer::legal_actions(s, m))
            .unwrap_or_default();
        let display = s
            .display
            .iter()
            .zip(COSTS)
            .map(|(&Shown { card, .. }, cost)| Place {
                card,
                cost,
                hireable: legal.contains(&Action::Hire { card }),
            })
            .collect();
        let publication_points = legal
            .iter()
            .filter_map(|a| match a {
                Action::Publish { groups, tribes } => {
                    me.map(|m| crate::publication_points(&s.hands[m.index()], groups, *tribes))
                }
                _ => None,
            })
            .collect();
        let over = s.phase == Phase::Over;
        Self {
            me,
            hand: me.map(|m| s.hands[m.index()].clone()).unwrap_or_default(),
            seats: (0..s.n())
                .map(|i| HandInfo {
                    score: s.scores[i],
                    cards: s.hands[i].len() as u8,
                    line: s.lines[i],
                    publication: s.visible_publication(i).cloned(),
                    published: s.publications[i].is_some(),
                })
                .collect(),
            phase: s.phase,
            round: s.round + 1,
            blessing: s.blessing,
            trump: s.trump(),
            display,
            deck: s.deck.len() as u8,
            bargaining: s.bargaining,
            current: (!over).then_some(s.current),
            recent_hires: s.recent_hires().to_vec(),
            preemptive: s.preemptive,
            trick: s.trick.clone(),
            last_trick: s.last_trick.clone(),
            last_round: s.last_round.clone(),
            sanctuary: (*s.config.sanctuary().start(), *s.config.sanctuary().end()),
            ranks: over.then(|| s.ranks()),
            publication_points,
            legal,
        }
    }
}

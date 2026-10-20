use ludo_core::{Seat, Viewer};
use serde::Serialize;

use crate::knowledge::{direct_facts, knows_trio, Note};
use crate::material::{CardId, Suspect, INVESTIGATIONS};
use crate::{reusable, special_allowed, Answer, Event, Phase, State};

/// What one seat sees: its own hand, every answer, the piles, its sheet's
/// automatic notes, and the trio once it has seen it. Never another hand,
/// the trio before that, the others' guesses before the end, nor the seed.
/// No statistics.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub phase: Phase,
    pub turn: Option<Seat>,
    pub players: u8,
    pub dealer: Seat,
    /// The viewer's hand (empty for a spectator).
    pub hand: Vec<Suspect>,
    pub hand_sizes: Vec<usize>,
    pub reserve: u8,
    pub piles: Vec<PileView>,
    /// The 27 suspects, in the sheet's order [F].
    pub suspects: Vec<Suspect>,
    /// The 36 investigation cards' letters, by card id.
    pub investigations: Vec<&'static str>,
    pub answers: Vec<Answer>,
    /// Who has accused wrongly.
    pub wrong: Vec<bool>,
    /// Who has been asked at least once.
    pub asked: Vec<bool>,
    pub can_accuse: bool,
    pub special_allowed: bool,
    /// Cards that may be laid again in a special investigation.
    pub reusable: Vec<CardId>,
    pub last: Option<Seat>,
    pub final_asked: bool,
    /// Guess: the viewer's own guess, once made.
    pub my_guess: Option<[Suspect; 3]>,
    /// Once the viewer has seen it.
    pub trio: Option<[Suspect; 3]>,
    pub accuser: Option<Seat>,
    /// Everyone's guess, at the end only.
    pub guesses: Vec<Option<[Suspect; 3]>>,
    pub scores: Vec<u16>,
    /// The sheet's automatic notes (direct facts only).
    pub notes: Vec<Note>,
    pub events: Vec<Event>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PileView {
    pub top: Option<CardId>,
    pub left: usize,
}

impl View {
    pub fn of(s: &State, viewer: Viewer) -> Self {
        let me = match viewer {
            Viewer::Seat(seat) if seat.index() < usize::from(s.players) => Some(seat),
            _ => None,
        };
        let over = s.phase == Phase::Over;
        Self {
            phase: s.phase,
            turn: (s.phase == Phase::Turn).then_some(s.turn),
            players: s.players,
            dealer: s.dealer,
            hand: me.map(|m| s.hands[m.index()].clone()).unwrap_or_default(),
            hand_sizes: s.hands.iter().map(Vec::len).collect(),
            reserve: s.reserve,
            piles: s
                .piles
                .iter()
                .map(|p| PileView {
                    top: p.last().copied(),
                    left: p.len(),
                })
                .collect(),
            suspects: Suspect::all().collect(),
            investigations: INVESTIGATIONS.to_vec(),
            answers: s.answers.clone(),
            wrong: s.wrong.clone(),
            asked: s.asked.clone(),
            can_accuse: me
                .is_some_and(|m| s.phase == Phase::Turn && s.turn == m && !s.wrong[m.index()]),
            special_allowed: special_allowed(s),
            reusable: reusable(s),
            last: s.last,
            final_asked: s.final_asked,
            my_guess: me.and_then(|m| s.guesses[m.index()]),
            trio: me.map_or(over, |m| knows_trio(s, m)).then_some(s.trio),
            accuser: s.accuser,
            guesses: if over { s.guesses.clone() } else { Vec::new() },
            scores: s.scores.clone(),
            notes: me.map(|m| direct_facts(s, m)).unwrap_or_default(),
            events: s.events.clone(),
        }
    }
}

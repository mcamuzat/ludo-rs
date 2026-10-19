use ludo_core::{Determinize, Game, Outcome, Seat, Viewer};
use rand::rngs::SmallRng;
use rand::SeedableRng;
use volltreffer::{
    gain, pack, Action, Card, Config, Phase, Shown, State, Tribe, Volltreffer, COSTS,
};

fn c(tribe: Tribe, value: u8) -> Card {
    Card::new(tribe, value)
}

fn new(players: u8, seed: u64) -> State {
    Volltreffer::new(&Config::default(), players, seed)
}

/// Plays the hiring phase by always taking the free card.
fn hire_all_free(s: &mut State) {
    while s.phase == Phase::Hire {
        let card = s.display[0].card;
        let seat = s.current;
        Volltreffer::apply(s, seat, &Action::Hire { card }).unwrap();
    }
}

#[test]
fn setup_follows_the_table_size() {
    let s = new(4, 1);
    assert_eq!(s.scores, vec![25; 4]);
    assert_eq!(new(5, 1).scores, vec![30; 5]);
    assert_eq!(s.display.len(), 4);
    assert_eq!(s.deck.len(), 50 - 1 - 4);
    assert_eq!(s.current, s.bargaining);
    assert_eq!(pack().len(), 50);
}

#[test]
fn the_display_puts_trumps_dearest_and_later_ties_left() {
    let mut s = new(3, 2);
    s.blessing = c(Tribe::Green, 1);
    s.display = vec![
        Shown {
            card: c(Tribe::Red, 9),
            arrival: 0,
        },
        Shown {
            card: c(Tribe::Green, 2),
            arrival: 1,
        },
        Shown {
            card: c(Tribe::Blue, 5),
            arrival: 2,
        },
    ];
    s.arrivals = 3;
    s.deck.push(c(Tribe::Yellow, 5));
    // Through a hire of nothing: reveal goes through the public path.
    let hand_before = s.hands.clone();
    s.display.push(Shown {
        card: c(Tribe::Purple, 1),
        arrival: 3,
    });
    s.arrivals = 4;
    let seat = s.current;
    Volltreffer::apply(
        &mut s,
        seat,
        &Action::Hire {
            card: c(Tribe::Purple, 1),
        },
    )
    .unwrap();
    assert_ne!(s.hands, hand_before);
    let order: Vec<Card> = s.display.iter().map(|x| x.card).collect();
    // The new yellow 5 ties the blue 5 and goes left of it; the green
    // trump, though only a 2, is dearest.
    assert_eq!(
        order,
        vec![
            c(Tribe::Yellow, 5),
            c(Tribe::Blue, 5),
            c(Tribe::Red, 9),
            c(Tribe::Green, 2)
        ]
    );
}

#[test]
fn a_card_beyond_your_means_cannot_be_hired() {
    let mut s = new(3, 3);
    let seat = s.current;
    s.scores[seat.index()] = 1;
    let legal = Volltreffer::legal_actions(&s, seat);
    assert_eq!(legal.len(), 2);
    let dear = s.display[3].card;
    assert!(Volltreffer::apply(&mut s, seat, &Action::Hire { card: dear }).is_err());
    let cheap = s.display[1].card;
    Volltreffer::apply(&mut s, seat, &Action::Hire { card: cheap }).unwrap();
    assert_eq!(s.scores[seat.index()], 1 - COSTS[1]);
}

#[test]
fn hiring_moves_back_and_refills() {
    let mut s = new(3, 4);
    let seat = s.current;
    let card = s.display[3].card;
    Volltreffer::apply(&mut s, seat, &Action::Hire { card }).unwrap();
    assert_eq!(s.scores[seat.index()], 21);
    assert_eq!(s.display.len(), 4);
    assert_eq!(s.current, s.next(seat));
    assert_eq!(s.hands[seat.index()], vec![card]);
}

#[test]
fn eight_cards_each_then_publication() {
    let mut s = new(4, 5);
    hire_all_free(&mut s);
    assert_eq!(s.phase, Phase::Publish);
    assert!(s.hands.iter().all(|h| h.len() == 8));
    assert_eq!(s.current, s.bargaining);
}

#[test]
fn teleport_past_eighty() {
    assert_eq!(gain(78, 5), 43);
    assert_eq!(gain(80, 1), 41);
    assert_eq!(gain(75, 5), 80);
    assert_eq!(gain(60, 25), 45);
}

fn at_publication(hands: [Vec<Card>; 3]) -> State {
    let mut s = new(3, 6);
    hire_all_free(&mut s);
    s.hands = hands.to_vec();
    for h in &mut s.hands {
        h.sort();
    }
    s.scores = vec![20; 3];
    s
}

#[test]
fn groups_score_their_full_size_and_five_tribes_score_five() {
    use Tribe::*;
    let a = vec![
        c(Red, 9),
        c(Green, 9),
        c(Blue, 9),
        c(Yellow, 9),
        c(Purple, 2),
        c(Red, 3),
        c(Green, 3),
        c(Blue, 3),
    ];
    let b = vec![
        c(Red, 1),
        c(Green, 1),
        c(Blue, 1),
        c(Yellow, 1),
        c(Purple, 1),
        c(Red, 4),
        c(Green, 4),
        c(Blue, 4),
    ];
    let z = vec![
        c(Red, 10),
        c(Green, 10),
        c(Blue, 8),
        c(Yellow, 8),
        c(Purple, 8),
        c(Red, 7),
        c(Green, 7),
        c(Blue, 6),
    ];
    let mut s = at_publication([a, b, z]);
    let first = s.current;
    let legal = Volltreffer::legal_actions(&s, first);
    // Groups of 3s and 9s, five tribes: 2 × 2 × 2 choices.
    assert_eq!(legal.len(), 8);
    let all = Action::Publish {
        groups: vec![3, 9],
        tribes: true,
    };
    assert!(legal.contains(&all));
    let before = s.scores[first.index()];
    Volltreffer::apply(&mut s, first, &all).unwrap();
    assert_eq!(s.scores[first.index()], before + 10 + 5 + 5);
    let view = Volltreffer::view(&s, Viewer::Seat(s.next(first)));
    assert!(view.seats[first.index()].publication.is_some());
}

#[test]
fn showing_is_optional_and_the_lowest_sum_takes_the_preemptive_action() {
    use Tribe::*;
    let low_a = vec![
        c(Red, 1),
        c(Green, 1),
        c(Blue, 2),
        c(Yellow, 2),
        c(Purple, 2),
        c(Red, 3),
        c(Green, 3),
        c(Blue, 3),
    ];
    // Same sum (17) but only one 1: loses the tie.
    let low_b = vec![
        c(Purple, 1),
        c(Yellow, 3),
        c(Red, 2),
        c(Green, 2),
        c(Blue, 1),
        c(Purple, 3),
        c(Yellow, 1),
        c(Red, 4),
    ];
    let high = vec![
        c(Red, 10),
        c(Green, 10),
        c(Blue, 8),
        c(Yellow, 8),
        c(Purple, 8),
        c(Red, 7),
        c(Green, 7),
        c(Blue, 6),
    ];
    let sum = |h: &Vec<Card>| h.iter().map(|c| u32::from(c.value)).sum::<u32>();
    assert_eq!(sum(&low_a), 17);
    assert_eq!(sum(&low_b), 17);
    let mut s = at_publication([low_a, low_b, high]);
    for _ in 0..3 {
        let seat = s.current;
        Volltreffer::apply(
            &mut s,
            seat,
            &Action::Publish {
                groups: vec![],
                tribes: false,
            },
        )
        .unwrap();
    }
    assert_eq!(s.phase, Phase::Tricks);
    // low_b has three 1s against low_a's two.
    assert_eq!(s.preemptive, Some(Seat(1)));
    assert_eq!(s.current, Seat(1));
    assert_eq!(s.scores, vec![20, 25, 20]);
    // Publications are hidden once the tricks start.
    let view = Volltreffer::view(&s, Viewer::Seat(Seat(0)));
    assert!(view.seats.iter().all(|h| h.publication.is_none()));
}

fn at_tricks(hands: [Vec<Card>; 3], trump: Tribe, leader: u8) -> State {
    let mut s = new(3, 7);
    hire_all_free(&mut s);
    s.hands = hands.to_vec();
    for h in &mut s.hands {
        h.sort();
    }
    s.blessing = c(trump, 1);
    s.phase = Phase::Tricks;
    s.current = Seat(leader);
    s.scores = vec![30; 3];
    s
}

#[test]
fn follow_the_tribe_and_trumps_win() {
    use Tribe::*;
    let mut s = at_tricks(
        [
            vec![c(Red, 5), c(Blue, 2)],
            vec![c(Red, 9), c(Green, 1)],
            vec![c(Green, 2), c(Blue, 10)],
        ],
        Green,
        0,
    );
    Volltreffer::apply(&mut s, Seat(0), &Action::Play { card: c(Red, 5) }).unwrap();
    // Seat 1 holds red: it must follow.
    assert_eq!(
        Volltreffer::legal_actions(&s, Seat(1)),
        vec![Action::Play { card: c(Red, 9) }]
    );
    Volltreffer::apply(&mut s, Seat(1), &Action::Play { card: c(Red, 9) }).unwrap();
    // Seat 2 has no red: anything goes, and a small trump wins.
    assert_eq!(Volltreffer::legal_actions(&s, Seat(2)).len(), 2);
    Volltreffer::apply(&mut s, Seat(2), &Action::Play { card: c(Green, 2) }).unwrap();
    assert_eq!(s.last_trick.as_ref().unwrap().winner, Seat(2));
    assert_eq!(s.scores, vec![30, 30, 35]);
    assert_eq!(s.current, Seat(2));
    assert!(s.voids[2][Red.index()]);
    // Without a trump, the highest card of the tribe led wins.
    Volltreffer::apply(&mut s, Seat(2), &Action::Play { card: c(Blue, 10) }).unwrap();
    Volltreffer::apply(&mut s, Seat(0), &Action::Play { card: c(Blue, 2) }).unwrap();
    Volltreffer::apply(&mut s, Seat(1), &Action::Play { card: c(Green, 1) }).unwrap();
    assert_eq!(s.last_trick.as_ref().unwrap().winner, Seat(1));
}

#[test]
fn a_round_ending_in_the_sanctuary_wins_closest_to_66() {
    use Tribe::*;
    let mut s = at_tricks(
        [vec![c(Red, 5)], vec![c(Red, 9)], vec![c(Blue, 2)]],
        Green,
        0,
    );
    s.scores = vec![64, 63, 68];
    for (seat, card) in [(0, c(Red, 5)), (1, c(Red, 9)), (2, c(Blue, 2))] {
        Volltreffer::apply(&mut s, Seat(seat), &Action::Play { card }).unwrap();
    }
    // Seat 1 takes the trick: 68. All three are 2 from 66; the higher
    // score wins, and 68 against 68 is a shared victory.
    assert_eq!(s.scores, vec![64, 68, 68]);
    assert_eq!(s.phase, Phase::Over);
    assert_eq!(
        Volltreffer::outcome(&s),
        Some(Outcome::Ranked(vec![2, 0, 0]))
    );
}

#[test]
fn closest_to_66_then_higher_score() {
    use Tribe::*;
    let mut s = at_tricks(
        [vec![c(Red, 5)], vec![c(Red, 9)], vec![c(Blue, 2)]],
        Green,
        0,
    );
    s.scores = vec![64, 40, 63];
    for (seat, card) in [(0, c(Red, 5)), (1, c(Red, 9)), (2, c(Blue, 2))] {
        Volltreffer::apply(&mut s, Seat(seat), &Action::Play { card }).unwrap();
    }
    // Seat 1 takes the trick but stays far: 64 is closer to 66 than 63.
    assert_eq!(s.scores, vec![64, 45, 63]);
    assert_eq!(
        Volltreffer::outcome(&s),
        Some(Outcome::Ranked(vec![0, 2, 1]))
    );
}

#[test]
fn no_one_in_the_sanctuary_starts_a_new_round() {
    use Tribe::*;
    let mut s = at_tricks(
        [vec![c(Red, 5)], vec![c(Red, 9)], vec![c(Blue, 2)]],
        Green,
        0,
    );
    let holder = s.bargaining;
    s.scores = vec![30, 72, 50];
    for (seat, card) in [(0, c(Red, 5)), (1, c(Red, 9)), (2, c(Blue, 2))] {
        Volltreffer::apply(&mut s, Seat(seat), &Action::Play { card }).unwrap();
    }
    assert_eq!(s.phase, Phase::Hire);
    assert_eq!(s.round, 1);
    assert_eq!(s.scores, vec![30, 77, 50]);
    assert_eq!(s.bargaining, s.next(holder));
    assert_eq!(s.current, s.bargaining);
    assert!(s.hands.iter().all(Vec::is_empty));
}

#[test]
fn the_narrow_sanctuary_is_63_to_69() {
    let config = Config {
        narrow_sanctuary: true,
    };
    assert!(!config.sanctuary().contains(&62));
    assert!(config.sanctuary().contains(&69));
    assert!(!config.sanctuary().contains(&70));
}

#[test]
fn determinize_keeps_recent_hires_and_voids() {
    let mut s = new(3, 8);
    for _ in 0..5 {
        let seat = s.current;
        let card = s.display[0].card;
        Volltreffer::apply(&mut s, seat, &Action::Hire { card }).unwrap();
    }
    let observer = Seat(0);
    let view = Volltreffer::view(&s, Viewer::Seat(observer));
    let recent = s.recent_hires().to_vec();
    let mut changed = false;
    for seed in 0..50 {
        let d = Volltreffer::determinize(&s, observer, &mut SmallRng::seed_from_u64(seed));
        assert_eq!(Volltreffer::view(&d, Viewer::Seat(observer)), view);
        for hire in &recent {
            assert!(d.hands[hire.seat.index()].contains(&hire.card));
        }
        changed |= d.hands != s.hands || d.deck != s.deck;
    }
    assert!(changed);
}

#[test]
fn the_deal_of_a_seed_is_pinned() {
    let s = new(4, 42);
    assert_eq!(s, new(4, 42));
    // Pinned: a change of shuffle would make browser logs unreadable.
    let shown: Vec<Card> = s.display.iter().map(|x| x.card).collect();
    use Tribe::*;
    assert_eq!(s.blessing, c(Blue, 9));
    assert_eq!(
        shown,
        vec![c(Yellow, 1), c(Red, 1), c(Purple, 1), c(Green, 5)]
    );
}

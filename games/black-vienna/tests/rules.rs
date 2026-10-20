use black_vienna::{deal, hand_sizes, suspects, CardId, Suspect, INVESTIGATIONS};
use ludo_core::Seat;

#[test]
fn every_suspect_is_on_four_investigations() {
    assert_eq!(INVESTIGATIONS.len(), 36);
    let mut seen = [0u8; 27];
    for id in 0..36u8 {
        let cards = suspects(CardId::new(id).unwrap());
        assert!(cards[0] != cards[1] && cards[1] != cards[2] && cards[0] != cards[2]);
        let letters: String = cards.iter().map(|s| s.letter()).collect();
        assert_eq!(letters, INVESTIGATIONS[usize::from(id)]);
        for s in cards {
            seen[s.index()] += 1;
        }
    }
    assert_eq!(seen, [4; 27]);
    assert!(CardId::new(36).is_none());
}

#[test]
fn hands_by_table_size() {
    assert_eq!(hand_sizes(3, Seat(0)), vec![8, 8, 8]);
    assert_eq!(hand_sizes(4, Seat(1)), vec![6; 4]);
    assert_eq!(hand_sizes(6, Seat(5)), vec![4; 6]);
    // [R] At five, the dealer gets four.
    assert_eq!(hand_sizes(5, Seat(2)), vec![5, 5, 4, 5, 5]);
}

#[test]
fn the_deal_splits_everything() {
    for players in 3..=6u8 {
        for seed in 0..10 {
            let dealer = Seat((seed % u64::from(players)) as u8);
            let d = deal(players, dealer, seed);
            let mut all: Vec<Suspect> = d.trio.to_vec();
            for (hand, size) in d.hands.iter().zip(hand_sizes(players, dealer)) {
                assert_eq!(hand.len(), size);
                all.extend(hand);
            }
            all.sort();
            all.dedup();
            assert_eq!(all.len(), 27);
            assert!(d.trio.windows(2).all(|w| w[0] < w[1]), "trio sorted");
            let mut cards: Vec<CardId> = d.piles.iter().flatten().copied().collect();
            assert!(d.piles.iter().all(|p| p.len() == 12));
            cards.sort();
            cards.dedup();
            assert_eq!(cards.len(), 36);
        }
    }
}

#[test]
fn suspects_serialize_as_letters() {
    let o = Suspect::new(26).unwrap();
    assert_eq!(o.letter(), 'Ö');
    assert_eq!(serde_json::to_string(&o).unwrap(), "\"Ö\"");
    assert_eq!(
        serde_json::from_str::<Suspect>("\"B\"").unwrap(),
        Suspect::new(1).unwrap()
    );
    assert!(serde_json::from_str::<Suspect>("\"0\"").is_err());
    assert!(serde_json::from_str::<Suspect>("\"ß\"").is_err());
    assert!(serde_json::from_str::<Suspect>("\"AB\"").is_err());
    assert!(Suspect::new(27).is_none());
}

#[test]
fn the_deal_of_a_seed_is_pinned() {
    let d = deal(4, Seat(0), 7);
    let trio: String = d.trio.iter().map(|s| s.letter()).collect();
    let hand: String = d.hands[0].iter().map(|s| s.letter()).collect();
    assert_eq!((trio.as_str(), hand.as_str()), ("CPÖ", "FGMSYZ"));
}

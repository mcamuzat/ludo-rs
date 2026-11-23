use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use wu_hsing::{deal, is_double, values, DOMINOES};

#[test]
fn the_deck_has_two_copies_of_fifteen_dominoes() {
    assert_eq!(DOMINOES, 30);
    let mut seen = std::collections::HashMap::new();
    for id in 0..DOMINOES as u8 {
        let (a, b) = values(id);
        assert!((1..=5).contains(&a) && a <= b && b <= 5, "{id}: {a}-{b}");
        *seen.entry((a, b)).or_insert(0) += 1;
    }
    assert_eq!(seen.len(), 15);
    assert!(seen.values().all(|&n| n == 2));
    assert!(is_double(0) && !is_double(1));
}

#[test]
fn deal_sizes_follow_the_rule() {
    for (players, size) in [(2u8, 14usize), (3, 9), (4, 7)] {
        let d = deal(players, &mut ChaCha8Rng::seed_from_u64(u64::from(players)));
        assert_eq!(d.hands.len(), usize::from(players));
        assert!(d.hands.iter().all(|h| h.len() == size));
        assert_eq!(d.discarded.is_some(), players == 3);
        let mut all: Vec<u8> = d.hands.concat();
        all.extend(d.starters);
        all.extend(d.discarded);
        all.sort_unstable();
        assert_eq!(all, (0..30).collect::<Vec<u8>>());
        assert!(d.hands.iter().all(|h| h.windows(2).all(|w| w[0] < w[1])));
    }
}

#[test]
fn no_misdeal_is_ever_dealt() {
    for seed in 0..2000 {
        let d = deal(2, &mut ChaCha8Rng::seed_from_u64(seed));
        let [x, y] = d.starters;
        assert!(
            !(is_double(x) && is_double(y) && values(x) == values(y)),
            "seed {seed}"
        );
    }
}

#[test]
fn the_deal_of_a_seed_is_pinned() {
    let d = deal(2, &mut ChaCha8Rng::seed_from_u64(7));
    assert_eq!(format!("{:?} {:?}", d.hands, d.starters), "[[0, 2, 8, 9, 10, 11, 12, 14, 16, 18, 20, 21, 26, 27], [1, 3, 5, 6, 7, 13, 15, 17, 19, 22, 23, 24, 25, 28]] [29, 4]");
}

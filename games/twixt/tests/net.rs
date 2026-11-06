//! The Rust network against PyTorch (fixtures from scripts/twixt-export.py).

use std::time::Instant;

use serde::Deserialize;
use twixt::encode::{encode, policy_index_point, policy_point_index, Inputs};
use twixt::net::Net;
use twixt::{replay, Color, Point, TwixtAction};

#[derive(Deserialize)]
struct Fixture {
    positions: Vec<Position>,
}

#[derive(Deserialize)]
struct Position {
    moves: Vec<String>,
    pegs: Vec<f32>,
    links: Vec<f32>,
    policy: Vec<f32>,
    value: Vec<f32>,
}

fn fixture() -> Fixture {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/net-ref.json"
    ))
    .unwrap();
    serde_json::from_str(&text).unwrap()
}

fn weights() -> Vec<u8> {
    std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../web/src/engine/twixt-net-f16.bin"
    ))
    .unwrap()
}

/// twixtbot's notation: column letter from `a`, then row from 1.
fn action(name: &str) -> TwixtAction {
    if name == "swap" {
        return TwixtAction::Swap;
    }
    let x = name.as_bytes()[0].to_ascii_lowercase() - b'a';
    let y: u8 = name[1..].parse::<u8>().unwrap() - 1;
    TwixtAction::Place(Point { x, y })
}

fn moves(p: &Position) -> Vec<TwixtAction> {
    p.moves.iter().map(|m| action(m)).collect()
}

#[test]
fn encoding_matches_twixtbot() {
    for (i, p) in fixture().positions.iter().enumerate() {
        let inputs = encode(&replay(&moves(p)).unwrap());
        let mut pegs = inputs.pegs_hwc();
        let mut links = inputs.links_hwc();
        pegs.iter_mut()
            .chain(links.iter_mut())
            .for_each(|v| *v = v.round());
        assert_eq!(pegs, p.pegs, "position {i}: pegs");
        assert_eq!(links, p.links, "position {i}: links");
    }
}

#[test]
fn net_matches_pytorch() {
    let net = Net::from_bytes(&weights()).unwrap();
    for (i, p) in fixture().positions.iter().enumerate() {
        let eval = net.eval(&Inputs::from_hwc(&p.pegs, &p.links));
        for (k, (a, b)) in eval.policy.iter().zip(&p.policy).enumerate() {
            assert!((a - b).abs() < 1e-3, "position {i}, policy {k}: {a} vs {b}");
        }
        for (k, (a, b)) in eval.value.iter().zip(&p.value).enumerate() {
            assert!((a - b).abs() < 1e-3, "position {i}, value {k}: {a} vs {b}");
        }
    }
}

#[test]
fn policy_index_round_trips() {
    for c in [Color::Red, Color::Black] {
        for i in 0..528 {
            assert_eq!(policy_point_index(c, policy_index_point(c, i)), i);
        }
    }
    assert_eq!(policy_index_point(Color::Red, 0), Point { x: 1, y: 0 });
    assert_eq!(policy_index_point(Color::Black, 0), Point { x: 0, y: 1 });
}

#[test]
fn bad_weights_are_refused() {
    assert!(Net::from_bytes(b"nope").is_err());
    let mut w = weights();
    w.truncate(w.len() - 2);
    assert!(Net::from_bytes(&w).is_err());
}

#[test]
#[ignore]
fn eval_speed() {
    let net = Net::from_bytes(&weights()).unwrap();
    let f = fixture();
    let inputs = Inputs::from_hwc(&f.positions[3].pegs, &f.positions[3].links);
    net.eval(&inputs);
    let start = Instant::now();
    for _ in 0..20 {
        net.eval(&inputs);
    }
    println!(
        "{:.1} ms per evaluation",
        start.elapsed().as_secs_f64() * 1000.0 / 20.0
    );
}

//! Network-guided search, after [TB] `nnmcts.py`: PUCT selection with the
//! policy as prior, the value head at the leaves, proven wins, losses and
//! draws taken out of the selection.

use ludo_core::ai::Budget;

use crate::board::{Board, Point};
use crate::encode::{encode, policy_index_point, score, POLICY};
use crate::net::Net;
use crate::{Color, TwixtAction, TwixtState};

const CPUCT: f32 = 1.0;

/// The legal holes for the side to move with the network's probabilities
/// (softmax over the legal holes only).
pub fn policy(net: &Net, board: &Board) -> Vec<(Point, f32)> {
    let logits = net.eval(&encode(board)).policy;
    softmax_legal(board, &logits)
}

fn softmax_legal(board: &Board, logits: &[f32]) -> Vec<(Point, f32)> {
    let legal: Vec<(Point, f32)> = (0..POLICY)
        .map(|i| (policy_index_point(board.turn, i), logits[i]))
        .filter(|&(p, _)| board.legal(p))
        .collect();
    let max = legal.iter().map(|&(_, l)| l).fold(f32::MIN, f32::max);
    let sum: f32 = legal.iter().map(|&(_, l)| (l - max).exp()).sum();
    legal
        .into_iter()
        .map(|(p, l)| (p, (l - max).exp() / sum))
        .collect()
}

pub struct SearchResult {
    pub best: TwixtAction,
    /// Visits of each move searched, most visited first.
    pub visits: Vec<(TwixtAction, u32)>,
    /// The network's estimate at the root, p(win) − p(loss) in [−1, 1],
    /// for the side to move.
    pub root_score: f32,
}

struct Node {
    turn: Color,
    prior: Vec<f32>,
    n: Vec<u32>,
    q: Vec<f32>,
    /// Still selectable: legal and not proven.
    open: Vec<bool>,
    child: Vec<Option<usize>>,
    /// For the side to move here.
    score: f32,
    proven: bool,
    winning: Option<usize>,
    drawing: bool,
}

struct Tree<'a> {
    net: &'a Net,
    nodes: Vec<Node>,
}

impl Tree<'_> {
    fn expand(&mut self, s: &TwixtState) -> usize {
        let turn = s.board.turn;
        let mut node = Node {
            turn,
            prior: vec![0.0; POLICY],
            n: vec![0; POLICY],
            q: vec![0.0; POLICY],
            open: vec![false; POLICY],
            child: vec![None; POLICY],
            score: 0.0,
            proven: false,
            winning: None,
            drawing: false,
        };
        match s.result {
            // The player who just moved won.
            Some(Some(_)) => {
                node.proven = true;
                node.score = -1.0;
            }
            Some(None) => node.proven = true,
            None => {
                let eval = self.net.eval(&encode(&s.board));
                node.score = score(&eval.value);
                for (p, prob) in softmax_legal(&s.board, &eval.policy) {
                    let i = crate::encode::policy_point_index(turn, p);
                    node.prior[i] = prob;
                    node.open[i] = true;
                }
                if !node.open.iter().any(|&o| o) {
                    node.proven = true;
                }
            }
        }
        self.nodes.push(node);
        self.nodes.len() - 1
    }

    /// One simulation from `id`; the result for the side to move at `id`.
    fn visit(&mut self, id: usize, s: &TwixtState) -> f32 {
        let node = &self.nodes[id];
        if !node.open.iter().any(|&o| o) {
            // Every move proven: a draw if one draws, else all lose.
            let v = if node.drawing { 0.0 } else { -1.0 };
            self.nodes[id].proven = true;
            self.nodes[id].score = v;
            return v;
        }
        let total: u32 = node.n.iter().sum();
        let stv = (total as f32 + 1.0).sqrt();
        let index = (0..POLICY)
            .filter(|&i| node.open[i])
            .max_by(|&a, &b| {
                let u =
                    |i: usize| node.q[i] + CPUCT * node.prior[i] * stv / (1.0 + node.n[i] as f32);
                u(a).total_cmp(&u(b))
            })
            .expect("an open move");
        let mut next = s.clone();
        next.play(&TwixtAction::Place(policy_index_point(node.turn, index)))
            .expect("open moves are legal");
        let (sub, child_proven) = match node.child[index] {
            Some(c) => {
                let v = -self.visit(c, &next);
                (v, self.nodes[c].proven)
            }
            None => {
                let c = self.expand(&next);
                self.nodes[id].child[index] = Some(c);
                (-self.nodes[c].score, self.nodes[c].proven)
            }
        };
        let node = &mut self.nodes[id];
        node.n[index] += 1;
        if child_proven {
            node.q[index] = sub;
            node.open[index] = false;
            if sub >= 1.0 {
                node.proven = true;
                node.winning = Some(index);
                node.score = 1.0;
            } else if sub == 0.0 {
                node.drawing = true;
            }
        } else {
            node.q[index] += (sub - node.q[index]) / node.n[index] as f32;
        }
        sub
    }
}

/// Searches the position after `s` for the side to move within `budget`.
pub fn mcts(net: &Net, s: &TwixtState, budget: Budget, now_ms: &dyn Fn() -> f64) -> SearchResult {
    let mut tree = Tree {
        net,
        nodes: Vec::new(),
    };
    let root = tree.expand(s);
    let start = now_ms();
    let mut done = 0u32;
    while !tree.nodes[root].proven {
        let more = match budget {
            Budget::Iterations(n) => done < n,
            Budget::Millis(ms) => done == 0 || now_ms() - start < f64::from(ms),
        };
        if !more {
            break;
        }
        tree.visit(root, s);
        done += 1;
    }
    let node = &tree.nodes[root];
    let mut visits: Vec<(usize, u32)> = (0..POLICY)
        .filter(|&i| node.n[i] > 0)
        .map(|i| (i, node.n[i]))
        .collect();
    visits.sort_by_key(|&(_, n)| std::cmp::Reverse(n));
    let best = node
        .winning
        .or_else(|| {
            // Most visited among moves not proven lost.
            (0..POLICY)
                .filter(|&i| node.n[i] > 0 && (node.open[i] || node.q[i] > -1.0))
                .max_by_key(|&i| node.n[i])
        })
        .or_else(|| visits.first().map(|&(i, _)| i))
        .or_else(|| {
            (0..POLICY)
                .filter(|&i| node.prior[i] > 0.0)
                .max_by(|&a, &b| node.prior[a].total_cmp(&node.prior[b]))
        })
        .map(|i| policy_index_point(node.turn, i))
        .or_else(|| crate::all_points().find(|&p| s.board.legal(p)))
        .expect("a legal hole");
    SearchResult {
        best: TwixtAction::Place(best),
        visits: visits
            .into_iter()
            .map(|(i, n)| (TwixtAction::Place(policy_index_point(node.turn, i)), n))
            .collect(),
        root_score: node.score,
    }
}

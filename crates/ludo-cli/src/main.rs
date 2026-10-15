//! `ludo list` | `ludo arena <jeu> <parties> <niveau A> <niveau B> [joueurs] [--config '<json>']` | `ludo play <jeu> <joueurs> <niveau>`

use std::io::{self, BufRead, Write};
use std::str::FromStr;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use ludo_core::ai::Level;
use ludo_core::{AnyGame, Seat, Viewer};
use serde_json::Value;

const USAGE: &str = "usage :
  ludo list
  ludo arena <jeu> <parties> <easy|medium|hard> <easy|medium|hard> [joueurs] [--config '<json>']
  ludo play <jeu> <joueurs> <easy|medium|hard>";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(e) = run(&args) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn run(args: &[String]) -> Result<(), String> {
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        ["list"] => {
            for game in ludo_registry::GAMES {
                let m = game.meta();
                println!(
                    "{}\t{}\t{}–{} joueurs",
                    m.id, m.name, m.min_players, m.max_players
                );
            }
            Ok(())
        }
        ["arena", game, games, a, b, rest @ ..] => {
            let (players, config) = arena_extras(rest)?;
            arena(game, parse(games)?, level(a)?, level(b)?, players, &config)
        }
        ["play", game, players, lvl] => play(game, parse(players)?, level(lvl)?),
        _ => Err(USAGE.into()),
    }
}

fn parse<T: FromStr>(s: &str) -> Result<T, String> {
    s.parse()
        .map_err(|_| format!("valeur invalide : {s}\n{USAGE}"))
}

fn level(s: &str) -> Result<Level, String> {
    match s {
        "easy" => Ok(Level::Easy),
        "medium" => Ok(Level::Medium),
        "hard" => Ok(Level::Hard),
        _ => Err(format!("niveau inconnu : {s}\n{USAGE}")),
    }
}

/// `[joueurs] [--config '<json>']`, in any order; 2 players and the default
/// config when absent.
fn arena_extras(rest: &[&str]) -> Result<(u8, Value), String> {
    let mut players = 2;
    let mut config = Value::Null;
    let mut args = rest.iter();
    while let Some(&arg) = args.next() {
        if arg == "--config" {
            let text = args
                .next()
                .ok_or_else(|| format!("--config attend un objet JSON\n{USAGE}"))?;
            config = serde_json::from_str(text).map_err(|e| format!("config invalide : {e}"))?;
        } else {
            players = parse(arg)?;
        }
    }
    Ok((players, config))
}

fn game(id: &str) -> Result<&'static dyn AnyGame, String> {
    ludo_registry::find(id).map_err(|e| e.to_string())
}

/// Level A against level B; A takes each seat in turn. Prints A's mean reward
/// (1 = always first, 0.5 = even).
fn arena(
    id: &str,
    games: u32,
    a: Level,
    b: Level,
    players: u8,
    config: &Value,
) -> Result<(), String> {
    let game = game(id)?;
    let score = |g: u32| -> Result<f64, String> {
        let a_seat = Seat((g % u32::from(players)) as u8);
        let mut m = game
            .new_match(config.clone(), players, u64::from(g))
            .map_err(|e| e.to_string())?;
        while let Some(&seat) = m.pending().first() {
            let lvl = if seat == a_seat { a } else { b };
            let action = game
                .ai_move(
                    m.state_json(),
                    seat,
                    lvl,
                    false,
                    (u64::from(g) << 32) | u64::from(m.rev()),
                    &|| 0.0,
                )
                .map_err(|e| e.to_string())?;
            m.apply_json(seat, action).map_err(|e| e.to_string())?;
        }
        Ok(m.outcome().expect("finished").rewards()[a_seat.index()])
    };
    // Games are independent and seeded by their index: play them in parallel.
    let workers = std::thread::available_parallelism().map_or(1, |n| n.get()) as u32;
    let done = std::sync::atomic::AtomicU32::new(0);
    let total = std::sync::Mutex::new(0.0);
    std::thread::scope(|scope| -> Result<(), String> {
        let handles: Vec<_> = (0..workers.min(games))
            .map(|w| {
                let (score, done, total) = (&score, &done, &total);
                scope.spawn(move || -> Result<(), String> {
                    for g in (w..games).step_by(workers as usize) {
                        let r = score(g)?;
                        let mut t = total.lock().expect("no poisoning");
                        *t += r;
                        let n = done.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                        eprint!("\r{n}/{games}  score A = {:.3}", *t / f64::from(n));
                    }
                    Ok(())
                })
            })
            .collect();
        handles
            .into_iter()
            .try_for_each(|h| h.join().expect("arena worker panicked"))
    })?;
    let total = total.into_inner().expect("no poisoning");
    eprintln!();
    println!(
        "{id} à {players} : {a:?} contre {b:?}, {games} parties, score A = {:.3}",
        total / f64::from(games)
    );
    Ok(())
}

/// Seat 0 is you (actions chosen by number); every other seat is the AI.
fn play(id: &str, players: u8, lvl: Level) -> Result<(), String> {
    let game = game(id)?;
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos() as u64);
    let mut m = game
        .new_match(Value::Null, players, seed)
        .map_err(|e| e.to_string())?;
    let clock = Instant::now();
    let now = move || clock.elapsed().as_secs_f64() * 1000.0;
    let me = Seat(0);
    let mut input = io::stdin().lock().lines();
    while let Some(&seat) = m.pending().first() {
        if seat == me {
            println!(
                "{}",
                serde_json::to_string_pretty(&m.view_json(Viewer::Seat(me))).expect("json")
            );
            let legal = m
                .legal_actions_json(me)
                .as_array()
                .cloned()
                .unwrap_or_default();
            for (i, action) in legal.iter().enumerate() {
                println!("  {i}: {action}");
            }
            print!("ton coup > ");
            io::stdout().flush().map_err(|e| e.to_string())?;
            let line = input
                .next()
                .ok_or("entrée fermée")?
                .map_err(|e| e.to_string())?;
            let Some(action) = line
                .trim()
                .parse::<usize>()
                .ok()
                .and_then(|i| legal.get(i).cloned())
            else {
                println!("numéro invalide");
                continue;
            };
            m.apply_json(me, action).map_err(|e| e.to_string())?;
        } else {
            let action = game
                .ai_move(
                    m.state_json(),
                    seat,
                    lvl,
                    true,
                    seed ^ u64::from(m.rev()),
                    &now,
                )
                .map_err(|e| e.to_string())?;
            println!("siège {} joue {action}", seat.0);
            m.apply_json(seat, action).map_err(|e| e.to_string())?;
        }
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&m.view_json(Viewer::Spectator)).expect("json")
    );
    println!("classement : {:?}", m.outcome());
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::arena_extras;

    #[test]
    fn arena_reads_players_and_config_in_any_order() {
        assert_eq!(arena_extras(&[]).unwrap(), (2, serde_json::Value::Null));
        assert_eq!(arena_extras(&["4"]).unwrap(), (4, serde_json::Value::Null));
        assert_eq!(
            arena_extras(&["--config", r#"{"hands":3}"#, "3"]).unwrap(),
            (3, json!({ "hands": 3 }))
        );
        assert!(arena_extras(&["--config"]).is_err());
        assert!(arena_extras(&["--config", "{"]).is_err());
    }
}

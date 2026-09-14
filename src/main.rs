mod engine;
mod transition_table;
mod constants;
mod tests;
use shakmaty::Color;
use vampirc_uci::{parse_one, UciMessage};
use std::io::{self, BufRead, Write};
use ::fishbait_engine::Fishbait;

fn main() {
    let stdin = io::stdin();
    let mut fishbait = None;

    for line in stdin.lock().lines() {
        let msg = parse_one(&line.unwrap());
        match msg {
            UciMessage::Uci => {
                println!("id name Fishbait");
                println!("id author Gianmaria Vasino");
                println!("uciok");
                io::stdout().flush().unwrap();
            }
            UciMessage::IsReady => {
                println!("readyok");
                io::stdout().flush().unwrap();
            }
            UciMessage::Position { startpos, fen, moves } => {
                if fishbait.is_none() || startpos {
                    // create new engine
                    let color = if moves.len() % 2 == 0 {
                        Color::White
                    } else {
                        Color::Black
                    };
                    fishbait = Some(if startpos {
                        Fishbait::new(color, 8)
                    } else if let Some(f) = fen {
                        Fishbait::from_fen(&f.to_string(), color, 8).unwrap()
                    } else {
                        Fishbait::new(color, 8)
                    });
                }

                if let Some(ref mut e) = fishbait {
                    let already_played = e.move_count();
                    for mv in moves.iter().skip(already_played) {
                        e.make_move_uci(&mv.to_string()).unwrap();
                    }
                }
            }
            UciMessage::Go { .. } => {
                if let Some(ref mut e) = fishbait {
                    if let Some(result) = e.search() {
                        let uci_move = shakmaty::uci::UciMove::from_move(
                            result.best_move,
                            shakmaty::CastlingMode::Standard
                        );
                        println!("bestmove {}", uci_move);
                        io::stdout().flush().unwrap();
                    }
                }
            }
            UciMessage::Quit => break,
            _ => {}
        }
    }
}
mod engine;
mod transition_table;
mod constants;

use std::env;
use shakmaty::Color;
use vampirc_uci::{parse_one, UciMessage, UciTimeControl};
use std::io::{self, BufRead, Write};
use polyglot_book_rs::PolyglotBook;
use ::fishbait_engine::Fishbait;

fn main() {
    let stdin = io::stdin();
    let mut fishbait = None;
    let mut opening_book = None;

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

                    // load opening book
                    let mut book_path = env::current_exe().expect("Could not find executable path");
                    book_path.pop();
                    book_path.push("Perfect2023.bin");
                    let book_path = book_path.to_string_lossy().into_owned();

                    opening_book = match PolyglotBook::load(&book_path) {
                        Ok(book) => Some(book),
                        Err(e) => {
                            eprintln!("Failed to load opening book '{}': {}", book_path, e);
                            None
                        }
                    };

                    // determine color
                    let color = if moves.len() % 2 == 0 {
                        Color::White
                    } else {
                        Color::Black
                    };
                    // create new engine
                    fishbait = Some(if let Some(f) = fen {
                        Fishbait::from_fen(&f.to_string(), color, opening_book).unwrap()
                    } else {
                        Fishbait::new(color, opening_book)
                    });
                }

                if let Some(ref mut e) = fishbait {
                    let already_played = e.move_count();
                    for mv in moves.iter().skip(already_played) {
                        e.make_move_uci(&mv.to_string()).unwrap();
                    }
                }
            }
            UciMessage::Go { time_control, .. } => {
                let time_limit = match time_control {
                    Some(UciTimeControl::MoveTime(duration)) => duration,
                    Some(UciTimeControl::TimeLeft { white_time, black_time, .. }) => {
                        let t = if fishbait.as_ref().unwrap().get_color() == Color::White {
                            white_time
                        } else {
                            black_time
                        };
                        match t {
                            Some(t) => t / 15,
                            None => chrono::Duration::seconds(30),
                        }
                    }
                    _ => chrono::Duration::seconds(30),
                };
                if let Some(ref mut e) = fishbait {
                    let result = e.search(time_limit.to_std().unwrap(), true);
                    let uci_move = shakmaty::uci::UciMove::from_move(
                        result.best_move,
                        shakmaty::CastlingMode::Standard
                    );
                    println!("bestmove {}", uci_move);
                    io::stdout().flush().unwrap();
                }
            }
            UciMessage::Quit => break,
            _ => {}
        }
    }
}
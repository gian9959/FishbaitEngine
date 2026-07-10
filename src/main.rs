use std::time::{Duration, Instant};
use shakmaty::{Color, Position, Square, Role, File, Rank, Chess, Outcome};
use crate::rusty_engine::{Game, State};

mod rusty_engine;
mod constants;
mod transition_table;

fn print_board(board: &Chess) {
    let b = board.board();
    println!("  a b c d e f g h");
    println!("  ───────────────");
    for rank in (0..8).rev() {
        print!("{} │", rank + 1);
        for file in 0..8 {
            let sq = Square::from_coords(File::new(file), Rank::new(rank));
            let symbol = match b.piece_at(sq) {
                Some(piece) => match (piece.role, piece.color) {
                    (Role::King,   Color::White) => '♔',
                    (Role::Queen,  Color::White) => '♕',
                    (Role::Rook,   Color::White) => '♖',
                    (Role::Bishop, Color::White) => '♗',
                    (Role::Knight, Color::White) => '♘',
                    (Role::Pawn,   Color::White) => '♙',
                    (Role::King,   Color::Black) => '♚',
                    (Role::Queen,  Color::Black) => '♛',
                    (Role::Rook,   Color::Black) => '♜',
                    (Role::Bishop, Color::Black) => '♝',
                    (Role::Knight, Color::Black) => '♞',
                    (Role::Pawn,   Color::Black) => '♟',
                },
                None => '·',
            };
            print!("{} ", symbol);
        }
        println!();
    }
    println!("  ───────────────");
    println!("  a b c d e f g h");
}

fn turn(game: &mut Game, state: &mut State, tot_time: &mut Duration) -> bool {
    match game.current().outcome() {
        Outcome::Known(result) => { println!("GAME OVER: {}", result); return false; },
        Outcome::Unknown => {   if game.is_threefold_repetition() {
                                    println!("GAME OVER: 0-0 by repetition");
                                    return false;
                                } else if game.maxed_moves() {
                                    println!("GAME OVER: 0-0 by move limit");
                                    return false;
                                }
                            }
    }

    state.set_game(game.clone());

    println!("{}'s turn", state.get_color());
    println!("Thinking...");
    let start_time = Instant::now();

    let m = match state.play() { None => {println!("NONE MOVE"); return false}, Some(x) => x };
    if game.push(m).is_err() { println!("PLAY ERROR"); return false };

    let delta_time = start_time.elapsed();
    *tot_time += delta_time;

    println!("{}", m);
    print_board(game.current());

    println!("Best score predicted: {}", state.get_best_score());

    println!("Turn time: {}s", delta_time.as_secs());
    println!();

    true
}

fn main() {
    let mut game = Game::new();
    let mut avg_time = Duration::new(0, 0);

    let mut w_state = State::new(game.clone(), Color::White, 8);
    let mut b_state = State::new(game.clone(), Color::Black, 8);

    println!("Starting test game!");
    println!("Playing against myself");
    println!();
    loop {
        if !turn(&mut game, &mut w_state, &mut avg_time) { break }
        if !turn(&mut game, &mut b_state, &mut avg_time) { break }
    }
    println!("END");
    println!();
    println!("Average turn time: {}s", avg_time.as_secs()/game.len() as u64);

    let w_stats = w_state.get_stats();
    let b_stats = b_state.get_stats();
    println!("Average white iterations -> normal: {}, memo: {}", w_stats.0/game.len() as i32, w_stats.1/game.len() as i32);
    println!("White transitions table length: {}", w_stats.2);
    println!("Average black iterations -> normal: {}, memo: {}", b_stats.0/game.len() as i32, b_stats.1/game.len() as i32);
    println!("Black transitions table length: {}", b_stats.2);
}
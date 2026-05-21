use shakmaty::{Color, Position, Square, Role, File, Rank, Chess};
use crate::rusty_engine::{Game, State};

mod rusty_engine;
mod constants;

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

fn turn(c: Color, g: &mut Game, d: i32) -> bool {
    println!("{}'s turn", c);
    println!("Thinking...");

    let mut s = State::new(g.clone(), c, d);

    let m = match s.play() { None => {println!("NONE MOVE"); return false}, Some(x) => x };
    if g.push(m).is_err() { println!("PLAY ERROR"); return false };

    println!("{}", m);
    print_board(g.current());

    if(g.current().is_game_over()) { println!("GAME OVER"); return false };
    true
}

fn main() {
    let mut game = Game::new();

    println!("Starting test game!");
    println!("Playing against myself");
    loop {
        if !turn(Color::White, &mut game, 4) { break }
        if !turn(Color::Black, &mut game, 5) { break }
    }
    println!("END");
}

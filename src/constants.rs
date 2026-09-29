use shakmaty::{Role, Color, Square, Move, Board};

pub const BOOK: &[u8] = include_bytes!("Perfect2023.bin");
pub const TABLE_SIZE: usize = 10_000_000;
pub const INF: i32 = 100_000_000;

// depth reduction value of LMR
pub const LMR: i32 = 3;

// LMR first threshold
pub const LMR_T1: i32 = 2;

// LMR second threshold
pub const LMR_T2: i32 = 5;

// depth reduction value of NMP
pub const NMP: i32 = 3;

pub const PAWN_TABLE_WHITE: [i32; 64] = [
    0,  0,  0,  0,  0,  0,  0,  0,
    5, 10, 10,-20,-20, 10, 10,  5,
    5, -5,-10,  0,  0,-10, -5,  5,
    0,  0,  0, 20, 20,  0,  0,  0,
    5,  5, 10, 25, 25, 10,  5,  5,
    10, 10, 20, 30, 30, 20, 10, 10,
    50, 50, 50, 50, 50, 50, 50, 50,
    0,  0,  0,  0,  0,  0,  0,  0,
];
pub const PAWN_TABLE_BLACK: [i32; 64] = reverse(PAWN_TABLE_WHITE);

pub const KNIGHT_TABLE: [i32; 64] = [
    -50,-40,-30,-30,-30,-30,-40,-50,
    -40,-20,  0,  0,  0,  0,-20,-40,
    -30,  0, 10, 15, 15, 10,  0,-30,
    -30,  5, 15, 20, 20, 15,  5,-30,
    -30,  0, 15, 20, 20, 15,  0,-30,
    -30,  5, 10, 15, 15, 10,  5,-30,
    -40,-20,  0,  5,  5,  0,-20,-40,
    -50,-40,-30,-30,-30,-30,-40,-50,
];

pub const BISHOP_TABLE_WHITE: [i32; 64] = [
    -20,-10,-10,-10,-10,-10,-10,-20,
    -10,  5,  0,  0,  0,  0,  5,-10,
    -10, 10, 10, 10, 10, 10, 10,-10,
    -10,  0, 10, 10, 10, 10,  0,-10,
    -10,  5,  5, 10, 10,  5,  5,-10,
    -10,  0,  5, 10, 10,  5,  0,-10,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -20,-10,-10,-10,-10,-10,-10,-20,
];
pub const BISHOP_TABLE_BLACK: [i32; 64] = reverse(BISHOP_TABLE_WHITE);

pub const ROOK_TABLE_WHITE: [i32; 64] = [
    0,  0,  0,  5,  5,  0,  0,  0,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    5, 10, 10, 10, 10, 10, 10,  5,
    0,  0,  0,  0,  0,  0,  0,  0,
];
pub const ROOK_TABLE_BLACK: [i32; 64] = reverse(ROOK_TABLE_WHITE);

pub const QUEEN_TABLE: [i32; 64] = [
    -20,-10,-10, -5, -5,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5,  5,  5,  5,  0,-10,
    -5,  0,  5,  5,  5,  5,  0, -5,
    0,  0,  5,  5,  5,  5,  0, -5,
    -10,  5,  5,  5,  5,  5,  0,-10,
    -10,  0,  5,  0,  0,  0,  0,-10,
    -20,-10,-10, -5, -5,-10,-10,-20,
];

pub const KING_TABLE_WHITE: [i32; 64] = [
    20, 40, 10,  0,  0, 10, 40, 20,
    20, 20,  0,  0,  0,  0, 20, 20,
    -10,-20,-20,-20,-20,-20,-20,-10,
    -20,-30,-30,-40,-40,-30,-30,-20,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
];
pub const KING_TABLE_BLACK: [i32; 64] = reverse(KING_TABLE_WHITE);

pub const KING_ENDGAME_TABLE_WHITE: [i32; 64] = [
    -50,-30,-30,-30,-30,-30,-30,-50,
    -30,-30,  0,  0,  0,  0,-30,-30,
    -30,-10, 20, 30, 30, 20,-10,-30,
    -30,-10, 30, 40, 40, 30,-10,-30,
    -30,-10, 30, 40, 40, 30,-10,-30,
    -30,-10, 20, 30, 30, 20,-10,-30,
    -30,-20,-10,  0,  0,-10,-20,-30,
    -50,-40,-30,-20,-20,-30,-40,-50,
];
pub const KING_ENDGAME_TABLE_BLACK: [i32; 64] = reverse(KING_ENDGAME_TABLE_WHITE);

pub fn piece_square_value(role: Role, color: Color, square: Square, board: &Board, endgame: bool) -> i32 {
    let p_value = piece_value(role);

    let idx = square as usize;
    let s_value = match (role, color) {
        (Role::Pawn,   Color::White) => PAWN_TABLE_WHITE[idx] + passed_pawn_bonus(color, square, board) + doubled_pawn_penalty(color, square, board),
        (Role::Pawn,   Color::Black) => PAWN_TABLE_BLACK[idx] + passed_pawn_bonus(color, square, board) + doubled_pawn_penalty(color, square, board),
        (Role::Knight, _)            => KNIGHT_TABLE[idx],
        (Role::Bishop, Color::White) => BISHOP_TABLE_WHITE[idx],
        (Role::Bishop, Color::Black) => BISHOP_TABLE_BLACK[idx],
        (Role::Rook,   Color::White) => ROOK_TABLE_WHITE[idx],
        (Role::Rook,   Color::Black) => ROOK_TABLE_BLACK[idx],
        (Role::Queen,  _)            => QUEEN_TABLE[idx],
        (Role::King,   Color::White) => if endgame { KING_ENDGAME_TABLE_WHITE[idx] } else { KING_TABLE_WHITE[idx] },
        (Role::King,   Color::Black) => if endgame { KING_ENDGAME_TABLE_BLACK[idx] } else { KING_TABLE_BLACK[idx] },
    };

    p_value + s_value
}

pub const fn piece_value(role: Role) -> i32 {
    match role {
        Role::Pawn   => 100,
        Role::Knight => 320,
        Role::Bishop => 350,
        Role::Rook   => 500,
        Role::Queen  => 900,
        Role::King   => 0,
    }
}

fn passed_pawn_bonus(color: Color, square: Square, board: &Board) -> i32 {
    let their_pawns = board.by_color(!color).intersect(board.by_role(Role::Pawn));
    let mut score = 0;
    let mut passed = true;

    for p in their_pawns {
        let file_diff = square.file() as i32 - p.file() as i32;
        if file_diff == 0 {
            passed = false;
            break;
        } else if file_diff.abs() <= 1 {
            match color {
                Color::White => {
                    if square.rank() < p.rank() {
                        passed = false;
                        break;
                    }
                },
                Color::Black => {
                    if square.rank() > p.rank() {
                        passed = false;
                        break;
                    }
                }
            }
        }
    }
    if passed {
        match color {
            Color::White => {
                score += square.rank() as i32 * 10;
            },
            Color::Black => {
                score += (7 - square.rank() as i32) * 10;
            },
        }
    }
    score
}

fn doubled_pawn_penalty(color: Color, square: Square, board: &Board) -> i32 {
    let my_pawns = board.by_color(color).intersect(board.by_role(Role::Pawn));
    let mut score = 0;
    let mut doubled = false;

    for p in my_pawns {
        if square.file() == p.file() {
            match color {
                Color::White => {
                    if square.rank() < p.rank() {
                        doubled = true;
                        break;
                    }
                },
                Color::Black => {
                    if square.rank() > p.rank() {
                        doubled = true;
                        break;
                    }
                }
            }
        }
    }
    if doubled {
        score -= 30;
    }
    score
}

pub fn move_score(mv: &Move) -> i32 {
    match mv {
        Move::Normal { capture: Some(victim), role, .. } => {
            piece_value(*victim) * 10 - piece_value(*role)
        },
        Move::Castle { .. } => 20,
        Move::EnPassant { .. } => 10,
        _ => 0,
    }
}

pub const fn reverse(table: [i32; 64]) -> [i32; 64] {
    let mut result = [0; 64];
    let mut i = 0;
    while i < 64 {
        result[i] = table[63 - i];
        i += 1;
    }
    result
}
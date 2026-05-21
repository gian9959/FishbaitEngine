use shakmaty::{Chess, Position, Move, Color, MoveList, Role, PlayError};
use crate::constants::{piece_value, piece_square_value};

#[derive(Clone)]
pub struct Game {
    history: Vec<Chess>,
}

impl Game {
    pub fn new() -> Self {
        Game {
            history: vec![Chess::default()],
        }
    }

    pub fn from_pos(c: Chess) -> Self {
        Game {
            history: vec![c],
        }
    }

    pub fn current(&self) -> &Chess {
        self.history.last().unwrap()
    }

    pub fn push(&mut self, mv: Move) -> Result<(), PlayError<Chess>> {
        let next = self.current().clone().play(mv)?;
        self.history.push(next);
        Ok(())
    }

    pub fn pop(&mut self) -> Option<Chess> {
        if self.history.len() > 1 {
            self.history.pop()
        } else {
            None
        }
    }
}

pub struct State {
    game: Game,
    best_move: Option<Move>,
    color: Color,
    max_depth: i32,
}

impl State {
    pub fn new(g: Game, c: Color, d: i32) -> Self {
        State {
            game: g,
            best_move: None,
            color: c,
            max_depth: d
        }
    }

    pub fn play(&mut self) -> Option<Move> {
        let mut score = 0;
        for depth in 1..=self.max_depth {
            score = self.minimax(depth, i32::MIN, i32::MAX, true);
        }
        println!("Best score: {}", score);
        self.best_move
    }

    fn order_moves(&self, moves: MoveList) -> MoveList {
        let mut moves = moves;
        moves.sort_by_key(|m| {
            if Some(m) == self.best_move.as_ref() {
                return i32::MIN;
            }
            -self.move_score(m)
        });
        moves
    }

    fn move_score(&self, mv: &Move) -> i32 {
        match mv {
            Move::Normal { capture: Some(victim), role, .. } => {
                piece_value(*victim) * 10 - piece_value(*role)
            },
            Move::Castle { .. } => 20,
            Move::EnPassant { .. } => 10,
            _ => 0,
        }
    }

    fn eval(&self) -> i32 {
        let board = self.game.current().board();
        let mut score = 0;

        for color in [Color::White, Color::Black] {
            for role in [Role::Pawn, Role::Knight, Role::Bishop, Role::Rook, Role::Queen] {
                let mut pieces = board.by_color(color).intersect(board.by_role(role));
                while let Some(square) = pieces.pop_front() {
                    let value = piece_value(role) + piece_square_value(role, color, square);
                    if color == self.color {
                        score += value;
                    } else {
                        score -= value;
                    }
                }
            }
        }
        score
    }

    pub fn minimax(&mut self, depth: i32, mut alpha: i32, mut beta: i32, is_max: bool) -> i32 {
        let moves = self.game.current().legal_moves();
        let moves = self.order_moves(moves);

        // max depth reached
        if depth == 0 {
            return self.eval();
        }

        // end condition reached
        if moves.is_empty() {
            return if self.game.current().is_checkmate() {
                if is_max { i32::MIN } else { i32::MAX }
            } else {
                0
            }
        }

        let mut best_score = if is_max { i32::MIN } else { i32::MAX };

        for m in moves {
            // push move and explore down the tree
            if self.game.push(m).is_err() { continue; }
            let score = self.minimax(depth - 1, alpha, beta, !is_max);
            // pop move to go up the tre
            self.game.pop();

            let is_better = if is_max { score > best_score } else { score < best_score };
            // if it's near the root of the tree (the possible next move) save the best move
            if depth == self.max_depth && (is_better || self.best_move.is_none()) {
               self.best_move = Some(m);
            }

            if is_max {
                best_score = i32::max(best_score, score);
                alpha = i32::max(alpha, best_score);
                if best_score >= beta { break; }
            } else {
                best_score = i32::min(best_score, score);
                beta = i32::min(beta, best_score);
                if best_score <= alpha { break; }
            }
        }
        best_score
    }
}
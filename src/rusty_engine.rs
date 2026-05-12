use shakmaty::{Chess, Position, Move, Color, MoveList, Role, PlayError};

fn piece_value(role: Role) -> i32 {
    match role {
        Role::Pawn   => 100,
        Role::Knight => 320,
        Role::Bishop => 330,
        Role::Rook   => 500,
        Role::Queen  => 900,
        Role::King   => 0,
    }
}

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
    fn new(g: Game, c: Color, d: i32) -> Self {
        State {
            game: g,
            best_move: None,
            color: c,
            max_depth: d
        }
    }

    fn eval(&self) -> i32 {
        let board = self.game.current().board();
        let mut score = 0;

        for color in [Color::White, Color::Black] {
            for role in [Role::Pawn, Role::Knight, Role::Bishop, Role::Rook, Role::Queen] {
                let count = board.by_color(color).intersect(board.by_role(role)).count();
                let value = piece_value(role) * count as i32;
                if color == self.color {
                    score += value;
                } else {
                    score -= value;
                }
            }
        }
        score
    }

    fn minimax(&mut self, depth: i32, mut alpha: i32, mut beta: i32, is_max: bool) -> i32 {
        let moves = self.game.current().legal_moves();

        if depth == 0 {
            return self.eval();
        }

        if moves.is_empty() {
            return if self.game.current().is_checkmate() {
                if is_max { i32::MIN } else { i32::MAX }
            } else {
                0
            }
        }

        let mut best_score = if is_max { i32::MIN } else { i32::MAX };

        for m in moves {
            if self.game.push(m.clone()).is_err() { continue; }
            let score = self.minimax(depth - 1, alpha, beta, !is_max);
            self.game.pop();

            if depth == self.max_depth {
                let is_better = if is_max { score > best_score } else { score < best_score };

                if is_better {
                    self.best_move = Some(m);
                }
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
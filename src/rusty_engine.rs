use shakmaty::{Chess, Position, Move, Color, MoveList, Role, PlayError, EnPassantMode, zobrist::Zobrist64};
use crate::constants::{piece_value, piece_square_value, move_score};
use crate::transition_table::{TranspositionTable, EntryType, TTEntry};

#[derive(Clone)]
pub struct Game {
    history: Vec<Chess>,
    hashes: Vec<Zobrist64>,
}

impl Game {
    pub fn new() -> Self {
        Game {
            history: vec![Chess::default()],
            hashes: vec![Chess::default().zobrist_hash(EnPassantMode::Legal)],
        }
    }

    pub fn from_pos(c: Chess) -> Self {
        let hash: Zobrist64 = c.zobrist_hash(EnPassantMode::Legal);
        Game {
            history: vec![c],
            hashes: vec![hash],
        }
    }

    pub fn current(&self) -> &Chess {
        self.history.last().unwrap()
    }
    
    pub fn len(&self) -> usize {
        self.history.len()
    }

    pub fn push(&mut self, mv: Move) -> Result<(), PlayError<Chess>> {
        let next = self.current().clone().play(mv)?;
        let hash = next.zobrist_hash::<Zobrist64>(EnPassantMode::Legal);
        self.history.push(next);
        self.hashes.push(hash);
        Ok(())
    }

    pub fn pop(&mut self) -> Option<Chess> {
        if self.history.len() > 1 {
            self.hashes.pop();
            self.history.pop()
        } else {
            None
        }
    }

    pub fn is_threefold_repetition(&self) -> bool {
        let current_hash = self.hashes.last().unwrap();
        self.hashes.iter().filter(|h| *h == current_hash).count() >= 3
    }

    pub fn maxed_moves(&self) -> bool {
        self.history.len() > 100
    }
}

pub struct State {
    game: Game,
    best_move: Option<Move>,
    color: Color,
    max_depth: i32,
    hash_table: TranspositionTable,

    // stats
    iterations: i32,
    memo_iterations: i32,
    best_move_found: i32,
}

impl State {
    pub fn new(g: Game, c: Color, d: i32) -> Self {
        State {
            game: g,
            best_move: None,
            color: c,
            max_depth: d,
            hash_table: TranspositionTable::new(10000000),

            iterations: 0,
            memo_iterations: 0,
            best_move_found: 0,
        }
    }

    pub fn set_game(&mut self, game: Game) {
        self.game = game;
        self.best_move = None;
    }

    pub fn get_color(&self) -> Color {
        self.color
    }

    pub fn get_stats(&self) -> (i32, i32, i32) {
        (self.iterations, self.memo_iterations, self.best_move_found)
    }

    // with iterative deepening
    pub fn play(&mut self) -> Option<Move> {
        let mut score = 0;
        for depth in 1..=self.max_depth {
            score = self.minimax(depth, i32::MIN, i32::MAX, true);
        }
        println!("Best score: {}", score);
        self.best_move
    }

    fn order_moves(&mut self, moves: MoveList, local_best: Option<Move>) -> MoveList {
        let mut moves = moves;
        moves.sort_by_key(|m| {
            if Some(m) == local_best.as_ref() {
                self.best_move_found += 1;
                return i32::MIN;
            }
            -move_score(m)
        });
        moves
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
        let h: Zobrist64 = self.game.current().zobrist_hash(EnPassantMode::Legal);
        let mut h_move: Option<Move> = None;

        // memoization with Zobrist hashes
        if let Some(entry) = self.hash_table.get(h) {
            self.memo_iterations += 1;
            h_move = entry.best_move;
            if entry.depth >= depth {
                if depth == self.max_depth {
                    self.best_move = h_move;
                }
                match entry.entry_type {
                    EntryType::Exact => return entry.score,
                    EntryType::LowerBound => alpha = i32::max(alpha, entry.score),
                    EntryType::UpperBound => beta = i32::min(beta, entry.score),
                }
                if alpha >= beta {
                    return entry.score;
                }
            }
        }

        self.iterations += 1;

        // max depth reached
        if depth == 0 {
            return self.eval();
        }

        // draw by a special case
        if self.game.is_threefold_repetition() || self.game.maxed_moves() {
            return 0;
        }

        let moves = self.game.current().legal_moves();
        let moves = self.order_moves(moves, h_move);

        // end condition by no other moves
        if moves.is_empty() {
            return if self.game.current().is_checkmate() {
                if is_max { i32::MIN } else { i32::MAX }
            } else {
                0
            }
        }

        let mut best_score = if is_max { i32::MIN } else { i32::MAX };
        let mut local_best_move: Option<Move> = None;
        let original_alpha = alpha;
        let original_beta = beta;

        for m in moves {
            // push move and explore down the tree
            if self.game.push(m).is_err() { continue; }
            let score = self.minimax(depth - 1, alpha, beta, !is_max);
            // pop move to go up the tre
            self.game.pop();

            let is_better = if is_max { score > best_score } else { score < best_score };
            // if it's near the root of the tree (the possible next move) save the best move
            if is_better || local_best_move.is_none() {
                if depth == self.max_depth || self.best_move.is_none() {
                    self.best_move = Some(m.clone());
                }
                local_best_move = Some(m);
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

        // save in the hash table the position explored
        let et = if best_score <= original_alpha {
            EntryType::UpperBound
        } else if best_score >= original_beta {
            EntryType::LowerBound
        } else {
            EntryType::Exact
        };

        self.hash_table.insert(TTEntry {
            hash: h,
            depth: depth,
            score: best_score,
            entry_type: et,
            best_move: local_best_move,
        });

        best_score
    }
}
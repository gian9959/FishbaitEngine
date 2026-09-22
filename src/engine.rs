use std::io;
use std::io::Write;
use std::time::{Duration, Instant};
use shakmaty::{Chess, Position, Move, Color, MoveList, Role, PlayError, EnPassantMode, zobrist::Zobrist64};
use rand;
use polyglot_book_rs::PolyglotBook;
use crate::constants::{TABLE_SIZE, INF, piece_value, piece_square_value, move_score};
use crate::transition_table::{TranspositionTable, EntryType, TTEntry};


#[derive(Clone)]
pub struct Game {
    history: Vec<Chess>,
    hashes: Vec<Zobrist64>,
    max_moves: Option<i32>,
}

impl Game {
    pub fn new() -> Self {
        Game {
            history: vec![Chess::default()],
            hashes: vec![Chess::default().zobrist_hash(EnPassantMode::Legal)],
            max_moves: None,
        }
    }

    pub fn from_pos(c: Chess) -> Self {
        let hash: Zobrist64 = c.zobrist_hash(EnPassantMode::Legal);
        Game {
            history: vec![c],
            hashes: vec![hash],
            max_moves: None,
        }
    }

    pub fn set_max_moves(&mut self, max_moves: Option<i32>) {
        self.max_moves = max_moves;
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
        match self.max_moves {
            Some(max_moves) => self.history.len() >= max_moves as usize,
            None => false,
        }
    }

    pub fn is_endgame(&self) -> bool {
        let board = self.current().board();
        let mut total = 0;
        for role in [Role::Pawn, Role::Knight, Role::Bishop, Role::Rook, Role::Queen] {
            let pieces = board.by_role(role).count();
            total += piece_value(role) * pieces as i32;
        }
        total <= 3000
    }
}
#[derive(Clone)]
pub struct SearchResult {
    pub best_move: Move,
    pub score: i32,
}

pub struct Engine {
    pub game: Game,
    color: Color,
    max_depth: i32,
    quiescence_depth: i32,
    max_time: Duration,

    best_move: Option<Move>,
    best_score: i32,
    start_timer: Instant,
    hash_table: TranspositionTable,
    q_hash_table: TranspositionTable,
    silent_history: [[i32; 64]; 64],

    // stats
    iterations: i32,
    q_iterations: i32,
    memo_iterations: i32,
}

impl Engine {
    pub fn new(g: Game, c: Color) -> Self {
        Engine {
            game: g,
            color: c,
            max_depth: 20,
            quiescence_depth: 5,
            max_time: Duration::from_secs(30),

            best_move: None,
            best_score: 0,
            start_timer: Instant::now(),
            hash_table: TranspositionTable::new(TABLE_SIZE),
            q_hash_table: TranspositionTable::new(TABLE_SIZE),
            silent_history: [[0; 64]; 64],

            iterations: 0,
            q_iterations: 0,
            memo_iterations: 0,
        }
    }

    pub fn set_game(&mut self, game: Game) {
        self.game = game;
        if self.game.current().turn() != self.color {
            panic!("Turn of game does not match color!");
        }
        self.best_move = None;
        self.best_score = 0;
        //self.silent_history = [[0; 64]; 64];
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn set_max_depth(&mut self, max_depth: i32) {
        self.max_depth = max_depth;
    }

    pub fn set_quiescence_depth(&mut self, depth: i32) {
        self.quiescence_depth = depth;
    }

    pub fn set_timer(&mut self, max_time: Duration) {
        self.max_time = max_time;
    }

    pub fn get_color(&self) -> Color {
        self.color
    }

    pub fn get_best_score(&self) -> i32 {
        self.best_score
    }

    pub fn get_stats(&self) -> (i32, i32, i32, i32, i32) {
        (self.iterations, self.q_iterations, self.memo_iterations, self.hash_table.get_len(), self.q_hash_table.get_len())
    }

    pub fn is_timeout(&self) -> bool {
        self.start_timer.elapsed() >= self.max_time
    }

    fn extract_pv(&self, depth: i32) -> Vec<Move> {
        let mut pv = Vec::new();
        let mut game = self.game.clone();

        for _ in 0..depth {
            let h = game.current().zobrist_hash::<Zobrist64>(EnPassantMode::Legal);
            if let Some(entry) = self.hash_table.get(h) {
                if let Some(mv) = entry.best_move {
                    if game.current().legal_moves().contains(&mv) {
                        pv.push(mv);
                        game.push(mv).ok();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        pv
    }

    pub fn play(&mut self, opening_book: &Option<PolyglotBook>, print: bool) -> SearchResult {

        // search opening book
        if let Some(book) = opening_book {
            let fen = shakmaty::fen::Fen::from_position(
                self.game.current(),
                EnPassantMode::Legal
            ).to_string();

            let moves = book.get_all_moves_from_fen(&fen);
            if !moves.is_empty() {
                // select random move from book
                // probability based on "weight" of move in the book
                let tot_weight = moves.iter().map(|m| m.weight).sum();
                let mut choice = rand::random_range(0..tot_weight);

                for entry in moves {
                    if choice < entry.weight {
                        if let Ok(mv) = entry.move_string.parse::<shakmaty::uci::UciMove>() {
                            if let Ok(mv) = mv.to_move(self.game.current()) {
                                self.game.push(mv).ok();
                                let score = self.eval();
                                self.game.pop();
                                return SearchResult { best_move: mv, score: score };
                            }
                        }
                    }
                    choice -= entry.weight;
                }
            }
        }

        let mut prev_res: Vec<SearchResult> = vec![];

        // start move timer
        self.start_timer = Instant::now();

        // with iterative deepening
        for depth in 1..=self.max_depth {

            let score = if depth <= 2 {
                // first two iterations have a "full" window
                self.minimax(depth, -INF, INF)
            } else {
                // use aspiration window on later iterations
                let mut delta = 50;
                let mut alpha = prev_res[depth as usize - 2].score - delta;
                let mut beta = prev_res[depth as usize - 2].score + delta;

                loop {
                    let score = self.minimax(depth, alpha, beta);

                    match score {
                        None => { return prev_res[depth as usize - 2].clone() }
                        _ => {}
                    }

                    if score <= Some(alpha) {
                        alpha -= delta;
                        delta *= 2;
                    } else if score >= Some(beta) {
                        beta += delta;
                        delta *= 2;
                    } else {
                        break score;
                    }
                }
            };

            self.best_score = score.unwrap();

            prev_res.push(SearchResult {
                best_move: self.best_move.unwrap(),
                score: self.best_score,
            });

            let root_hash = self.game.current().zobrist_hash(EnPassantMode::Legal);
            if let Some(entry) = self.hash_table.get(root_hash) {
                self.best_move = entry.best_move;
            }

            if let Some(bm) = self.best_move && print {
                let pv = self.extract_pv(depth);
                let pv_str = pv.iter()
                    .map(|m| shakmaty::uci::UciMove::from_move(*m, shakmaty::CastlingMode::Standard).to_string())
                    .collect::<Vec<_>>()
                    .join(" ");

                println!("info depth {} score cp {} nodes {} time {} pv {}",
                    depth,
                    self.best_score,
                    self.iterations + self.q_iterations,
                    self.start_timer.elapsed().as_millis(),
                    pv_str
                );
                io::stdout().flush().unwrap();
            }
        }
        SearchResult {
            best_move: self.best_move.unwrap(),
            score: self.best_score,
        }
    }

    fn order_moves(&mut self, moves: MoveList, h_move: Option<Move>) -> MoveList {
        let mut moves = moves;
        moves.sort_by_key(|m| {
            if Some(m) == h_move.as_ref() {
                return -INF;
            }
            if m.is_capture() {
                return -move_score(m);
            }
            let from = m.from().unwrap() as usize;
            let to = m.to() as usize;
            -self.silent_history[from][to]
        });
        moves
    }

    fn eval(&self) -> i32 {
        let board = self.game.current().board();
        let endgame = self.game.is_endgame();
        let mut score = 0;

        for color in [Color::White, Color::Black] {
            for role in [Role::Pawn, Role::Knight, Role::Bishop, Role::Rook, Role::Queen, Role::King] {
                let mut pieces = board.by_color(color).intersect(board.by_role(role));
                while let Some(square) = pieces.pop_front() {
                    let value = piece_value(role) + piece_square_value(role, color, square, endgame);
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

    fn quiescence_search(&mut self, depth: i32, mut alpha: i32, mut beta: i32) -> Option<i32> {

        if self.is_timeout() {
            return None;
        }

        let is_max = self.game.current().turn() == self.color;

        let h: Zobrist64 = self.game.current().zobrist_hash(EnPassantMode::Legal);
        let mut h_move: Option<Move> = None;

        // memoization with Zobrist hashes
        if let Some(entry) = self.q_hash_table.get(h) {
            self.memo_iterations += 1;
            h_move = entry.best_move;
            if entry.depth >= depth {
                match entry.entry_type {
                    EntryType::Exact => return Some(entry.score),
                    EntryType::LowerBound => alpha = alpha.max(entry.score),
                    EntryType::UpperBound => beta = beta.min(entry.score),
                }
                if alpha >= beta {
                    return Some(entry.score);
                }
            }

        }

        self.q_iterations += 1;

        // draw by special case
        if self.game.maxed_moves() || self.game.is_threefold_repetition() {
            return Some(0);
        }

        let moves = self.game.current().legal_moves();

        // check if game ended
        if moves.is_empty() {
            return if self.game.current().is_checkmate() {
                if is_max { Some(-INF) } else { Some(INF) }
            } else {
                Some(0)
            };
        }

        let mut best_score = self.eval();
        let mut active_moves = moves;

        // score of position with no capture if not in check
        if !self.game.current().is_check(){
            if is_max {
                if best_score >= beta { return Some(beta); }
                alpha = alpha.max(best_score);
            } else {
                if best_score <= alpha { return Some(alpha); }
                beta = beta.min(best_score);
            }
            // if not in check explore only capture moves
            active_moves= active_moves.into_iter()
                .filter(|m| m.is_capture())
                .collect();
            active_moves = self.order_moves(active_moves, h_move);
        }

        let original_alpha = alpha;
        let original_beta = beta;
        let mut local_best_move: Option<Move> = None;

        for m in active_moves {
            if self.is_timeout() { return None }

            if self.game.push(m).is_err() { continue; }

            let score = self.quiescence_search(depth - 1, alpha, beta);

            self.game.pop();

            match score {
                None => { return None; },
                _ => {}
            }

            let is_better = if is_max { score > Some(best_score) } else { score < Some(best_score) };
            if is_better {
                best_score = score.unwrap();
                local_best_move = Some(m);
            }

            if is_max {
                alpha = alpha.max(best_score);
                if best_score >= beta { break; }
            } else {
                beta = beta.min(best_score);
                if best_score <= alpha { break; }
            }
        }

        let et = if best_score <= original_alpha {
            EntryType::UpperBound
        } else if best_score >= original_beta {
            EntryType::LowerBound
        } else {
            EntryType::Exact
        };

        self.q_hash_table.insert(TTEntry {
            hash: h,
            depth: depth,
            score: best_score,
            entry_type: et,
            best_move: local_best_move,
        });

        Some(best_score)
    }

    pub fn minimax(&mut self, depth: i32, mut alpha: i32, mut beta: i32) -> Option<i32> {

        if self.is_timeout() {
            return None;
        }

        let is_max = self.game.current().turn() == self.color;

        let h: Zobrist64 = self.game.current().zobrist_hash(EnPassantMode::Legal);
        let mut h_move: Option<Move> = None;

        // memoization with Zobrist hashes
        if let Some(entry) = self.hash_table.get(h) {
            h_move = entry.best_move;
            if entry.depth >= depth {
                self.memo_iterations += 1;
                if depth == self.max_depth {
                    self.best_move = h_move;
                }
                match entry.entry_type {
                    EntryType::Exact => return Some(entry.score),
                    EntryType::LowerBound => alpha = alpha.max(entry.score),
                    EntryType::UpperBound => beta = beta.min(entry.score),
                }
                if alpha >= beta {
                    return Some(entry.score);
                }
            }

        }

        self.iterations += 1;

        // draw by a special case
        if self.game.maxed_moves() || self.game.is_threefold_repetition() {
            return Some(0);
        }

        // max depth reached
        // start quiescence search
        if depth <= 0 {
            return self.quiescence_search(-1, alpha, beta)
        }

        let mut moves = self.game.current().legal_moves();

        // end condition by no other moves
        if moves.is_empty() {
            return if self.game.current().is_checkmate() {
                if is_max { Some(-INF) } else { Some(INF) }
            } else {
                Some(0)
            }
        }

        moves = self.order_moves(moves, h_move);

        let mut best_score = if is_max { -INF } else { INF };
        let mut local_best_move: Option<Move> = None;
        let original_alpha = alpha;
        let original_beta = beta;
        let is_endgame = self.game.is_endgame();

        for (i, m) in moves.iter().enumerate() {
            if self.is_timeout() { return None }

            // push move and explore down the tree
            if self.game.push(*m).is_err() { continue; }

            // Late Move Reduction (LMR)
            let score = if i >= 2 && depth >= 3 && !self.game.current().is_check() && !m.is_capture() && !is_endgame {
                let r_depth = if i >= 5 { 3 } else { 2 };
                let red_score = self.minimax(depth - r_depth, alpha, beta);
                if red_score > Some(alpha) {
                    // move is promising despite being late in the order, full search
                    self.minimax(depth-1, alpha, beta)
                } else {
                    red_score
                }
            } else {
                // normal search
                self.minimax(depth-1, alpha, beta)
            };

            // pop move to go up the tre
            self.game.pop();

            match score {
                None => { return None; },
                _ => {}
            }

            let is_better = if is_max { score > Some(best_score) } else { score < Some(best_score) };
            // if it's near the root of the tree (the possible next move) save the best move
            if is_better || local_best_move.is_none() {
                if depth == self.max_depth || self.best_move.is_none() {
                    self.best_move = Some(m.clone());
                }
                local_best_move = Some(*m);
            }

            if is_max {
                best_score = best_score.max(score.unwrap());
                alpha = alpha.max(best_score);
                if best_score >= beta {
                    if !m.is_capture() && !self.game.current().is_check() {
                        // save "silent" move in history
                        let from = m.from().unwrap() as usize;
                        let to = m.to() as usize;
                        self.silent_history[from][to] += depth * depth;
                    }
                    break;
                }
            } else {
                best_score = best_score.min(score.unwrap());
                beta = beta.min(best_score);
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

        Some(best_score)
    }
}
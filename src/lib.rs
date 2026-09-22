mod engine;
mod transition_table;
mod constants;

use std::error::Error;
use std::time::Duration;
use polyglot_book_rs::PolyglotBook;
use shakmaty::{Color, Move, Position, MoveList, Chess, PlayError};
use shakmaty::{fen::Fen, CastlingMode};
use shakmaty::uci::UciMove;
use engine::{Game, Engine, SearchResult};

pub struct Fishbait {
    engine: Engine,
    opening_book: Option<PolyglotBook>
}

impl Fishbait {

    pub fn new(color: Color, polyglot_book: Option<PolyglotBook>) -> Self {
        let game = Game::new();
        Fishbait {
            engine: Engine::new(game.clone(), color),
            opening_book: polyglot_book
        }
    }

    pub fn from_fen(fen: &str, color: Color, polyglot_book: Option<PolyglotBook>) -> Result<Self, Box<dyn Error>> {
        let fen: Fen = fen.parse()?;
        let pos = fen.into_position(CastlingMode::Standard)?;
        let game = Game::from_pos(pos);
        Ok(Fishbait {
            engine: Engine::new(game.clone(), color),
            opening_book: polyglot_book
        })
    }

    pub fn set_color(&mut self, color: Color) {
        self.engine.set_color(color);
    }

    pub fn get_color(&self) -> Color {
        self.engine.get_color()
    }

    pub fn move_count(&self) -> usize {
        self.engine.game.len() - 1
    }

    pub fn make_move(&mut self, mv: Move) -> Result<(), PlayError<Chess>> {
        self.engine.game.push(mv)
    }

    pub fn make_move_uci(&mut self, uci_str: &str) -> Result<(), Box<dyn Error>> {
        let uci: UciMove = uci_str.parse()?;
        let mv = uci.to_move(self.engine.game.current())?;
        self.make_move(mv)?;
        Ok(())
    }

    pub fn search(&mut self, max_time: Duration, print: bool) -> SearchResult {
        self.engine.set_timer(max_time);
        self.engine.play(&self.opening_book, print)
    }

    pub fn legal_moves(&self) -> MoveList {
        self.engine.game.current().legal_moves()
    }

    pub fn is_game_over(&self) -> bool {
        self.engine.game.current().is_game_over()
    }

    pub fn current_fen(&self) -> String {
        Fen::from_position(self.engine.game.current(), shakmaty::EnPassantMode::Legal).to_string()
    }
}
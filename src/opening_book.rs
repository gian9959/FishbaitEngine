use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt};
use shakmaty::{Chess, File, Move, Rank, Square};
use crate::constants::BOOK;

pub struct OpeningBook {
    entries: Vec<BookEntry>,
}

pub struct BookEntry {
    hash: u64,
    pub mv: u16,
    pub weight: u16,
}

impl OpeningBook {
    pub fn from_embedded() -> Self {
        let mut cursor = Cursor::new(BOOK);
        let mut entries = Vec::new();

        while let (Ok(hash), Ok(mv), Ok(weight)) = (
            cursor.read_u64::<BigEndian>(),
            cursor.read_u16::<BigEndian>(),
            cursor.read_u16::<BigEndian>(),
        ) {
            cursor.read_u32::<BigEndian>().ok();
            entries.push(BookEntry {hash: hash, mv: mv, weight: weight});
        }
        OpeningBook { entries: entries }
    }

    pub fn get_moves(&self, hash: u64) -> Vec<&BookEntry> {
        self.entries.iter().filter(|e| e.hash == hash).collect()
    }
}

impl BookEntry {
    pub(crate) fn decode_move(&self, pos: &Chess) -> Option<Move> {
        let to_file = ((self.mv >> 0) & 7) as u8;
        let to_rank = ((self.mv >> 3) & 7) as u8;
        let from_file = ((self.mv >> 6) & 7) as u8;
        let from_rank = ((self.mv >> 9) & 7) as u8;
        let promotion = (self.mv >> 12) & 7;

        let from = Square::from_coords(File::new(from_file as u32), Rank::new(from_rank as u32));
        let to = Square::from_coords(File::new(to_file as u32), Rank::new(to_rank as u32));

        let uci_str = match promotion {
            1 => format!("{}{}n", from, to),
            2 => format!("{}{}b", from, to),
            3 => format!("{}{}r", from, to),
            4 => format!("{}{}q", from, to),
            _ => format!("{}{}", from, to),
        };

        let uci: shakmaty::uci::UciMove = uci_str.parse().ok()?;
        uci.to_move(pos).ok()
    }
}
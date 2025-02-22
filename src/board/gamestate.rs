use crate::search::perft::str_to_square;

use super::{Piece, PieceColor};

#[derive(Copy, Clone, Debug)]
pub struct Gamestate {
    pub last_piece_captured: Option<Piece>,
    pub enpassant_square: Option<u64>,
    castling_rights: u8,
    pub zobrist_key: u64,
    pub halfmove_clock: u8,
}

impl Gamestate {
    pub fn new(castling_rights_str: &str, enpassant_square_str: &str) -> Gamestate {
        let mut castling_rights = 0u8;
        for c in castling_rights_str.chars() {
            match c {
                'K' => castling_rights += 1,
                'Q' => castling_rights += 2,
                'k' => castling_rights += 4,
                'q' => castling_rights += 8,
                '-' => break,
                _ => panic!("unrecognized character in FEN castling rights parsing"),
            };
        }

        let enpassant_square = if enpassant_square_str == "-" {
            None
        } else {
            Some(str_to_square(enpassant_square_str))
        };

        Gamestate {
            last_piece_captured: None,
            enpassant_square,
            castling_rights,
            zobrist_key: 0,
            halfmove_clock: 0,
        }
    }

    #[inline]
    pub fn get_last_piece_captured(self) -> Option<Piece> {
        self.last_piece_captured
    }

    #[inline]
    pub fn get_enpassant_square(self) -> Option<u64> {
        self.enpassant_square
    }

    #[inline]
    pub fn get_castling_rights(self) -> u8 {
        self.castling_rights
    }

    #[inline]
    pub fn can_black_castle_queenside(self) -> bool {
        self.castling_rights & 8 != 0
    }

    #[inline]
    pub fn can_black_castle_kingside(self) -> bool {
        self.castling_rights & 4 != 0
    }

    #[inline]
    pub fn can_white_castle_queenside(self) -> bool {
        self.castling_rights & 2 != 0
    }

    #[inline]
    pub fn can_white_castle_kingside(self) -> bool {
        self.castling_rights & 1 != 0
    }

    #[inline]
    pub fn can_castle_kingside(self, color_to_move: PieceColor) -> bool {
        match color_to_move {
            PieceColor::White => self.can_white_castle_kingside(),
            PieceColor::Black => self.can_black_castle_kingside(),
        }
    }

    #[inline]
    pub fn can_castle_queenside(self, color_to_move: PieceColor) -> bool {
        match color_to_move {
            PieceColor::White => self.can_white_castle_queenside(),
            PieceColor::Black => self.can_black_castle_queenside(),
        }
    }

    #[inline]
    pub fn remove_castle_kingside(&mut self, color_to_move: PieceColor) {
        match color_to_move {
            PieceColor::White => self.castling_rights &= !(1),
            PieceColor::Black => self.castling_rights &= !(4),
        }
    }

    #[inline]
    pub fn remove_castle_queenside(&mut self, color_to_move: PieceColor) {
        match color_to_move {
            PieceColor::White => self.castling_rights &= !(2),
            PieceColor::Black => self.castling_rights &= !(8),
        }
    }
}

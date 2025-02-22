use std::{fmt::Display, ops::*};

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum PieceType {
    Rook,
    Bishop,
    Queen,
    Knight,
    Pawn,
    King,
}

impl PieceType {
    pub fn to_char(self) -> char {
        match self {
            PieceType::Rook => 'r',
            PieceType::Bishop => 'b',
            PieceType::Queen => 'q',
            PieceType::Knight => 'n',
            PieceType::Pawn => 'p',
            PieceType::King => 'k',
        }
    }
}

impl From<u8> for PieceType {
    fn from(value: u8) -> Self {
        PieceType::from(value as u64)
    }
}

impl From<u64> for PieceType {
    fn from(value: u64) -> Self {
        match value {
            0 => Self::Rook,
            1 => Self::Bishop,
            2 => Self::Queen,
            3 => Self::Knight,
            4 => Self::Pawn,
            5 => Self::King,
            _ => panic!("unrecognized pieceType in From trait"),
        }
    }
}

impl From<usize> for PieceType {
    fn from(value: usize) -> Self {
        PieceType::from(value as u64)
    }
}

impl From<char> for PieceType {
    fn from(value: char) -> Self {
        match value {
            'r' => PieceType::Rook,
            'n' => PieceType::Knight,
            'b' => PieceType::Bishop,
            'q' => PieceType::Queen,
            'k' => PieceType::King,
            'p' => PieceType::Pawn,
            _ => panic!("wrong char in PieceType::From<char>"),
        }
    }
}

impl<T, const N: usize> Index<PieceType> for [T; N] {
    type Output = T;

    fn index(&self, index: PieceType) -> &Self::Output {
        &self[index as usize]
    }
}

impl<T, const N: usize> IndexMut<PieceType> for [T; N] {
    fn index_mut(&mut self, index: PieceType) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

// ---------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum PieceColor {
    White,
    Black,
}

impl From<u8> for PieceColor {
    fn from(value: u8) -> Self {
        PieceColor::from(value as u64)
    }
}

impl From<u64> for PieceColor {
    fn from(value: u64) -> Self {
        match value {
            0 => Self::White,
            1 => Self::Black,
            _ => panic!("unrecognized pieceColor in From trait"),
        }
    }
}

impl From<usize> for PieceColor {
    fn from(value: usize) -> Self {
        PieceColor::from(value as u64)
    }
}

impl<T, const N: usize> Index<PieceColor> for [T; N] {
    type Output = T;

    fn index(&self, index: PieceColor) -> &Self::Output {
        &self[index as usize]
    }
}

impl<T, const N: usize> IndexMut<PieceColor> for [T; N] {
    fn index_mut(&mut self, index: PieceColor) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

impl Not for PieceColor {
    type Output = PieceColor;

    fn not(self) -> Self::Output {
        match self {
            PieceColor::White => PieceColor::Black,
            PieceColor::Black => PieceColor::White,
        }
    }
}

// ---------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, Debug, Default)]
pub struct Piece {
    piece_code: u8,
}

impl Piece {
    pub fn new(ptype: PieceType, color: PieceColor) -> Piece {
        let val: u8 = ptype as u8 + ((color as u8) << 3);
        Piece { piece_code: val }
    }

    pub fn get_type(self) -> PieceType {
        PieceType::from(self.piece_code & 7)
    }

    pub fn get_color(self) -> PieceColor {
        PieceColor::from(self.piece_code >> 3)
    }

    #[inline]
    pub fn is_slider(self) -> bool {
        matches!(
            self.get_type(),
            PieceType::Bishop | PieceType::Rook | PieceType::Queen
        )
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = if self.get_color() == PieceColor::Black {
            self.get_type().to_char()
        } else {
            self.get_type().to_char().to_ascii_uppercase()
        };
        write!(f, "{}", c)
    }
}

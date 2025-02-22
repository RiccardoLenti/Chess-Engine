use crate::board::piece::PieceType;

#[derive(Clone, Copy, Debug)]
pub struct Move {
    move_code: u16,
}

impl Move {
    #[inline]
    pub fn new(start_square: u64, land_square: u64) -> Move {
        Move {
            move_code: ((land_square << 6) + start_square) as u16,
        }
    }

    #[inline]
    pub fn get_from(self) -> u64 {
        (self.move_code & 0x3f) as u64
    }

    #[inline]
    pub fn get_to(self) -> u64 {
        ((self.move_code >> 6) & 0x3f) as u64
    }

    #[inline]
    pub fn add_enpassant(&mut self) {
        self.move_code += 8192;
    }

    #[inline]
    pub fn is_enpassant(self) -> bool {
        !self.is_promotion() && (self.move_code >> 13) & 1 == 1
    }

    #[inline]
    pub fn add_castle_kingside(&mut self) {
        self.move_code += 16384;
    }

    #[inline]
    pub fn is_castle_kingside(self) -> bool {
        !self.is_promotion() && (self.move_code >> 14) & 1 == 1
    }

    #[inline]
    pub fn add_castle_queenside(&mut self) {
        self.move_code += 32768;
    }

    #[inline]
    pub fn is_castle_queenside(self) -> bool {
        !self.is_promotion() && (self.move_code >> 15) & 1 == 1
    }

    #[inline]
    pub fn add_promotion(&mut self, piece_to_promote_to: PieceType) {
        self.move_code += 4096;
        self.move_code += (piece_to_promote_to as u16) << 13;
    }

    #[inline]
    pub fn is_promotion(self) -> bool {
        (self.move_code >> 12) & 1 == 1
    }

    #[inline]
    pub fn get_promotion_type(self) -> PieceType {
        PieceType::from(((self.move_code >> 13) & 7) as u8)
    }

    pub fn to_long_algebraic_notation(self) -> String {
        const XCHARS: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
        const YCHARS: [char; 8] = ['1', '2', '3', '4', '5', '6', '7', '8'];
        let mut res: String = String::new();
        let from_y = (self.get_from() / 8) as usize;
        let from_x = (self.get_from() % 8) as usize;
        let to_y = (self.get_to() / 8) as usize;
        let to_x = (self.get_to() % 8) as usize;

        res.push(XCHARS[from_x]);
        res.push(YCHARS[from_y]);
        res.push(XCHARS[to_x]);
        res.push(YCHARS[to_y]);

        if self.is_promotion() {
            res.push(self.get_promotion_type().to_char());
        }

        res
    }
}

impl From<&str> for Move {
    fn from(value: &str) -> Self {
        let mut res;
        let chars_vec: Vec<char> = value.chars().collect();

        if chars_vec.len() < 4 {
            panic!("called Move::from with a wrong str");
        }

        let from_x = chars_vec[0] as u8 - b'a';
        let from_y = chars_vec[1] as u8 - b'1';
        let to_x = chars_vec[2] as u8 - b'a';
        let to_y = chars_vec[3] as u8 - b'1';

        res = Move::new((from_y * 8 + from_x) as u64, (to_y * 8 + to_x) as u64);
        if chars_vec.len() == 5 {
            res.add_promotion(PieceType::from(chars_vec[4]));
        }
        res
    }
}

impl PartialEq for Move {
    fn eq(&self, other: &Self) -> bool {
        (self.move_code & 8191) == (other.move_code & 8191)
            && (!self.is_promotion() || (self.get_promotion_type() == other.get_promotion_type()))
    }
}

impl Default for Move {
    fn default() -> Self {
        Move::new(0, 0)
    }
}

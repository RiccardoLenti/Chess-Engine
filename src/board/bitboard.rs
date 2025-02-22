pub trait Bitmanip {
    fn set_square(&mut self, square_index: u64);
    fn toggle_square(&mut self, square_index: u64);
    fn toggle_squares(&mut self, square_index_1: u64, square_index_2: u64);
    fn bitscan(self) -> u64;
    fn bitscan_reset(&mut self) -> u64;
    fn contains_index(self, square_index: u64) -> bool;
    fn contains_bit(self, bit: u64) -> bool;
    fn isolate_ls1b(self) -> u64;
}

impl Bitmanip for u64 {
    #[inline]
    fn set_square(&mut self, square_index: u64) {
        *self |= 1 << square_index;
    }

    #[inline]
    fn toggle_square(&mut self, square_index: u64) {
        *self ^= 1 << square_index;
    }

    #[inline]
    fn toggle_squares(&mut self, square_index_1: u64, square_index_2: u64) {
        *self ^= (1 << square_index_1) | (1 << square_index_2);
    }

    #[inline]
    fn bitscan(self) -> u64 {
        self.trailing_zeros() as u64
    }

    #[inline]
    fn bitscan_reset(&mut self) -> u64 {
        let id = self.trailing_zeros();
        *self &= *self - 1;
        id as u64
    }

    #[inline]
    fn contains_index(self, square_index: u64) -> bool {
        ((self >> square_index) & 1) != 0
    }

    #[inline]
    fn isolate_ls1b(self) -> u64 {
        //self & (-(self as i64)) as u64
        self & self.wrapping_neg()
    }

    #[inline]
    fn contains_bit(self, bit: u64) -> bool {
        (self & bit) != 0
    }
}

pub fn print_bitboard(bitboard: u64) {
    println!();
    const LAST_BIT: u64 = 63;
    for rank in 0..8 {
        for file in (0..8).rev() {
            let mask = 1u64 << (LAST_BIT - (rank * 8) - file);
            let char = if bitboard & mask != 0 { '1' } else { '0' };
            print!("{char} ");
        }
        println!();
    }
}

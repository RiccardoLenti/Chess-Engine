use crate::{
    board::{Board, bitboard::Bitmanip, piece::PieceColor},
    move_gen::{chess_move::Move, move_list::MoveList},
};

const PIECE_WEIGHTS: [i32; 6] = [500, 330, 900, 300, 100, 0];
const COLOR_MULTIPLIERS: [i32; 2] = [1, -1];

// tables' values are taken from https://www.chessprogramming.org/PeSTO%27s_Evaluation_Function

#[rustfmt::skip]
static PAWNS_MG_TABLE: [i16; 64] = [
    0,   0,   0,   0,   0,   0,  0,   0,
    98, 134,  61,  95,  68, 126, 34, -11,
    -6,   7,  26,  31,  65,  56, 25, -20,
   -14,  13,   6,  21,  23,  12, 17, -23,
   -27,  -2,  -5,  12,  17,   6, 10, -25,
   -26,  -4,  -4, -10,   3,   3, 33, -12,
   -35,  -1, -20, -23, -15,  24, 38, -22,
     0,   0,   0,   0,   0,   0,  0,   0,
];

#[rustfmt::skip]
static PAWNS_EG_TABLE: [i16; 64] = [
    0,   0,   0,   0,   0,   0,   0,   0,
    178, 173, 158, 134, 147, 132, 165, 187,
     94, 100,  85,  67,  56,  53,  82,  84,
     32,  24,  13,   5,  -2,   4,  17,  17,
     13,   9,  -3,  -7,  -7,  -8,   3,  -1,
      4,   7,  -6,   1,   0,  -5,  -1,  -8,
     13,   8,   8,  10,  13,   0,   2,  -7,
      0,   0,   0,   0,   0,   0,   0,   0,
];

#[rustfmt::skip]
static ROOKS_MG_TABLE: [i16; 64] = [
    32,  42,  32,  51, 63,  9,  31,  43,
    27,  32,  58,  62, 80, 67,  26,  44,
    -5,  19,  26,  36, 17, 45,  61,  16,
   -24, -11,   7,  26, 24, 35,  -8, -20,
   -36, -26, -12,  -1,  9, -7,   6, -23,
   -45, -25, -16, -17,  3,  0,  -5, -33,
   -44, -16, -20,  -9, -1, 11,  -6, -71,
   -19, -13,   1,  17, 16,  7, -37, -26,
];

#[rustfmt::skip]
static ROOKS_EG_TABLE: [i16; 64] = [
    13, 10, 18, 15, 12,  12,   8,   5,
    11, 13, 13, 11, -3,   3,   8,   3,
     7,  7,  7,  5,  4,  -3,  -5,  -3,
     4,  3, 13,  1,  2,   1,  -1,   2,
     3,  5,  8,  4, -5,  -6,  -8, -11,
    -4,  0, -5, -1, -7, -12,  -8, -16,
    -6, -6,  0,  2, -9,  -9, -11,  -3,
    -9,  2,  3, -1, -5, -13,   4, -20,
];

#[rustfmt::skip]
static KNIGHTS_MG_TABLE: [i16; 64] = [    
    -167, -89, -34, -49,  61, -97, -15, -107,
     -73, -41,  72,  36,  23,  62,   7,  -17,
     -47,  60,  37,  65,  84, 129,  73,   44,
      -9,  17,  19,  53,  37,  69,  18,   22,
     -13,   4,  16,  13,  28,  19,  21,   -8,
     -23,  -9,  12,  10,  19,  17,  25,  -16,
     -29, -53, -12,  -3,  -1,  18, -14,  -19,
    -105, -21, -58, -33, -17, -28, -19,  -23,
];

#[rustfmt::skip]
static KNIGHTS_EG_TABLE: [i16; 64] = [
    -58, -38, -13, -28, -31, -27, -63, -99,
    -25,  -8, -25,  -2,  -9, -25, -24, -52,
    -24, -20,  10,   9,  -1,  -9, -19, -41,
    -17,   3,  22,  22,  22,  11,   8, -18,
    -18,  -6,  16,  25,  16,  17,   4, -18,
    -23,  -3,  -1,  15,  10,  -3, -20, -22,
    -42, -20, -10,  -5,  -2, -20, -23, -44,
    -29, -51, -23, -15, -22, -18, -50, -64,
];

#[rustfmt::skip]
static BISHOPS_MG_TABLE: [i16; 64] = [
    -29,   4, -82, -37, -25, -42,   7,  -8,
    -26,  16, -18, -13,  30,  59,  18, -47,
    -16,  37,  43,  40,  35,  50,  37,  -2,
     -4,   5,  19,  50,  37,  37,   7,  -2,
     -6,  13,  13,  26,  34,  12,  10,   4,
      0,  15,  15,  15,  14,  27,  18,  10,
      4,  15,  16,   0,   7,  21,  33,   1,
    -33,  -3, -14, -21, -13, -12, -39, -21,
];

#[rustfmt::skip]
static BISHOPS_EG_TABLE: [i16; 64] = [
    -14, -21, -11,  -8, -7,  -9, -17, -24,
     -8,  -4,   7, -12, -3, -13,  -4, -14,
      2,  -8,   0,  -1, -2,   6,   0,   4,
     -3,   9,  12,   9, 14,  10,   3,   2,
     -6,   3,  13,  19,  7,  10,  -3,  -9,
    -12,  -3,   8,  10, 13,   3,  -7, -15,
    -14, -18,  -7,  -1,  4,  -9, -15, -27,
    -23,  -9, -23,  -5, -9, -16,  -5, -17,
];

#[rustfmt::skip]
static QUEENS_MG_TABLE: [i16; 64] = [
    -28,   0,  29,  12,  59,  44,  43,  45,
    -24, -39,  -5,   1, -16,  57,  28,  54,
    -13, -17,   7,   8,  29,  56,  47,  57,
    -27, -27, -16, -16,  -1,  17,  -2,   1,
     -9, -26,  -9, -10,  -2,  -4,   3,  -3,
    -14,   2, -11,  -2,  -5,   2,  14,   5,
    -35,  -8,  11,   2,   8,  15,  -3,   1,
     -1, -18,  -9,  10, -15, -25, -31, -50,
];

#[rustfmt::skip]
static QUEENS_EG_TABLE: [i16; 64] = [
    -9,  22,  22,  27,  27,  19,  10,  20,
    -17,  20,  32,  41,  58,  25,  30,   0,
    -20,   6,   9,  49,  47,  35,  19,   9,
      3,  22,  24,  45,  57,  40,  57,  36,
    -18,  28,  19,  47,  31,  34,  39,  23,
    -16, -27,  15,   6,   9,  17,  10,   5,
    -22, -23, -30, -16, -16, -23, -36, -32,
    -33, -28, -22, -43,  -5, -32, -20, -41,
];

#[rustfmt::skip]
static KINGS_MG_TABLE: [i16; 64] = [
    -65,  23,  16, -15, -56, -34,   2,  13,
     29,  -1, -20,  -7,  -8,  -4, -38, -29,
     -9,  24,   2, -16, -20,   6,  22, -22,
    -17, -20, -12, -27, -30, -25, -14, -36,
    -49,  -1, -27, -39, -46, -44, -33, -51,
    -14, -14, -22, -46, -44, -30, -15, -27,
      1,   7,  -8, -64, -43, -16,   9,   8,
    -15,  36,  12, -54,   8, -28,  24,  14,

];

#[rustfmt::skip]
static KINGS_EG_TABLE: [i16; 64] = [
    -74, -35, -18, -18, -11,  15,   4, -17,
    -12,  17,  14,  17,  17,  38,  23,  11,
     10,  17,  23,  15,  20,  45,  44,  13,
     -8,  22,  24,  27,  26,  33,  26,   3,
    -18,  -4,  21,  24,  27,  23,   9, -11,
    -19,  -3,  11,  21,  23,  16,   7,  -9,
    -27, -11,   4,  13,  14,   4,  -5, -17,
    -53, -34, -21, -11, -28, -14, -24, -43
];

static PIECE_TABLES: [[[i16; 64]; 2]; 6] = [
    [ROOKS_MG_TABLE, ROOKS_EG_TABLE],
    [BISHOPS_MG_TABLE, BISHOPS_EG_TABLE],
    [QUEENS_MG_TABLE, QUEENS_EG_TABLE],
    [KNIGHTS_MG_TABLE, KNIGHTS_EG_TABLE],
    [PAWNS_MG_TABLE, PAWNS_EG_TABLE],
    [KINGS_MG_TABLE, KINGS_EG_TABLE],
];

pub fn eval(board: &Board) -> i32 {
    const TOTAL_WEIGHT: i32 = 2
        * (PIECE_WEIGHTS[0] * 2
            + PIECE_WEIGHTS[1] * 2
            + PIECE_WEIGHTS[2]
            + PIECE_WEIGHTS[3] * 2
            + PIECE_WEIGHTS[4] * 8
            + PIECE_WEIGHTS[5] * 2);

    let mut material_score = 0i32;
    let mut positional_score = 0i32;
    let mut total_material = 0i32;

    let (white_bb, black_bb, _, _) = board.get_us_enemy_bitboards(PieceColor::White);

    for (piece_type, bb) in white_bb.iter().enumerate() {
        let piece_count = bb.count_ones() as i32;
        material_score += PIECE_WEIGHTS[piece_type] * piece_count;
        total_material += PIECE_WEIGHTS[piece_type] * piece_count;
    }

    for (piece_type, bb) in black_bb.iter().enumerate() {
        let piece_count = bb.count_ones() as i32;
        material_score -= PIECE_WEIGHTS[piece_type] * piece_count;
        total_material += PIECE_WEIGHTS[piece_type] * piece_count;
    }

    let endgame_weight: f32 = (TOTAL_WEIGHT - total_material) as f32 / TOTAL_WEIGHT as f32;

    // spaghetti code
    for (piece_type, mut bb) in white_bb.into_iter().enumerate() {
        while bb != 0 {
            let index = bb.bitscan_reset() ^ 56; // ^ 56 because it needs to be flipped because it's white

            let mg_score = PIECE_TABLES[piece_type][0][index as usize] as f32;
            let eg_score = PIECE_TABLES[piece_type][1][index as usize] as f32;

            positional_score +=
                (mg_score * (1.0 - endgame_weight) + eg_score * endgame_weight) as i32;
        }
    }

    for (piece_type, mut bb) in black_bb.into_iter().enumerate() {
        while bb != 0 {
            let index = bb.bitscan_reset();

            let mg_score = PIECE_TABLES[piece_type][0][index as usize] as f32;
            let eg_score = PIECE_TABLES[piece_type][1][index as usize] as f32;

            positional_score -=
                (mg_score * (1.0 - endgame_weight) + eg_score * endgame_weight) as i32;
        }
    }

    (material_score + positional_score) * COLOR_MULTIPLIERS[board.get_color_to_move()]
}

pub fn order_moves(moves: &mut MoveList, board: &Board, best_tt_move: Option<Move>) {
    let mut scores: Vec<i32> = vec![0; moves.len() as usize];

    for (i, m) in moves.iter().enumerate() {
        if let Some(tt_move) = best_tt_move {
            if m == tt_move {
                scores[i] = i32::MAX;
                continue; // skip otherwise it's overwritten or may overflow
            }
        }

        if let Some(captured_piece) = board.get_piece_at(m.get_to()) {
            scores[i] = PIECE_WEIGHTS[captured_piece.get_type()] * 10
                - PIECE_WEIGHTS[board.get_piece_at(m.get_from()).unwrap().get_type()];
        }

        if m.is_promotion() {
            scores[i] += PIECE_WEIGHTS[m.get_promotion_type()];
        }
    }

    quick_sort(moves, &mut scores, 0, moves.len() as isize - 1);
}

fn quick_sort(moves: &mut MoveList, scores: &mut Vec<i32>, low: isize, high: isize) {
    if low < high {
        let pivot_index = partition(moves, scores, low, high);
        quick_sort(moves, scores, low, pivot_index - 1);
        quick_sort(moves, scores, pivot_index + 1, high);
    }
}

fn partition(moves: &mut MoveList, scores: &mut [i32], low: isize, high: isize) -> isize {
    let pivot_score = scores[high as usize];
    let mut i = low - 1;

    for j in low..high {
        if scores[j as usize] > pivot_score {
            i += 1;
            moves.swap(i as usize, j as usize);
            scores.swap(i as usize, j as usize);
        }
    }

    moves.swap((i + 1) as usize, high as usize);
    scores.swap((i + 1) as usize, high as usize);

    i + 1
}

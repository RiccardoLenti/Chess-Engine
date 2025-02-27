pub mod perft;
pub mod tt;

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use tt::{EntryType, TranspositionTable};

use crate::{
    board::Board,
    move_gen::{chess_move::Move, generate_legal_moves, move_list::MoveList},
};

const PIECE_WEIGHTS: [i32; 6] = [500, 330, 900, 300, 100, 0];
const COLOR_MULTIPLIERS: [i32; 2] = [1, -1];
const MATE_THRESHOLD: i32 = i32::MAX - 50;

pub fn iterative_deepening_search(
    board: &mut Board,
    think_time_ms: u64,
    stop_flag: &Arc<AtomicBool>,
) {
    let mut best_search_res: (Move, i32) = (board.get_legal_moves().index(0), i32::MIN);
    let mut best_depth = 0u8;
    let think_time = Duration::from_millis(think_time_ms);

    let mut tt = TranspositionTable::new(64); // 64 MB

    let now = Instant::now();
    for current_depth in 1..=32 {
        let search_res_opt =
            alpha_beta_root_node(board, current_depth, &mut tt, now, think_time, stop_flag);
        if let Some(search_res) = search_res_opt {
            best_search_res = search_res;
            best_depth = current_depth;

            if best_search_res.1 >= MATE_THRESHOLD {
                break;
            }
        } else {
            break;
        }
    }

    println!("info depth {} score cp {}", best_depth, best_search_res.1);
    println!(
        "bestmove {}",
        best_search_res.0.to_long_algebraic_notation()
    );
}

fn alpha_beta_root_node(
    board: &mut Board,
    max_depth: u8,
    tt: &mut TranspositionTable,
    now: Instant,
    think_time: Duration,
    stop_flag: &Arc<AtomicBool>,
) -> Option<(Move, i32)> {
    let mut alpha = -MATE_THRESHOLD;
    let beta = MATE_THRESHOLD;
    let mut best_move: Option<Move> = None;

    tt.increment_age();

    let zobrist_key = board.get_zobrist_key();
    let tt_best_move = tt.probe(zobrist_key).and_then(|entry| entry.best_move);

    let mut legal_moves = generate_legal_moves(board);
    order_moves(&mut legal_moves, board, tt_best_move);

    for m in legal_moves.iter() {
        board.make_move(m);

        let this_move_eval: i32 =
            if board.is_threefold_repetition() || board.draw_by_fifty_moves_rule() {
                0
            } else {
                -alpha_beta(
                    board,
                    -beta,
                    -alpha,
                    max_depth - 1,
                    tt,
                    now,
                    think_time,
                    stop_flag,
                )
            };

        board.unmake_move(m);

        if now.elapsed() >= think_time || stop_flag.load(Ordering::SeqCst) {
            return None;
        }

        if this_move_eval > alpha {
            best_move = Some(m);
            alpha = this_move_eval;
        }
    }

    // store best move in tt and return the tuple (m, alpha)
    best_move.map(|m| {
        tt.store(zobrist_key, max_depth, alpha, EntryType::Exact, Some(m));
        (m, alpha)
    })
}

fn alpha_beta(
    board: &mut Board,
    mut alpha: i32,
    mut beta: i32,
    depth: u8,
    tt: &mut TranspositionTable,
    now: Instant,
    think_time: Duration,
    stop_flag: &Arc<AtomicBool>,
) -> i32 {
    if now.elapsed() >= think_time || stop_flag.load(Ordering::SeqCst) {
        return 0;
    }

    let original_alpha = alpha;
    let original_beta = beta;
    let zobrist_key = board.get_zobrist_key();
    let tt_entry = tt.probe(zobrist_key);

    if let Some(entry) = tt_entry {
        if entry.depth >= depth {
            match entry.entry_type {
                tt::EntryType::Exact => return entry.score,
                tt::EntryType::LowerBound => alpha = std::cmp::max(alpha, entry.score),
                tt::EntryType::UpperBound => beta = std::cmp::min(beta, entry.score),
            }

            if alpha >= beta {
                return match entry.entry_type {
                    EntryType::LowerBound => beta,
                    EntryType::UpperBound => alpha,
                    EntryType::Exact => unreachable!(),
                };
            }
        }
    }

    let mut max_eval = -MATE_THRESHOLD;
    let mut best_move: Option<Move> = None;
    let mut legal_moves = generate_legal_moves(board);

    if legal_moves.len() == 0 {
        if board.is_in_check() {
            return max_eval - depth as i32; // shorter mates are preferred
        } else {
            return 0;
        }
    }

    if depth == 0 {
        return eval(board);
    }

    let tt_best_move = tt_entry.and_then(|entry| entry.best_move);
    order_moves(&mut legal_moves, board, tt_best_move);

    for m in legal_moves.iter() {
        board.make_move(m);

        let this_move_eval = if board.is_threefold_repetition() || board.draw_by_fifty_moves_rule()
        {
            0
        } else {
            -alpha_beta(
                board,
                -beta,
                -alpha,
                depth - 1,
                tt,
                now,
                think_time,
                stop_flag,
            )
        };
        board.unmake_move(m);

        if this_move_eval > max_eval {
            max_eval = this_move_eval;
            best_move = Some(m);

            if this_move_eval > alpha {
                alpha = this_move_eval;
            }
        }

        if alpha >= beta {
            // beta cutoff
            tt.store(zobrist_key, depth, beta, EntryType::LowerBound, best_move);
            return max_eval;
        }
    }

    let entry_type = if max_eval <= original_alpha {
        EntryType::UpperBound
    } else if max_eval >= original_beta {
        EntryType::LowerBound
    } else {
        EntryType::Exact
    };

    tt.store(zobrist_key, depth, max_eval, entry_type, best_move);

    max_eval
}

fn eval(board: &Board) -> i32 {
    let mut res = 0;

    for (piece_color, bbs_ar) in board.get_pieces_bb().iter().enumerate() {
        for (piece_type, bb) in bbs_ar.iter().enumerate() {
            res +=
                PIECE_WEIGHTS[piece_type] * COLOR_MULTIPLIERS[piece_color] * bb.count_ones() as i32;
        }
    }

    res * COLOR_MULTIPLIERS[board.get_color_to_move()]
}

fn order_moves(moves: &mut MoveList, board: &Board, best_tt_move: Option<Move>) {
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

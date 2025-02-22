use crate::move_gen::generate_legal_moves;
use std::sync::atomic::Ordering;
use std::{
    sync::{atomic::AtomicBool, Arc},
    time::Instant,
};

use crate::board::Board;

pub fn perft_test(max_depth: u8, board: &mut Board, stop: &Arc<AtomicBool>) {
    println!("---------------------------------------");
    println!("|                                     |");
    println!("|  Running Perft test with depth: {:>2}  |", max_depth);
    println!("|                                     |");
    println!("---------------------------------------");
    for i in 1..=max_depth {
        let now = Instant::now();
        let num_positions = perft_test_r(i, board, i, stop);

        if stop.load(Ordering::SeqCst) {
            return;
        }

        println!(
            "Depth: {:>2} | Nodes: {:>12} | Time: {}s",
            i,
            num_positions,
            now.elapsed().as_secs_f32()
        );
    }
}

fn perft_test_r(depth: u8, board: &mut Board, max_depth: u8, stop: &Arc<AtomicBool>) -> u128 {
    if depth == 0 || stop.load(Ordering::SeqCst) {
        return 1;
    }

    let legal_moves = generate_legal_moves(board);
    if depth == 1 {
        return legal_moves.len() as u128;
    }

    let mut res = 0;
    for m in legal_moves.iter() {
        board.make_move(m);
        let positions_after_this_move = perft_test_r(depth - 1, board, max_depth, stop);
        board.unmake_move(m);

        #[cfg(debug_assertions)]
        {
            if depth == max_depth {
                println!(
                    "{} : {}",
                    m.to_long_algebraic_notation(),
                    positions_after_this_move
                );
            }
        }

        res += positions_after_this_move;
    }

    res
}

fn _square_to_str(index: u64) -> String {
    let y = index / 8;
    let x = index % 8;

    ((b'a' + x as u8) as char).to_string() + &((y as u8 + b'1') as char).to_string()
}

pub fn str_to_square(name: &str) -> u64 {
    let mut chars = name.chars();

    ((chars.next().unwrap() as u32 - 'a' as u32)
        + (chars.next().unwrap().to_digit(10).unwrap() - 1) * 8) as u64
}

mod board;
mod consts;
mod move_gen;
mod search;

use std::{
    io,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread::{self},
};

use crate::board::Board;

use board::{bitboard::print_bitboard, piece::*};
use move_gen::chess_move::Move;
use move_gen::generate_legal_moves;
use search::perft::perft_test;

const _INITIAL_FEN_STRING: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"; //KQkq -";
const _TEST_FEN_STRING: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ";

fn main() {
    let mut board = Board::new(_INITIAL_FEN_STRING);

    let (tx, rx) = mpsc::channel();
    let stop_flag = Arc::new(AtomicBool::from(false));
    let stop_clone = Arc::clone(&stop_flag);
    let _ = thread::spawn(move || handle_stdin(tx, stop_clone));

    loop {
        stop_flag.store(false, Ordering::SeqCst);
        let uci_command = rx.recv().unwrap();
        match uci_command[0].as_str() {
            "ucinewgame" => board = Board::new(_INITIAL_FEN_STRING),
            "position" => board = handle_position(uci_command[1..].to_vec()),
            "perft" => {
                let depth = uci_command[1].parse::<u8>().unwrap();
                perft_test(depth, &mut board, &stop_flag);
            }
            "captures" => {
                let captures = generate_legal_moves(&board, false);
                println!("captures: {}", captures.len());
            }
            "go" => {
                handle_go(uci_command[1..].to_vec(), &mut board, &stop_flag);
            }
            "isready" => println!("readyok"),
            "quit" => return,
            _ => {}
        }
    }
}

fn handle_go(command: Vec<String>, board: &mut Board, stop_flag: &Arc<AtomicBool>) {
    let mut wtime = 20u64;
    let mut btime = 20u64;
    let mut winc = 0u64;
    let mut binc = 0u64;

    let mut tokens = command.into_iter();

    while let Some(token) = tokens.next() {
        match token.as_str() {
            "wtime" => wtime = tokens.next().unwrap().parse::<u64>().unwrap(),
            "btime" => btime = tokens.next().unwrap().parse::<u64>().unwrap(),
            "winc" => winc = tokens.next().unwrap().parse::<u64>().unwrap(),
            "binc" => binc = tokens.next().unwrap().parse::<u64>().unwrap(),
            _ => {}
        }
    }

    let think_time: u64 = if board.get_color_to_move() == PieceColor::White {
        wtime / 20 + winc / 2
    } else {
        btime / 20 + binc / 2
    };

    search::iterative_deepening_search(board, think_time, stop_flag);
}

fn handle_position(command: Vec<String>) -> Board {
    let mut res;
    let mut moves_index: usize = 0;

    match command[0].as_str() {
        "startpos" => {
            res = Board::new(_INITIAL_FEN_STRING);
            moves_index = 1;
        }
        "fen" => {
            let mut found_moves = false;
            for (index, token) in command[1..].iter().enumerate() {
                if token.as_str() == "moves" {
                    moves_index = index + 1;
                    found_moves = true;
                    break;
                }
            }
            if found_moves {
                res = Board::new(command[1..moves_index].join(" ").as_str());
            } else {
                res = Board::new(command[1..].join(" ").as_str());
            }
        }
        _ => {
            panic!("wrong format in position command")
        }
    }

    if moves_index != 0
        && command.len() > moves_index + 1
        && command[moves_index].as_str() == "moves"
    {
        for ucimove in &command[(moves_index + 1)..] {
            let chess_move = Move::from(ucimove.as_str());
            if let Some(move_in_legal_moves) = generate_legal_moves(&res, true)
                .iter()
                .find(|&m| m == chess_move)
            {
                res.make_move(move_in_legal_moves);
            } else {
                panic!("move in position command was not found");
            }
        }
        res.generate_legal_moves();
    }
    res
}

fn handle_stdin(tx: mpsc::Sender<Vec<String>>, stop_flag: Arc<AtomicBool>) {
    loop {
        let mut line = String::new();
        let _ = io::stdin().read_line(&mut line);
        let splits: Vec<String> = line.split_whitespace().map(|s| s.to_string()).collect();

        match splits[0].as_str() {
            "stop" => stop_flag.store(true, Ordering::SeqCst),
            "uci" => {
                println!("id name {}", env!("CARGO_PKG_NAME"));
                println!("id author Rick");
                println!("uciok");
            }
            _ => tx.send(splits).unwrap(),
        }
    }
}

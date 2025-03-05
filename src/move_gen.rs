pub mod chess_move;
pub mod move_list;

use crate::{
    board::{
        Board,
        bitboard::{Bitmanip, print_bitboard},
        gamestate::Gamestate,
        piece::*,
    },
    consts::CONSTS,
};

use self::{chess_move::Move, move_list::MoveList};

pub fn generate_legal_moves(board: &Board) -> MoveList {
    let mut res = MoveList::new();
    let us_color = board.get_color_to_move();
    let enemy_color = !us_color;
    let (us_pieces_bb, enemy_pieces_bb, us_color_bb, enemy_color_bb) =
        board.get_us_enemy_bitboards(us_color);
    let king_bit = us_pieces_bb[PieceType::King].isolate_ls1b();

    let attacks_per_piece_bb = generate_attacks(
        enemy_pieces_bb,
        enemy_color_bb | (us_color_bb ^ king_bit),
        enemy_color,
    );
    let attacks_bb = attacks_per_piece_bb
        .iter()
        .copied()
        .fold(0u64, |acc, bb| acc | bb);

    let attackers = find_attackers(
        king_bit,
        us_color,
        enemy_pieces_bb,
        enemy_color_bb | us_color_bb,
    );
    let num_attackers = attackers.count_ones();

    let capture_mask = if num_attackers == 0 {
        0xFFFFFFFFFFFFFFFF
    } else {
        attackers
    };
    let block_mask =
        if num_attackers == 1 && board.get_piece_at(attackers.bitscan()).unwrap().is_slider() {
            CONSTS::SQUARES_BETWEEN[king_bit.bitscan() as usize][attackers.bitscan() as usize]
        } else {
            0u64
        };

    let mut legal_squares = capture_mask | block_mask;

    if num_attackers <= 1 {
        let pinned_pieces = recognize_pinned_pieces(king_bit, board, us_color);
        let pinned_pieces_mask = pinned_pieces
            .iter()
            .map(|(pinned, _)| pinned)
            .fold(0, |acc, bb| acc | bb);

        if num_attackers == 0 {
            generate_moves_for_pinned_pieces(&pinned_pieces, us_color, board, &mut res);

            generate_castles(
                king_bit,
                us_pieces_bb[PieceType::Rook],
                us_color_bb | enemy_color_bb,
                attacks_bb,
                us_color,
                board.current_gamestate,
                &mut res,
            )
        }

        // Sliding Piece Moves are generated using Hyperbola Quintessence
        for i in 0u8..=3u8 {
            let piece_type = PieceType::from(i);
            generate_moves_for_piece(
                piece_type,
                us_color,
                !pinned_pieces_mask,
                legal_squares,
                board,
                &mut res,
            );
        }

        // this is handled separately because it needs to consider the case in which an enpassant capture removes
        // an enemy attacker (in this specific case the landing index of the move is different from the attacker index)
        if let Some(enpassant_index) = board.current_gamestate.get_enpassant_square() {
            let enemy_pawn_index = match us_color {
                PieceColor::White => enpassant_index - 8,
                PieceColor::Black => enpassant_index + 8,
            };

            if legal_squares.contains_index(enemy_pawn_index) {
                legal_squares.set_square(enpassant_index);
            }
        }
        generate_moves_for_piece(
            PieceType::Pawn,
            us_color,
            !pinned_pieces_mask,
            legal_squares,
            board,
            &mut res,
        );
    }

    generate_king_moves(
        us_pieces_bb[PieceType::King],
        us_color_bb,
        attacks_bb,
        &mut res,
    );

    res
}

pub fn generate_attacks(
    pieces_bb: [u64; 6],
    occupied_bb: u64,
    piece_color: PieceColor,
) -> [u64; 6] {
    let mut res = [0u64; 6];

    res[PieceType::Rook] = generate_rook_attacks(pieces_bb[PieceType::Rook], occupied_bb);
    if pieces_bb[PieceType::Knight] != 0 {
        res[PieceType::Knight] = generate_knight_attacks(pieces_bb[PieceType::Knight]);
    }
    res[PieceType::Bishop] = generate_bishop_attacks(pieces_bb[PieceType::Bishop], occupied_bb);
    res[PieceType::Queen] = generate_queen_attacks(pieces_bb[PieceType::Queen], occupied_bb);
    if pieces_bb[PieceType::King] != 0 {
        res[PieceType::King] = generate_king_attacks(pieces_bb[PieceType::King]);
    }
    res[PieceType::Pawn] = generate_pawn_attacks(pieces_bb[PieceType::Pawn], piece_color);

    res
}

fn generate_knight_attacks(mut knight_bb: u64) -> u64 {
    let mut res = 0u64;

    while knight_bb != 0 {
        let index = knight_bb.bitscan_reset();
        let moves_bb = CONSTS::KNIGHT_TABLE[index as usize];

        res |= moves_bb;
    }

    res
}

fn generate_rook_attacks(mut rook_bb: u64, occupied_bb: u64) -> u64 {
    let mut res = 0u64;

    while rook_bb != 0 {
        let index = rook_bb.bitscan_reset();
        let moves_bb = file_moves(occupied_bb, index) + rank_moves(occupied_bb, index);

        res |= moves_bb;
    }

    res
}

fn generate_bishop_attacks(mut bishop_bb: u64, occupied_bb: u64) -> u64 {
    let mut res = 0u64;

    while bishop_bb != 0 {
        let index = bishop_bb.bitscan_reset();
        let moves_bb = diagonal_moves(occupied_bb, index) + antidiagonal_moves(occupied_bb, index);

        res |= moves_bb;
    }
    res
}

fn generate_queen_attacks(queen_bb: u64, occupied_bb: u64) -> u64 {
    generate_bishop_attacks(queen_bb, occupied_bb) | generate_rook_attacks(queen_bb, occupied_bb)
}

fn generate_king_attacks(mut king_bb: u64) -> u64 {
    CONSTS::KING_TABLE[king_bb.bitscan_reset() as usize]
}

fn generate_pawn_attacks(pawn_bb: u64, piece_color: PieceColor) -> u64 {
    let (east_attacks, west_attacks): (u64, u64);

    if piece_color == PieceColor::White {
        east_attacks = (pawn_bb << 9) & CONSTS::NOT_A_FILE;
        west_attacks = (pawn_bb << 7) & CONSTS::NOT_H_FILE;
    } else {
        east_attacks = (pawn_bb >> 7) & CONSTS::NOT_A_FILE;
        west_attacks = (pawn_bb >> 9) & CONSTS::NOT_H_FILE;
    }

    east_attacks | west_attacks
}

fn find_attackers(
    king_bb: u64,
    king_color: PieceColor,
    enemy_pieces_bb: [u64; 6],
    occupied_bb: u64,
) -> u64 {
    let mut attackers = 0u64;
    let mut attacks: u64;

    attacks = generate_knight_attacks(king_bb);
    attackers |= attacks & enemy_pieces_bb[PieceType::Knight];

    attacks = generate_bishop_attacks(king_bb, occupied_bb);
    attackers |= attacks & enemy_pieces_bb[PieceType::Bishop];

    attacks = generate_rook_attacks(king_bb, occupied_bb);
    attackers |= attacks & enemy_pieces_bb[PieceType::Rook];

    attacks = generate_queen_attacks(king_bb, occupied_bb);
    attackers |= attacks & enemy_pieces_bb[PieceType::Queen];

    attacks = generate_pawn_attacks(king_bb, king_color);
    attackers |= attacks & enemy_pieces_bb[PieceType::Pawn];

    attackers
}

fn xray_rook_attacks(occupied_bb: u64, mut blockers_bb: u64, rook_bit: u64) -> u64 {
    let attacks = generate_rook_attacks(rook_bit, occupied_bb);
    blockers_bb &= attacks;
    attacks ^ generate_rook_attacks(rook_bit, occupied_bb ^ blockers_bb)
}

fn xray_bishop_attacks(occupied_bb: u64, mut blockers_bb: u64, bishop_bit: u64) -> u64 {
    let attacks = generate_bishop_attacks(bishop_bit, occupied_bb);
    blockers_bb &= attacks;
    attacks ^ generate_bishop_attacks(bishop_bit, occupied_bb ^ blockers_bb)
}

fn recognize_pinned_pieces(
    king_bit: u64,
    board: &Board,
    color_to_move: PieceColor,
) -> Vec<(u64, u64)> {
    let mut pinned_pieces = Vec::new();
    let (us_pieces_bb, enemy_pieces_bb, us_color_bb, enemy_color_bb) =
        board.get_us_enemy_bitboards(color_to_move);
    let occupied_bb = us_color_bb | enemy_color_bb;

    // Find rook and queen pins
    let mut pinner = xray_rook_attacks(occupied_bb, us_color_bb, king_bit)
        & (enemy_pieces_bb[PieceType::Rook] | enemy_pieces_bb[PieceType::Queen]);

    while pinner != 0 {
        let index = pinner.bitscan_reset();
        let mut squares_between =
            CONSTS::SQUARES_BETWEEN[index as usize][king_bit.bitscan() as usize];
        let pinned = squares_between & us_color_bb;

        squares_between.set_square(index); // Add capturing the pinning piece to the list of legal moves

        if pinned != 0 {
            pinned_pieces.push((pinned, squares_between));
        }
    }

    // Find bishop and queen pins
    pinner = xray_bishop_attacks(occupied_bb, us_color_bb, king_bit)
        & (enemy_pieces_bb[PieceType::Bishop] | enemy_pieces_bb[PieceType::Queen]);

    while pinner != 0 {
        let index = pinner.bitscan_reset();
        let mut squares_between =
            CONSTS::SQUARES_BETWEEN[index as usize][king_bit.bitscan() as usize];
        let pinned = squares_between & us_color_bb;

        squares_between.set_square(index); // Add capturing the pinning piece to the list of legal moves

        if pinned != 0 {
            pinned_pieces.push((pinned, squares_between));
        }
    }

    // Check enpassant discovered check
    if let Some(enpassant_index) = board.current_gamestate.get_enpassant_square() {
        let king_index = king_bit.bitscan();
        if (king_index as i64 / 8 - enpassant_index as i64 / 8).abs() == 1    //on consecutive rows
            && generate_pawn_attacks(us_pieces_bb[PieceType::Pawn], color_to_move).contains_index(enpassant_index)
        {
            let mut enemy_rooks_queens_bb =
                enemy_pieces_bb[PieceType::Queen] | enemy_pieces_bb[PieceType::Rook];

            while enemy_rooks_queens_bb != 0 {
                let enemy_piece_index = enemy_rooks_queens_bb.bitscan_reset();

                if king_index / 8 != enemy_piece_index / 8 {
                    // not on the same row
                    continue;
                }

                let pinned_pieces_bb = CONSTS::SQUARES_BETWEEN[king_index as usize]
                    [enemy_piece_index as usize]
                    & occupied_bb;
                if pinned_pieces_bb.count_ones() == 2 {
                    pinned_pieces.push((pinned_pieces_bb & us_color_bb, !(1 << enpassant_index)));
                    break;
                }
            }
        }
    }

    pinned_pieces
}

fn generate_moves_for_pinned_pieces(
    pinned_pieces: &[(u64, u64)],
    us_color: PieceColor,
    board: &Board,
    move_list: &mut MoveList,
) {
    for &(pinned_bb, restriction_mask) in pinned_pieces {
        let pinned_piece = board.get_piece_at(pinned_bb.bitscan());

        if let Some(piece) = pinned_piece {
            generate_moves_for_piece(
                piece.get_type(),
                us_color,
                pinned_bb,
                restriction_mask,
                board,
                move_list,
            );
        }
    }
}

fn generate_knights_moves(
    mut knights_bb: u64,
    us_color_bb: u64,
    legal_squares_bb: u64,
    move_list: &mut MoveList,
) {
    while knights_bb != 0 {
        let index = knights_bb.bitscan_reset();
        let moves_bb = CONSTS::KNIGHT_TABLE[index as usize] & !us_color_bb & legal_squares_bb;

        move_list.append_bb(moves_bb, index);
    }
}

fn generate_king_moves(
    mut king_bb: u64,
    us_color_bb: u64,
    attacks_bb: u64,
    move_list: &mut MoveList,
) {
    let index = king_bb.bitscan_reset();
    let moves_bb = CONSTS::KING_TABLE[index as usize] & !us_color_bb & !attacks_bb;

    move_list.append_bb(moves_bb, index);
}

fn generate_white_pawns_moves(
    pawns_bb: u64,
    us_color_bb: u64,
    mut enemy_color_bb: u64,
    legal_squares_bb: u64,
    opt_enpassant_square: Option<u64>,
    move_list: &mut MoveList,
) {
    const RANK_4: u64 = CONSTS::MASKS[3 * 8].rank_mask;

    let empty_bb = !(us_color_bb | enemy_color_bb);
    let mut single_pushes = (pawns_bb << 8) & empty_bb;
    let double_pushes = (single_pushes << 8) & RANK_4 & empty_bb & legal_squares_bb;
    single_pushes &= legal_squares_bb; // need to do this after generating the double pushes because of some limit case (rnbqkbnr/ppp1pppp/8/3p4/8/2P5/PP1PPPPP/RNBQKBNR w KQkq d6 0 2)

    if let Some(enpassant_square) = opt_enpassant_square {
        // count an enemy piece on the enpassant square
        enemy_color_bb.toggle_square(enpassant_square);
    }
    let east_attacks = (pawns_bb << 9) & CONSTS::NOT_A_FILE & enemy_color_bb & legal_squares_bb;
    let west_attacks = (pawns_bb << 7) & CONSTS::NOT_H_FILE & enemy_color_bb & legal_squares_bb;

    move_list.append_bb_pawn_pushes(single_pushes, -8);
    move_list.append_bb_pawn_pushes(double_pushes, -16);
    move_list.append_bb_pawn_attacks(east_attacks, -9, opt_enpassant_square);
    move_list.append_bb_pawn_attacks(west_attacks, -7, opt_enpassant_square);
}

fn generate_black_pawns_moves(
    pawns_bb: u64,
    us_color_bb: u64,
    mut enemy_color_bb: u64,
    legal_squares_bb: u64,
    opt_enpassant_square: Option<u64>,
    move_list: &mut MoveList,
) {
    const RANK_5: u64 = CONSTS::MASKS[4 * 8].rank_mask;

    let empty_bb = !(us_color_bb | enemy_color_bb);
    let mut single_pushes = (pawns_bb >> 8) & empty_bb;
    let double_pushes = (single_pushes >> 8) & RANK_5 & empty_bb & legal_squares_bb;
    single_pushes &= legal_squares_bb; // need to do this after generating the double pushes because of some limit case (rnbqkbnr/ppp1pppp/8/3p4/8/2P5/PP1PPPPP/RNBQKBNR w KQkq d6 0 2)

    if let Some(enpassant_square) = opt_enpassant_square {
        // count an enemy piece on the enpassant square
        enemy_color_bb.toggle_square(enpassant_square);
    }
    let east_attacks = (pawns_bb >> 7) & CONSTS::NOT_A_FILE & enemy_color_bb & legal_squares_bb;
    let west_attacks = (pawns_bb >> 9) & CONSTS::NOT_H_FILE & enemy_color_bb & legal_squares_bb;

    move_list.append_bb_pawn_pushes(single_pushes, 8);
    move_list.append_bb_pawn_pushes(double_pushes, 16);
    move_list.append_bb_pawn_attacks(east_attacks, 7, opt_enpassant_square);
    move_list.append_bb_pawn_attacks(west_attacks, 9, opt_enpassant_square);
}

pub fn generate_promotion_moves(mut promotions: u64, offset: i64, move_list: &mut MoveList) {
    while promotions != 0 {
        let to = promotions.bitscan_reset();

        for i in 0u8..=3u8 {
            let mut m = Move::new((to as i64 + offset) as u64, to);
            m.add_promotion(PieceType::from(i));

            move_list.push(m);
        }
    }
}

fn generate_bishop_moves(
    mut bishop_bb: u64,
    us_color_bb: u64,
    enemy_color_bb: u64,
    legal_squares_bb: u64,
    move_list: &mut MoveList,
) {
    let blockers_bb = us_color_bb | enemy_color_bb;

    while bishop_bb != 0 {
        let index = bishop_bb.bitscan_reset();
        let moves_bb = (diagonal_moves(blockers_bb, index)
            + antidiagonal_moves(blockers_bb, index))
            & !us_color_bb
            & legal_squares_bb;

        move_list.append_bb(moves_bb, index);
    }
}

fn generate_rook_moves(
    mut rook_bb: u64,
    us_color_bb: u64,
    enemy_color_bb: u64,
    legal_squares_bb: u64,
    move_list: &mut MoveList,
) {
    let blockers_bb = us_color_bb | enemy_color_bb;

    while rook_bb != 0 {
        let index = rook_bb.bitscan_reset();
        let moves_bb = (file_moves(blockers_bb, index) + rank_moves(blockers_bb, index))
            & !us_color_bb
            & legal_squares_bb;

        move_list.append_bb(moves_bb, index);
    }
}

fn generate_queen_moves(
    queen_bb: u64,
    us_color_bb: u64,
    enemy_color_bb: u64,
    legal_squares_bb: u64,
    move_list: &mut MoveList,
) {
    generate_bishop_moves(
        queen_bb,
        us_color_bb,
        enemy_color_bb,
        legal_squares_bb,
        move_list,
    );
    generate_rook_moves(
        queen_bb,
        us_color_bb,
        enemy_color_bb,
        legal_squares_bb,
        move_list,
    );
}

fn diagonal_moves(blockers_bb: u64, index: u64) -> u64 {
    let mut forward: u64;
    let mut reverse: u64;

    forward = blockers_bb & CONSTS::MASKS[index as usize].diagonal_mask_ex;
    reverse = forward.swap_bytes();
    forward = forward.wrapping_sub(1u64 << index);
    reverse = reverse.wrapping_sub((1u64 << index).swap_bytes());
    forward ^= reverse.swap_bytes();
    forward &= CONSTS::MASKS[index as usize].diagonal_mask_ex;

    forward
}

fn antidiagonal_moves(blockers_bb: u64, index: u64) -> u64 {
    let mut forward: u64;
    let mut reverse: u64;

    forward = blockers_bb & CONSTS::MASKS[index as usize].antidiag_mask_ex;
    reverse = forward.swap_bytes();
    forward = forward.wrapping_sub(1u64 << index);
    reverse = reverse.wrapping_sub((1u64 << index).swap_bytes());
    forward ^= reverse.swap_bytes();
    forward &= CONSTS::MASKS[index as usize].antidiag_mask_ex;

    forward
}

fn file_moves(blockers_bb: u64, index: u64) -> u64 {
    let mut forward: u64;
    let mut reverse: u64;

    forward = blockers_bb & CONSTS::MASKS[index as usize].file_mask_ex;
    reverse = forward.swap_bytes();
    forward = forward.wrapping_sub(1u64 << index);
    reverse = reverse.wrapping_sub((1u64 << index).swap_bytes());
    forward ^= reverse.swap_bytes();
    forward &= CONSTS::MASKS[index as usize].file_mask_ex;

    forward
}

/// https://timcooijmans.blogspot.com/2014/04/hyperbola-quintessence-for-rooks-along.html
fn rank_moves(blockers_bb: u64, index: u64) -> u64 {
    let rank_index = index / 8;
    let mut occupancy = (blockers_bb & CONSTS::MASKS[index as usize].rank_mask) >> (rank_index * 8);
    let mut piece = (1u64 << index) >> (rank_index * 8);

    occupancy *= 0x0101010101010101;
    piece *= 0x0101010101010101;

    occupancy &= 0x8040201008040201;
    piece &= 0x8040201008040201;

    let diag_index = piece.bitscan_reset();

    let mut moves = diagonal_moves(occupancy, diag_index);

    moves = moves.wrapping_mul(0x0101010101010101);
    moves /= 0x0100000000000000;
    moves << (rank_index * 8)
}

fn generate_moves_for_piece(
    piece_type: PieceType,
    us_color: PieceColor,
    valid_pieces_mask: u64,
    restricted_squares: u64,
    board: &Board,
    move_list: &mut MoveList,
) {
    let piece_bb = board.get_pieces_bb()[us_color][piece_type] & valid_pieces_mask;
    let (us_color_bb, enemy_color_bb) = board.get_us_enemy_colors_bb(us_color);

    match piece_type {
        PieceType::Rook => generate_rook_moves(
            piece_bb,
            us_color_bb,
            enemy_color_bb,
            restricted_squares,
            move_list,
        ),
        PieceType::Bishop => generate_bishop_moves(
            piece_bb,
            us_color_bb,
            enemy_color_bb,
            restricted_squares,
            move_list,
        ),
        PieceType::Queen => generate_queen_moves(
            piece_bb,
            us_color_bb,
            enemy_color_bb,
            restricted_squares,
            move_list,
        ),
        PieceType::Knight => {
            if piece_bb != 0 {
                generate_knights_moves(piece_bb, us_color_bb, restricted_squares, move_list)
            }
        }
        PieceType::Pawn => match us_color {
            PieceColor::White => generate_white_pawns_moves(
                piece_bb,
                us_color_bb,
                enemy_color_bb,
                restricted_squares,
                board.current_gamestate.get_enpassant_square(),
                move_list,
            ),
            PieceColor::Black => generate_black_pawns_moves(
                piece_bb,
                us_color_bb,
                enemy_color_bb,
                restricted_squares,
                board.current_gamestate.get_enpassant_square(),
                move_list,
            ),
        },
        _ => panic!("Is generating pinned moves for kings"),
    }
}

fn generate_castles(
    king_bit: u64,
    rooks_bb: u64,
    occupied_bb: u64,
    attacked_bb: u64,
    us_color: PieceColor,
    gamestate: Gamestate,
    move_list: &mut MoveList,
) {
    if gamestate.can_castle_kingside(us_color)
        && CONSTS::CASTLING_MASKS_KINGSIDE[us_color] & (occupied_bb | attacked_bb) == 0
        && CONSTS::CASTLING_ROOK_INDEX_KINGSIDE[us_color] & rooks_bb != 0
    {
        let king_index = king_bit.bitscan();
        let mut m = Move::new(king_index, king_index + 2);
        m.add_castle_kingside();

        move_list.push(m);
    }

    if gamestate.can_castle_queenside(us_color)
        && CONSTS::CASTLING_EMPTY_MASKS_QUEENSIDE[us_color] & occupied_bb == 0
        && CONSTS::CASTLING_ATTACKED_MASKS_QUEENSIDE[us_color] & attacked_bb == 0
        && CONSTS::CASTLING_ROOK_INDEX_QUEENSIDE[us_color] & rooks_bb != 0
    {
        let king_index = king_bit.bitscan();
        let mut m = Move::new(king_index, king_index - 2);
        m.add_castle_queenside();

        move_list.push(m);
    }
}

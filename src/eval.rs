use chess::{Board, BoardStatus, ChessMove, Color, Piece};

use crate::{
    helpers::{color_to_num, N_INF},
    score::ScoreTy,
};

#[inline(always)]
fn count_piece(board: Board, piece: Piece, color: Color) -> ScoreTy {
    let ps = board.pieces(piece);
    ((ps & board.color_combined(color)).0.count_ones()) as ScoreTy
}

const QUEEN_WT: ScoreTy = 975;
const ROOK_WT: ScoreTy = 500;
const BISHOP_WT: ScoreTy = 335;
const KNIGHT_WT: ScoreTy = 325;
const PAWN_WT: ScoreTy = 100;

#[inline(always)]
pub fn get_score(piece: Piece) -> ScoreTy {
    match piece {
        Piece::Pawn => PAWN_WT,
        Piece::Knight => KNIGHT_WT,
        Piece::Bishop => BISHOP_WT,
        Piece::Rook => ROOK_WT,
        Piece::Queen => QUEEN_WT,
        Piece::King => 1500,
    }
}

#[inline(always)]
fn material(board: Board, color: Color) -> ScoreTy {
    let queen_s = QUEEN_WT * count_piece(board, Piece::Queen, color);
    let rook_s = ROOK_WT * count_piece(board, Piece::Rook, color);
    let bishop_s = BISHOP_WT * count_piece(board, Piece::Bishop, color);
    let knight_s = KNIGHT_WT * count_piece(board, Piece::Knight, color);
    let pawn_s = PAWN_WT * count_piece(board, Piece::Pawn, color);
    queen_s + rook_s + bishop_s + knight_s + pawn_s
}

const ENDGAME_MAT: ScoreTy = 1300;

#[inline(always)]
pub fn is_endgame(board: Board) -> bool {
    material(board, board.side_to_move()) < ENDGAME_MAT
}

const ROOK_PAIR: ScoreTy = -16;
const KNIGHT_PAIR: ScoreTy = -8;
const BISHOP_PAIR: ScoreTy = 30;

#[inline(always)]
fn pairs(board: Board, color: Color) -> ScoreTy {
    count_piece(board, Piece::Bishop, color) % 2 * BISHOP_PAIR
        + count_piece(board, Piece::Knight, color) % 2 * KNIGHT_PAIR
        + count_piece(board, Piece::Rook, color) % 2 * ROOK_PAIR
}

const MOBILITY_WT: ScoreTy = 1;

#[inline(always)]
#[allow(deprecated)]
fn mobility(board: Board, color: Color) -> ScoreTy {
    let new_board = if board.side_to_move() == color {
        Some(board)
    } else {
        board.null_move()
    };

    return (new_board
        .map(|b| b.enumerate_moves(&mut [ChessMove::default(); 256]))
        .unwrap_or(20) as ScoreTy)
        * MOBILITY_WT;
}

#[inline(always)]
pub fn evaluate(board: Board) -> ScoreTy {
    match board.status() {
        BoardStatus::Ongoing => {
            let material_delta = material(board, Color::White) - material(board, Color::Black);
            let pairs_delta = pairs(board, Color::White) - pairs(board, Color::Black);
            let mobilty_delta = mobility(board, Color::White) - mobility(board, Color::Black);
            let score = material_delta + pairs_delta + mobilty_delta;
            score * color_to_num(board.side_to_move())
        }
        BoardStatus::Checkmate => N_INF + 1,
        BoardStatus::Stalemate => 0,
    }
}

use chess::{Board, BoardStatus, Color, Piece};

use crate::{
    helpers::{color_to_num, N_INF},
    score::ScoreTy,
};

const QUEEN_WT: ScoreTy = 975;
const ROOK_WT: ScoreTy = 500;
const BISHOP_WT: ScoreTy = 335;
const KNIGHT_WT: ScoreTy = 325;
const PAWN_WT: ScoreTy = 100;

#[inline(always)]
fn delta(board: Board, piece: Piece) -> ScoreTy {
    let ps = board.pieces(piece);
    ((ps & board.color_combined(Color::White)).0.count_ones() as ScoreTy)
        - ((ps & board.color_combined(Color::Black)).0.count_ones() as ScoreTy)
}

#[inline(always)]
pub fn evaluate(board: Board) -> ScoreTy {
    match board.status() {
        BoardStatus::Ongoing => {
            let queen_s = QUEEN_WT * delta(board, Piece::Queen);
            let rook_s = ROOK_WT * delta(board, Piece::Rook);
            let bishop_s = BISHOP_WT * delta(board, Piece::Bishop);
            let knight_s = KNIGHT_WT * delta(board, Piece::Knight);
            let pawn_s = PAWN_WT * delta(board, Piece::Pawn);
            (queen_s + rook_s + bishop_s + knight_s + pawn_s) * color_to_num(board.side_to_move())
        }
        BoardStatus::Checkmate => N_INF + 1,
        BoardStatus::Stalemate => 0,
    }
}

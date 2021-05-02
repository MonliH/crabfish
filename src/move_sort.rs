use chess::{BitBoard, Board, ChessMove, EMPTY};
use std::cmp::Reverse;

use crate::eval::get_score;

#[inline(always)]
fn no_capture(board: &Board, m: &ChessMove) -> bool {
    let bit = BitBoard::from_square(m.get_dest());
    return board.combined() & bit == EMPTY;
}

#[inline(always)]
pub fn sort_moves(board: &Board, moves: &mut [ChessMove]) {
    moves.sort_by_key(|m| no_capture(&board, m));
}

#[inline(always)]
pub fn sort_qs(board: &Board, moves: &mut [ChessMove]) {
    moves.sort_by_key(|m| {
        (
            Reverse(board.piece_on(m.get_dest()).map(|p| get_score(p))),
            board.piece_on(m.get_source()).map(|p| get_score(p)),
        )
    })
}

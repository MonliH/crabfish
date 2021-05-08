use chess::{Board, ChessMove};
use std::cmp::Reverse;

use crate::{eval::get_score, score::ScoreTy, search::KILLER_MOVES};

#[inline(always)]
fn mvv_lva(board: &Board, m: &ChessMove) -> Reverse<Option<(ScoreTy, Reverse<ScoreTy>)>> {
    return Reverse(board.piece_on(m.get_dest()).map(|p| {
        (
            get_score(p),
            Reverse(get_score(board.piece_on(m.get_source()).unwrap())),
        )
    }));
}

#[inline(always)]
pub fn sort_moves(
    board: &Board,
    moves: &mut [ChessMove],
    killer_moves: &[Option<ChessMove>; KILLER_MOVES],
) {
    moves.sort_by_key(|m| {
        let mvv_lva = mvv_lva(board, m);
        (
            mvv_lva,
            killer_moves
                .iter()
                .position(|v| v.as_ref() == Some(m))
                .unwrap_or(KILLER_MOVES + 1),
        )
    });
}

#[inline(always)]
pub fn sort_qs(board: &Board, moves: &mut [ChessMove]) {
    moves.sort_by_key(|m| mvv_lva(board, m));
}

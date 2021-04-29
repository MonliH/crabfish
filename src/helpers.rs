use chess::{Board, BoardStatus, Color};

use crate::score::ScoreTy;

#[inline(always)]
pub fn game_over(board: Board) -> bool {
    match board.status() {
        BoardStatus::Ongoing => false,
        BoardStatus::Stalemate | BoardStatus::Checkmate => true,
    }
}

#[inline(always)]
pub fn color_to_num(color: Color) -> ScoreTy {
    match color {
        Color::White => 1,
        Color::Black => -1,
    }
}

pub const N_INF: ScoreTy = ScoreTy::MIN + 1;
pub const P_INF: ScoreTy = ScoreTy::MAX;

use chess::{Board, ChessMove, MoveGen};
use smallvec::{smallvec, SmallVec};

use crate::{
    eval::{evaluate, is_endgame},
    helpers::{game_over, N_INF, P_INF},
    move_sort::{sort_moves, sort_qs},
    score::ScoreTy,
    transposition::{CacheItem, Flag, TTable},
};

const R: u8 = 2;
const DEPTH: usize = 12;
pub const KILLER_MOVES: usize = 3;

#[derive(Clone)]
pub struct Engine {
    killer_moves: SmallVec<[[Option<ChessMove>; KILLER_MOVES]; DEPTH]>,
    memo: TTable,
    pub nodes_searched: usize,
    pub search_id: usize,
}

impl Engine {
    pub fn new(search_id: usize, memo: TTable) -> Self {
        Self {
            nodes_searched: 0,
            memo,
            killer_moves: smallvec![[None; KILLER_MOVES]; DEPTH],
            search_id,
        }
    }

    #[inline]
    fn quiesce(&mut self, board: Board, mut alpha: ScoreTy, beta: ScoreTy) -> ScoreTy {
        self.nodes_searched += 1;
        let standing_pat = evaluate(board);
        if standing_pat >= beta {
            return beta;
        }
        if alpha < standing_pat {
            alpha = standing_pat
        }

        let mut movegen = MoveGen::new_legal(&board);
        let targets = board.color_combined(!board.side_to_move());
        // Filter down to attacking moves
        movegen.set_iterator_mask(*targets);
        let mut possible_moves = [ChessMove::default(); 256];
        let mut count = 0;
        for m in movegen {
            possible_moves[count] = m;
            count += 1;
        }
        sort_qs(&board, &mut possible_moves[..count]);

        for i in 0..count {
            let m = possible_moves[i];
            let new_board = board.make_move_new(m);
            let score = -self.quiesce(new_board, -beta, -alpha);
            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
            }
        }

        alpha
    }

    #[inline]
    #[allow(deprecated)]
    fn pvs(
        &mut self,
        start_depth: u8,
        depth: u8,
        board: Board,
        mut alpha: ScoreTy,
        mut beta: ScoreTy,
        pv: Option<ChessMove>,
        can_null: bool,
        time_up: &dyn Fn() -> bool,
    ) -> Option<ScoreTy> {
        if time_up() {
            return None;
        }

        let orig_alpha = alpha;
        let ply = (start_depth - depth) as usize;

        if let Some(entry) = self.memo.get(board.get_hash()) {
            if entry.depth >= depth {
                match entry.flag {
                    Flag::Exact => return Some(entry.value),
                    Flag::LowerBound => alpha = ScoreTy::max(alpha, entry.value),
                    Flag::UpperBound => beta = ScoreTy::max(beta, entry.value),
                }

                if alpha >= beta {
                    return Some(entry.value);
                }
            }
        }

        self.nodes_searched += 1;

        if depth == 0 || game_over(board) {
            return Some(self.quiesce(board, alpha, beta));
        }

        let not_checked = board.checkers().0 == 0;
        let not_endgame = !is_endgame(board);

        // Null Move Pruning
        if not_checked
            && can_null
            && depth > R
            && (ScoreTy::abs(beta - 1) > N_INF + 100)
            && not_endgame
        {
            let adapt_r = if depth > 6 { R + 1 } else { R };
            let nulled = board.null_move().unwrap();
            let score = -self.pvs(
                start_depth,
                depth - 1 - adapt_r,
                nulled,
                -beta,
                -beta + 1,
                None,
                false,
                time_up,
            )?;
            if score >= beta {
                return Some(score);
            }
        }

        // Reverse Futility Pruning
        if depth < 3 && not_checked && (ScoreTy::abs(beta - 1) > N_INF + 100) {
            let static_eval = evaluate(board);

            let eval_margin = 120 * depth as ScoreTy;
            if (static_eval - eval_margin) >= beta {
                return Some(static_eval - eval_margin);
            }
        }

        let mut possible_moves = [ChessMove::default(); 256];
        let count = board.enumerate_moves(&mut possible_moves);
        let killer_moves = self
            .killer_moves
            .get(ply as usize)
            .unwrap_or(&[None; KILLER_MOVES]);
        sort_moves(&board, &mut possible_moves[..count], killer_moves);
        let mut is_pv = true;

        for i in 0..count {
            let m = possible_moves[i];
            let new_board = board.make_move_new(m);
            let best_score = if Some(m) == pv && is_pv {
                is_pv = false;
                -self.pvs(
                    start_depth,
                    depth - 1,
                    new_board,
                    -beta,
                    -alpha,
                    None,
                    true,
                    time_up,
                )?
            } else {
                // Null Window Search
                let s = -self.pvs(
                    start_depth,
                    depth - 1,
                    new_board,
                    -alpha - 1,
                    -alpha,
                    None,
                    true,
                    time_up,
                )?;
                if alpha < s && s < beta {
                    -self.pvs(
                        start_depth,
                        depth - 1,
                        new_board,
                        -beta,
                        -s,
                        None,
                        true,
                        time_up,
                    )?
                } else {
                    s
                }
            };
            alpha = ScoreTy::max(alpha, best_score);
            if alpha >= beta {
                while self.killer_moves.len() <= ply {
                    self.killer_moves.push([None; KILLER_MOVES]);
                }
                self.killer_moves[ply].rotate_right(1);
                self.killer_moves[ply][0] = Some(m);
                break;
            }
        }

        let entry_flag = if alpha <= orig_alpha {
            Flag::UpperBound
        } else if alpha >= beta {
            Flag::LowerBound
        } else {
            Flag::Exact
        };

        self.memo
            .set(CacheItem::new(depth, entry_flag, alpha, board.get_hash()));

        Some(alpha)
    }

    pub fn pvs_root(
        &mut self,
        depth: u8,
        board: Board,
        pv: Option<ChessMove>,
        time_up: &dyn Fn() -> bool,
    ) -> Option<(ChessMove, ScoreTy)> {
        let start_depth = depth;
        if depth == 0 || game_over(board) {
            return None;
        }

        let mut alpha = N_INF;
        let beta = P_INF;

        let possible_moves = MoveGen::new_legal(&board);

        let mut best_move = None;
        for m in possible_moves {
            let new_board = board.make_move_new(m);
            let score = if let Some(score) = self.pvs(
                start_depth,
                depth - 1,
                new_board,
                -beta,
                -alpha,
                pv,
                true,
                time_up,
            ) {
                -score
            } else {
                return None;
            };
            if score > alpha {
                alpha = score;
                best_move = Some((m, alpha));
            }
        }

        best_move
    }
}

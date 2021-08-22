use std::sync::atomic::Ordering;

use smallvec::{smallvec, SmallVec};

use crate::{
    eval::{evaluate, is_endgame},
    helpers::{game_over, N_INF, P_INF},
    move_sort::{sort_moves, sort_qs},
    score::ScoreTy,
    transposition::{CacheItem, Flag},
    TIME_UP,
};

const R: u8 = 2;
const DEPTH: usize = 12;
pub const KILLER_MOVES: usize = 3;

pub struct Engine {
    memo: CacheTable<CacheItem>,
    killer_moves: SmallVec<[[Option<ChessMove>; KILLER_MOVES]; DEPTH]>,
    nodes_searched: usize,
    cached_timeup: bool,
}

impl Engine {
    pub fn new(size: usize) -> Self {
        Self {
            memo: CacheTable::new(size, CacheItem::default()),
            nodes_searched: 0,
            killer_moves: smallvec![[None; KILLER_MOVES]; DEPTH],
            cached_timeup: TIME_UP.load(Ordering::SeqCst),
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
    ) -> ScoreTy {
        if !self.cached_timeup && ((self.nodes_searched & 4095) == 0) {
            self.cached_timeup = TIME_UP.load(Ordering::SeqCst);
        }
        if self.cached_timeup {
            return 0;
        }

        let orig_alpha = alpha;
        let ply = (start_depth - depth) as usize;

        if let Some(entry) = self.memo.get(board.get_hash()) {
            if entry.depth >= depth {
                match entry.flag {
                    Flag::Exact => return entry.value,
                    Flag::LowerBound => alpha = ScoreTy::max(alpha, entry.value),
                    Flag::UpperBound => beta = ScoreTy::max(beta, entry.value),
                }

                if alpha >= beta {
                    return entry.value;
                }
            }
        }

        self.nodes_searched += 1;

        if depth == 0 || game_over(board) {
            return self.quiesce(board, alpha, beta);
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
            );
            if score >= beta {
                return score;
            }
        }

        // Reverse Futility Pruning
        if depth < 3 && not_checked && (ScoreTy::abs(beta - 1) > N_INF + 100) {
            let static_eval = evaluate(board);

            let eval_margin = 120 * depth as ScoreTy;
            if (static_eval - eval_margin) >= beta {
                return static_eval - eval_margin;
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
                -self.pvs(start_depth, depth - 1, new_board, -beta, -alpha, None, true)
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
                );
                if alpha < s && s < beta {
                    -self.pvs(start_depth, depth - 1, new_board, -beta, -s, None, true)
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

        self.memo.add(
            board.get_hash(),
            CacheItem {
                depth,
                flag: entry_flag,
                value: alpha,
            },
        );

        alpha
    }

    fn pvs_root(
        &mut self,
        depth: u8,
        board: Board,
        pv: Option<ChessMove>,
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
            let score = -self.pvs(start_depth, depth - 1, new_board, -beta, -alpha, pv, true);
            if score > alpha {
                alpha = score;
                best_move = Some((m, alpha));
            }
        }

        best_move
    }

    pub fn best_move(&mut self, max_depth: u8, board: Board) -> Option<(ChessMove, ScoreTy)> {
        let mut best_move: Option<(ChessMove, ScoreTy)> = None;

        // Iterative Deepening
        for depth in 1..(max_depth + 1) {
            if !self.cached_timeup {
                self.cached_timeup = TIME_UP.load(Ordering::SeqCst);
            }
            if self.cached_timeup {
                break;
            }
            let pvs_res = self.pvs_root(depth, board, best_move.map(|(a, _)| a));
            if let Some((_, new_analysis)) = pvs_res {
                best_move = pvs_res;
                println!(
                    "info depth {} nodes {} score cp {}",
                    depth, self.nodes_searched, new_analysis
                );
            }
            self.nodes_searched = 0;
        }

        best_move
    }
}

use chess::{Board, CacheTable, ChessMove, MoveGen};

use crate::{
    eval::evaluate,
    helpers::{game_over, N_INF, P_INF},
    score::ScoreTy,
    transposition::{CacheItem, Flag},
};

pub struct Engine {
    memo: CacheTable<CacheItem>,
}

impl Engine {
    pub fn new(size: usize) -> Self {
        Self {
            memo: CacheTable::new(size, CacheItem::default()),
        }
    }

    fn quiesce(&mut self, board: Board, mut alpha: ScoreTy, beta: ScoreTy) -> ScoreTy {
        let standing_pat = evaluate(board);
        if standing_pat >= beta {
            return beta;
        }
        if alpha < standing_pat {
            alpha = standing_pat
        }

        let mut possible_moves = MoveGen::new_legal(&board);
        let targets = board.color_combined(!board.side_to_move());
        // Filter down to attacking moves
        possible_moves.set_iterator_mask(*targets);

        for m in possible_moves {
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
    fn pvs(&mut self, depth: u8, board: Board, mut alpha: ScoreTy, mut beta: ScoreTy) -> ScoreTy {
        let orig_alpha = alpha;

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

        if depth == 0 || game_over(board) {
            return self.quiesce(board, alpha, beta);
        }

        let not_checked = board.checkers().0 == 0;

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

        for i in 0..count {
            let new_board = board.make_move_new(possible_moves[i]);
            let best_score = if i == 0 {
                -self.pvs(depth - 1, new_board, -beta, -alpha)
            } else {
                let s = -self.pvs(depth - 1, new_board, -alpha - 1, -alpha);
                if alpha < s && s < beta {
                    -self.pvs(depth - 1, new_board, -beta, -s)
                } else {
                    s
                }
            };
            alpha = ScoreTy::max(alpha, best_score);
            if alpha >= beta {
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

    fn pvs_root(&mut self, depth: u8, board: Board) -> Option<ChessMove> {
        if depth == 0 || game_over(board) {
            return None;
        }

        let mut alpha = N_INF;
        let beta = P_INF;

        let possible_moves = MoveGen::new_legal(&board);

        let mut best_move = None;
        for m in possible_moves {
            let new_board = board.make_move_new(m);
            let score = -self.pvs(depth - 1, new_board, -beta, -alpha);
            if score > alpha {
                alpha = score;
                best_move = Some(m);
            }
        }

        best_move
    }

    pub fn best_move(&mut self, max_depth: u8, board: Board) -> Option<ChessMove> {
        let mut best_move = None;

        // Iterative Deepening
        for depth in 1..max_depth {
            best_move = self.pvs_root(depth, board);
        }

        best_move
    }
}

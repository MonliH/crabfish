use std::sync::{Arc, Mutex};

use crate::{score::ScoreTy, search::Engine, transposition::TTable};
use chess::{Board, ChessMove};

pub struct ThreadLauncher {
    pub memo: TTable,
    n_jobs: usize,
}

impl ThreadLauncher {
    pub fn new(tt_size: usize, n_jobs: usize) -> Self {
        Self {
            memo: TTable::new(tt_size),
            n_jobs,
        }
    }

    pub fn best_move(&mut self, max_depth: u8, board: Board) -> Option<(ChessMove, ScoreTy)> {
        let best_move: Arc<Mutex<(Option<(ChessMove, ScoreTy)>, u8)>> =
            Arc::new(Mutex::new((None, 0)));
        let mut searchers: Vec<Engine> = (1..(self.n_jobs + 1))
            .map(|id| (Engine::new(id, self.memo.clone())))
            .collect();

        let best_move_ref = &best_move;

        let _ = crossbeam::scope(|scope| {
            // Iterative Deepening
            // While current_depth < max_depth
            // spawn threads to set current depth to the depth searched, also adding to the best move
            for searcher in searchers.iter_mut() {
                scope.spawn(move |_| {
                    let best_move = Arc::clone(best_move_ref);
                    while best_move.lock().unwrap().1 < max_depth {
                        let best_move_guard = best_move.lock().unwrap();
                        let trailing_0s = searcher.search_id.trailing_zeros() as u8;
                        let sdepth = best_move_guard.1 + 1 + trailing_0s;
                        let pv = best_move_guard.0.map(|(m, _)| m);
                        std::mem::drop(best_move_guard);
                        let res = searcher.pvs_root(
                            sdepth,
                            board,
                            if trailing_0s == 0 { pv } else { None },
                            &|| false,
                        );
                        if res.is_some() {
                            let mut best_move_guard = best_move.lock().unwrap();
                            if best_move_guard.1 < sdepth {
                                *best_move_guard = (res, sdepth);
                                std::mem::drop(best_move_guard);
                                if let Some((bmove, analysis)) = res {
                                    eprintln!(
                                        "Depth {}; Best move: {}; Analysis: {};",
                                        sdepth, bmove, analysis
                                    );
                                }
                            }
                        }
                    }
                });
            }
        });

        Arc::try_unwrap(best_move).unwrap().into_inner().unwrap().0
    }
}

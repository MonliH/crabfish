use chess::{Board, BoardStatus, CacheTable, ChessMove, Color, MoveGen, Piece};

use std::io::BufRead;
use std::{io, str::FromStr};

type ScoreTy = i16;

const QUEEN_WT: ScoreTy = 9;
const ROOK_WT: ScoreTy = 5;
const BISHOP_WT: ScoreTy = 3;
const KNIGHT_WT: ScoreTy = 3;
const PAWN_WT: ScoreTy = 1;

#[inline(always)]
fn delta(board: Board, piece: Piece) -> ScoreTy {
    let ps = board.pieces(piece);
    ((ps & board.color_combined(Color::White)).0.count_ones() as ScoreTy)
        - ((ps & board.color_combined(Color::Black)).0.count_ones() as ScoreTy)
}

#[inline(always)]
fn evaluate(board: Board) -> ScoreTy {
    (match board.status() {
        BoardStatus::Ongoing => {
            let queen_s = QUEEN_WT * delta(board, Piece::Queen);
            let rook_s = ROOK_WT * delta(board, Piece::Rook);
            let bishop_s = BISHOP_WT * delta(board, Piece::Bishop);
            let knight_s = KNIGHT_WT * delta(board, Piece::Knight);
            let pawn_s = PAWN_WT * delta(board, Piece::Pawn);
            queen_s + rook_s + bishop_s + knight_s + pawn_s
        }
        BoardStatus::Checkmate => 1000 * -color_to_num(board.side_to_move()),
        BoardStatus::Stalemate => 0,
    }) * color_to_num(board.side_to_move())
}

#[inline(always)]
fn game_over(board: Board) -> bool {
    match board.status() {
        BoardStatus::Ongoing => false,
        BoardStatus::Stalemate | BoardStatus::Checkmate => true,
    }
}

#[inline(always)]
fn color_to_num(color: Color) -> ScoreTy {
    match color {
        Color::White => 1,
        Color::Black => -1,
    }
}

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
enum Flag {
    Exact,
    LowerBound,
    UpperBound,
}

impl Default for Flag {
    fn default() -> Self {
        Flag::LowerBound
    }
}

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug, Default)]
struct CacheItem {
    depth: u8,
    flag: Flag,
    value: ScoreTy,
}

pub struct Engine {
    memo: CacheTable<CacheItem>,
}

impl Engine {
    fn new(size: usize) -> Self {
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
    fn negamax(
        &mut self,
        depth: u8,
        board: Board,
        mut alpha: ScoreTy,
        mut beta: ScoreTy,
    ) -> ScoreTy {
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

        let possible_moves = MoveGen::new_legal(&board);

        let mut best_score = ScoreTy::MIN + 1;
        for m in possible_moves {
            let new_board = board.make_move_new(m);
            best_score = ScoreTy::max(
                best_score,
                -self.negamax(depth - 1, new_board, -beta, -alpha),
            );
            alpha = ScoreTy::max(alpha, best_score);
            if alpha >= beta {
                break;
            }
        }

        let entry_flag = if best_score <= orig_alpha {
            Flag::UpperBound
        } else if best_score >= beta {
            Flag::LowerBound
        } else {
            Flag::Exact
        };

        self.memo.add(
            board.get_hash(),
            CacheItem {
                depth,
                flag: entry_flag,
                value: best_score,
            },
        );

        best_score
    }

    fn negamax_root(&mut self, depth: u8, board: Board) -> Option<ChessMove> {
        if depth == 0 || game_over(board) {
            return None;
        }

        let mut alpha = ScoreTy::MIN + 1;
        let beta = ScoreTy::MAX;

        let possible_moves = MoveGen::new_legal(&board);

        let mut best_move = None;
        for m in possible_moves {
            let new_board = board.make_move_new(m);
            let score = -self.negamax(depth - 1, new_board, -beta, -alpha);
            if score > alpha {
                alpha = score;
                best_move = Some(m);
            }
        }

        best_move
    }

    fn best_move(&mut self, max_depth: u8, board: Board) -> Option<ChessMove> {
        let mut best_move = None;

        // Iterative Deepening
        for depth in 1..max_depth {
            best_move = self.negamax_root(depth, board);
        }

        best_move
    }
}

fn main() {
    let fen = io::stdin()
        .lock()
        .lines()
        .next()
        .unwrap()
        .expect("Failed to read from stdin");
    let board = Board::from_str(&fen).expect("Invalid FEN position");
    let mut engine = Engine::new(32768);

    println!("Best move: {}", engine.best_move(5, board).unwrap());
}

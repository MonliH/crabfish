use chess::{Board, BoardStatus, CacheTable, Color, MoveGen, Piece};
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
    ((ps & board.color_combined(Color::White)).0.count_ones()
        - (ps & board.color_combined(Color::Black)).0.count_ones()) as i16
}

#[inline(always)]
fn evaluate(board: Board) -> ScoreTy {
    match board.status() {
        BoardStatus::Ongoing => {
            let queen_s = QUEEN_WT * delta(board, Piece::Queen);
            let rook_s = ROOK_WT * delta(board, Piece::Rook);
            let bishop_s = BISHOP_WT * delta(board, Piece::Bishop);
            let knight_s = KNIGHT_WT * delta(board, Piece::Knight);
            let pawn_s = PAWN_WT * delta(board, Piece::Pawn);
            queen_s + rook_s + bishop_s + knight_s + pawn_s
        }
        BoardStatus::Checkmate => 1000,
        BoardStatus::Stalemate => 0,
    }
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

pub struct Engine {
    memo: CacheTable<ScoreTy>,
}

impl Engine {
    fn new(size: usize) -> Self {
        Self {
            memo: CacheTable::new(size, 0),
        }
    }

    #[inline]
    fn maxi(&mut self, depth: u8, board: Board, mut a: ScoreTy, b: ScoreTy) -> ScoreTy {
        if depth == 0 || game_over(board) {
            return evaluate(board);
        }

        let possible_moves = MoveGen::new_legal(&board);

        for m in possible_moves {
            let new_board = board.make_move_new(m);
            let score = self.mini(depth - 1, new_board, a, b);
            if score >= b {
                return b;
            }
            if score > a {
                a = score;
            }
        }

        return a;
    }

    #[inline]
    fn mini(&mut self, depth: u8, board: Board, a: ScoreTy, mut b: ScoreTy) -> ScoreTy {
        if depth == 0 || game_over(board) {
            return evaluate(board);
        }

        let possible_moves = MoveGen::new_legal(&board);

        for m in possible_moves {
            let new_board = board.make_move_new(m);
            let score = self.maxi(depth - 1, new_board, a, b);
            if score <= a {
                return a;
            }
            if score < b {
                b = score;
            }
        }

        return b;
    }

    #[inline]
    fn negamax(&mut self, depth: u8, board: Board, mut a: ScoreTy, b: ScoreTy) -> ScoreTy {
        if depth == 0 || game_over(board) {
            return evaluate(board) * (-color_to_num(board.side_to_move()));
        }

        let possible_moves = MoveGen::new_legal(&board);

        for m in possible_moves {
            let new_board = board.make_move_new(m);
            let score = -self.negamax(depth - 1, new_board, -b, -a);
            if score >= b {
                return b;
            }
            if score > a {
                a = score;
            }
        }

        return a;
    }

    fn analyze(&mut self, board: Board, depth: u8) -> ScoreTy {
        if board.side_to_move() == Color::White {
            self.maxi(depth, board, ScoreTy::MIN, ScoreTy::MAX)
        } else {
            self.maxi(depth, board, ScoreTy::MIN, ScoreTy::MAX)
        }
    }
}

fn main() {
    let mut fen = String::new();
    let stdin = io::stdin();
    stdin
        .read_line(&mut fen)
        .expect("Failed to read from stdin");
    let board = Board::from_str(&fen).expect("Invalid FEN position");
    let mut engine = Engine::new(2048);

    println!(
        "Score: {}",
        engine.negamax(3, board, ScoreTy::MIN, ScoreTy::MAX)
    );
}

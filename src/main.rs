mod eval;
mod helpers;
mod move_sort;
mod score;
mod search;
mod transposition;

use chess::Board;
use helpers::game_over;

use std::io::BufRead;
use std::{io, str::FromStr};

fn main() {
    let mut engine = search::Engine::new(32768);

    loop {
        let fen = io::stdin()
            .lock()
            .lines()
            .next()
            .unwrap()
            .expect("Failed to read from stdin");
        let board = Board::from_str(&fen).expect("Invalid FEN position");
        if game_over(board) {
            break;
        }
        println!("Best move: {}", engine.best_move(7, board).unwrap());
    }
}

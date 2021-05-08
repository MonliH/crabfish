mod eval;
mod flags;
mod helpers;
mod move_sort;
mod score;
mod search;
mod thread_launcher;
mod transposition;

use clap::Clap;

use chess::Board;
use helpers::game_over;
use thread_launcher::ThreadLauncher;

use std::io::BufRead;
use std::{io, str::FromStr};

fn eval_from_fen(engine: &mut ThreadLauncher, depth: u8, fen: &str) -> bool {
    let board = Board::from_str(&fen).expect("Invalid FEN position");
    if game_over(board) {
        return true;
    }
    let (best_move, eval) = engine.best_move(depth, board).unwrap();
    println!("Best move: {}; Analysis: {}", best_move, eval);

    false
}

fn main() {
    let opts = flags::App::parse();
    match opts.subcmd {
        flags::SubCommand::Uci => todo!(),
        flags::SubCommand::Move(conf) => {
            let mut thread_launcher = ThreadLauncher::new(conf.memo, conf.jobs);

            if conf.interactive {
                loop {
                    if let Some(line) = io::stdin().lock().lines().next() {
                        let fen = line.expect("Failed to read from stdin");
                        let game_over = eval_from_fen(&mut thread_launcher, conf.depth, &fen);
                        if game_over {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            } else {
                let fen = if let Some(fen) = conf.fen {
                    fen
                } else {
                    io::stdin()
                        .lock()
                        .lines()
                        .next()
                        .unwrap()
                        .expect("Failed to read from stdin")
                };

                eval_from_fen(&mut thread_launcher, conf.depth, &fen);
            }
            thread_launcher.memo.free();
        }
    }
}

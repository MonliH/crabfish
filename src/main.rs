mod eval;
mod flags;
mod helpers;
mod move_sort;
mod score;
mod search;
mod transposition;

use clap::Clap;

use chess::{Board, ChessMove};
use helpers::game_over;

use std::{
    io,
    io::BufRead,
    mem,
    process::exit,
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
};

pub static TIME_UP: AtomicBool = AtomicBool::new(false);

#[derive(Default, Debug)]
pub struct UciConfig {
    ponder: bool,
    wtime: i64,
    btime: i64,
    nodes: Option<usize>,
    depth: Option<u8>,
    movetime: Option<u64>,
    infinite: bool,
}

fn eval_from_fen(engine: &mut search::Engine, depth: u8, fen: &str) -> bool {
    let board = Board::from_str(&fen).expect("Invalid FEN position");
    if game_over(board) {
        return true;
    }
    let (best_move, eval) = engine.best_move(depth, board).unwrap();
    println!("Best move: {}; Analysis: {}", best_move, eval);

    false
}

fn main() {
    let conf = flags::App::parse();

    match conf.subcmd {
        flags::SubCommand::Uci => {
            let mut internal_board = Board::default();
            let engine = Arc::new(Mutex::new(search::Engine::new(33554432)));
            // SAFTEY: This is static because we never use the static reference in a way where the
            // value behind it would be dropped before other threads use it.
            let eng: &'static Arc<Mutex<search::Engine>> = unsafe { mem::transmute(&engine) };
            let mut joins = Vec::new();
            loop {
                if let Some(line) = io::stdin().lock().lines().next() {
                    let input = line.expect("Failed to read from stdin");
                    let mut items = input.split(" ");
                    let cmd = items.next().unwrap();
                    match cmd {
                        "uci" => {
                            println!("id name Crabfish {}", env!("CARGO_PKG_VERSION"));
                            println!("id author Jonathan Li");
                        }
                        "isready" => {
                            println!("readyok");
                        }
                        "position" => {
                            let mode = items.next().unwrap_or("");
                            let mut board = if mode == "fen" {
                                let mut fen = String::new();
                                let mut next = items.next();
                                while let Some(s) = next {
                                    if s == "moves" {
                                        break;
                                    }
                                    fen.push_str(s);
                                    next = items.next();
                                }
                                Board::from_str(&fen).expect("Invalid FEN")
                            } else if mode == "startpos" {
                                // eat moves
                                items.next().unwrap();
                                Board::default()
                            } else {
                                Board::default()
                            };

                            for cmove in items {
                                board = board.make_move_new(
                                    ChessMove::from_str(cmove).expect("invalid move"),
                                );
                            }

                            internal_board = board;
                        }
                        "go" => {
                            let mut config = UciConfig::default();
                            while let Some(token) = items.next() {
                                match token {
                                    "movestogo" | "winc" | "binc" | "mate" => {
                                        items.next().unwrap();
                                    }
                                    "ponder" => {
                                        config.ponder = true;
                                    }
                                    "wtime" => {
                                        config.wtime = items.next().unwrap().parse().unwrap();
                                    }
                                    "btime" => {
                                        config.btime = items.next().unwrap().parse().unwrap();
                                    }
                                    "depth" => {
                                        config.depth =
                                            Some(items.next().unwrap().parse().unwrap());
                                    }
                                    "nodes" => {
                                        config.nodes =
                                            Some(items.next().unwrap().parse().unwrap());
                                    }
                                    "movetime" => {
                                        config.movetime =
                                            Some(items.next().unwrap().parse().unwrap());
                                    }
                                    "infinite" => {
                                        config.infinite = true;
                                    }
                                    _ => {}
                                }
                            }
                            let depth = if config.infinite {
                                u8::MAX - 1
                            } else {
                                config.depth.unwrap_or(7)
                            };
                            dbg!(&config);
                            joins.push(thread::spawn(move || {
                                let (best_move, _) = Arc::clone(&eng)
                                    .lock()
                                    .unwrap()
                                    .best_move(depth, internal_board)
                                    .unwrap();
                                println!("bestmove {}", best_move);
                            }));
                        }
                        "stop" => {
                            TIME_UP.store(true, Ordering::SeqCst);
                            for join in mem::take(&mut joins) {
                                join.join().unwrap();
                            }
                        }
                        "quit" => {
                            TIME_UP.store(true, Ordering::SeqCst);
                            for join in joins {
                                join.join().unwrap();
                            }
                            exit(0);
                        }
                        _ => {}
                    }
                } else {
                    break;
                }
            }
        }
        flags::SubCommand::Move(conf) => {
            let mut engine = search::Engine::new(conf.memo);
            if conf.interactive {
                loop {
                    if let Some(line) = io::stdin().lock().lines().next() {
                        let fen = line.expect("Failed to read from stdin");
                        let game_over = eval_from_fen(&mut engine, conf.depth, &fen);
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

                eval_from_fen(&mut engine, conf.depth, &fen);
            }
        }
    }
}

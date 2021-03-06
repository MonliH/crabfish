use clap::Parser;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[clap(version = VERSION, about = "The crabfish chess engine.")]
pub struct App {
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Parser)]
pub enum SubCommand {
    #[clap(about = "Play the best move")]
    Move(Move),

    #[clap(about = "The UCI engine protocol")]
    Uci,
}

#[derive(Parser)]
pub struct Move {
    #[clap(
        short,
        long,
        about = "An FEN string. Will read from stdin if not provided",
        conflicts_with = "interactive"
    )]
    pub fen: Option<String>,

    #[clap(short, long, about = "Interactive mode", conflicts_with = "fen")]
    pub interactive: bool,

    #[clap(short, long, about = "Max depth of search", default_value = "9")]
    pub depth: u8,

    #[clap(
        short,
        long,
        about = "Size of the transposition table. Must be power of 2",
        default_value = "33554432"
    )]
    pub memo: usize,
}

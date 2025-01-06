use clap::Parser;

#[derive(Parser)]
pub struct Args {
    /// Optional path to list (default to current directory)
    #[arg(default_value = ".")]
    pub path: String,

    #[arg(short, long, help = "Show hidden files")]
    pub all: bool,

    #[arg(short, long, help = "Long listing format")]
    pub long: bool,

    #[arg(short = 'R', long, help = "Recursive listing")]
    pub recursive: bool,
    // #[arg(short = '', long, help = "Human-readable sizes")]
    // human_readable: bool,
}

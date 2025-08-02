use clap::Parser;

#[derive(Parser)]
pub struct Args {
    /// Optional path to list (default to current directory)
    #[arg(default_value = ".")]
    pub path: String,

    #[arg(short, long, help = "Show hidden files")]
    pub all: bool,

    #[arg(short = 'A', long, help = "Do not list implied . and ..")]
    pub almost_all: bool,

    #[arg(short, long, help = "Long listing format")]
    pub long: bool,

    #[arg(short = 'R', long, help = "Recursive listing")]
    pub recursive: bool,

    #[arg(long, help = "Human-readable sizes")]
    pub human_readable: bool,

    #[arg(short = 'F', long, help = "Append indicator (one of */=>@|) to entries")]
    pub classify: bool,

    #[arg(short = '1', help = "List one file per line")]
    pub one_per_line: bool,

    #[arg(short = 't', help = "Sort by modification time, newest first")]
    pub sort_time: bool,

    #[arg(short = 'S', help = "Sort by file size, largest first")]
    pub sort_size: bool,

    #[arg(short = 'r', long, help = "Reverse order while sorting")]
    pub reverse: bool,

    #[arg(short = 'U', help = "Do not sort; list entries in directory order")]
    pub unsorted: bool,
}

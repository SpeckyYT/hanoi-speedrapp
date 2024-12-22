use clap::Parser;

/// Hanoi-Speedrapp
#[derive(Parser)]
pub struct Cli {
    /// Enable performance profiler
    #[arg(long, short)]
    pub profile: bool,

    /// Backup savefile
    #[arg(long, short)]
    pub backup: bool,

    /// Enable VSync
    #[arg(long, short)]
    pub vsync: bool,
}

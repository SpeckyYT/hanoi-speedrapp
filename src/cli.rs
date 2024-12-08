use clap::Parser;

/// Hanoi-Speedrapp
#[derive(Parser)]
pub struct Cli {
    /// Enable performance profiler
    #[arg(long, short)]
    pub profile: bool,

    /// Enable VSync
    #[arg(long, short)]
    pub vsync: bool,
}

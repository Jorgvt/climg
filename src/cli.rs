use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ProtocolChoice {
    /// Auto-detect terminal graphics protocol (Kitty -> iTerm2)
    Auto,
    /// Kitty graphics protocol
    Kitty,
    /// iTerm2 inline image protocol
    Iterm2,
}

#[derive(Debug, Parser)]
#[command(
    name = "climg",
    author = "jorgvt",
    version,
    about = "Fast, high-resolution terminal image viewer for remote servers",
    long_about = "A standalone high-resolution terminal image viewer designed for remote server workflows.\n\
                  Supports Kitty graphics protocol and iTerm2 inline images protocol with tmux passthrough.\n\
                  Reads from files or standard input (pipe support)."
)]
pub struct Cli {
    /// Image file path(s) to view. If omitted or '-', reads from stdin.
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,

    /// Target display width in columns (cells), pixels ('800px'), or terminal percent ('50%')
    #[arg(short = 'w', long = "width", value_name = "WIDTH")]
    pub width: Option<String>,

    /// Target display height in rows (cells), pixels ('600px'), or terminal percent ('50%')
    #[arg(short = 'H', long = "height", value_name = "HEIGHT")]
    pub height: Option<String>,

    /// Maximum display width
    #[arg(long = "max-width", value_name = "WIDTH")]
    pub max_width: Option<String>,

    /// Maximum display height
    #[arg(long = "max-height", value_name = "HEIGHT")]
    pub max_height: Option<String>,

    /// Explicit graphics protocol override
    #[arg(short = 'p', long = "protocol", value_enum, default_value_t = ProtocolChoice::Auto)]
    pub protocol: ProtocolChoice,

    /// Suppress filename and image metadata header
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,

    /// Watch mode: monitor file and re-render in-place when modified
    #[arg(short = 'W', long = "watch")]
    pub watch: bool,
}

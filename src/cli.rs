use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
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

    /// Explicit graphics protocol override (can also be set via CLIMG_PROTOCOL env var)
    #[arg(
        short = 'p',
        long = "protocol",
        value_enum,
        env = "CLIMG_PROTOCOL"
    )]
    pub protocol: Option<ProtocolChoice>,

    /// Suppress filename and image metadata header
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,

    /// Watch mode: monitor file and re-render in-place when modified
    #[arg(short = 'W', long = "watch")]
    pub watch: bool,

    /// Only display the first frame of animated images (e.g. GIFs)
    #[arg(long = "static")]
    pub r#static: bool,

    /// Number of animation loops (0 for infinite loop, default: 0)
    #[arg(long = "loop", default_value_t = 0)]
    pub loop_count: u32,

    /// Generate a persistent configuration file in ~/.config/climg/config.toml
    #[arg(long = "init-config", value_name = "PROTOCOL", num_args = 0..=1, default_missing_value = "kitty")]
    pub init_config: Option<ProtocolChoice>,

    /// Update climg to the latest release from GitHub
    #[arg(short = 'u', long = "update")]
    pub update: bool,
}

use crate::cli::ProtocolChoice;
use anyhow::{Result, bail};
use std::env;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphicsProtocol {
    Kitty,
    Iterm2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Multiplexer {
    None,
    Tmux,
    Screen,
}

impl Multiplexer {
    pub fn detect() -> Self {
        if env::var("TMUX").is_ok() {
            return Multiplexer::Tmux;
        }
        if let Ok(term) = env::var("TERM") {
            if term.starts_with("tmux") {
                return Multiplexer::Tmux;
            }
            if term.starts_with("screen") {
                return Multiplexer::Screen;
            }
        }
        Multiplexer::None
    }

    /// Wrap an escape sequence if running under tmux or screen so the terminal receives it
    pub fn wrap(&self, escape_seq: &str) -> String {
        match self {
            Multiplexer::None => escape_seq.to_string(),
            Multiplexer::Tmux => {
                // In tmux passthrough, every ESC (\x1b) inside must be doubled
                let doubled = escape_seq.replace("\x1b", "\x1b\x1b");
                format!("\x1bPtmux;\x1b{}\x1b\\", doubled)
            }
            Multiplexer::Screen => {
                let doubled = escape_seq.replace("\x1b", "\x1b\x1b");
                format!("\x1bP{}\x1b\\", doubled)
            }
        }
    }
}

pub struct TerminalInfo {
    pub protocol: GraphicsProtocol,
    pub multiplexer: Multiplexer,
}

impl TerminalInfo {
    pub fn detect(choice: ProtocolChoice) -> Result<Self> {
        let multiplexer = Multiplexer::detect();

        let protocol = match choice {
            ProtocolChoice::Kitty => GraphicsProtocol::Kitty,
            ProtocolChoice::Iterm2 => GraphicsProtocol::Iterm2,
            ProtocolChoice::Auto => Self::detect_protocol(multiplexer)?,
        };

        Ok(Self {
            protocol,
            multiplexer,
        })
    }

    fn detect_protocol(multiplexer: Multiplexer) -> Result<GraphicsProtocol> {
        // 1. Fast path: check environment variables first (instant, zero TTY side-effects)
        if let Some(proto) = Self::detect_from_env() {
            return Ok(proto);
        }

        // 2. Query /dev/tty if environment didn't give an answer (e.g. SSH session without forwarded env)
        if let Some(proto) = Self::query_tty_for_protocol(multiplexer) {
            return Ok(proto);
        }

        bail!(
            "Could not detect a terminal supporting high-resolution graphics (Kitty or iTerm2 protocol).\n\n\
             Supported terminals include Kitty, Ghostty, WezTerm, iTerm2, and VS Code Terminal.\n\
             If you are connecting over SSH and environment variables were not forwarded, you can manually specify:\n  \
             climg --protocol kitty <FILE>\n  \
             climg --protocol iterm2 <FILE>"
        )
    }

    fn detect_from_env() -> Option<GraphicsProtocol> {
        Self::detect_from_env_lookup(|k| env::var(k).ok())
    }

    pub fn detect_from_env_lookup<F>(lookup: F) -> Option<GraphicsProtocol>
    where
        F: Fn(&str) -> Option<String>,
    {
        // Kitty checks
        if lookup("KITTY_WINDOW_ID").is_some() || lookup("KITTY_PID").is_some() {
            return Some(GraphicsProtocol::Kitty);
        }
        if lookup("GHOSTTY_RESOURCES_DIR").is_some() {
            return Some(GraphicsProtocol::Kitty);
        }
        if let Some(term) = lookup("TERM") {
            if term == "xterm-kitty" || term == "xterm-ghostty" {
                return Some(GraphicsProtocol::Kitty);
            }
        }
        if let Some(terminfo) = lookup("TERMINFO") {
            if terminfo.contains("kitty") {
                return Some(GraphicsProtocol::Kitty);
            }
        }

        // Wezterm checks (supports Kitty and iTerm2)
        if lookup("WEZTERM_PANE").is_some() || lookup("WEZTERM_UNIX_SOCKET").is_some() {
            return Some(GraphicsProtocol::Kitty);
        }

        // TERM_PROGRAM checks
        if let Some(prog) = lookup("TERM_PROGRAM") {
            let lower = prog.to_lowercase();
            if lower.contains("kitty") || lower.contains("ghostty") || lower.contains("wezterm") {
                return Some(GraphicsProtocol::Kitty);
            }
            if lower.contains("iterm") || lower.contains("vscode") {
                return Some(GraphicsProtocol::Iterm2);
            }
        }

        // LC_TERMINAL checks
        if let Some(lc_term) = lookup("LC_TERMINAL") {
            let lower = lc_term.to_lowercase();
            if lower.contains("kitty") || lower.contains("ghostty") {
                return Some(GraphicsProtocol::Kitty);
            }
            if lower.contains("iterm") {
                return Some(GraphicsProtocol::Iterm2);
            }
        }

        None
    }

    fn query_tty_for_protocol(multiplexer: Multiplexer) -> Option<GraphicsProtocol> {
        use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

        // Attempt to open /dev/tty for querying
        let mut tty = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/tty")
            .ok()?;

        // Enable raw mode so terminal responses are never echoed to screen
        if enable_raw_mode().is_err() {
            return None;
        }

        // Send a well-formed 1x1 24-bit test pixel query to Kitty graphics protocol
        let kitty_query = multiplexer.wrap("\x1b_Gi=1,s=1,v=1,a=q,t=d,f=24;AAAA\x1b\\");
        let write_res = tty.write_all(kitty_query.as_bytes()).and_then(|_| tty.flush());

        let mut detected = None;

        if write_res.is_ok() {
            // Read with short timeout using crossterm event poll
            if crossterm::event::poll(Duration::from_millis(50)).unwrap_or(false) {
                let mut buf = [0u8; 512];
                if let Ok(n) = tty.read(&mut buf) {
                    let resp = String::from_utf8_lossy(&buf[..n]);
                    if resp.contains("_Gi=1;OK") || resp.contains("_Gi=1") {
                        detected = Some(GraphicsProtocol::Kitty);
                    }
                }
            }
        }

        // Always restore terminal mode
        let _ = disable_raw_mode();

        detected
    }

    /// Helper to wrap an escape sequence with the active multiplexer wrapper
    pub fn wrap_escape(&self, seq: &str) -> String {
        self.multiplexer.wrap(seq)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiplexer_wrap_none() {
        let mux = Multiplexer::None;
        let seq = "\x1b]1337;File=inline=1:...\x07";
        assert_eq!(mux.wrap(seq), seq);
    }

    #[test]
    fn test_multiplexer_wrap_tmux() {
        let mux = Multiplexer::Tmux;
        let seq = "\x1b_Ga=d\x1b\\";
        let wrapped = mux.wrap(seq);
        assert_eq!(wrapped, "\x1bPtmux;\x1b\x1b\x1b_Ga=d\x1b\x1b\\\x1b\\");
    }

    #[test]
    fn test_env_detection_kitty() {
        let proto = TerminalInfo::detect_from_env_lookup(|k| {
            if k == "KITTY_WINDOW_ID" {
                Some("123".to_string())
            } else {
                None
            }
        });
        assert_eq!(proto, Some(GraphicsProtocol::Kitty));
    }

    #[test]
    fn test_env_detection_iterm2() {
        let proto = TerminalInfo::detect_from_env_lookup(|k| {
            if k == "TERM_PROGRAM" {
                Some("iTerm.app".to_string())
            } else {
                None
            }
        });
        assert_eq!(proto, Some(GraphicsProtocol::Iterm2));
    }
}

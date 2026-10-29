use crate::image_loader::LoadedImage;
use crate::renderer::{RenderOptions, Renderer};
use crate::terminal::{GraphicsProtocol, TerminalInfo};
use crate::ui::Ui;
use anyhow::{Context, Result, anyhow};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

pub struct LiveWatcher;

impl LiveWatcher {
    pub fn watch(
        files: &[PathBuf],
        render_opts: &RenderOptions,
        term_info: &TerminalInfo,
        quiet: bool,
    ) -> Result<()> {
        if files.is_empty() {
            return Err(anyhow!("Cannot use --watch without file paths (stdin cannot be watched)."));
        }

        let (tx, rx) = channel();
        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        })
        .context("Failed to initialize file watcher")?;

        for file in files {
            if let Some(parent) = file.parent().filter(|p| !p.as_os_str().is_empty()) {
                watcher
                    .watch(parent, RecursiveMode::NonRecursive)
                    .with_context(|| format!("Failed to watch directory: {}", parent.display()))?;
            } else {
                watcher
                    .watch(file, RecursiveMode::NonRecursive)
                    .with_context(|| format!("Failed to watch file: {}", file.display()))?;
            }
        }

        let mut stdout = io::stdout().lock();

        // Initial render
        Self::render_frame(&mut stdout, files, render_opts, term_info, quiet)?;

        let debounce_duration = Duration::from_millis(200);
        let mut last_render = Instant::now();

        loop {
            // Block until an event arrives
            match rx.recv_timeout(Duration::from_millis(500)) {
                Ok(event) => {
                    // Check if event is a modification or create event
                    let is_relevant = match event.kind {
                        EventKind::Modify(_) | EventKind::Create(_) => true,
                        _ => false,
                    };

                    if is_relevant && last_render.elapsed() >= debounce_duration {
                        // Sleep a tiny amount for filesystem write flush
                        std::thread::sleep(Duration::from_millis(50));
                        Self::render_frame(&mut stdout, files, render_opts, term_info, quiet)?;
                        last_render = Instant::now();
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    // Periodic poll continue
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    break;
                }
            }
        }

        Ok(())
    }

    fn render_frame<W: Write>(
        writer: &mut W,
        files: &[PathBuf],
        render_opts: &RenderOptions,
        term_info: &TerminalInfo,
        quiet: bool,
    ) -> Result<()> {
        // Clear terminal screen and reset cursor to home (1, 1)
        write!(writer, "\x1b[2J\x1b[H")?;

        // If Kitty protocol, clear previous graphics images to avoid stacking
        if term_info.protocol == GraphicsProtocol::Kitty {
            let clear_kitty = term_info.wrap_escape("\x1b_Ga=d,d=A\x1b\\");
            write!(writer, "{}", clear_kitty)?;
        }

        for (idx, path) in files.iter().enumerate() {
            if idx > 0 {
                writeln!(writer)?;
            }

            // If the file is temporarily being written, give it a retry
            let img = match LoadedImage::from_file(path) {
                Ok(img) => img,
                Err(_) => {
                    std::thread::sleep(Duration::from_millis(50));
                    LoadedImage::from_file(path)?
                }
            };

            if !quiet {
                Ui::print_banner(writer, &img)?;
            }

            Renderer::render(writer, &img, render_opts, term_info)?;
        }

        writer.flush()?;
        Ok(())
    }
}

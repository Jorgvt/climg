mod cli;
mod image_loader;
mod protocols;
mod renderer;
mod terminal;
mod ui;
mod watcher;

use clap::Parser;
use cli::Cli;
use image_loader::LoadedImage;
use renderer::{RenderOptions, Renderer};
use terminal::TerminalInfo;
use ui::Ui;
use watcher::LiveWatcher;

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    let term_info = TerminalInfo::detect(args.protocol)?;
    let render_opts = RenderOptions::from_cli(&args)?;

    let is_stdin = args.files.is_empty()
        || (args.files.len() == 1 && args.files[0].to_string_lossy() == "-");

    if args.watch {
        if is_stdin {
            anyhow::bail!("Cannot use --watch with standard input.");
        }
        return LiveWatcher::watch(&args.files, &render_opts, &term_info, args.quiet);
    }

    let mut stdout = std::io::stdout().lock();

    if is_stdin {
        let img = LoadedImage::from_stdin()?;
        if !args.quiet {
            Ui::print_banner(&mut stdout, &img)?;
        }
        Renderer::render(&mut stdout, &img, &render_opts, &term_info)?;
        return Ok(());
    }

    for (idx, path) in args.files.iter().enumerate() {
        let img = LoadedImage::from_file(path)?;

        if idx > 0 {
            // Visual separation between consecutive images
            println!();
        }

        if !args.quiet {
            Ui::print_banner(&mut stdout, &img)?;
        }

        Renderer::render(&mut stdout, &img, &render_opts, &term_info)?;
    }

    Ok(())
}




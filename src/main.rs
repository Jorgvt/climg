mod cli;
mod config;
mod image_loader;
mod protocols;
mod renderer;
mod terminal;
mod ui;
mod watcher;

use clap::Parser;
use cli::{Cli, ProtocolChoice};
use config::AppConfig;
use image_loader::LoadedImage;
use renderer::{RenderOptions, Renderer};
use terminal::TerminalInfo;
use ui::Ui;
use watcher::LiveWatcher;

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    // Handle --init-config
    if let Some(proto) = args.init_config {
        let path = AppConfig::init_config(Some(proto))?;
        println!("Configuration file created successfully at: {}", path.display());
        println!("Default protocol set to: {:?}", proto);
        return Ok(());
    }

    let config = AppConfig::load();

    // Resolve protocol precedence: CLI flag / env var > config file > Auto
    let protocol = args
        .protocol
        .or(config.protocol)
        .unwrap_or(ProtocolChoice::Auto);

    let quiet = args.quiet || config.quiet.unwrap_or(false);

    let term_info = TerminalInfo::detect(protocol)?;
    let render_opts = RenderOptions::from_cli(&args, &config)?;

    let is_stdin = args.files.is_empty()
        || (args.files.len() == 1 && args.files[0].to_string_lossy() == "-");

    if args.watch {
        if is_stdin {
            anyhow::bail!("Cannot use --watch with standard input.");
        }
        return LiveWatcher::watch(&args.files, &render_opts, &term_info, quiet);
    }

    let mut stdout = std::io::stdout().lock();

    if is_stdin {
        let img = LoadedImage::from_stdin()?;
        if !quiet {
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

        if !quiet {
            Ui::print_banner(&mut stdout, &img)?;
        }

        Renderer::render(&mut stdout, &img, &render_opts, &term_info)?;
    }

    Ok(())
}





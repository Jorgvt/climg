# climg

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

A lightweight, blazing-fast standalone CLI utility for visualizing high-resolution images and plots directly in your terminal — built specifically for machine learning and remote server workflows.

When training models on remote machines over SSH, copying plots to your local laptop or setting up web view servers is tedious. `climg` lets you inspect training curves, confusion matrices, and feature maps instantly from your terminal with native pixel fidelity.

---

## Features

- **True High-Resolution Graphics**: Uses modern terminal protocols (Kitty graphics protocol and iTerm2 inline protocol) to render full-resolution raster images with crystal clarity.
- **Remote Server Friendly**:
  - Standalone single binary (~2.7 MB) with zero external runtime dependencies.
  - Automatically queries the terminal over `/dev/tty` and environment variables so it works over SSH sessions without requiring forwarded variables.
  - Automatic `tmux` and `screen` passthrough wrapping (`\ePtmux;...\e\\`).
- **Live Watch Mode (`-W` / `--watch`)**: Watches plot files on disk and re-renders them in-place as training scripts update them.
- **Flexible Input**: Supports local files, multiple files sequentially, and UNIX pipes (`cat plot.png | climg` or `curl ... | climg`).
- **Aspect-Ratio Preserving Auto-Fit**: Scales images to comfortably fit your terminal window, with flags to customize width, height, or maximum bounds.
- **Wide Format Support**: PNG, JPEG, WebP, and GIF.

---

## Terminal Compatibility

| Terminal Emulator | Protocol | Notes |
|-------------------|----------|-------|
| **Kitty** | Kitty graphics protocol | Native chunked graphics, fully supported |
| **Ghostty** | Kitty graphics protocol | Native support |
| **WezTerm** | Kitty & iTerm2 protocols | Native support |
| **iTerm2** | iTerm2 inline images | Native OSC 1337 support |
| **VS Code Terminal** | iTerm2 inline images | Supported in integrated terminal |

> **Note for `tmux` users:**
> In tmux 3.3+, enable escape sequence passthrough in your `~/.tmux.conf`:
> ```tmux
> set -g allow-passthrough on
> ```

---

## Installation

### 1. One-Line Install (Recommended for Remote Servers)

No Rust, Cargo, or root/sudo access required. On any remote Linux server or local Mac, run:

```bash
curl -fsSL https://raw.githubusercontent.com/Jorgvt/climg/master/install.sh | bash
```

This automatically detects your OS and architecture, downloads the latest pre-compiled static binary from GitHub Releases, and installs it into `~/.local/bin/climg`.

*(Make sure `~/.local/bin` is in your `$PATH`, e.g. `export PATH="$HOME/.local/bin:$PATH"`)*.

> **💡 Remote Server Tip (Setting your protocol once):**
> Because SSH connections often strip terminal environment variables like `KITTY_WINDOW_ID`, initialize your preferred protocol once right after installing:
> ```bash
> climg --init-config kitty
> # or for iTerm2 / VS Code:
> climg --init-config iterm2
> ```
> This creates `~/.config/climg/config.toml` so you can simply run `climg plot.png` without extra flags.

---

### 2. Direct Release Download

You can also download pre-built binaries directly from the [GitHub Releases page](https://github.com/Jorgvt/climg/releases):

| Platform | Target | Archive |
|---|---|---|
| **Linux (x86_64)** | `x86_64-unknown-linux-musl` | `climg-x86_64-unknown-linux-musl.tar.gz` |
| **Linux (ARM64)** | `aarch64-unknown-linux-musl` | `climg-aarch64-unknown-linux-musl.tar.gz` |
| **macOS (Apple Silicon)** | `aarch64-apple-darwin` | `climg-aarch64-apple-darwin.tar.gz` |
| **macOS (Intel)** | `x86_64-apple-darwin` | `climg-x86_64-apple-darwin.tar.gz` |

To unpack and place into `~/.local/bin`:
```bash
curl -fsSL https://github.com/Jorgvt/climg/releases/latest/download/climg-x86_64-unknown-linux-musl.tar.gz | tar -xz -C ~/.local/bin/
```

---

### 3. Build from Source (with Cargo)

If you have Rust installed locally or on the remote machine:

```bash
cargo install --path .
```
or directly from GitHub:
```bash
cargo install --git https://github.com/Jorgvt/climg.git
```

---

### 4. Updating `climg`

Once `climg` is installed, you can update it to the latest version at any time directly from the CLI:

```bash
climg --update
# or short flag:
climg -u
```

This checks GitHub Releases for new releases, downloads the appropriate static binary for your OS/architecture, and replaces the binary in-place without touching your configuration.

---

## Usage

### 1. View a Single Plot
```bash
climg loss.png
```

Displays the image scaled to fit your terminal window, prefixed with a styled metadata header showing filename, dimensions, format, and file size.

### 2. Live Watch Mode (During Model Training)
```bash
climg --watch checkpoints/loss_curve.png
```

Watches the image file on disk. Whenever your PyTorch/TensorFlow training loop updates `loss_curve.png`, `climg` clears the screen and re-renders the latest plot in-place.

### 3. Multiple Images
```bash
climg loss.png learning_rate.png roc_curve.png
```

Renders images sequentially with clear visual separation and individual metadata banners.

### 4. Reading from STDIN / Pipes
```bash
# Pipe from a file or download
cat evaluation.png | climg

# Pipe directly from a remote URL or API
curl -s https://example.com/plot.png | climg
```

### 5. Custom Sizing
You can specify width or height in terminal columns/rows, percentages of your terminal window, or approximate pixels:

```bash
# 50% of terminal width
climg --width 50% plot.png

# Exactly 80 terminal columns wide
climg -w 80 plot.png

# Constrain max height to 20 rows
climg --max-height 20 plot.png
```

### 6. Animated GIFs
`climg` natively supports animated GIFs. For Kitty terminals, it transmits all animation frames with their exact millisecond delays and initiates hardware-accelerated looping handled entirely by the terminal.

```bash
# Play animated GIF (infinite loop by default)
climg training_progress.gif

# Limit playback to 3 loops
climg --loop 3 training_progress.gif

# Only view the first frame as a static image
climg --static training_progress.gif
```

### 7. Quiet Mode (Scripting)
```bash
climg --quiet plot.png
```
Suppresses metadata headers and prints only the raw inline graphic sequence.

### 8. Configuration & Remote SSH Defaults

When connecting over SSH, remote servers frequently strip terminal environment variables like `KITTY_WINDOW_ID` or `TERM_PROGRAM`. To avoid having to type `--protocol kitty` every time, you can configure your default protocol once using either method:

#### Option A: Generate a Persistent Config File (Recommended)
Run this command once on your remote server:
```bash
climg --init-config kitty
```
This creates `~/.config/climg/config.toml`:
```toml
# ~/.config/climg/config.toml
protocol = "kitty"
```
From then on, simply running `climg plot.png` will always use Kitty graphics automatically.

#### Option B: Environment Variable
Add this to your remote server's `~/.bashrc` or `~/.zshrc`:
```bash
export CLIMG_PROTOCOL=kitty
```

---

## Configuration File Reference

`climg` looks for configuration in `$XDG_CONFIG_HOME/climg/config.toml` or `~/.config/climg/config.toml`:

```toml
# Default graphics protocol: "kitty", "iterm2", or "auto"
protocol = "kitty"

# Always suppress filename/metadata banner (default: false)
quiet = false

# Default maximum width / height constraints
max_width = "80"
max_height = "35"
```

---

## CLI Options

```text
Usage: climg [OPTIONS] [FILE]...

Arguments:
  [FILE]...  Image file path(s) to view. If omitted or '-', reads from stdin

Options:
  -w, --width <WIDTH>              Target display width (cells, '50%', or '800px')
  -H, --height <HEIGHT>            Target display height (cells, '50%', or '600px')
      --max-width <WIDTH>          Maximum display width
      --max-height <HEIGHT>        Maximum display height
  -p, --protocol <PROTOCOL>        Explicit graphics protocol: [auto, kitty, iterm2] [env: CLIMG_PROTOCOL=]
      --init-config [<PROTOCOL>]   Generate persistent config file in ~/.config/climg/config.toml
  -q, --quiet                      Suppress metadata headers
  -W, --watch                      Watch mode: monitor file and re-render in-place
      --static                     Only display first frame of animated GIF
      --loop <COUNT>               Number of animation loops (0 = infinite)
  -h, --help                       Print help
  -V, --version                    Print version
```
  -q, --quiet                Suppress metadata headers
  -W, --watch                Watch mode: monitor file and re-render in-place
  -h, --help                 Print help
  -V, --version              Print version
```

---

## License

Licensed under either of [Apache License, Version 2.0](LICENSE) or [MIT License](LICENSE) at your option.

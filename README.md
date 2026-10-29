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

### 1. Install Locally (into `~/.cargo/bin`)

If you want `climg` available globally on your current machine:

```bash
cargo install --path .
```

Ensure `~/.cargo/bin` is in your `PATH` (typically added by Rust installation).

---

### 2. Install on a Remote Server

#### Option A: Direct Build on Remote Server (Recommended)
If Rust / Cargo is installed on the remote machine:

```bash
# Clone and install directly
git clone https://github.com/your-username/climg.git
cd climg
cargo install --path .
```

#### Option B: Standalone Binary Transfer (Zero Remote Rust Dependency)
Since `climg` is a standalone static binary without dynamic C dependencies, you can compile and transfer the binary directly.

**If your remote server matches your build OS:**
```bash
cargo build --release

# If you have sudo / root access:
scp target/release/climg user@remote-server:/usr/local/bin/

# Or without sudo (into user binary directory):
ssh user@remote-server "mkdir -p ~/.local/bin"
scp target/release/climg user@remote-server:~/.local/bin/
```

**If developing on macOS and targeting a remote Linux server (Cross-Compilation):**
You can use [`cross`](https://github.com/cross-rs/cross) or standard target compilation:
```bash
# Build a static Linux binary from macOS
cross build --target x86_64-unknown-linux-musl --release
scp target/x86_64-unknown-linux-musl/release/climg user@remote-server:~/.local/bin/
```
Make sure `~/.local/bin` is in your remote `$PATH` (e.g., in `~/.bashrc` or `~/.zshrc`):
```bash
export PATH="$HOME/.local/bin:$PATH"
```

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

### 6. Quiet Mode (Scripting)
```bash
climg --quiet plot.png
```
Suppresses metadata headers and prints only the raw inline graphic sequence.

### 7. Explicit Protocol Override
If terminal auto-detection fails in a non-standard SSH environment:
```bash
climg --protocol kitty plot.png
# or
climg --protocol iterm2 plot.png
```

---

## CLI Options

```text
Usage: climg [OPTIONS] [FILE]...

Arguments:
  [FILE]...  Image file path(s) to view. If omitted or '-', reads from stdin

Options:
  -w, --width <WIDTH>        Target display width (cells, '50%', or '800px')
  -H, --height <HEIGHT>      Target display height (cells, '50%', or '600px')
      --max-width <WIDTH>    Maximum display width
      --max-height <HEIGHT>  Maximum display height
  -p, --protocol <PROTOCOL>  Explicit graphics protocol: [auto, kitty, iterm2] (default: auto)
  -q, --quiet                Suppress metadata headers
  -W, --watch                Watch mode: monitor file and re-render in-place
  -h, --help                 Print help
  -V, --version              Print version
```

---

## License

Licensed under either of [Apache License, Version 2.0](LICENSE) or [MIT License](LICENSE) at your option.

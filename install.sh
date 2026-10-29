#!/usr/bin/env bash
set -e

# climg installer script
# Installs pre-compiled static binary from GitHub Releases into ~/.local/bin

REPO="Jorgvt/climg"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# 1. Detect OS
OS="$(uname -s)"
case "$OS" in
  Linux*)  PLATFORM="unknown-linux-musl" ;;
  Darwin*) PLATFORM="apple-darwin" ;;
  *)
    echo "Error: Unsupported operating system '$OS'."
    exit 1
    ;;
esac

# 2. Detect Architecture
ARCH="$(uname -m)"
case "$ARCH" in
  x86_64|amd64) ARCH_TARGET="x86_64" ;;
  arm64|aarch64) ARCH_TARGET="aarch64" ;;
  *)
    echo "Error: Unsupported architecture '$ARCH'."
    exit 1
    ;;
esac

TARGET="${ARCH_TARGET}-${PLATFORM}"
DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/climg-${TARGET}.tar.gz"

echo "Detected platform: ${TARGET}"
echo "Downloading climg from ${DOWNLOAD_URL}..."

mkdir -p "$INSTALL_DIR"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if command -v curl >/dev/null 2>&1; then
  curl -fsSL "$DOWNLOAD_URL" -o "$TMP_DIR/climg.tar.gz"
elif command -v wget >/dev/null 2>&1; then
  wget -qO "$TMP_DIR/climg.tar.gz" "$DOWNLOAD_URL"
else
  echo "Error: Neither curl nor wget was found. Please install one of them."
  exit 1
fi

tar -xzf "$TMP_DIR/climg.tar.gz" -C "$TMP_DIR"
mv "$TMP_DIR/climg" "$INSTALL_DIR/climg"
chmod +x "$INSTALL_DIR/climg"

echo " Successfully installed climg to ${INSTALL_DIR}/climg"

# Check if INSTALL_DIR is in PATH
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    echo ""
    echo "Note: '${INSTALL_DIR}' is not currently in your PATH."
    echo "Add it by appending this line to your ~/.bashrc or ~/.zshrc:"
    echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
    ;;
esac

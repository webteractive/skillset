#!/usr/bin/env sh
# Install the skillset binary to a directory on your PATH.
# Usage:
#   ./install.sh              # use ./target/release/skillset (run from repo after cargo build --release)
#   ./install.sh [path]       # copy binary from path (e.g. downloaded release binary)
#   ./install.sh --download   # download latest release from GitHub and install (no cargo needed)
# One-liner (download and install): curl -sSL https://raw.githubusercontent.com/webteractive/skillset/main/install.sh | sh -s -- --download

set -e

REPO="webteractive/skillset"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DEFAULT_BINARY="${SCRIPT_DIR}/target/release/skillset"

# --download: fetch latest release from GitHub
if [ "${1:-}" = "--download" ] || [ "${1:-}" = "-d" ]; then
  OS="$(uname -s | tr 'A-Z' 'a-z')"
  ARCH="$(uname -m)"
  case "$ARCH" in
    x86_64|amd64) ARCH="x86_64" ;;
    arm64|aarch64) ARCH="aarch64" ;;
  esac
  ASSET="skillset-${OS}-${ARCH}"
  LATEST="$(curl -sSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')"
  if [ -z "$LATEST" ]; then
    echo "Could not find latest release. Check https://github.com/${REPO}/releases"
    exit 1
  fi
  URL="https://github.com/${REPO}/releases/download/${LATEST}/${ASSET}"
  echo "Downloading ${ASSET} from ${LATEST}..."
  BINARY="${TMPDIR:-/tmp}/skillset-$$"
  curl -sSLf "$URL" -o "$BINARY"
  chmod +x "$BINARY"
  CLEANUP=1
else
  BINARY="${1:-$DEFAULT_BINARY}"
  CLEANUP=0
fi

if [ ! -f "$BINARY" ]; then
  echo "Binary not found: $BINARY"
  echo ""
  echo "Either:"
  echo "  1. Build in this repo: cargo build --release   then run: ./install.sh"
  echo "  2. Pass a path to a binary: ./install.sh /path/to/skillset"
  echo "  3. Download latest release: ./install.sh --download"
  exit 1
fi

# Prefer ~/.local/bin, then ~/bin, or PREFIX/bin if set
if [ -n "$PREFIX" ]; then
  INSTALL_DIR="${PREFIX}/bin"
else
  if [ -d "$HOME/.local/bin" ]; then
    INSTALL_DIR="$HOME/.local/bin"
  else
    INSTALL_DIR="$HOME/bin"
  fi
fi

mkdir -p "$INSTALL_DIR"
cp "$BINARY" "$INSTALL_DIR/skillset"
chmod +x "$INSTALL_DIR/skillset"

echo "Installed skillset to $INSTALL_DIR/skillset"
echo ""
if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
  echo "Ensure $INSTALL_DIR is on your PATH, e.g. add to ~/.bashrc or ~/.zshrc:"
  echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
fi

[ "$CLEANUP" = 1 ] && rm -f "$BINARY"

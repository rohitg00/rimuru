#!/usr/bin/env bash
set -euo pipefail

REPO="rohitg00/rimuru"
III_REPO="iii-hq/iii"
INSTALL_DIR="${RIMURU_INSTALL_DIR:-$HOME/.local/bin}"

get_latest_version() {
  curl -fsSL "https://api.github.com/repos/$1/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/'
}

detect_target() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Linux)  os="unknown-linux-gnu" ;;
    Darwin) os="apple-darwin" ;;
    *)      echo "Unsupported OS: $os" >&2; exit 1 ;;
  esac

  case "$arch" in
    x86_64|amd64)  arch="x86_64" ;;
    aarch64|arm64) arch="aarch64" ;;
    *)             echo "Unsupported architecture: $arch" >&2; exit 1 ;;
  esac

  echo "${arch}-${os}"
}

detect_rimuru_platform() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Linux)  os="linux" ;;
    Darwin) os="macos" ;;
    *)      echo "Unsupported OS: $os" >&2; exit 1 ;;
  esac

  case "$arch" in
    x86_64|amd64)  arch="x64" ;;
    aarch64|arm64) arch="x64" ;;
    *)             echo "Unsupported architecture: $arch" >&2; exit 1 ;;
  esac

  echo "${os}-${arch}"
}

install_iii() {
  if command -v iii &>/dev/null; then
    local current_version
    current_version="$(iii --version 2>/dev/null || echo "unknown")"
    echo "iii engine already installed (v$current_version)"
    return
  fi

  echo ""
  echo "Installing iii engine (required dependency)..."
  local iii_version target filename url
  iii_version="$(get_latest_version "$III_REPO")"
  target="$(detect_target)"
  filename="iii-${target}.tar.gz"
  url="https://github.com/$III_REPO/releases/download/${iii_version}/${filename}"

  local tmpdir
  tmpdir="$(mktemp -d)"

  echo "Downloading iii engine $iii_version for $target..."
  curl -fsSL "$url" -o "$tmpdir/$filename"

  mkdir -p "$INSTALL_DIR"
  tar -xzf "$tmpdir/$filename" -C "$INSTALL_DIR" 2>/dev/null || \
    tar -xzf "$tmpdir/$filename" --strip-components=1 -C "$INSTALL_DIR" 2>/dev/null || true
  chmod +x "$INSTALL_DIR/iii" 2>/dev/null || true
  rm -rf "$tmpdir"

  if [ -f "$INSTALL_DIR/iii" ]; then
    echo "iii engine $iii_version installed to $INSTALL_DIR/iii"
  else
    echo "Warning: iii binary not found after extraction. You may need to install it manually." >&2
    echo "  See: https://github.com/$III_REPO/releases" >&2
  fi
}

install_rimuru() {
  local version="${1:-}"
  if [ -z "$version" ]; then
    echo "Fetching latest rimuru release..."
    version="$(get_latest_version "$REPO")"
  fi

  local platform filename url
  platform="$(detect_rimuru_platform)"
  filename="rimuru-${version}-${platform}.tar.gz"
  url="https://github.com/$REPO/releases/download/${version}/${filename}"

  echo "Downloading rimuru $version for $platform..."
  local tmpdir
  tmpdir="$(mktemp -d)"
  trap 'rm -rf "$tmpdir"' EXIT

  curl -fsSL "$url" -o "$tmpdir/$filename"

  echo "Extracting to $INSTALL_DIR..."
  mkdir -p "$INSTALL_DIR"
  tar -xzf "$tmpdir/$filename" -C "$INSTALL_DIR" --exclude='README.md' --exclude='LICENSE'

  chmod +x "$INSTALL_DIR/rimuru-worker" "$INSTALL_DIR/rimuru" "$INSTALL_DIR/rimuru-tui" 2>/dev/null || true
}

main() {
  echo "Rimuru installer"
  echo "================"
  echo ""

  install_iii
  install_rimuru "$@"

  echo ""
  echo "Installed to $INSTALL_DIR:"
  for bin in iii rimuru-worker rimuru rimuru-tui; do
    if [ -f "$INSTALL_DIR/$bin" ]; then
      echo "  $bin"
    fi
  done

  if ! echo "$PATH" | tr ':' '\n' | grep -qx "$INSTALL_DIR"; then
    echo ""
    echo "Add to your PATH:"
    echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
  fi

  echo ""
  echo "Get started:"
  echo "  iii                        # start the iii engine"
  echo "  rimuru-worker              # start the worker (serves UI on :3100)"
  echo "  rimuru agents detect       # auto-detect installed agents"
  echo "  rimuru-tui                 # launch terminal UI"
}

main "$@"

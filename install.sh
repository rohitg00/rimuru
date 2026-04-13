#!/usr/bin/env bash
set -euo pipefail

REPO="rohitg00/rimuru"
III_REPO="iii-hq/iii"
INSTALL_DIR="${RIMURU_INSTALL_DIR:-$HOME/.local/bin}"
CONFIG_DIR="${RIMURU_CONFIG_DIR:-$HOME/.config/rimuru}"
DATA_DIR="${RIMURU_DATA_DIR:-$HOME/.local/share/rimuru}"
# Default path baked into the shipped config.yaml. install_config() does
# an in-place substitution after downloading so RIMURU_DATA_DIR actually
# takes effect instead of being silently shadowed.
DEFAULT_DATA_DIR_IN_CONFIG='${HOME}/.local/share/rimuru'
# Set at runtime by resolve_rimuru_version so install_config and
# install_rimuru fetch artifacts from the same release.
RIMURU_VERSION=""

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

resolve_rimuru_version() {
  # Resolve the rimuru release tag once so install_rimuru and
  # install_config pull matching artifacts. Callers can override by
  # passing a tag as the first positional argument to main.
  if [ -n "${1:-}" ]; then
    RIMURU_VERSION="$1"
    return
  fi
  echo "Fetching latest rimuru release..."
  RIMURU_VERSION="$(get_latest_version "$REPO")"
}

install_rimuru() {
  if [ -z "$RIMURU_VERSION" ]; then
    echo "install_rimuru called before resolve_rimuru_version" >&2
    exit 1
  fi

  local platform filename url
  platform="$(detect_rimuru_platform)"
  filename="rimuru-${RIMURU_VERSION}-${platform}.tar.gz"
  url="https://github.com/$REPO/releases/download/${RIMURU_VERSION}/${filename}"

  echo "Downloading rimuru $RIMURU_VERSION for $platform..."
  local tmpdir
  tmpdir="$(mktemp -d)"
  trap 'rm -rf "$tmpdir"' EXIT

  curl -fsSL "$url" -o "$tmpdir/$filename"

  echo "Extracting to $INSTALL_DIR..."
  mkdir -p "$INSTALL_DIR"
  tar -xzf "$tmpdir/$filename" -C "$INSTALL_DIR" --exclude='README.md' --exclude='LICENSE'

  chmod +x "$INSTALL_DIR/rimuru-worker" "$INSTALL_DIR/rimuru" "$INSTALL_DIR/rimuru-tui" 2>/dev/null || true
}

install_config() {
  # Rimuru's iii config pins every KV worker to file_based storage under
  # $DATA_DIR. Without this, iii falls back to the in-memory default and
  # every cost record / budget counter / guard entry vanishes on restart.
  mkdir -p "$CONFIG_DIR" "$DATA_DIR"

  if [ -f "$CONFIG_DIR/config.yaml" ] && [ -z "${RIMURU_FORCE_CONFIG:-}" ]; then
    echo "Config already present at $CONFIG_DIR/config.yaml (set RIMURU_FORCE_CONFIG=1 to overwrite)"
    return
  fi

  # Pull the config.yaml that matches the installed release, not main.
  local config_url="https://raw.githubusercontent.com/$REPO/${RIMURU_VERSION}/config.yaml"

  echo "Installing iii config to $CONFIG_DIR/config.yaml..."
  if ! curl -fsSL "$config_url" -o "$CONFIG_DIR/config.yaml"; then
    echo "Warning: failed to download config.yaml from $config_url" >&2
    echo "Rimuru will start with in-memory state until you supply a config manually." >&2
    return
  fi

  # Honor RIMURU_DATA_DIR. The shipped config hardcodes
  # ${HOME}/.local/share/rimuru because iii's YAML env-var regex doesn't
  # support nested ${A:${B}/...} defaults, so we substitute after download.
  if [ "$DATA_DIR" != "$HOME/.local/share/rimuru" ]; then
    local tmpfile
    tmpfile="$(mktemp)"
    sed "s|${DEFAULT_DATA_DIR_IN_CONFIG}|${DATA_DIR}|g" "$CONFIG_DIR/config.yaml" > "$tmpfile"
    mv "$tmpfile" "$CONFIG_DIR/config.yaml"
  fi

  echo "Durable state will be written under $DATA_DIR"
}

main() {
  echo "Rimuru installer"
  echo "================"
  echo ""

  install_iii
  resolve_rimuru_version "${1:-}"
  install_rimuru
  install_config

  echo ""
  echo "Installed to $INSTALL_DIR:"
  for bin in iii rimuru-worker rimuru rimuru-tui; do
    if [ -f "$INSTALL_DIR/$bin" ]; then
      echo "  $bin"
    fi
  done
  echo "Config:    $CONFIG_DIR/config.yaml"
  echo "Data dir:  $DATA_DIR"

  if ! echo "$PATH" | tr ':' '\n' | grep -qx "$INSTALL_DIR"; then
    echo ""
    echo "Add to your PATH:"
    echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
  fi

  echo ""
  echo "Get started:"
  echo "  iii --config $CONFIG_DIR/config.yaml   # start iii with durable state"
  echo "  rimuru-worker                          # start the worker (serves UI on :3100)"
  echo "  rimuru agents detect                   # auto-detect installed agents"
  echo "  rimuru-tui                             # launch terminal UI"
}

main "$@"

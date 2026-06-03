#!/usr/bin/env bash
set -euo pipefail

TARGET="${1:-$HOME/.local/bin}"
MAN_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/man/man1"
BIN_NAME="trex"
REPO="tahasadough/trex"
VERSION="latest"

detect_platform() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Linux) os="unknown-linux-gnu" ;;
    Darwin) os="apple-darwin" ;;
    *) return 1 ;;
  esac

  case "$arch" in
    x86_64|amd64) arch="x86_64" ;;
    aarch64|arm64) arch="aarch64" ;;
    *) return 1 ;;
  esac

  echo "${arch}-${os}"
}

download_and_install() {
  local platform url archive tmp_dir

  platform="$(detect_platform)" || return 1

  if [ "$VERSION" = "latest" ]; then
    url="https://github.com/${REPO}/releases/latest/download/${BIN_NAME}-${platform}.tar.gz"
  else
    url="https://github.com/${REPO}/releases/download/${VERSION}/${BIN_NAME}-${platform}.tar.gz"
  fi

  tmp_dir="$(mktemp -d)"
  archive="${tmp_dir}/${BIN_NAME}.tar.gz"

  if command -v curl &>/dev/null; then
    curl -fsSL "$url" -o "$archive" || { rm -rf "$tmp_dir"; return 1; }
  elif command -v wget &>/dev/null; then
    wget -q "$url" -O "$archive" || { rm -rf "$tmp_dir"; return 1; }
  else
    rm -rf "$tmp_dir"
    echo "Error: need curl or wget to download." >&2
    return 1
  fi

  mkdir -p "$TARGET"
  tar -xzf "$archive" -C "$TARGET" "${BIN_NAME}" 2>/dev/null || {
    tar -xzf "$archive" -C "$tmp_dir"
    find "$tmp_dir" -name "${BIN_NAME}" -type f -exec cp {} "$TARGET/${BIN_NAME}" \;
  }
  chmod +x "$TARGET/${BIN_NAME}"

  # install man page if present in tarball
  if tar -tzf "$archive" | grep -q 'trex\.1'; then
    mkdir -p "$MAN_DIR"
    tar -xzf "$archive" -C "$MAN_DIR" "trex.1" 2>/dev/null || \
      tar -xzf "$archive" -C "$tmp_dir" && \
      find "$tmp_dir" -name "trex.1" -type f -exec cp {} "$MAN_DIR/trex.1" \;
    echo "Installed man page -> ${MAN_DIR}/trex.1"
  fi

  rm -rf "$tmp_dir"
  echo "Installed ${BIN_NAME} -> ${TARGET}/${BIN_NAME}"
}

check_tmux() {
  if ! command -v tmux &>/dev/null; then
    echo "Error: tmux is not installed." >&2
    echo "Install tmux via your package manager before using this tool." >&2
    exit 1
  fi
}

ensure_path() {
  case ":${PATH}:" in
  *:"$TARGET":*) return ;;
  esac

  local rc_file
  rc_file="${ZDOTDIR:-$HOME}/.zshrc"
  [[ -f "$rc_file" ]] || rc_file="$HOME/.bashrc"
  [[ -f "$rc_file" ]] || rc_file="$HOME/.profile"

  {
    echo ''
    echo "export PATH=\"\$PATH:$TARGET\""
  } >> "$rc_file"
  echo "Added $TARGET to PATH in $rc_file"
  echo "Restart your shell or run: export PATH=\"\$PATH:$TARGET\""
}

enable_service() {
  if ! command -v systemctl &>/dev/null; then
    echo "systemd not found - skipping auto-restore setup."
    echo "Restore manually with: trex restore"
    return
  fi

  mkdir -p "$HOME/.config/systemd/user"

  cat > "$HOME/.config/systemd/user/trex.service" << 'SERVICE'
[Unit]
Description=Restore tmux sessions after reboot (trex)
After=network.target

[Service]
Type=oneshot
ExecStart=%h/.local/bin/trex restore --quiet
RemainAfterExit=yes

[Install]
WantedBy=default.target
SERVICE

  systemctl --user daemon-reload
  systemctl --user enable trex.service
  echo "Auto-restore enabled (systemd)."

  "$TARGET/trex" auto enable 2>/dev/null || {
    rc_file="${ZDOTDIR:-$HOME}/.zshrc"
    [[ -f "$rc_file" ]] || rc_file="$HOME/.bashrc"
    [[ -f "$rc_file" ]] || rc_file="$HOME/.profile"
    printf '\n# trex: auto-restore tmux sessions\ncommand -v trex &>/dev/null && trex restore --quiet\n' >> "$rc_file"
    echo "Auto-restore enabled in $rc_file"
  }
}

build_from_source() {
  local script_dir manifest
  script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" 2>/dev/null && pwd)"
  manifest="$script_dir/Cargo.toml"

  if [ ! -f "$manifest" ]; then
    echo "Cloning repository to build from source..."
    local tmp_dir
    tmp_dir="$(mktemp -d)"
    git clone "https://github.com/${REPO}.git" "$tmp_dir" 2>/dev/null || {
      echo "Error: failed to clone repository." >&2
      exit 1
    }
    script_dir="$tmp_dir"
    manifest="$script_dir/Cargo.toml"
  fi

  echo "Building trex from source..."
  cargo build --release --manifest-path "$manifest"
  mkdir -p "$TARGET"
  cp "$script_dir/target/release/trex" "$TARGET/trex"

  if [ -f "$script_dir/man/trex.1" ]; then
    mkdir -p "$MAN_DIR"
    cp "$script_dir/man/trex.1" "$MAN_DIR/trex.1"
    echo "Installed man page -> $MAN_DIR/trex.1"
  fi

  echo "Installed trex -> $TARGET/trex"
}

main() {
  check_tmux

  if ! download_and_install; then
    if command -v cargo &>/dev/null; then
      echo "No prebuilt binary available. Building from source..."
      build_from_source
    else
      echo "Error: no prebuilt binary available for $(uname -s) $(uname -m)." >&2
      echo "" >&2
      echo "Install Rust first: https://rustup.rs" >&2
      exit 1
    fi
  fi

  ensure_path

  if command -v systemctl &>/dev/null; then
    echo ""
    read -r -p "Enable auto-restore on reboot? [Y/n] " reply </dev/tty
    case "$reply" in
      n|N|no|NO)
        echo "Skipping auto-restore."
        ;;
      *) enable_service ;;
    esac
  fi

  echo ""
  echo "Install complete. Run 'trex --help' or 'man trex' to get started."

  [[ -f "$0" ]] && rm -- "$0"
}

main

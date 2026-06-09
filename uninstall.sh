#!/usr/bin/env bash
set -euo pipefail

BIN_NAME="trex"
MAN_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/man"
REMOVED_ANY=false

remove_binary() {
  local dir label="$1"
  shift
  for dir in "$@"; do
    if [[ -f "$dir/$BIN_NAME" ]]; then
      rm -f "$dir/$BIN_NAME"
      echo "Removed $dir/$BIN_NAME"
      REMOVED_ANY=true
    fi
  done
}

remove_man_page() {
  local d
  for d in "$MAN_DIR/man1" "/usr/local/share/man/man1" "/usr/share/man/man1"; do
    if [[ -f "$d/$BIN_NAME.1" ]]; then
      rm -f "$d/$BIN_NAME.1"
      echo "Removed man page $d/$BIN_NAME.1"
      REMOVED_ANY=true
    fi
  done
}

remove_systemd_service() {
  local service="$HOME/.config/systemd/user/trex.service"
  if [[ -f "$service" ]]; then
    systemctl --user disable trex.service 2>/dev/null || true
    systemctl --user daemon-reload 2>/dev/null || true
    rm -f "$service"
    echo "Removed systemd user service: $service"
    REMOVED_ANY=true
  fi
}

remove_shell_hooks() {
  local rc_file line
  for rc_file in "${ZDOTDIR:-$HOME}/.zshrc" "$HOME/.bashrc" "$HOME/.profile" "$HOME/.bash_profile"; do
    [[ -f "$rc_file" ]] || continue
    line="$(grep -n 'command -v trex' "$rc_file" || true)"
    if [[ -n "$line" ]]; then
      # remove the trex auto-restore hook line and the blank line before it
      if [[ "$(uname -s)" = "Darwin" ]]; then
        sed -i '' -e '/# trex: auto-restore tmux sessions/d' -e '/command -v trex.*trex restore --quiet/d' "$rc_file"
      else
        sed -i '/# trex: auto-restore tmux sessions/d;/command -v trex.*trex restore --quiet/d' "$rc_file"
      fi
      echo "Removed trex auto-restore hook from $rc_file"
      REMOVED_ANY=true
    fi
    # remove PATH addition if it points to a dir that only contains trex or is now empty
    local target_line
    target_line="$(grep -n "export PATH=\".*\$PATH:$HOME/.local/bin\"" "$rc_file" || true)"
    if [[ -n "$target_line" ]] && [[ ! -f "$HOME/.local/bin/trex" ]]; then
      # only remove the PATH line if trex no longer exists there
      if [[ "$(uname -s)" = "Darwin" ]]; then
        sed -i '' -e "\|export PATH=\"\\\$PATH:$HOME/.local/bin\"|d" "$rc_file"
      else
        sed -i "\|export PATH=\"\\\$PATH:$HOME/.local/bin\"|d" "$rc_file"
      fi
      echo "Removed $HOME/.local/bin from PATH in $rc_file (trex no longer installed there)"
    fi
    # remove blank line before the removed hook (GNU sed)
  done
}

remove_cargo_bin() {
  local cargo_bin
  cargo_bin="${CARGO_HOME:-$HOME/.cargo}/bin"
  if [[ -f "$cargo_bin/$BIN_NAME" ]]; then
    rm -f "$cargo_bin/$BIN_NAME"
    echo "Removed $cargo_bin/$BIN_NAME"
    REMOVED_ANY=true
  fi
}

remove_cached_data() {
  local data_dir
  data_dir="${XDG_DATA_HOME:-$HOME/.local/share}/trex"
  if [[ -d "$data_dir" ]]; then
    rm -rf "$data_dir"
    echo "Removed saved session data: $data_dir"
    REMOVED_ANY=true
  fi
}

main() {
  echo "Uninstalling trex..."

  remove_binary "local bin" "$HOME/.local/bin" "/usr/local/bin" "/usr/bin"
  remove_cargo_bin
  remove_man_page
  remove_systemd_service
  remove_shell_hooks
  remove_cached_data

  # also check if there's a piped install in a custom target
  if [[ $# -gt 0 ]]; then
    for custom in "$@"; do
      if [[ -f "$custom/$BIN_NAME" ]]; then
        rm -f "$custom/$BIN_NAME"
        echo "Removed $custom/$BIN_NAME"
        REMOVED_ANY=true
      fi
    done
  fi

  if [[ "$REMOVED_ANY" = false ]]; then
    echo "trex is not installed anywhere."
  else
    echo ""
    echo "trex has been removed."
  fi
}

main "$@"

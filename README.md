# trex — Tmux Restore Extreme

![Written in Rust](https://img.shields.io/badge/written%20in-Rust-orange?logo=rust)

Persist your tmux sessions across reboots. Windows, panes, layouts, working directories, session options, and running commands come back exactly as you left them.

## Install

### Package managers

| Platform | Manager | Command |
|---|---|---|
| **Arch Linux** | AUR (yay/paru) | `yay -S trex` |
| **macOS / Linux** | Homebrew | `brew install tahasadough/trex/trex` |
| **Ubuntu / Debian** | APT (PPA) | `sudo add-apt-repository ppa:tahasadough/trex && sudo apt update && sudo apt install trex` |
| **Fedora** | DNF (COPR) | `sudo dnf copr enable tahasadough/trex && sudo dnf install trex` |

### From source (requires Rust toolchain)

```sh
cargo install --git https://github.com/tahasadough/trex
```

> The `.deb` and `.rpm` packages are attached to each [GitHub Release](https://github.com/tahasadough/trex/releases) for manual download. The PPA and COPR repositories are still being set up — watch the repo for announcements.

## Requirements

- **tmux** (install via your package manager)

## Usage

```
trex save                # save all sessions
trex save --current      # save only current session
trex save my-session     # save a specific session by name
trex restore             # restore saved sessions
trex restore --quiet     # restore silently
trex ls                  # list saved sessions
trex status              # show session info & timestamps
trex remove my-session   # remove one saved session
trex remove --all        # remove all saved data
trex ignore my-session   # exclude session from saves
trex ignore --list       # list ignored sessions
trex unignore my-session # stop ignoring a session
trex auto enable         # auto-restore on shell start
trex auto disable        # disable auto-restore
trex update              # update to latest version
```

Flags: `-q`/`--quiet`, `-c`/`--current`, `-a`/`--all`, `-l`/`--list`, `-h`/`--help`, `-V`/`--version`

Aliases: `s` (save), `r` (restore), `l` (ls), `st` (status), `rm` (remove), `ig` (ignore), `uig` (unignore), `a` (auto), `up` (update)

See also: `man trex` for the full manual page.

## Auto-restore

- **systemd**: The installer can create `~/.config/systemd/user/trex.service` — a oneshot service that runs `trex restore --quiet` after the network is up on login.
- **Shell hook**: `trex auto enable` appends a silent restore hook to `.bashrc`/`.zshrc`/`.profile`. `trex auto disable` removes it and cleans up the systemd service.

Both use `--quiet` so you never see output during startup.

## How it works

`trex save` serializes every session, window, and pane (names, layouts, directories, options, active panes, running commands) to a JSON file at `$XDG_DATA_HOME/trex/sessions.json`. `trex restore` rebuilds the session tree in three passes: (1) create sessions and split panes, (2) apply layouts and options, (3) re-issue saved commands via `tmux send-keys`.

## Uninstall

```sh
trex auto disable                                  # remove shell hooks & systemd service
rm -f "$(which trex)" ~/.local/bin/trex \
      "${CARGO_HOME:-$HOME/.cargo}/bin/trex"       # remove binary
rm -f "$HOME/.local/share/man/man1/trex.1"         # remove man page
rm -rf "${XDG_DATA_HOME:-$HOME/.local/share}/trex" # remove saved data
```

### Contributing & Development Guidelines

Contributions are welcome! To maintain code quality and consistency, please adhere to the following project standards:

- **Understand the Codebase:** Read through the existing code to familiarize yourself with current traits, error types, and test patterns before making changes.
- **Keep it Focused:** Prefer small, atomic edits. Please open an issue to discuss major refactors, component renames, or architectural changes before starting work.
- **Match Conventions:** Follow existing code styles: use crate granularity for imports, include `/// # Errors` sections in rustdoc comments, and use `#[serial]` with single assertions for integration tests.
- **Test:** Always write or update tests alongside any new feature or bug fix.

### AI-Assisted Development

This repository includes AI skill definitions in `.agents/skills/`. AI coding assistants (such as opencode) consume these skills to provide context-aware guidance — e.g., `rust-best-practices` ensures generated code follows idiomatic Rust conventions.

When using AI assistance, remember: **AI is a tool, not a replacement for judgment.** Always review, test, and validate AI-generated code. Treat it like any other contribution — write high-quality, production-grade code with proper tests.

See [`AGENTS.md`](AGENTS.md) for detailed project conventions and AI guidelines.

### Commands

```sh
cargo build --release        # release build
cargo test --all-features    # run all tests
cargo clippy -- -D warnings  # lint
cargo bench                  # benchmarks
cargo fmt                    # format
```

### CI

Two workflows: `ci.yml` runs clippy + tests on every push/PR; `release.yml` builds 4-target binaries and creates a GitHub release on `v*` tags.

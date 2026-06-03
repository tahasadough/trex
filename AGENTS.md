# AI Instructions — trex

## Project

[trex](https://github.com/tahasadough/trex) is a Rust CLI that persists and restores tmux sessions (windows, panes, layouts, working directories, running commands). Single binary, no runtime deps beyond tmux.

## Architecture

```
trex/
├── src/
│   ├── main.rs          — entry point, parses CLI, dispatches to commands
│   ├── lib.rs           — public module declarations
│   ├── cli.rs           — clap derive: Cli, Command, AutoAction enums
│   ├── error.rs         — TrexError enum + TrexResult<T> alias
│   ├── model.rs         — Sessions, SavedSession, SavedWindow, SavedPane
│   ├── storage.rs       — filesystem helpers (trex_dir, load/save sessions + ignore list)
│   ├── tmux.rs          — TmuxClient trait + Tmux (real) + MockTmux (test double); cross-platform ps/pgrep
│   ├── test_helpers.rs  — with_trex_dir, with_temp_home
│   └── commands/
│       ├── mod.rs       — execute() dispatcher
│       ├── save.rs      — save sessions
│       ├── restore.rs   — restore sessions (3-pass)
│       ├── list.rs      — ls command
│       ├── status.rs    — status command
│       ├── remove.rs    — remove saved data
│       ├── ignore.rs    — ignore/unignore sessions
│       ├── auto.rs      — enable/disable auto-restore (shell hooks + systemd)
│       └── update.rs    — self-update
├── tests/
│   ├── common/
│   │   └── mod.rs       — shared test helpers (with_temp_trex_dir)
│   ├── save_test.rs
│   ├── restore_test.rs
│   ├── auto_test.rs
│   ├── ignore_test.rs
│   ├── remove_test.rs
│   ├── status_test.rs
│   └── session_test.rs
└── benches/
    └── bench.rs          — criterion benchmarks (serialization, ignore ops)
```

## Conventions

### Imports

Crate-granularity: `use crate::error::TrexError;`, not deep paths. Group: std → extern crates → crate. No blank lines between groups.

### Error handling

- Use `TrexError` (thiserror enum) for all fallible operations. Never `unwrap()`/`expect()` outside tests.
- Include `/// # Errors` section in every public function that returns `TrexResult`.
- Use `?` for propagation, not match chains.

### Testing

- Unit tests in `#[cfg(test)] mod tests` inside source files. Integration tests in `tests/*.rs`.
- Shared test utilities live in `tests/common/mod.rs`; use `with_temp_trex_dir()` for integration tests.
- Use `#[serial]` from `serial_test` on any test mutating env/filesystem.
- One assertion per test where practical.
- Use `MockTmux` for command tests; `with_trex_dir()` for unit filesystem tests.
- Name: `fn save_all_sessions()`, `fn restore_with_no_data_returns_error()`.
- Benchmarks live in `benches/bench.rs` (criterion). Run with `cargo bench`.

### CLI

- New subcommands go in `commands/` and are registered in `commands/mod.rs` + `cli.rs`.
- Always support `--quiet` flag pattern on mutating commands.

### Code quality

- `cargo clippy --all-targets --all-features -- -D warnings` — must pass.
- `clippy::pedantic` is set to `warn` in Cargo.toml; prefer fixing pedantic lints over silencing them.
- `cargo test --all-features` — must pass.
- `cargo fmt` — match existing style.
- MSRV: 1.75.

## Skills

This repo ships `.agents/skills/rust-best-practices/` — an AI skill loaded by opencode for idiomatic Rust guidance. When generating code, always reference it.

## AI Principles

- AI is an assistant. Review every line before committing.
- Write high-quality, idiomatic Rust. No shortcuts.
- Match existing patterns. Don't introduce new dependencies without discussion.
- Prefer small, focused edits over large rewrites.

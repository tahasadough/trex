# Changelog

## [0.6.18] — 2026-06-08

### Fixed

- **save**: preserve last-known commands from previous save so panes whose application was closed before a re-save still have their commands restored.
- **restore**: send an initial Enter keystroke before each command to ensure the shell is at a fresh prompt.

### Changed

- Bumped version to 0.7.0

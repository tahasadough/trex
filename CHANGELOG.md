# Changelog

## [0.7.1] — 2026-06-09

### Added

- **packaging**: AUR (Arch Linux), Homebrew (macOS), Debian/Ubuntu PPA, and Copr (Fedora/RHEL) packages.
- **macOS**: launchd agent support for auto-restore.
- **ci**: automatic `.deb` and `.rpm` package builds in the release workflow.
- **scripts**: `scripts/update-formula.sh` for Homebrew formula updates.

### Removed

- **install.sh** and **uninstall.sh** — distribution is now handled by package managers.

### Changed

- Bumped version to 0.7.1.

## [0.6.18] — 2026-06-08

### Fixed

- **save**: preserve last-known commands from previous save so panes whose application was closed before a re-save still have their commands restored.
- **restore**: send an initial Enter keystroke before each command to ensure the shell is at a fresh prompt.

### Changed

- Bumped version to 0.7.0.

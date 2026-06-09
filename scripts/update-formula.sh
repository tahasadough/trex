#!/usr/bin/env bash
# Updates packaging files after a new release.
# Usage: scripts/update-formula.sh <version> <sha256-x86_64-linux> <sha256-aarch64-linux> <sha256-x86_64-macos> <sha256-aarch64-macos>
set -euo pipefail

if [ $# -ne 5 ]; then
  echo "Usage: $0 <version> <sha256-x86_64-linux> <sha256-aarch64-linux> <sha256-x86_64-macos> <sha256-aarch64-macos>" >&2
  exit 1
fi

VERSION="$1"
SHA_LINUX_X64="$2"
SHA_LINUX_ARM64="$3"
SHA_MACOS_X64="$4"
SHA_MACOS_ARM64="$5"
SED_CMD="sed -i"

if [[ "$(uname -s)" == "Darwin" ]]; then
  SED_CMD="sed -i ''"
fi

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

# --- AUR PKGBUILD ---
$SED_CMD "s/^pkgver=.*/pkgver=${VERSION}/" "$ROOT/packaging/aur/PKGBUILD"

# --- Homebrew formula ---
$SED_CMD "s/version \"[0-9.]*\"/version \"${VERSION}\"/" "$ROOT/packaging/homebrew/trex.rb"
$SED_CMD "s|sha256 \".*\"|sha256 \"${SHA_MACOS_ARM64}\"|" "$ROOT/packaging/homebrew/trex.rb"

# --- Debian changelog ---
DEB_DATE="$(date -R)"
DEB_VERSION="${VERSION}-1"
# Insert new entry at top of changelog
TMP="$(mktemp)"
{
  echo "trex (${DEB_VERSION}) unstable; urgency=medium"
  echo ""
  echo "  * Release ${VERSION}."
  echo ""
  echo " -- Taha Sadough <taha@sadough.dev>  ${DEB_DATE}"
  echo ""
  cat "$ROOT/packaging/debian/changelog"
} > "$TMP"
mv "$TMP" "$ROOT/packaging/debian/changelog"

# --- RPM spec ---
$SED_CMD "s/^Version:.*/Version:    ${VERSION}/" "$ROOT/packaging/copr/trex.spec"

echo "Updated packaging files to version ${VERSION}."
echo ""
echo "Verify and commit:"
echo "  git add packaging/ && git commit -m \"Bump packaging files to v${VERSION}\""

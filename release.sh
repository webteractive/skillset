#!/usr/bin/env sh
# Build release binary and create/update a GitHub release via gh.
# Usage:
#   ./release.sh <tag>              # create release and upload binary (first machine)
#   ./release.sh <tag> --upload      # build and upload binary only (add to existing release)
# Example: ./release.sh v0.1.0
# Requires: cargo, gh (GitHub CLI), and gh auth login.

set -e

TAG="${1:-}"
UPLOAD_ONLY=false
if [ "${2:-}" = "--upload" ]; then
  UPLOAD_ONLY=true
fi

if [ -z "$TAG" ]; then
  echo "Usage: ./release.sh <tag> [--upload]"
  echo "  <tag>     e.g. v0.1.0"
  echo "  --upload  only build and upload asset (for adding from another OS/arch)"
  exit 1
fi

# Build
echo "Building release binary..."
cargo build --release

# Asset name by OS and arch so multiple uploads don't overwrite
OS="$(uname -s | tr 'A-Z' 'a-z')"
ARCH="$(uname -m)"
case "$ARCH" in
  x86_64|amd64) ARCH="x86_64" ;;
  arm64|aarch64) ARCH="aarch64" ;;
esac
ASSET_NAME="skillset-${OS}-${ARCH}"

cp target/release/skillset "$ASSET_NAME"
echo "Built: $ASSET_NAME"

if [ "$UPLOAD_ONLY" = true ]; then
  echo "Uploading to existing release $TAG..."
  gh release upload "$TAG" "$ASSET_NAME" --clobber
  rm -f "$ASSET_NAME"
  echo "Done. Run ./install.sh $ASSET_NAME after downloading, or download from the release page."
  exit 0
fi

# Create release and upload (or upload only if release exists)
echo "Creating release $TAG and uploading $ASSET_NAME..."
if ! gh release create "$TAG" "$ASSET_NAME" \
  --title "$TAG" \
  --notes "Release $TAG. Download the binary for your OS/arch, then run \`./install.sh <binary>\` or put it on your PATH as \`skillset\`."; then
  echo "Release $TAG already exists, uploading asset..."
  gh release upload "$TAG" "$ASSET_NAME" --clobber
fi
rm -f "$ASSET_NAME"
echo "Done. To add another OS/arch, build on that machine and run: ./release.sh $TAG --upload"
echo "Release: $(gh release view "$TAG" --json url -q .url)"
exit 0

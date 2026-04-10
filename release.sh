#!/usr/bin/env sh
# Build release binary and create/update a GitHub release via gh.
# Usage:
#   ./release.sh patch|minor|major   # bump version, commit, tag, build, release, upload
#   ./release.sh --upload <tag>      # build and upload binary only (add to existing release from another OS/arch)
# Requires: cargo, gh (GitHub CLI), and gh auth login.

set -e

# --- Parse args ---
UPLOAD_ONLY=false
BUMP=""
TAG=""

if [ "${1:-}" = "--upload" ]; then
  UPLOAD_ONLY=true
  TAG="${2:-}"
  if [ -z "$TAG" ]; then
    echo "Usage: ./release.sh --upload <tag>"
    exit 1
  fi
else
  BUMP="${1:-}"
fi

if [ "$UPLOAD_ONLY" = false ] && [ -z "$BUMP" ]; then
  echo "Usage: ./release.sh patch|minor|major"
  echo "       ./release.sh --upload <tag>"
  exit 1
fi

# --- Bump version (unless --upload) ---
if [ "$UPLOAD_ONLY" = false ]; then
  # Abort if working tree is dirty
  if [ -n "$(git status --porcelain)" ]; then
    echo "ERROR: Working tree is dirty. Commit or stash changes before releasing."
    exit 1
  fi

  # Read current version from Cargo.toml
  CURRENT="$(grep '^version' Cargo.toml | head -1 | sed -E 's/.*"([^"]+)".*/\1/')"
  MAJOR="$(echo "$CURRENT" | cut -d. -f1)"
  MINOR="$(echo "$CURRENT" | cut -d. -f2)"
  PATCH="$(echo "$CURRENT" | cut -d. -f3)"

  # Validate parsed version
  if [ -z "$CURRENT" ] || [ -z "$MAJOR" ] || [ -z "$MINOR" ] || [ -z "$PATCH" ]; then
    echo "ERROR: Could not parse version from Cargo.toml (got: '$CURRENT')"
    exit 1
  fi
  case "$MAJOR$MINOR$PATCH" in
    *[!0-9]*) echo "ERROR: Version components are not numeric: $MAJOR.$MINOR.$PATCH"; exit 1 ;;
  esac

  case "$BUMP" in
    patch) PATCH=$((PATCH + 1)) ;;
    minor) MINOR=$((MINOR + 1)); PATCH=0 ;;
    major) MAJOR=$((MAJOR + 1)); MINOR=0; PATCH=0 ;;
    *)
      echo "Invalid bump type: $BUMP"
      echo "Usage: ./release.sh patch|minor|major"
      exit 1
      ;;
  esac

  NEXT="${MAJOR}.${MINOR}.${PATCH}"
  TAG="v${NEXT}"

  echo "Bumping version: ${CURRENT} → ${NEXT} (${BUMP})"

  # Update Cargo.toml (portable: write to temp file then move)
  sed -E "s/^version = \"${CURRENT}\"/version = \"${NEXT}\"/" Cargo.toml > Cargo.toml.tmp && mv Cargo.toml.tmp Cargo.toml

  # Verify the substitution actually worked
  VERIFY="$(grep '^version' Cargo.toml | head -1 | sed -E 's/.*"([^"]+)".*/\1/')"
  if [ "$VERIFY" != "$NEXT" ]; then
    echo "ERROR: Failed to update version in Cargo.toml (expected $NEXT, got $VERIFY)"
    exit 1
  fi

  # Update Cargo.lock
  cargo generate-lockfile --quiet

  # Commit and tag
  git add Cargo.toml Cargo.lock
  git commit -m "Release version ${NEXT}"
  git tag "$TAG"
  echo "Tagged: $TAG"
fi

# --- Build ---
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
  echo "Done."
  exit 0
fi

# --- Push and release ---
if ! git push origin HEAD "$TAG"; then
  echo ""
  echo "ERROR: Push failed. Your local commit and tag ($TAG) are still present."
  echo "To retry:  git push origin HEAD $TAG"
  echo "To undo:   git tag -d $TAG && git reset HEAD~1"
  rm -f "$ASSET_NAME"
  exit 1
fi

echo "Creating release $TAG and uploading $ASSET_NAME..."
GH_VIEW_OUTPUT=$(gh release view "$TAG" 2>&1) && {
  echo "Release $TAG already exists, uploading asset..."
  gh release upload "$TAG" "$ASSET_NAME" --clobber
} || {
  if echo "$GH_VIEW_OUTPUT" | grep -qi "not found"; then
    gh release create "$TAG" "$ASSET_NAME" \
      --title "$TAG" \
      --notes "Release $TAG. Download the binary for your OS/arch, then run \`./install.sh <binary>\` or put it on your PATH as \`skillset\`."
  else
    echo "ERROR: gh release view failed:"
    echo "$GH_VIEW_OUTPUT"
    rm -f "$ASSET_NAME"
    exit 1
  fi
}
rm -f "$ASSET_NAME"
echo "Done. To add another OS/arch, build on that machine and run: ./release.sh --upload $TAG"
echo "Release: $(gh release view "$TAG" --json url -q .url)"
exit 0

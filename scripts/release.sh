#!/bin/sh
# Release script: bump version, update changelog, tag.
set -e

if [ -z "$1" ]; then
    echo "Usage: scripts/release.sh <version>  (e.g. 0.2.0)"
    exit 1
fi

VERSION="$1"
echo "Releasing $VERSION..."

# Bump Cargo.toml
sed -i.bak "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
rm -f Cargo.toml.bak

# Regenerate lock
cargo build --offline

# Commit
git add -A
git commit -m "chore: release $VERSION"

# Tag
git tag -a "v$VERSION" -m "Release $VERSION"

echo "Released v$VERSION. Push with: git push && git push --tags"

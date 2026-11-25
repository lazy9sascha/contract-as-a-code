#!/bin/sh
# Commit the reviewed version bump and changelog before running this helper.
set -eu

fail() {
    printf '%s\n' "$1" >&2
    exit 1
}

require_clean() {
    status=$(git status --porcelain=v1 --untracked-files=all --ignore-submodules=none)
    [ -z "$status" ] || fail "Release requires a clean index and working tree."
}

[ "$#" -eq 1 ] || fail "Usage: scripts/release.sh <version>"
version=$1
tag="v$version"
git check-ref-format "refs/tags/$tag" || fail "Invalid release tag: $tag"

root=$(git rev-parse --show-toplevel)
cd "$root"
require_clean
start=$(git rev-parse --verify HEAD)
existing_tag=$(git tag --list "$tag")
[ -z "$existing_tag" ] || fail "Tag already exists: $tag"

scratch=$(mktemp -d "${TMPDIR:-/tmp}/caac-release.XXXXXX")
checkout="$scratch/checkout"
cleanup() {
    if [ -f "$checkout/.git" ]; then
        git worktree remove --force "$checkout"
    fi
    rmdir "$scratch"
}
trap cleanup EXIT
trap 'exit 1' HUP INT TERM

git worktree add --quiet --detach "$checkout" "$start"
(
    cd "$checkout"
    package=$(cargo pkgid --locked)
    case "$package" in
        *"#caac@$version") ;;
        *) fail "Commit the Cargo.toml and Cargo.lock version bump to $version first." ;;
    esac
    cargo fmt --all -- --check
    cargo test --locked
    cargo clippy --all-targets --locked -- -D warnings
    sh scripts/check_contracts.sh
    require_clean
)
cleanup
trap - EXIT HUP INT TERM

[ "$(git rev-parse HEAD)" = "$start" ] || fail "HEAD changed during the release checks."
require_clean
git tag -a "$tag" -m "Release $version" "$start"
printf 'Created local tag %s.\n' "$tag"

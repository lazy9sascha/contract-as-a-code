#!/bin/sh
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/caac-release.XXXXXX")
trap 'rm -rf "$scratch"' EXIT
trap 'exit 1' HUP INT TERM
export CARGO_NET_OFFLINE=true CARGO_TARGET_DIR="$scratch/target"

git -C "$root" archive HEAD > "$scratch/project.tar"
mkdir "$scratch/repo" "$scratch/hooks"
cd "$scratch/repo"
tar -xf "$scratch/project.tar"
cp "$root/scripts/release.sh" scripts/release.sh
git init --quiet
git config user.name "Release Regression"
git config user.email "release-regression@example.invalid"
git config core.hooksPath "$scratch/hooks"
git config commit.gpgSign false
git config tag.gpgSign false
git add -- .
git commit --quiet -m "test: create release fixture"
[ -z "$(git remote)" ]

expect_failure() {
    if sh scripts/release.sh "$1" > "$scratch/release.log" 2>&1; then
        cat "$scratch/release.log"
        printf 'Release %s unexpectedly succeeded.\n' "$1" >&2
        exit 1
    fi
    if ! grep -F "$2" "$scratch/release.log"; then
        cat "$scratch/release.log"
        exit 1
    fi
    [ "$(git rev-parse HEAD)" = "$before" ]
    [ -z "$(git tag --list "v$1")" ]
}

expect_success() {
    if ! sh scripts/release.sh "$1" > "$scratch/release.log" 2>&1; then
        cat "$scratch/release.log"
        exit 1
    fi
    [ "$(git cat-file -t "refs/tags/v$1")" = tag ]
    [ "$(git rev-parse "refs/tags/v$1^{commit}")" = "$(git rev-parse HEAD)" ]
    [ -z "$(git status --porcelain)" ]
}

before=$(git rev-parse HEAD)
expect_failure 99.0.1 "Commit the Cargo.toml and Cargo.lock version bump to 99.0.1 first."
[ -z "$(git status --porcelain)" ]

sed 's/^version = ".*"$/version = "99.0.1"/' Cargo.toml > "$scratch/Cargo.toml"
mv "$scratch/Cargo.toml" Cargo.toml
cargo update --workspace --offline
git add -- Cargo.toml Cargo.lock
git commit --quiet -m "test: prepare source version in a patch"
before=$(git rev-parse HEAD)

printf 'private fixture\n' > private.txt
expect_failure 99.0.1 "Release requires a clean index and working tree."
[ "$(cat private.txt)" = "private fixture" ]
[ "$(git status --porcelain)" = "?? private.txt" ]
mv private.txt "$scratch/private.txt"

printf 'staged fixture\n' > staged.txt
git add -- staged.txt
printf 'working copy fixture\n' > staged.txt
expect_failure 99.0.1 "Release requires a clean index and working tree."
[ "$(git show :staged.txt)" = "staged fixture" ]
[ "$(cat staged.txt)" = "working copy fixture" ]
[ "$(git status --porcelain)" = "AM staged.txt" ]
git rm --cached --force --quiet -- staged.txt
rm staged.txt
printf 'Preserved the untracked file and both versions of the staged file.\n'

git switch --quiet -c failing-check
printf '%s\n' \
    '#[test]' \
    'fn deliberately_failing_test() {' \
    '    assert_eq!(1, 2, "intentional release-check failure");' \
    '}' > tests/release_fixture_failure.rs
git add -- tests/release_fixture_failure.rs
git commit --quiet -m "test: add a failing Rust test"
before=$(git rev-parse HEAD)
expect_failure 99.0.1 "intentional release-check failure"
[ -z "$(git status --porcelain)" ]
git switch --quiet -
printf 'Passed check failure without a commit, tag, or manifest edits.\n'

git switch --quiet -c concurrent-edit
export CAAC_RELEASE_FIXTURE_SOURCE="$PWD"
printf '%s\n' \
    'printf "\n# concurrent manifest edit\n" >> "$CAAC_RELEASE_FIXTURE_SOURCE/Cargo.toml"' \
    'git -C "$CAAC_RELEASE_FIXTURE_SOURCE" add -- Cargo.toml' \
    'printf "\n# concurrent lockfile edit\n" >> "$CAAC_RELEASE_FIXTURE_SOURCE/Cargo.lock"' \
    >> scripts/check_contracts.sh
git add -- scripts/check_contracts.sh
git commit --quiet -m "test: edit the source checkout during release checks"
before=$(git rev-parse HEAD)
expect_failure 99.0.1 "Release requires a clean index and working tree."
[ "$(git show :Cargo.toml | tail -1)" = "# concurrent manifest edit" ]
[ "$(tail -1 Cargo.toml)" = "# concurrent manifest edit" ]
[ "$(tail -1 Cargo.lock)" = "# concurrent lockfile edit" ]
[ "$(git status --porcelain)" = "$(printf ' M Cargo.lock\nM  Cargo.toml')" ]
git restore --staged --worktree -- Cargo.toml Cargo.lock
git switch --quiet -
printf 'Preserved manifest and lockfile edits made during the checks.\n'

printf 'private.txt\n' >> .git/info/exclude
cp "$scratch/private.txt" private.txt
before=$(git rev-parse HEAD)
expect_success 99.0.1
[ "$(git rev-parse HEAD)" = "$before" ]
[ "$(git diff-tree --no-commit-id -r --name-only HEAD)" = "$(printf 'Cargo.lock\nCargo.toml')" ]
[ "$(cat private.txt)" = "private fixture" ]
printf 'Tagged the checked version bump and preserved the ignored private file.\n'

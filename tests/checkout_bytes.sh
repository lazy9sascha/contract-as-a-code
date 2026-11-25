#!/bin/sh
set -eu

cd "$(git rev-parse --show-toplevel)"
scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT

for autocrlf in false true; do
    git -c core.autocrlf="$autocrlf" -c core.eol=lf checkout-index \
        --all --prefix="$scratch/$autocrlf/"
done

if ! diff -qr "$scratch/false" "$scratch/true"; then
    echo 'Checkout bytes differ with core.autocrlf. Set the checkout policy in .gitattributes.' >&2
    exit 1
fi

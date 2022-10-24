#!/usr/bin/env bash
# ------------------------------------------------------------------
# Builds ink! contract source code and generates a compressed
# archive suitable for uploading to the verification service.
# ------------------------------------------------------------------
set -eu

SRC_ROOT="${SRC_ROOT:-/build}"
TMP_PACKAGE_BASE="$SRC_ROOT/package"
TMP_PACKAGE_SRC="$TMP_PACKAGE_BASE/src"
TMP_PACKAGE_TARGET="$TMP_PACKAGE_SRC/target"
PACKAGE_DST="${SRC_ROOT}/target/ink/package.zip"

# We pin the root source path to emit stable dir names
# on debug info (to `/build/package/src`).
# When/if cargo-contracts supports adding RUSTC_FLAGS we could
# use 'remap-path-prefix'
mkdir -p "$TMP_PACKAGE_SRC"
cp -r ${SRC_ROOT}/* "$TMP_PACKAGE_SRC"
SRC_ROOT="$TMP_PACKAGE_SRC"

# Build contract
. /usr/local/bin/build-contract

# Build verification package
mv ${TMP_PACKAGE_TARGET}/ink/*.contract "${TMP_PACKAGE_BASE}"
rm -rf "$TMP_PACKAGE_TARGET"
(cd "${TMP_PACKAGE_BASE}" && zip -r - src/ *.contract) > "$PACKAGE_DST"
rm -rf  "$TMP_PACKAGE_BASE"

echo "Verification package in $PACKAGE_DST"
unzip -l "$PACKAGE_DST"
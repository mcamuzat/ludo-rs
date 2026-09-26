#!/usr/bin/env bash
# Builds ludo-wasm and generates the JS glue into web/src/wasm/.
#   ./scripts/build-wasm.sh          optimized build (profile wasm-release + wasm-opt)
#   ./scripts/build-wasm.sh dev      fast build, no optimization
set -euo pipefail
cd "$(dirname "$0")/.."

PROFILE="wasm-release"
[ "${1:-}" = "dev" ] && PROFILE="dev"
OUT="web/src/wasm"
TARGET_DIR="${CARGO_TARGET_DIR:-target}"

cargo build -p ludo-wasm --target wasm32-unknown-unknown --profile "$PROFILE"

# The "dev" profile writes to target/.../debug/.
PROFILE_DIR="$PROFILE"
[ "$PROFILE" = "dev" ] && PROFILE_DIR="debug"

wasm-bindgen --target web --out-dir "$OUT" --out-name ludo \
  "$TARGET_DIR/wasm32-unknown-unknown/$PROFILE_DIR/ludo_wasm.wasm"

if [ "$PROFILE" = "wasm-release" ] && command -v wasm-opt >/dev/null; then
  wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int \
    --enable-sign-ext --enable-mutable-globals --enable-reference-types --enable-multivalue \
    "$OUT/ludo_bg.wasm" -o "$OUT/ludo_bg.wasm"
fi

ls -lh "$OUT"/ludo_bg.wasm

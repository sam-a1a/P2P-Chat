#!/usr/bin/env bash
set -euo pipefail
CORE_DIR="$(cd "$(dirname "$0")" && pwd)"
: "${ANDROID_NDK_HOME:?Set ANDROID_NDK_HOME to an installed Android NDK}"
command -v cargo-ndk >/dev/null || { echo 'Install cargo-ndk: cargo install cargo-ndk --locked'; exit 1; }
cd "$CORE_DIR"
if [ "$#" -eq 0 ]; then set -- aarch64-linux-android armv7-linux-androideabi x86_64-linux-android; fi
args=()
for target in "$@"; do rustup target add "$target"; args+=(--target "$target"); done
cargo ndk "${args[@]}" --platform 33 --output-dir ../mobile/native-android/app/src/main/jniLibs build --release --locked -p p2p --lib

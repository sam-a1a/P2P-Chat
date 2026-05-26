#!/bin/bash
set -e

ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-$HOME/Library/Android/sdk/ndk/$(ls $HOME/Library/Android/sdk/ndk | tail -1)}"
OUTPUT_DIR="../Android-Kotlin/app/src/main/jniLibs"

echo "Using NDK: $ANDROID_NDK_HOME"
echo "Output: $OUTPUT_DIR"

mkdir -p "$OUTPUT_DIR"

cargo ndk \
  --target aarch64-linux-android \
  --target armv7-linux-androideabi \
  --target x86_64-linux-android \
  --output-dir "$OUTPUT_DIR" \
  build --release
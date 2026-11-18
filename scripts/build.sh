#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"
git -C "$ROOT" submodule update --init third_party/probe-rs third_party/nusb
NDK=${ANDROID_NDK_HOME:-$HOME/android-sdk/ndk/26.1.10909125}
TOOLCHAIN="$NDK/toolchains/llvm/prebuilt/linux-x86_64"
API=${ANDROID_API:-30}
TARGET=aarch64-linux-android

export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$TOOLCHAIN/bin/${TARGET}${API}-clang"
export CC_aarch64_linux_android="$CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER"
export AR_aarch64_linux_android="$TOOLCHAIN/bin/llvm-ar"

cd "$ROOT"
cargo build --target "$TARGET" --release
printf '\nbuilt: %s\n' "$ROOT/target/$TARGET/release/probe-rs-android"

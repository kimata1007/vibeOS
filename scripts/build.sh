#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
export CARGO_TARGET_DIR="$PWD/target"
cargo rustc -p vibeos-kernel --bin vibeos-kernel --target x86_64-unknown-none --release -- -C relocation-model=static -C link-arg=-T"$PWD/kernel/linker.ld"
export VIBEOS_KERNEL="$CARGO_TARGET_DIR/x86_64-unknown-none/release/vibeos-kernel"
cargo build -p vibeos-bootloader --target x86_64-unknown-uefi
mkdir -p build/esp/EFI/BOOT
cp target/x86_64-unknown-uefi/debug/vibeos-bootloader.efi build/esp/EFI/BOOT/BOOTX64.EFI

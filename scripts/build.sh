#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
export CARGO_TARGET_DIR="$PWD/target"
cargo build -p vibeos-bootloader --target x86_64-unknown-uefi
mkdir -p build/esp/EFI/BOOT
cp target/x86_64-unknown-uefi/debug/vibeos-bootloader.efi build/esp/EFI/BOOT/BOOTX64.EFI

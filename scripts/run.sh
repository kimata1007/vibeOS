#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
if ! command -v qemu-system-x86_64 >/dev/null 2>&1; then
  echo "QEMU is not installed or not on PATH. On macOS, run: brew install qemu" >&2
  exit 1
fi
bash scripts/build.sh
firmware="${QEMU_FIRMWARE_DIR:-$(brew --prefix qemu)/share/qemu}"
code="$firmware/edk2-x86_64-code.fd"
vars="$firmware/edk2-i386-vars.fd"
test -f "$code" && test -f "$vars" || { echo "Set QEMU_FIRMWARE_DIR to the firmware directory" >&2; exit 1; }
cp "$vars" build/vars.fd
chmod u+w build/vars.fd
exec qemu-system-x86_64 -machine q35,accel=tcg -m 256M \
  -drive "if=pflash,format=raw,readonly=on,file=$code" \
  -drive "if=pflash,format=raw,file=build/vars.fd" \
  -drive "format=raw,file=fat:rw:build/esp" -net none -no-reboot -serial stdio

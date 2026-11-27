# Development

VibeOS is a from-scratch x86-64 OS written in Rust, developed through vibe coding. The host is Apple Silicon macOS; QEMU will emulate x86-64. UEFI is the firmware interface, not the kernel.

## Setup

Install Rust via rustup and QEMU via Homebrew (`brew install qemu`). From the repository root, run:

```sh
cargo build -p vibeos-bootloader --target x86_64-unknown-uefi
cargo build -p vibeos-kernel --target x86_64-unknown-none
```

The bootloader currently returns success to firmware; it does not load the kernel. The kernel is only a library scaffold. Next milestones: UEFI console output, separate kernel loading, ExitBootServices, and QEMU boot verification.

License is pending a project decision.

# Development

VibeOS is a from-scratch x86-64 OS written in Rust through vibe coding. Apple Silicon macOS runs an emulated x86-64 machine in QEMU.

## Setup and run

Install stable Rust via rustup and QEMU via Homebrew (`brew install qemu`). Then run:

```sh
bash scripts/build.sh
bash scripts/run.sh
```

Set `QEMU_FIRMWARE_DIR` if the firmware is not in Homebrew's default directory. The scripts stage a FAT boot volume under build/esp, without writing physical disks.

## Boot sequence

UEFI loads BOOTX64.EFI, prints a boot message, and exposes the memory map. The loader embeds a separately linked ELF kernel, copies its loadable segments at 32 MiB, captures GOP information, obtains the final memory map, and exits Boot Services. The kernel switches to its own stack, writes to COM1, and draws directly to the RGB/BGR framebuffer.

Expected terminal: VibeOS kernel entered after ExitBootServices; Framebuffer rendered.
Expected QEMU display: VIBEOS / KERNEL OK.

These are expected outputs, not verified test results. The current implementation assumes QEMU/OVMF identity mappings, a free fixed load address, and a supported GOP mode. No allocator, independent page tables, IDT, filesystem or runtime services integration yet. The loader's memory and page tables must remain reserved. License is pending a project decision.

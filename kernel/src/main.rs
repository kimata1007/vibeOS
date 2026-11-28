#![no_std]
#![no_main]

use core::{arch::global_asm, panic::PanicInfo};

global_asm!(
    ".section .bss.stack,\"aw\",@nobits",
    ".balign 16",
    "stack_bottom:",
    ".skip 65536",
    "stack_top:",
    ".section .text._start,\"ax\"",
    ".global _start",
    "_start:",
    "cli",
    "cld",
    "lea rsp, [rip + stack_top]",
    "call kernel_main",
    "2: hlt",
    "jmp 2b",
);

#[unsafe(no_mangle)]
extern "C" fn kernel_main(info: *const vibeos_kernel::BootInfo) -> ! {
    // The loader stack and map remain reserved until the kernel no longer needs them.
    if !info.is_null() {
        serial(b"VibeOS kernel entered after ExitBootServices\r\n");
    }
    loop {
        unsafe { core::arch::asm!("hlt", options(nomem, nostack)) };
    }
}

fn out(port: u16, value: u8) {
    unsafe {
        core::arch::asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
    }
}

fn input(port: u16) -> u8 {
    let value: u8;
    unsafe {
        core::arch::asm!("in al, dx", in("dx") port, out("al") value, options(nomem, nostack, preserves_flags));
    }
    value
}

fn serial(message: &[u8]) {
    const COM: u16 = 0x3f8;
    out(COM + 1, 0);
    out(COM + 3, 0x80);
    out(COM, 3);
    out(COM + 1, 0);
    out(COM + 3, 3);
    out(COM + 2, 0xc7);
    out(COM + 4, 0x0b);
    for &byte in message {
        for _ in 0..100000 {
            if input(COM + 5) & 0x20 != 0 {
                out(COM, byte);
                break;
            }
            core::hint::spin_loop();
        }
    }
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {
        unsafe { core::arch::asm!("cli; hlt", options(nomem, nostack)) };
    }
}

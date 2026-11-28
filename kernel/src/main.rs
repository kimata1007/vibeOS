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
extern "C" fn kernel_main() -> ! {
    loop { unsafe { core::arch::asm!("hlt", options(nomem, nostack)); } }
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop { unsafe { core::arch::asm!("cli; hlt", options(nomem, nostack)); } }
}

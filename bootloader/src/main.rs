#![no_std]
#![no_main]

use core::panic::PanicInfo;

// Initial UEFI entry point; loading the kernel is a future milestone.
#[unsafe(no_mangle)]
pub extern "efiapi" fn efi_main(_image: usize, _system_table: *mut core::ffi::c_void) -> usize {
    0
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop { core::hint::spin_loop(); }
}

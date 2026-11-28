#![no_std]
#![no_main]

mod uefi;
use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "efiapi" fn efi_main(_image: usize, table: *mut uefi::SystemTable) -> usize {
    // SAFETY: UEFI supplies the system table at application entry.
    let status =
        unsafe { uefi::print(table, "VibeOS v0.0.1\r\n[ OK ] Bootloader initialized.\r\n") };
    if status & uefi::ERROR != 0 {
        return status;
    }
    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

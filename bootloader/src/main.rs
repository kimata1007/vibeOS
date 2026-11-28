#![no_std]
#![no_main]

mod uefi;
mod elf;
static KERNEL: &[u8] = include_bytes!(env!("VIBEOS_KERNEL"));
use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "efiapi" fn efi_main(_image: usize, table: *mut uefi::SystemTable) -> usize {
    // SAFETY: UEFI supplies the system table at application entry.
    let status = unsafe { uefi::print(table, "VibeOS v0.0.1\r\n[ OK ] Bootloader initialized.\r\n") };
    if status & uefi::ERROR != 0 { return status; }
    let bs = unsafe { (*table).boot_services };
    if bs.is_null() { return uefi::INVALID_PARAMETER; }
    let mut buffer = uefi::MapBuffer([0; 65536]);
    let _map = match unsafe { uefi::memory_map(bs, &mut buffer) } {
        Ok(map) => map,
        Err(status) => return status,
    };
    let status = unsafe { uefi::print(table, "[ OK ] Memory map acquired.\r\n") };
    if status & uefi::ERROR != 0 { return status; }
    let _entry = match unsafe { elf::load(bs, KERNEL) } {
        Ok(entry) => entry,
        Err(status) => return status,
    };
    let status = unsafe { uefi::print(table, "[ OK ] Kernel ELF loaded.\r\n") };
    if status & uefi::ERROR != 0 { return status; }
    loop { core::hint::spin_loop(); }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop { core::hint::spin_loop(); }
}

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
    let entry = match unsafe { elf::load(bs, KERNEL) } {
        Ok(entry) => entry,
        Err(status) => return status,
    };
    let status = unsafe { uefi::print(table, "[ OK ] Kernel ELF loaded.\r\n") };
    if status & uefi::ERROR != 0 { return status; }
    let framebuffer = unsafe { uefi::framebuffer(bs) };
    // GOP information is captured before ExitBootServices.
    // No firmware calls between GetMemoryMap and ExitBootServices except ExitBootServices.
    // Retry once if firmware changed the map key.
    let mut attempts = 0;
    loop {
        let map = match unsafe { uefi::memory_map(bs, &mut buffer) } {
            Ok(map) => map,
            Err(status) => {
                if attempts == 0 { return status; }
                halt();
            }
        };
        let info = vibeos_kernel::BootInfo {
            memory_map: buffer.0.as_ptr(), memory_map_size: map.size,
            descriptor_size: map.stride, descriptor_version: map.version,
            framebuffer,
        };
        let status = unsafe { ((*bs).exit_boot_services)(_image, map.key) };
        if status == 0 {
            // SAFETY: trusted ELF entry, SysV x86-64 ABI, identity mapping from firmware.
            // The kernel switches to its own stack and never returns.
            let start: unsafe extern "sysv64" fn(*const vibeos_kernel::BootInfo) -> ! =
                unsafe { core::mem::transmute(entry) };
            unsafe { start(&info) }
        }
        if status != uefi::INVALID_PARAMETER || attempts != 0 { halt(); }
        attempts += 1;
    }
}

fn halt() -> ! {
    loop { unsafe { core::arch::asm!("cli; hlt", options(nomem, nostack)); } }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop { core::hint::spin_loop(); }
}

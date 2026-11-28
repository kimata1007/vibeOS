#![no_std]
#![no_main]

mod elf;
mod uefi;

static KERNEL: &[u8] = include_bytes!(env!("VIBEOS_KERNEL"));
use core::panic::PanicInfo;

/// UEFI application entry point.
///
/// # Safety
/// The firmware must provide a valid system table with active boot services.
#[unsafe(no_mangle)]
pub unsafe extern "efiapi" fn efi_main(image: usize, table: *mut uefi::SystemTable) -> usize {
    // SAFETY: UEFI supplies the system table at application entry.
    let status =
        unsafe { uefi::print(table, "VibeOS v0.0.1\r\n[ OK ] Bootloader initialized.\r\n") };
    if status & uefi::ERROR != 0 {
        return status;
    }
    let bs = unsafe { (*table).boot_services };
    let mut buffer = uefi::MapBuffer([0; 16384]);
    if let Err(status) = unsafe { uefi::memory_map(bs, &mut buffer) } {
        let _ = unsafe { uefi::print(table, "[FAIL] Memory map acquisition failed.\r\n") };
        return status;
    }
    let status = unsafe { uefi::print(table, "[ OK ] Memory map acquired.\r\n") };
    if status & uefi::ERROR != 0 {
        return status;
    }
    let entry = match unsafe { elf::load(bs, KERNEL) } {
        Ok(entry) => entry,
        Err(status) => {
            let _ = unsafe { uefi::print(table, "[FAIL] Kernel ELF loading failed.\r\n") };
            return status;
        }
    };
    let status = unsafe { uefi::print(table, "[ OK ] Kernel ELF loaded.\r\n") };
    if status & uefi::ERROR != 0 {
        return status;
    }
    let status = unsafe { uefi::print(table, "[ OK ] Exiting boot services.\r\n") };
    if status & uefi::ERROR != 0 {
        return status;
    }

    // Refresh the map immediately before exit; retry once on a stale key.
    for attempt in 0..2 {
        let map = match unsafe { uefi::memory_map(bs, &mut buffer) } {
            Ok(map) => map,
            Err(status) if attempt == 0 => return status,
            Err(_) => halt(),
        };
        let info = vibeos_kernel::BootInfo {
            memory_map: buffer.0.as_ptr(),
            memory_map_size: map.size,
            descriptor_size: map.stride,
            descriptor_version: map.version,
        };
        let status = unsafe { ((*bs).exit_boot_services)(image, map.key) };
        if status == 0 {
            // SAFETY: the validated ELF uses the SysV ABI and firmware identity mapping.
            // The kernel switches stacks and never returns.
            let start: unsafe extern "sysv64" fn(*const vibeos_kernel::BootInfo) -> ! =
                unsafe { core::mem::transmute(entry) };
            unsafe { start(&info) }
        }
        if status != uefi::INVALID_PARAMETER {
            halt();
        }
    }
    halt()
}

fn halt() -> ! {
    loop {
        unsafe { core::arch::asm!("cli; hlt", options(nomem, nostack)) };
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

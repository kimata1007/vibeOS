//! Minimal UEFI ABI definitions for x86-64.
use core::ffi::c_void;

pub type Status = usize;
pub const ERROR: usize = 1 << (usize::BITS - 1);
pub const INVALID_PARAMETER: Status = ERROR | 2;

#[repr(C)]
pub struct TableHeader {
    signature: u64,
    revision: u32,
    header_size: u32,
    crc32: u32,
    reserved: u32,
}

#[repr(C)]
pub struct TextOutput {
    reset: usize,
    output_string: unsafe extern "efiapi" fn(*mut TextOutput, *const u16) -> Status,
}

#[repr(C)]
pub struct SystemTable {
    header: TableHeader,
    firmware_vendor: *const u16,
    firmware_revision: u32,
    console_in_handle: usize,
    console_in: *mut c_void,
    console_out_handle: usize,
    pub console_out: *mut TextOutput,
    stderr_handle: usize,
    stderr: *mut c_void,
    runtime_services: *mut c_void,
    pub boot_services: *mut BootServices,
    number_of_table_entries: usize,
    configuration_table: *mut c_void,
}

/// Print a string through the firmware console.
///
/// # Safety
///
/// Firmware must supply a valid system table and active boot services.
pub unsafe fn print(table: *mut SystemTable, text: &str) -> Status {
    if table.is_null() {
        return INVALID_PARAMETER;
    }
    let out = unsafe { (*table).console_out };
    if out.is_null() {
        return INVALID_PARAMETER;
    }
    let mut buffer = [0u16; 128];
    for (len, unit) in text.encode_utf16().enumerate() {
        if unit == 0 || len == buffer.len() - 1 {
            return INVALID_PARAMETER;
        }
        buffer[len] = unit;
    }
    unsafe { ((*out).output_string)(out, buffer.as_ptr()) }
}

#[repr(C)]
pub struct BootServices {
    header: TableHeader,
    raise_tpl: usize,
    restore_tpl: usize,
    pub allocate_pages: unsafe extern "efiapi" fn(u32, u32, usize, *mut u64) -> Status,
    free_pages: usize,
    get_memory_map:
        unsafe extern "efiapi" fn(*mut usize, *mut u8, *mut usize, *mut usize, *mut u32) -> Status,
}

// An aligned scratch buffer for the memory descriptors.
#[repr(align(8))]
pub struct MapBuffer(pub [u8; 16384]);

/// Retrieve and validate the firmware memory map.
///
/// # Safety
/// The caller must provide valid firmware boot services that are still active.
pub unsafe fn memory_map(bs: *mut BootServices, buffer: &mut MapBuffer) -> Result<(), Status> {
    if bs.is_null() {
        return Err(INVALID_PARAMETER);
    }
    let mut size = buffer.0.len();
    let mut key = 0;
    let mut stride = 0;
    let mut version = 0;
    let status = unsafe {
        ((*bs).get_memory_map)(
            &mut size,
            buffer.0.as_mut_ptr(),
            &mut key,
            &mut stride,
            &mut version,
        )
    };
    if status & ERROR != 0 {
        return Err(status);
    }
    if stride < 40 || size > buffer.0.len() || size == 0 || !size.is_multiple_of(stride) {
        return Err(INVALID_PARAMETER);
    }
    Ok(())
}

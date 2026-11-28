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
    pub boot_services: *mut c_void,
    number_of_table_entries: usize,
    configuration_table: *mut c_void,
}

/// Firmware must supply a valid system table and active boot services.
pub unsafe fn print(table: *mut SystemTable, text: &str) -> Status {
    if table.is_null() { return INVALID_PARAMETER; }
    let out = unsafe { (*table).console_out };
    if out.is_null() { return INVALID_PARAMETER; }
    let mut buffer = [0u16; 128];
    let mut len = 0;
    for unit in text.encode_utf16() {
        if unit == 0 || len == buffer.len() - 1 { return INVALID_PARAMETER; }
        buffer[len] = unit;
        len += 1;
    }
    unsafe { ((*out).output_string)(out, buffer.as_ptr()) }
}

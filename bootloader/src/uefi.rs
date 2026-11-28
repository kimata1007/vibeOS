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

#[repr(C)]
pub struct BootServices {
    header: TableHeader,
    raise_tpl: usize,
    restore_tpl: usize,
    pub allocate_pages: unsafe extern "efiapi" fn(u32, u32, usize, *mut u64) -> Status,
    free_pages: usize,
    get_memory_map: unsafe extern "efiapi" fn(*mut usize, *mut u8, *mut usize, *mut usize, *mut u32) -> Status,
    // Entries from AllocatePool through UnloadImage, in UEFI specification order.
    before_exit: [usize; 21],
    pub exit_boot_services: unsafe extern "efiapi" fn(usize, usize) -> Status,
    before_locate_protocol: [usize; 10],
    locate_protocol: unsafe extern "efiapi" fn(*const Guid, *mut c_void, *mut *mut c_void) -> Status,
}
pub const BUFFER_TOO_SMALL: Status = ERROR | 5;
#[repr(align(8))]
pub struct MapBuffer(pub [u8; 65536]);
pub struct MemoryMap {
    pub size: usize,
    pub key: usize,
    pub stride: usize,
    pub version: u32,
}
/// The caller must provide valid firmware services and an aligned buffer.
pub unsafe fn memory_map(bs: *mut BootServices, buffer: &mut MapBuffer) -> Result<MemoryMap, Status> {
    let mut map = MemoryMap { size: buffer.0.len(), key: 0, stride: 0, version: 0 };
    let status = unsafe { ((*bs).get_memory_map)(&mut map.size, buffer.0.as_mut_ptr(), &mut map.key, &mut map.stride, &mut map.version) };
    if status & ERROR != 0 { return Err(status); }
    if map.stride < 40 || map.size > buffer.0.len() || map.size % map.stride != 0 { return Err(INVALID_PARAMETER); }
    Ok(map)
}

#[repr(C)]
struct Guid { data1: u32, data2: u16, data3: u16, data4: [u8; 8] }
const GOP_GUID: Guid = Guid {
    data1: 0x9042a9de, data2: 0x23dc, data3: 0x4a38,
    data4: [0x96, 0xfb, 0x7a, 0xde, 0xd0, 0x80, 0x51, 0x6a],
};

#[repr(C)]
struct Gop { query_mode: usize, set_mode: usize, blt: usize, mode: *const GopMode }
#[repr(C)]
struct GopMode {
    max_mode: u32, mode: u32, info: *const GopInfo, size_of_info: usize,
    framebuffer_base: u64, framebuffer_size: usize,
}
#[repr(C)]
struct GopInfo {
    version: u32, width: u32, height: u32, format: u32,
    pixel_information: [u32; 4], pixels_per_scan_line: u32,
}

/// Firmware services must be active. Only direct 32-bit RGB/BGR modes are supported.
pub unsafe fn framebuffer(bs: *mut BootServices) -> vibeos_kernel::Framebuffer {
    use vibeos_kernel::Framebuffer;
    let mut interface = core::ptr::null_mut();
    let status = unsafe { ((*bs).locate_protocol)(&GOP_GUID, core::ptr::null_mut(), &mut interface) };
    if status & ERROR != 0 || interface.is_null() { return Framebuffer::EMPTY; }
    let mode = unsafe { (*(interface as *const Gop)).mode };
    if mode.is_null() { return Framebuffer::EMPTY; }
    let mode = unsafe { &*mode };
    if mode.info.is_null() || mode.size_of_info < core::mem::size_of::<GopInfo>() {
        return Framebuffer::EMPTY;
    }
    let info = unsafe { &*mode.info };
    let fb = Framebuffer {
        base: mode.framebuffer_base, size: mode.framebuffer_size,
        width: info.width, height: info.height,
        stride: info.pixels_per_scan_line, format: info.format,
    };
    if fb.valid() { fb } else { Framebuffer::EMPTY }
}

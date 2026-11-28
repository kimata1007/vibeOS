#![no_std]

//! Shared boot handoff ABI. Addresses remain valid while loader memory is reserved.
#[repr(C)]
pub struct BootInfo {
    pub memory_map: *const u8,
    pub memory_map_size: usize,
    pub descriptor_size: usize,
    pub descriptor_version: u32,
}

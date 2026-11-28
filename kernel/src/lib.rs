#![no_std]

//! Shared boot handoff ABI. Addresses remain valid while loader memory is reserved.
#[repr(C)]
pub struct BootInfo {
    pub memory_map: *const u8,
    pub memory_map_size: usize,
    pub descriptor_size: usize,
    pub descriptor_version: u32,
    pub framebuffer: Framebuffer,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Framebuffer {
    pub base: u64,
    pub size: usize,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    // UEFI PixelRedGreenBlueReserved8BitPerColor = 0, BGR = 1.
    pub format: u32,
}

impl Framebuffer {
    pub const EMPTY: Self = Self {
        base: 0,
        size: 0,
        width: 0,
        height: 0,
        stride: 0,
        format: 0,
    };

    pub fn valid(&self) -> bool {
        self.base != 0
            && self.base.is_multiple_of(4)
            && self.width != 0
            && self.height != 0
            && self.width <= self.stride
            && self.format <= 1
            && self.base.checked_add(self.size as u64).is_some()
            && self.size <= isize::MAX as usize
            && (self.stride as usize)
                .checked_mul(self.height as usize)
                .and_then(|n| n.checked_mul(4))
                .is_some_and(|n| n <= self.size)
    }
}

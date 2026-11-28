//! Loader for our own fixed-address, statically linked ELF64 kernel.
use crate::uefi::{BootServices, ERROR, INVALID_PARAMETER, Status};

const PAGE: u64 = 4096;
const BASE: u64 = 0x2000000;
const LIMIT: u64 = BASE + 16 * 1024 * 1024;

fn read<const N: usize>(bytes: &[u8], at: usize) -> Option<[u8; N]> {
    bytes.get(at..at.checked_add(N)?)?.try_into().ok()
}

fn u16at(b: &[u8], n: usize) -> Option<u16> {
    Some(u16::from_le_bytes(read(b, n)?))
}

fn u32at(b: &[u8], n: usize) -> Option<u32> {
    Some(u32::from_le_bytes(read(b, n)?))
}

fn u64at(b: &[u8], n: usize) -> Option<u64> {
    Some(u64::from_le_bytes(read(b, n)?))
}

struct Segment {
    offset: usize,
    address: u64,
    files: usize,
    memory: usize,
    flags: u32,
}

fn segment(b: &[u8], p: usize) -> Option<Option<Segment>> {
    if u32at(b, p)? != 1 {
        return Some(None);
    }
    let s = Segment {
        flags: u32at(b, p + 4)?,
        offset: usize::try_from(u64at(b, p + 8)?).ok()?,
        address: u64at(b, p + 16)?,
        files: usize::try_from(u64at(b, p + 32)?).ok()?,
        memory: usize::try_from(u64at(b, p + 40)?).ok()?,
    };
    if s.files > s.memory
        || s.address < BASE
        || s.address.checked_add(s.memory as u64)? > LIMIT
        || s.address != u64at(b, p + 24)?
    {
        return None;
    }
    b.get(s.offset..s.offset.checked_add(s.files)?)?;
    Some(Some(s))
}

fn header(b: &[u8]) -> Option<(u64, usize, usize)> {
    if b.get(..7)? != b"\x7fELF\x02\x01\x01"
        || u16at(b, 16)? != 2
        || u16at(b, 18)? != 62
        || u32at(b, 20)? != 1
        || u16at(b, 54)? != 56
    {
        return None;
    }
    let entry = u64at(b, 24)?;
    let start = usize::try_from(u64at(b, 32)?).ok()?;
    let count = usize::from(u16at(b, 56)?);
    b.get(start..start.checked_add(count.checked_mul(56)?)?)?;
    Some((entry, start, count))
}

/// Load the trusted kernel built with our linker script.
///
/// # Safety
/// Boot services must be valid and active. The allocated region must be
/// identity mapped and writable, and the kernel image must not overlap it.
pub unsafe fn load(bs: *mut BootServices, b: &[u8]) -> Result<usize, Status> {
    if bs.is_null() {
        return Err(INVALID_PARAMETER);
    }
    let (entry, start, count) = header(b).ok_or(INVALID_PARAMETER)?;
    let mut end = BASE;
    let mut executable = false;
    for i in 0..count {
        if let Some(s) = segment(b, start + i * 56).ok_or(INVALID_PARAMETER)? {
            let top = s.address + s.memory as u64;
            end = end.max(top);
            executable |=
                s.flags & 1 != 0 && entry >= s.address && entry < s.address + s.files as u64;
        }
    }
    if !executable || end == BASE {
        return Err(INVALID_PARAMETER);
    }
    let pages = (end - BASE).div_ceil(PAGE) as usize;
    let mut address = BASE;
    // AllocateAddress=2, EfiLoaderCode=1.
    let status = unsafe { ((*bs).allocate_pages)(2, 1, pages, &mut address) };
    if status & ERROR != 0 {
        return Err(status);
    }
    if address != BASE {
        return Err(INVALID_PARAMETER);
    }
    unsafe {
        core::ptr::write_bytes(address as *mut u8, 0, pages * PAGE as usize);
    }
    for i in 0..count {
        if let Some(s) = segment(b, start + i * 56).ok_or(INVALID_PARAMETER)? {
            unsafe {
                core::ptr::copy_nonoverlapping(
                    b.as_ptr().add(s.offset),
                    s.address as *mut u8,
                    s.files,
                );
            }
        }
    }
    Ok(entry as usize)
}

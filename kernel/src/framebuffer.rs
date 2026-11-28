//! Direct framebuffer rendering: no UEFI calls.
use vibeos_kernel::Framebuffer;

struct Screen(Framebuffer);
impl Screen {
    fn pixel(&self, x: u32, y: u32, rgb: [u8; 3]) {
        let fb = &self.0;
        if x >= fb.width || y >= fb.height { return; }
        let [r,g,b] = rgb;
        let bytes = if fb.format == 0 { [r,g,b,0] } else { [b,g,r,0] };
        let offset = (y as usize * fb.stride as usize + x as usize) * 4;
        // SAFETY: caller validated framebuffer bounds and identity-mapped writable VRAM.
        unsafe { core::ptr::write_volatile((fb.base as usize + offset) as *mut u32, u32::from_le_bytes(bytes)); }
    }
    fn fill(&self, color: [u8; 3]) {
        for y in 0..self.0.height {
            for x in 0..self.0.width { self.pixel(x,y,color); }
        }
    }
    fn text(&self, mut x: u32, y: u32, message: &[u8], color: [u8; 3], scale: u32) {
        for &ch in message {
            for (row, bits) in glyph(ch).iter().enumerate() {
                for col in 0..5 {
                    if bits & (1 << (4-col)) == 0 { continue; }
                    for dy in 0..scale {
                        for dx in 0..scale {
                            self.pixel(x+col*scale+dx,y+row as u32*scale+dy,color);
                        }
                    }
                }
            }
            x += 6*scale;
        }
    }
}
fn glyph(ch: u8) -> [u8; 7] {
    match ch {
        b'V' => [17,17,17,17,17,10,4],
        b'I' => [31,4,4,4,4,4,31],
        b'B' => [30,17,17,30,17,17,30],
        b'E' => [31,16,16,30,16,16,31],
        b'O' => [14,17,17,17,17,17,14],
        b'S' => [15,16,16,14,1,1,30],
        b'K' => [17,18,20,24,20,18,17],
        b'R' => [30,17,17,30,20,18,17],
        b'N' => [17,25,25,21,19,19,17],
        b'L' => [16,16,16,16,16,16,31],
        _ => [0;7],
    }
}

/// # Safety
/// The framebuffer must be writable, identity-mapped, and owned exclusively by the kernel.
pub unsafe fn render(fb: Framebuffer) -> bool {
    if !fb.valid() { return false; }
    let screen = Screen(fb);
    screen.fill([12,19,27]);
    screen.text(24,24,b"VIBEOS",[230,242,255],4);
    screen.text(24,68,b"KERNEL OK",[95,220,165],2);
    true
}

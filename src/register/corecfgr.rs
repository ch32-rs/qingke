//! corecfgr, core configuration register
use core::arch::asm;

#[inline]
pub fn read() -> usize {
    let ans: usize;
    unsafe { asm!("csrr {}, 0xBC0", out(reg) ans) };
    ans
}

/// Write to corecfgr
#[inline]
pub unsafe fn write(bits: usize) {
    asm!("csrs 0xBC0, {}", in(reg) bits);
}

/// Write 0x1f to ??? (in EVT code)
#[inline]
pub unsafe fn set_default() {
    write(0x1f)
}

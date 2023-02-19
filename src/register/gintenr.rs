//! gintenr, global interrupt enable register

use core::arch::asm;

#[inline]
pub fn read() -> usize {
    let ans: usize;
    unsafe { asm!("csrr {}, 0x800", out(reg) ans) };
    ans
}

#[inline]
pub unsafe fn write(bits: usize) {
    asm!("csrs 0x800, {}", in(reg) bits);
}

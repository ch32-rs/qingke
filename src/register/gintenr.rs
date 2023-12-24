//! gintenr, global interrupt enable register
//!
//! This is a mapping to mie & mpie
//!
//! 全局中断使能寄存器
//!
//! Write 0x08 to enable global interrupt

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

#[inline]
pub unsafe fn set_enable() {
    write(0x08)
}

#[inline]
pub unsafe fn set_disable() {
    write(0x00)
}

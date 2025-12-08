//! gintenr, global interrupt enable register (CSR 0x800)
//!
//! This is a mapping to mie & mpie
//!
//! 全局中断使能寄存器
//!
//! Write 0x08 to enable global interrupt
//!
//! NOTE: This register is NOT available on qingke_v2 (CH32V003).
//! Use mstatus.MIE instead for qingke_v2.

#[cfg(not(qingke_v2))]
use core::arch::asm;

#[cfg(not(qingke_v2))]
#[inline]
pub fn read() -> usize {
    let ans: usize;
    unsafe { asm!("csrr {}, 0x800", out(reg) ans) };
    ans
}

#[cfg(not(qingke_v2))]
#[inline]
pub unsafe fn write(bits: usize) {
    asm!("csrw 0x800, {}", in(reg) bits);
}

#[cfg(not(qingke_v2))]
#[inline]
pub unsafe fn set_enable() {
    let mask = 0x8;
    asm!("csrs 0x800, {}", in(reg) mask);
}

#[cfg(not(qingke_v2))]
#[inline]
/// Disable interrupt and return the old `GINTENR` value
pub fn set_disable() -> usize {
    let prev: usize;
    let mask = 0x8usize;
    unsafe { asm!("csrrc {}, 0x800, {}", out(reg) prev, in(reg) mask) };
    prev
}

//! gintenr, global interrupt enable register (CSR 0x800)
//!
//! This is a mapping to mie & mpie
//!
//! 全局中断使能寄存器
//!
//! Write 0x08 to enable global interrupt
//!
//! NOTE: This register is NOT available on v2 (CH32V003).
//! Use mstatus.MIE instead for v2.

#[cfg(csr_gintenr)]
use core::arch::asm;

#[cfg(csr_gintenr)]
#[inline]
pub fn read() -> usize {
    let ans: usize;
    unsafe { asm!("csrr {}, 0x800", out(reg) ans) };
    ans
}

#[cfg(csr_gintenr)]
#[inline]
pub unsafe fn write(bits: usize) {
    unsafe { asm!("csrw 0x800, {}", in(reg) bits) };
}

#[cfg(csr_gintenr)]
#[inline]
pub unsafe fn set_enable() {
    let mask = 0x8;
    unsafe { asm!("csrs 0x800, {}", in(reg) mask) };
}

#[cfg(csr_gintenr)]
#[inline]
/// Disable interrupt and return the old `GINTENR` value
pub fn set_disable() -> usize {
    let prev: usize;
    let mask = 0x8usize;
    unsafe { asm!("csrrc {}, 0x800, {}", out(reg) prev, in(reg) mask) };
    prev
}

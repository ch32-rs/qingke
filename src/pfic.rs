//! PFIC, Programmable Fast Interrupt Controller
#![allow(unused)]

use core::ptr;

/// Interrupt Status Register
const PFIC_ISR0: *mut u32 = 0xE000E000 as *mut u32;
/// Interrupt Pending Register
const PFIC_IPR0: *mut u32 = 0xE000E020 as *mut u32;

/// Interrupt priority threshold configure register
/// 中断优先级阈值设置
const PFIC_ITHRESDR: *mut u32 = 0xE000E040 as *mut u32;
/// Interrupt configure register
/// 中断配置寄存器
const PFIC_CFGR: *mut u32 = 0xE000E048 as *mut u32;
/// Interrupt global status register
/// 中断全局状态寄存器
const PFIC_GISR: *mut u32 = 0xE000E04C as *mut u32;

/// VTF ID configure register
/// 免表中断 ID, 8-bit for each entry, max 4 entries
const PFIC_VTFIDR: *mut u8 = 0xE000E050 as *mut u8;
/// VTF interrupt x offset address register
/// 免表中断地址寄存器
const PFIC_VTFADDRR0: *mut u32 = 0xE000E060 as *mut u32;
const PFIC_VTFADDRR1: *mut u32 = 0xE000E064 as *mut u32;
const PFIC_VTFADDRR2: *mut u32 = 0xE000E068 as *mut u32;
const PFIC_VTFADDRR3: *mut u32 = 0xE000E06C as *mut u32;

/// Interrupt Enable Register
const PFIC_IENR0: *mut u32 = 0xE000E100 as *mut u32;
/// Interrupt reset enable register
const PFIC_IRER0: *mut u32 = 0xE000E180 as *mut u32;
/// Interrupt pending set register
const PFIC_IPSR0: *mut u32 = 0xE000E200 as *mut u32;
/// Interrupt pending reset register
const PFIC_IPRR0: *mut u32 = 0xE000E280 as *mut u32;
/// Interrupt active register
const PFIC_IACTR0: *mut u32 = 0xE000E300 as *mut u32;
/// Interrupt priority configure register, 8-bit for each interrupt
const PFIC_IPRIOR0: *mut u8 = 0xE000E400 as *mut u8;

/// System control register
/// 系统控制寄存器
const PFIC_SCTLR: *mut u32 = 0xE000ED10 as *mut u32;

#[inline]
pub unsafe fn enable_interrupt(irq: u8) {
    let offset = (irq / 32) as isize;
    let bit = irq % 32;
    unsafe {
        ptr::write_volatile(PFIC_IENR0.offset(offset), 1 << bit);
    }
}

#[inline]
pub unsafe fn disable_interrupt(irq: u8) {
    let offset = (irq / 32) as isize;
    let bit = irq % 32;
    unsafe {
        ptr::write_volatile(PFIC_IRER0.offset(offset), 1 << bit);
    }
}

#[inline]
pub unsafe fn pend_interrupt(irq: u8) -> bool {
    let offset = (irq / 32) as isize;
    let bit = irq % 32;
    unsafe { ptr::read_volatile(PFIC_IPSR0.offset(offset)) & (1 << bit) != 0 }
}

#[inline]
pub unsafe fn unpend_interrupt(irq: u8) {
    let offset = (irq / 32) as isize;
    let bit = irq % 32;
    unsafe {
        ptr::write_volatile(PFIC_IPRR0.offset(offset), 1 << bit);
    }
}

#[inline]
pub fn is_active(irq: u8) -> bool {
    let offset = (irq / 32) as isize;
    let bit = irq % 32;
    unsafe { ptr::read_volatile(PFIC_IACTR0.offset(offset)) & (1 << bit) != 0 }
}

#[inline]
pub unsafe fn set_priority(irq: u8, priority: u8) {
    let offset = irq as isize;
    unsafe {
        ptr::write_volatile(PFIC_IPRIOR0.offset(offset), priority);
    }
}

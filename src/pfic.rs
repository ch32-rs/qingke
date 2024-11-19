//! PFIC, Programmable Fast Interrupt Controller
//!
//! V3 core seems older, so it has different VTF configuration
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

// /// VTF ID configure register
// /// 免表中断 ID, 8-bit for each entry, max 4 entries
// const PFIC_VTFIDR: *mut u32 = 0xE000E050 as *mut u32;

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
pub fn is_enabled(irq: u8) -> bool {
    let offset = (irq / 32) as isize;
    let bit = irq % 32;
    unsafe { ptr::read_volatile(PFIC_ISR0.offset(offset)) & (1 << bit) != 0 }
}

#[inline]
pub fn is_disabled(irq: u8) -> bool {
    !is_enabled(irq)
}

#[inline]
pub fn is_pending(irq: u8) -> bool {
    let offset = (irq / 32) as isize;
    let bit = irq % 32;
    unsafe { ptr::read_volatile(PFIC_IPR0.offset(offset)) & (1 << bit) != 0 }
}

#[inline]
pub unsafe fn pend_interrupt(irq: u8) {
    let offset = (irq / 32) as isize;
    let bit = irq % 32;
    unsafe { ptr::write_volatile(PFIC_IPSR0.offset(offset), 1 << bit) }
}

#[inline]
pub unsafe fn unpend_interrupt(irq: u8) {
    let offset = (irq / 32) as isize;
    let bit = irq % 32;
    unsafe { ptr::write_volatile(PFIC_IPRR0.offset(offset), 1 << bit) }
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

#[inline]
pub fn get_priority(irq: u8) -> u8 {
    let offset = irq as isize;
    unsafe { ptr::read_volatile(PFIC_IPRIOR0.offset(offset)) }
}

/// Enable VTF0, VTFBADDRR will be overwritten
#[cfg(feature = "v3")]
pub unsafe fn enable_vtf(channel: u8, irq: u8, address: u32) {
    assert!(channel < 4, "VTF channel must be less than 4");
    const PFIC_VTFBADDRR: *mut u32 = 0xE000E044 as *mut u32;

    unsafe {
        ptr::write_volatile(PFIC_VTFBADDRR, address & 0xF000_0000);

        ptr::write_volatile(
            PFIC_VTFADDRR0.offset(channel as isize),
            ((irq as u32) << 24) | (address & 0x00FF_FFFF),
        );
    }
}

#[cfg(feature = "v3")]
pub unsafe fn disable_vtf(channel: u8) {
    assert!(channel < 4, "VTF channel must be less than 4");
    let val = ptr::read_volatile(PFIC_VTFADDRR0.offset(channel as isize));
    ptr::write_volatile(PFIC_VTFADDRR0.offset(channel as isize), val & 0x00FF_FFFF);
}

#[cfg(not(feature = "v3"))]
pub unsafe fn enable_vtf(channel: u8, irq: u8, address: u32) {
    assert!(channel < 4, "VTF channel must be less than 4");

    // [31:24]: Numbering of VTF interrupt 3
    // [23:16]: Numbering of VTF interrupt 2
    // [15:8]: Numbering of VTF interrupt 1
    // [7:0]: Numbering of VTF interrupt 0
    const PFIC_VTFIDR: *mut u32 = 0xE000E050 as *mut u32;

    let irq_bits = (irq as u32) << ((channel as u32) * 8);
    let irq_mask = 0xFF << ((channel as u32) * 8);

    let irq = ptr::read_volatile(PFIC_VTFIDR);
    ptr::write_volatile(PFIC_VTFIDR, irq & !irq_mask | irq_bits);

    ptr::write_volatile(
        PFIC_VTFADDRR0.offset(channel as isize),
        address | 0x0000_0001,
    );
}

#[cfg(not(feature = "v3"))]
pub unsafe fn disable_vtf(channel: u8) {
    assert!(channel < 4, "VTF channel must be less than 4");
    let val = ptr::read_volatile(PFIC_VTFADDRR0.offset(channel as isize));
    ptr::write_volatile(PFIC_VTFADDRR0.offset(channel as isize), val & 0xFFFF_FFFE);
}

#[cfg(feature = "critical-section-impl")]
pub unsafe fn wfi_to_wfe(v: bool) {
    critical_section::with(|_| {
        let mut val = ptr::read_volatile(PFIC_SCTLR);
        // 0x8 is WFITOWFE bit
        if v {
            val |= 0x8;
        } else {
            val &= !0x8;
        }
        ptr::write_volatile(PFIC_SCTLR, val);
    });
}

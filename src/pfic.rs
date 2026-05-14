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

/// Wake-up instruction pointer register for hart 0 (C0).
/// 内核 C0 唤醒指令指针寄存器.
///
/// Chip-specific multi-core extension — currently CH32H417 only.
/// Not documented in any generic QingKe IP manual (V2/V3/V4/V5);
/// see CH32H417 RM V1.7 §4.7.5.51.
#[cfg(feature = "dual-core")]
const PFIC_WAKEIP0: *mut u32 = 0xE000E720 as *mut u32;
/// Wake-up instruction pointer register for hart 1 (C1).
/// 内核 C1 唤醒指令指针寄存器.
/// See [`PFIC_WAKEIP0`] for caveats.
#[cfg(feature = "dual-core")]
const PFIC_WAKEIP1: *mut u32 = 0xE000E724 as *mut u32;

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
#[cfg(feature = "_v3")]
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

#[cfg(feature = "_v3")]
pub unsafe fn disable_vtf(channel: u8) {
    assert!(channel < 4, "VTF channel must be less than 4");
    let val = ptr::read_volatile(PFIC_VTFADDRR0.offset(channel as isize));
    ptr::write_volatile(PFIC_VTFADDRR0.offset(channel as isize), val & 0x00FF_FFFF);
}

#[cfg(not(feature = "_v3"))]
pub unsafe fn enable_vtf(channel: u8, irq: u8, address: u32) {
    assert!(channel < 4, "VTF channel must be less than 4");

    // [31:24]: Numbering of VTF interrupt 3
    // [23:16]: Numbering of VTF interrupt 2
    // [15:8]: Numbering of VTF interrupt 1
    // [7:0]: Numbering of VTF interrupt 0
    const PFIC_VTFIDR: *mut u32 = 0xE000E050 as *mut u32;

    unsafe {
        ptr::write_volatile(PFIC_VTFIDR, (irq as u32) << ((channel as u32) * 8));

        ptr::write_volatile(
            PFIC_VTFADDRR0.offset(channel as isize),
            address | 0x0000_0001,
        );
    }
}

#[cfg(not(feature = "_v3"))]
pub unsafe fn disable_vtf(channel: u8) {
    assert!(channel < 4, "VTF channel must be less than 4");
    unsafe {
        let val = ptr::read_volatile(PFIC_VTFADDRR0.offset(channel as isize));
        ptr::write_volatile(PFIC_VTFADDRR0.offset(channel as isize), val & 0xFFFF_FFFE);
    }
}

#[cfg(feature = "critical-section-impl")]
pub unsafe fn wfi_to_wfe(v: bool) {
    critical_section::with(|_| unsafe {
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

/// Identifies the current QingKe hart in a multi-core PFIC.
///
/// Read from `PFIC_SCTLR[23:16]` (`HART_ID` field). Only the LSB is
/// meaningful — the upper bits of that field are reserved.
///
/// `HART_ID` is documented in the QingKe V5 IP manual §8.1 and the
/// CH32H417 RM §4.7.5.58, but is **not** defined in the QingKe
/// V2 / V3 / V4 manuals, so this type is gated behind the
/// `dual-core` feature. On CH32H417 the primary boot core (V3F) is
/// `C0` and the secondary (V5F) is `C1`.
#[cfg(feature = "dual-core")]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum HartId {
    /// Core 0 (V3F on CH32H417)
    C0 = 0,
    /// Core 1 (V5F on CH32H417)
    C1 = 1,
}

#[cfg(feature = "dual-core")]
impl HartId {
    /// Read the current hart ID from `PFIC_SCTLR`.
    #[inline]
    pub fn current() -> Self {
        let sctlr = unsafe { ptr::read_volatile(PFIC_SCTLR) };
        if (sctlr & (1 << 16)) != 0 {
            HartId::C1
        } else {
            HartId::C0
        }
    }

    /// Return the *other* hart's ID.
    #[inline]
    pub const fn other(self) -> Self {
        match self {
            HartId::C0 => HartId::C1,
            HartId::C1 => HartId::C0,
        }
    }

    /// 0 or 1, suitable for indexing per-hart resources.
    #[inline]
    pub const fn to_index(self) -> usize {
        self as usize
    }
}

/// Wake the *other* hart from its deep-sleep lock and set its entry PC
/// to `entry`.
///
/// Rust equivalent of WCH SDK's `NVIC_WakeUp_V3F` / `NVIC_WakeUp_V5F`.
/// It writes `entry` into the other hart's `PFIC_WAKEIPx` register —
/// because `entry` is required to be 1KB-aligned, the write also
/// clears the `SHUTDOWN_x` bit (bit 0), releasing the hart from its
/// deep-sleep lock — then sets `PFIC_SCTLR.SENDEVENT` (bit 5) to
/// deliver the wake event.
///
/// Gated behind `dual-core` because `WAKEIPx` are not documented in
/// any generic QingKe IP manual — they are a chip-level multi-core
/// extension currently known only on CH32H417.
///
/// # Safety
/// - `entry` must be 1KB-aligned (bottom 10 bits zero); debug builds
///   panic otherwise.
/// - The image at `entry` must already be programmed in Flash and be
///   reachable from the other hart's address space.
/// - Typically called once by the primary hart (`C0`) during boot,
///   before any cross-core synchronization protocol begins.
#[cfg(feature = "dual-core")]
#[inline]
pub unsafe fn wake_other_core(entry: u32) {
    debug_assert!(entry & 0x3FF == 0, "entry must be 1KB-aligned");
    let wakeip = match HartId::current().other() {
        HartId::C0 => PFIC_WAKEIP0,
        HartId::C1 => PFIC_WAKEIP1,
    };
    unsafe {
        ptr::write_volatile(wakeip, entry);
        let sctlr = ptr::read_volatile(PFIC_SCTLR);
        ptr::write_volatile(PFIC_SCTLR, sctlr | (1 << 5));
    }
}

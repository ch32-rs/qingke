//! inestcr, interrupt nest control register.
//!
//! QingKe-specific CSR at address 0xBC1. Controls the maximum
//! interrupt nest depth and exposes nest status / overflow flags.
//!
//! Documented in QingKe V5 IP manual §8.1 (and used by V3F/V5F on
//! CH32H417). V3 and earlier manuals don't describe this register
//! in detail, but V3F's startup writes it the same way.

use bit_field::BitField;
use core::arch::asm;

/// Maximum number of interrupt nesting levels.
///
/// Programmed into the low 3 bits of inestcr. On QingKe V3, only
/// levels up to 2 are supported (`NestLevel::Disabled` / `Two`);
/// values 3–8 are V5-only.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum NestLevel {
    /// No interrupt nesting (one in-flight interrupt at a time).
    Disabled = 0b000,
    Two = 0b001,
    Three = 0b010,
    Four = 0b011,
    Five = 0b100,
    Six = 0b101,
    Seven = 0b110,
    /// Eight levels — the maximum, supported on QingKe V5.
    Eight = 0b111,
}

impl NestLevel {
    /// Decode the `NEST_STA` (status) field — a unary-style encoding
    /// where each bit represents a "currently active at level N" flag.
    /// Returns the count of consecutive low bits set, i.e. the
    /// current nesting depth.
    #[inline]
    pub const fn from_status(bits: u8) -> u8 {
        // Pattern: 0x00 → 0, 0x01 → 1, 0x03 → 2, 0x07 → 3, ...
        // Equivalent to "count of trailing ones" plus 0.
        bits.trailing_ones() as u8
    }
}

/// inestcr register (CSR 0xBC1)
#[derive(Clone, Copy, Debug)]
pub struct Inestcr {
    bits: usize,
}

impl Inestcr {
    /// LSU NMI status flag. Set when the load/store unit raises a
    /// non-maskable interrupt (typically a write-bus error). Write 1
    /// to clear via [`clear_lsu_nmi`].
    #[inline]
    pub fn lsu_nmi_status(&self) -> bool {
        self.bits.get_bit(31)
    }

    /// Nest overflow flag. Set when a level-2 ISR raises an
    /// instruction exception or NMI causing the hardware stack to
    /// overflow. Write 1 to clear via [`clear_nest_overflow`].
    #[inline]
    pub fn nest_overflow(&self) -> bool {
        self.bits.get_bit(30)
    }

    /// Current nesting depth (0 .. 8). Encoded in `NEST_STA`
    /// (bits 11:8) as trailing-ones.
    #[inline]
    pub fn nest_depth(&self) -> u8 {
        NestLevel::from_status(self.bits.get_bits(8..=11) as u8)
    }

    /// Raw `NEST_STA` bits, unary-encoded.
    #[inline]
    pub fn nest_status_raw(&self) -> u8 {
        self.bits.get_bits(8..=11) as u8
    }

    /// Configured maximum nest level.
    #[inline]
    pub fn nest_level(&self) -> NestLevel {
        match self.bits.get_bits(0..=2) as u8 {
            0b000 => NestLevel::Disabled,
            0b001 => NestLevel::Two,
            0b010 => NestLevel::Three,
            0b011 => NestLevel::Four,
            0b100 => NestLevel::Five,
            0b101 => NestLevel::Six,
            0b110 => NestLevel::Seven,
            _ => NestLevel::Eight,
        }
    }

    /// Raw register bits.
    #[inline]
    pub fn bits(&self) -> usize {
        self.bits
    }
}

read_csr_as!(Inestcr, 0xBC1);

/// Overwrite the entire register.
///
/// Common values:
/// - `0x01` on QingKe V3F (2-level nesting)
/// - `0x07` on QingKe V5F (8-level nesting)
///
/// # Safety
/// Caller must ensure the value matches the hardware capabilities of
/// the running core. Writing reserved bits is allowed but may have
/// undefined effects on future silicon revisions.
#[inline]
pub unsafe fn write(bits: usize) {
    unsafe { asm!("csrw 0xBC1, {0}", in(reg) bits) }
}

/// Program `NEST_LVL` (max nesting depth) while preserving other
/// bits.
///
/// # Safety
/// See [`write`].
#[inline]
pub unsafe fn set_max_level(level: NestLevel) {
    let mut v = read().bits;
    v.set_bits(0..=2, level as usize);
    unsafe { write(v) };
}

/// Write 1 to `NEST_OV` to clear the overflow latch.
///
/// # Safety
/// Only meaningful while the overflow has been observed and the
/// software is recovering. The other W1C bit (`LSU_NMI_STA`) is left
/// untouched.
#[inline]
pub unsafe fn clear_nest_overflow() {
    unsafe { asm!("csrw 0xBC1, {0}", in(reg) 1usize << 30) }
}

/// Write 1 to `LSU_NMI_STA` to clear the LSU-NMI latch.
///
/// # Safety
/// See [`clear_nest_overflow`].
#[inline]
pub unsafe fn clear_lsu_nmi() {
    unsafe { asm!("csrw 0xBC1, {0}", in(reg) 1usize << 31) }
}

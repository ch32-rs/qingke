//! cache_pmp_ovr, ICache strategy PMP override register.
//!
//! QingKe V5-specific CSR at address 0xBC3. Provides a per-PMP-channel
//! override of the global cacheability policy in
//! [`cache_strtg_ctlr`](super::cache_strtg_ctlr): when an instruction
//! fetch falls under a PMP channel, the matching bit here decides
//! cacheability instead of the address-space policy in 0xBC2.
//!
//! **Availability**: only on cores with ICache hardware, gated
//! behind the `_v5` feature.
//!
//! Field layout follows QingKe V5 IP manual §8.1.

use bit_field::BitField;
use core::arch::asm;

/// PMP channel number used by [`CachePmpOvr`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum PmpChannel {
    Pmp0 = 0,
    Pmp1 = 4,
    Pmp2 = 8,
    Pmp3 = 12,
}

/// cache_pmp_ovr register view (CSR 0xBC3).
#[derive(Clone, Copy, Debug)]
pub struct CachePmpOvr {
    bits: usize,
}

impl CachePmpOvr {
    /// Whether instructions falling under the given PMP channel are
    /// cacheable (`true`) or non-cacheable (`false`).
    #[inline]
    pub fn pmp_cacheable(&self, channel: PmpChannel) -> bool {
        self.bits.get_bit(channel as usize)
    }

    /// Raw register bits.
    #[inline]
    pub fn bits(&self) -> usize {
        self.bits
    }
}

read_csr_as!(CachePmpOvr, 0xBC3);

/// Overwrite the entire register.
///
/// # Safety
/// Reserved bits should be kept at their reset values to remain
/// forward-compatible.
#[inline]
pub unsafe fn write(bits: usize) {
    unsafe { asm!("csrw 0xBC3, {0}", in(reg) bits) }
}

/// Set the listed bits via `csrs` (atomic set).
///
/// # Safety
/// See [`write`].
#[inline]
pub unsafe fn set_bits(mask: usize) {
    unsafe { asm!("csrs 0xBC3, {0}", in(reg) mask) }
}

/// Clear the listed bits via `csrc` (atomic clear).
///
/// # Safety
/// See [`write`].
#[inline]
pub unsafe fn clear_bits(mask: usize) {
    unsafe { asm!("csrc 0xBC3, {0}", in(reg) mask) }
}

/// Mark instructions covered by `channel` as cacheable.
///
/// # Safety
/// See [`write`].
#[inline]
pub unsafe fn enable_channel(channel: PmpChannel) {
    unsafe { set_bits(1usize << channel as usize) }
}

/// Mark instructions covered by `channel` as non-cacheable.
///
/// # Safety
/// See [`write`].
#[inline]
pub unsafe fn disable_channel(channel: PmpChannel) {
    unsafe { clear_bits(1usize << channel as usize) }
}

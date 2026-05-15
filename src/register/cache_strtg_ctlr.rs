//! cache_strtg_ctlr, instruction cache strategy control register.
//!
//! QingKe V5-specific CSR at address 0xBC2. Controls whether the
//! instruction cache (ICache) is enabled at all, plus per-region
//! cacheability for four standard address spaces.
//!
//! **Availability**: only on cores with ICache hardware, gated
//! behind the `_v5` feature. V2 / V3 / V4 do not have this CSR.
//!
//! Field layout follows QingKe V5 IP manual §8.1.

use bit_field::BitField;
use core::arch::asm;

/// Code region affected by a per-region cacheability bit in
/// `cache_strtg_ctlr`. Address ranges are RISC-V 32-bit defaults.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum CacheRegion {
    /// 0x00000000 - 0x1FFFFFFF (code Flash on CH32H417)
    Code = 24,
    /// 0x20000000 - 0x3FFFFFFF (SRAM, includes ITCM/DTCM on CH32H417)
    Sram = 25,
    /// 0x60000000 - 0x7FFFFFFF (external memory bank 0, FSMC)
    Mem0 = 26,
    /// 0x80000000 - 0x9FFFFFFF (external memory bank 1)
    Mem1 = 27,
}

/// cache_strtg_ctlr register view (CSR 0xBC2).
#[derive(Clone, Copy, Debug)]
pub struct CacheStrtgCtlr {
    bits: usize,
}

impl CacheStrtgCtlr {
    /// Whether the given address-space region is currently allowed
    /// to be cached.
    #[inline]
    pub fn region_cacheable(&self, region: CacheRegion) -> bool {
        self.bits.get_bit(region as usize)
    }

    /// `ic_disable` (bit 1). When `true`, the entire ICache is
    /// disabled regardless of per-region settings. Reset value is
    /// `true` — software must clear this bit to start using ICache.
    #[inline]
    pub fn icache_disabled(&self) -> bool {
        self.bits.get_bit(1)
    }

    /// Raw register bits.
    #[inline]
    pub fn bits(&self) -> usize {
        self.bits
    }
}

read_csr_as!(CacheStrtgCtlr, 0xBC2);

/// Overwrite the entire register.
///
/// # Safety
/// Caller must ensure the value is valid; reserved bits should be
/// kept at their reset values to remain forward-compatible.
#[inline]
pub unsafe fn write(bits: usize) {
    unsafe { asm!("csrw 0xBC2, {0}", in(reg) bits) }
}

/// Clear the listed bits via `csrc` (atomic clear).
///
/// # Safety
/// See [`write`].
#[inline]
pub unsafe fn clear_bits(mask: usize) {
    unsafe { asm!("csrc 0xBC2, {0}", in(reg) mask) }
}

/// Set the listed bits via `csrs` (atomic set).
///
/// # Safety
/// See [`write`].
#[inline]
pub unsafe fn set_bits(mask: usize) {
    unsafe { asm!("csrs 0xBC2, {0}", in(reg) mask) }
}

/// Enable the ICache globally by clearing `ic_disable` (bit 1).
/// Per-region bits remain unchanged.
///
/// # Safety
/// Caller must have set up the PMP and cache_pmp_ovr appropriately
/// for any region they care about, and have invalidated the cache
/// at least once before relying on its contents.
#[inline]
pub unsafe fn enable_icache() {
    unsafe { clear_bits(1 << 1) }
}

/// Disable the ICache globally by setting `ic_disable` (bit 1).
///
/// # Safety
/// See [`enable_icache`].
#[inline]
pub unsafe fn disable_icache() {
    unsafe { set_bits(1 << 1) }
}

/// Enable caching for `region` while leaving other regions and the
/// global `ic_disable` flag unchanged.
///
/// # Safety
/// See [`write`].
#[inline]
pub unsafe fn enable_region(region: CacheRegion) {
    unsafe { set_bits(1usize << region as usize) }
}

/// Disable caching for `region`.
///
/// # Safety
/// See [`write`].
#[inline]
pub unsafe fn disable_region(region: CacheRegion) {
    unsafe { clear_bits(1usize << region as usize) }
}

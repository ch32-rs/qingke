//! opcache_ctlr, instruction cache operation control register.
//!
//! QingKe V5-specific write-only CSR at address 0xBD0. Each write
//! performs a single ICache maintenance operation (currently only
//! "invalidate"), selected by the `Opcode` field and parametrized
//! by either a virtual address or a way/set index.
//!
//! **Availability**: only on cores with ICache hardware, gated
//! behind the `_v5` feature.
//!
//! Field layout follows QingKe V5 IP manual §8.1.

use core::arch::asm;

/// Operation requested by a write to `opcache_ctlr`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(usize)]
pub enum Opcode {
    /// Invalidate one or more ICache lines, parametrized by
    /// `vaddr` and the index-mode flag.
    Invalidate = 0b00,
}

/// Whether `vaddr` is interpreted as a virtual address or a
/// way/set index.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AddressMode {
    /// `vaddr` is a virtual address; the cache line containing it
    /// is selected.
    Address,
    /// `vaddr` encodes a way/set index pair; the entries are
    /// selected by index.
    Index,
}

impl AddressMode {
    #[inline]
    const fn bit(self) -> usize {
        match self {
            AddressMode::Address => 0,
            AddressMode::Index => 1 << 2,
        }
    }
}

/// Issue an ICache maintenance operation.
///
/// The bottom 5 bits of `vaddr` are ignored by the hardware (line
/// alignment); higher bits select the line in `Address` mode or the
/// way/set in `Index` mode.
///
/// # Safety
/// - Invalidating cache lines that the current core is about to
///   execute may cause undefined behavior; typically called once
///   during ICache initialization before enabling the cache, or
///   after explicit instruction-memory rewrites.
/// - Must only be called on cores that have ICache hardware.
#[inline]
pub unsafe fn issue(vaddr: usize, mode: AddressMode, op: Opcode) {
    let bits = (vaddr & !0x1F) | mode.bit() | op as usize;
    unsafe { asm!("csrw 0xBD0, {0}", in(reg) bits) }
}

/// Invalidate the entire ICache by issuing an index-mode
/// invalidate sweep (the value `0x4` matches the CH32H417 SDK
/// startup pattern).
///
/// # Safety
/// See [`issue`].
#[inline]
pub unsafe fn invalidate_all() {
    unsafe { issue(0, AddressMode::Index, Opcode::Invalidate) }
}

/// Invalidate the ICache line containing the given virtual address.
///
/// # Safety
/// See [`issue`].
#[inline]
pub unsafe fn invalidate_addr(vaddr: usize) {
    unsafe { issue(vaddr, AddressMode::Address, Opcode::Invalidate) }
}

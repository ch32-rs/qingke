//! `critical-section` implementation for QingKe cores.
//!
//! Two variants, selected at compile time:
//!
//! - the default, `single_hart`, masks the calling hart's interrupts. That is
//!   all a single-core chip needs.
//! - `dual_core`, enabled by the `dual-core-critical-section` feature,
//!   additionally takes a spinlock that both harts share. It is meant for the
//!   single-image CH32H417 ("homogeneous dual core") model.

#[cfg(all(feature = "dual-core-critical-section", cs_mstatus))]
compile_error!("dual-core-critical-section requires a gintenr-capable core (QingKe V3/V4/V5)");

#[cfg(not(feature = "dual-core-critical-section"))]
critical_section::set_impl!(single_hart::Section);
#[cfg(feature = "dual-core-critical-section")]
critical_section::set_impl!(dual_core::Section);

/// Masks `MIE` and `MPIE` on the calling hart and returns whether `MIE` was
/// set. Shared by both implementations.
#[cfg(not(cs_mstatus))]
#[inline(always)]
pub(crate) unsafe fn mask_interrupts() -> bool {
    // critical_section: mask MIE+MPIE together (0x88), matching openwch SDK
    // https://github.com/openwch/ch32v20x/blob/main/EVT/EXAM/SRC/Core/core_riscv.h
    // Fixes silent MIE-stuck-at-0 wedge when critical_section is invoked inside an
    // ISR body with INTSYSCR.INESTEN=1 (see qingke PR #25).
    // gintenr::set_disable/set_enable keep 0x8 for general-purpose use.
    let prior: usize;
    unsafe { core::arch::asm!("csrrc {}, 0x800, {}", out(reg) prior, in(reg) 0x88usize) };
    // `MPIE` is the value `mret` loads back into `MIE`. When `MIE` is already
    // clear — the deepest nesting level, since the vendor settings allow only
    // one preemption — `release` leaves `MPIE` alone, so put it back here.
    // Leaving it clear returns from the interrupt with global interrupts
    // disabled and the executor never wakes up again. Restoring it early is
    // equivalent: it cannot be cleared again while `MIE` is clear, because no
    // interrupt is taken and an exception's `mret` leaves it set.
    if prior & 0x88 == 0x80 {
        unsafe { core::arch::asm!("csrs 0x800, {}", in(reg) 0x80usize) };
    }
    (prior & 0x8) != 0
}

/// Restores what [`mask_interrupts`] returned.
#[cfg(not(cs_mstatus))]
#[inline(always)]
pub(crate) unsafe fn unmask_interrupts(irq_state: bool) {
    if irq_state {
        unsafe { core::arch::asm!("csrs 0x800, {}", in(reg) 0x88usize) };
    }
}

#[cfg(not(feature = "dual-core-critical-section"))]
mod single_hart {
    use critical_section::Impl;

    pub struct Section;

    unsafe impl Impl for Section {
        unsafe fn acquire() -> critical_section::RawRestoreState {
            cfg_if::cfg_if! {
                if #[cfg(cs_mstatus)] {
                    // CH32V003 (qingke_v2) does not have gintenr register
                    // v3a has "invalid" gintenr register - according to QingKeV3_Processor_Manual.pdf page 26
                    // Use standard RISC-V mstatus.MIE instead
                    let mut mstatus: usize;
                    unsafe { core::arch::asm!("csrrci {}, mstatus, 0b1000", out(reg) mstatus) };
                    (mstatus & 0b1000) != 0
                } else if #[cfg(csr_gintenr)] {
                    unsafe { super::mask_interrupts() }
                } else {
                    compile_error!(
                        "qingke critical-section-impl requires exactly one leaf core feature (v2a, v3f, v5f, …)"
                    );
                }
            }
        }

        unsafe fn release(irq_state: critical_section::RawRestoreState) {
            // Only re-enable interrupts if they were enabled before the critical section.
            if irq_state {
                cfg_if::cfg_if! {
                    if #[cfg(cs_mstatus)] {
                        unsafe { core::arch::asm!("csrsi mstatus, 0b1000") };
                    } else if #[cfg(csr_gintenr)] {
                        unsafe { super::unmask_interrupts(irq_state) };
                    } else {
                        compile_error!(
                            "qingke critical-section-impl requires exactly one leaf core feature (v2a, v3f, v5f, …)"
                        );
                    }
                }
            }
        }
    }
}

/// Critical section for the two harts of a single-image CH32H417 build.
///
/// The single-hart variant only masks the calling hart's interrupts, so it
/// cannot
/// serialise the two cores. This variant keeps that behaviour and adds a
/// spinlock that both harts share. Two preconditions:
///
/// - the two harts execute the *same image*, so `LOCKED`/`DEPTH` are the same
///   memory. Two separately flashed images each get their own copy and no
///   exclusion at all.
/// - the lock is held for short, non-blocking sections. A hart that sleeps,
///   waits on the other hart, or is halted by the debugger while inside a
///   critical section deadlocks the other one.
///
/// Nesting is tracked per hart, so a `with` inside a `with` (or in an
/// interrupt handler that runs after the outer one disabled interrupts) does
/// not re-acquire and cannot deadlock against itself.
#[cfg(feature = "dual-core-critical-section")]
mod dual_core {
    use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

    use critical_section::Impl;

    /// `PFIC_SCTLR`, whose `[23:16]` field carries the hart id.
    const PFIC_SCTLR: *const u32 = 0xE000_ED10 as *const u32;

    pub struct Section;

    /// Cross-hart ownership. In `.bss`, therefore shared only in the
    /// single-image model.
    static LOCKED: AtomicBool = AtomicBool::new(false);

    /// Per-hart nesting depth. Each hart only writes its own slot.
    static DEPTH: [AtomicU32; 2] = [AtomicU32::new(0), AtomicU32::new(0)];

    #[inline(always)]
    fn hart() -> usize {
        unsafe { (core::ptr::read_volatile(PFIC_SCTLR) as usize >> 16) & 1 }
    }

    unsafe impl Impl for Section {
        unsafe fn acquire() -> critical_section::RawRestoreState {
            // Local interrupts first: an interrupt handler that enters a
            // critical section on this hart must not spin on a lock its own
            // thread holds.
            let irq_state = unsafe { super::mask_interrupts() };

            let me = hart();
            let depth = DEPTH[me].load(Ordering::Relaxed);
            if depth != 0 {
                DEPTH[me].store(depth + 1, Ordering::Relaxed);
                return irq_state;
            }

            while LOCKED
                .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
                .is_err()
            {
                core::hint::spin_loop();
            }
            DEPTH[me].store(1, Ordering::Relaxed);
            irq_state
        }

        unsafe fn release(irq_state: critical_section::RawRestoreState) {
            let me = hart();
            let depth = DEPTH[me].load(Ordering::Relaxed);
            if depth <= 1 {
                DEPTH[me].store(0, Ordering::Relaxed);
                LOCKED.store(false, Ordering::Release);
            } else {
                DEPTH[me].store(depth - 1, Ordering::Relaxed);
            }

            // Release the shared lock before re-enabling local interrupts, so
            // a handler woken by the unmask cannot observe it still held.
            unsafe { super::unmask_interrupts(irq_state) };
        }
    }
}

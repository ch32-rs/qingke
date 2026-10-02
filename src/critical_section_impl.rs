use critical_section::{set_impl, Impl, RawRestoreState};

struct SingleHartCriticalSection;
set_impl!(SingleHartCriticalSection);

unsafe impl Impl for SingleHartCriticalSection {
    unsafe fn acquire() -> RawRestoreState {
        cfg_if::cfg_if! {
            if #[cfg(cs_mstatus)] {
                // CH32V003 (qingke_v2) does not have gintenr register
                // v3a has "invalid" gintenr register - according to QingKeV3_Processor_Manual.pdf page 26
                // Use standard RISC-V mstatus.MIE instead
                let mut mstatus: usize;
                unsafe { core::arch::asm!("csrrci {}, mstatus, 0b1000", out(reg) mstatus) };
                (mstatus & 0b1000) != 0
            } else if #[cfg(csr_gintenr)] {
                // critical_section: mask MIE+MPIE together (0x88), matching openwch SDK
                // https://github.com/openwch/ch32v20x/blob/main/EVT/EXAM/SRC/Core/core_riscv.h
                // Fixes silent MIE-stuck-at-0 wedge when critical_section is invoked inside an
                // ISR body with INTSYSCR.INESTEN=1 (see qingke PR #25).
                // gintenr::set_disable/set_enable keep 0x8 for general-purpose use.
                let prior: usize;
                unsafe { core::arch::asm!("csrrc {}, 0x800, {}", out(reg) prior, in(reg) 0x88usize) };
                (prior & 0x8) != 0
            } else {
                compile_error!(
                    "qingke critical-section-impl requires exactly one leaf core feature (v2a, v3f, v5f, …)"
                );
            }
        }
    }

    unsafe fn release(irq_state: RawRestoreState) {
        // Only re-enable interrupts if they were enabled before the critical section.
        if irq_state {
            cfg_if::cfg_if! {
                if #[cfg(cs_mstatus)] {
                    unsafe { core::arch::asm!("csrsi mstatus, 0b1000") };
                } else if #[cfg(csr_gintenr)] {
                    unsafe { core::arch::asm!("csrs 0x800, {}", in(reg) 0x88usize) };
                } else {
                    compile_error!(
                        "qingke critical-section-impl requires exactly one leaf core feature (v2a, v3f, v5f, …)"
                    );
                }
            }
        }
    }
}

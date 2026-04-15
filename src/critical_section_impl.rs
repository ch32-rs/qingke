use critical_section::{set_impl, Impl, RawRestoreState};

struct SingleHartCriticalSection;
set_impl!(SingleHartCriticalSection);

unsafe impl Impl for SingleHartCriticalSection {
    unsafe fn acquire() -> RawRestoreState {
        cfg_if::cfg_if! {
            if #[cfg(any(feature = "v2", feature = "v3a"))] {
                // CH32V003 (qingke_v2) does not have gintenr register
                // v3a has "invalid" gintenr register - according to QingKeV3_Processor_Manual.pdf page 26
                // Use standard RISC-V mstatus.MIE instead
                let mut mstatus: usize;
                unsafe { core::arch::asm!("csrrci {}, mstatus, 0b1000", out(reg) mstatus) };
                (mstatus & 0b1000) != 0
            } else {
                // Other QingKe cores have gintenr register
                use crate::register::gintenr;
                (gintenr::set_disable() & 0x8) != 0
            }
        }
    }

    unsafe fn release(irq_state: RawRestoreState) {
        // Only re-enable interrupts if they were enabled before the critical section.
        if irq_state {
            cfg_if::cfg_if! {
                if #[cfg(any(feature = "v2", feature = "v3a"))] {
                    unsafe { core::arch::asm!("csrsi mstatus, 0b1000") };
                } else {
                    use crate::register::gintenr;
                    unsafe { gintenr::set_enable() };
                }
            }
        }
    }
}

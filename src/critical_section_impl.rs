use crate::register::gintenr;
use critical_section::{set_impl, Impl, RawRestoreState};

struct SingleHartCriticalSection;
set_impl!(SingleHartCriticalSection);

unsafe impl Impl for SingleHartCriticalSection {
    unsafe fn acquire() -> RawRestoreState {
        (gintenr::set_disable() & 0x8) != 0
    }

    unsafe fn release(irq_state: RawRestoreState) {
        // Only re-enable interrupts if they were enabled before the critical section.
        if irq_state {
            gintenr::set_enable();
        }
    }
}

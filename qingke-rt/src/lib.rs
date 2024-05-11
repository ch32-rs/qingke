#![no_std]
//! # Differences vs the riscv-rt version
//!
//! - The structure of exception handlers is different
//! - The structure of core interrupt handlers is different
//! - Hardware stack push is available, so no need to push manually
use qingke::{
    register::mtvec::{self, TrapMode},
    riscv::register::mcause,
};
#[cfg(feature = "highcode")]
pub use qingke_rt_macros::highcode;
pub use qingke_rt_macros::{entry, interrupt};

use core::arch::global_asm;

mod asm;

// Let this crate conflicts with riscv-rt
#[export_name = "error: riscv-rt appears more than once in the dependency graph"]
#[doc(hidden)]
pub static __ONCE__: () = ();

extern "C" {
    fn Exception();

    fn InstructionMisaligned();
    fn InstructionFault();
    fn IllegalInstruction();
    fn LoadMisaligned();
    fn LoadFault();
    fn StoreMisaligned();
    fn StoreFault();

    fn NonMaskableInt();
    fn MachineEnvCall();
    fn UserEnvCall();
    fn Breakpoint();
    fn SysTick();
    fn Software();
}

#[doc(hidden)]
#[no_mangle]
#[link_section = ".vector_table.exceptions"]
pub static __EXCEPTIONS: [Option<unsafe extern "C" fn()>; 12] = [
    Some(InstructionMisaligned), // 0
    Some(InstructionFault),
    Some(IllegalInstruction),
    Some(Breakpoint),
    Some(LoadMisaligned),
    Some(LoadFault), // 5, Not accurate, async
    Some(StoreMisaligned),
    Some(StoreFault),  // 7, Not accurate, async
    Some(UserEnvCall), // not available for Qingke V2
    None,
    None,
    Some(MachineEnvCall),
];

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum CoreInterrupt {
    NonMaskableInt = 2,
    Exception = 3,
    MachineEnvCall = 5,
    UserEnvCall = 8,
    Breakpoint = 9,
    SysTick = 12,
    Software = 14,
}

impl CoreInterrupt {
    pub fn try_from(irq: u8) -> Result<Self, u8> {
        match irq {
            2 => Ok(CoreInterrupt::NonMaskableInt),
            3 => Ok(CoreInterrupt::Exception),
            5 => Ok(CoreInterrupt::MachineEnvCall),
            8 => Ok(CoreInterrupt::UserEnvCall),
            9 => Ok(CoreInterrupt::Breakpoint),
            12 => Ok(CoreInterrupt::SysTick),
            14 => Ok(CoreInterrupt::Software),

            _ => Err(irq),
        }
    }
}

/// Core interrupts, without the first one
#[doc(hidden)]
#[no_mangle]
#[used]
#[link_section = ".vector_table.core_interrupts"]
pub static __CORE_INTERRUPTS: [Option<unsafe extern "C" fn()>; 15] = [
    // None, // skip 0
    None,
    Some(NonMaskableInt), // 2
    Some(Exception),      // 3
    None,
    Some(MachineEnvCall), // 5
    None,
    None,
    Some(UserEnvCall), // 8
    Some(Breakpoint),  // 9
    None,
    None,
    Some(SysTick), // 12
    None,
    Some(Software), // 14
    None,
];
// followed by .vector_table.external_interrupts

#[no_mangle]
#[link_section = ".init.rust"]
#[export_name = "_setup_interrupts"]
unsafe extern "C" fn qingke_setup_interrupts() {
    // enable hardware stack push
    // intsyscr(0x804): Open nested interrupts and hardware stack functions
    // 0x3 both nested interrupts and hardware stack
    // 0x1 only hardware stack

    // Qingke V2A, V2C
    #[cfg(feature = "v2")]
    {
        core::arch::asm!(
            "
            li t0, 0x80
            csrw mstatus, t0
            li t0, 0x3
            csrw 0x804, t0
            "
        );
    }

    // Qingke V3A
    #[cfg(feature = "v3")]
    {
        core::arch::asm!(
            "
            li t0, 0x88
            csrs mstatus, t0
        "
        );
    }

    // return to user mode
    // mstate
    // - use 0x88 to set mpp=0, return to user mode
    // - use 0x1888 to set mpp=3, return to machine mode

    // corecfgr(0xbc0): 流水线控制位 & 动态预测控制位
    // corecfgr(0xbc0): Pipeline control bit & Dynamic prediction control
    #[cfg(any(
        feature = "v4",
        not(any(feature = "v2", feature = "v3", feature = "v4"))     // Fallback condition
    ))]
    {
        core::arch::asm!(
            "
            li t0, 0x1f
            csrw 0xbc0, t0
            li t0, 0x3
            csrw 0x804, t0
            li t0, 0x88
            csrs mstatus, t0
            "
        );
        qingke::register::gintenr::set_enable();
    }

    // Qingke V2's mtvec must be 1KB aligned.

    #[cfg(feature = "highcode")]
    mtvec::write(0x20000000, TrapMode::VectoredAddress);

    #[cfg(not(feature = "highcode"))]
    mtvec::write(0x00000000, TrapMode::VectoredAddress);
}

#[doc(hidden)]
#[no_mangle]
#[allow(non_snake_case)]
pub fn DefaultInterruptHandler() {
    loop {
        // Prevent this from turning into a UDF instruction
        // see rust-lang/rust#28728 for details
        continue;
    }
}

#[doc(hidden)]
#[no_mangle]
#[allow(non_snake_case)]
pub fn DefaultExceptionHandler() -> ! {
    loop {
        // Prevent this from turning into a UDF instruction
        // see rust-lang/rust#28728 for details
        continue;
    }
}

// override _start_trap in riscv-rt
global_asm!(
    r#"
        .section .trap, "ax"
        .global _exception_handler
    _exception_handler:
        addi sp, sp, -4
        sw ra, 0(sp)
        jal _exception_handler_rust
        lw ra, 0(sp)
        addi sp, sp, 4
        mret
    "#
);

#[doc(hidden)]
#[link_section = ".trap.rust"]
#[export_name = "_exception_handler_rust"]
pub unsafe extern "C" fn qingke_exception_handler() {
    // jump according to the __EXCEPTIONS table
    extern "C" {
        fn ExceptionHandler();
    }

    let cause = mcause::read();
    let code = cause.code();

    if cause.is_exception() {
        if code < __EXCEPTIONS.len() {
            let h = &__EXCEPTIONS[code];
            if let Some(handler) = h {
                handler();
            } else {
                ExceptionHandler();
            }
        } else {
            ExceptionHandler();
        }
    } else {
        loop {
            // Prevent this from turning into a UDF instruction
            // see rust-lang/rust#28728 for details
            continue;
        }
    }
}

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
#[unsafe(export_name = "error: riscv-rt appears more than once in the dependency graph")]
#[doc(hidden)]
pub static __ONCE__: () = ();

unsafe extern "C" {
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
#[unsafe(no_mangle)]
#[unsafe(link_section = ".vector_table.exceptions")]
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
#[unsafe(no_mangle)]
#[used]
#[unsafe(link_section = ".vector_table.core_interrupts")]
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

#[unsafe(link_section = ".init.rust")]
#[unsafe(export_name = "_setup_interrupts")]
unsafe extern "C" fn qingke_setup_interrupts() {
    // enable hardware stack push
    // intsyscr(0x804): Open nested interrupts and hardware stack functions
    // 0x3 both nested interrupts and hardware stack
    // 0x1 only hardware stack

    // for user mode: mstatus = 0x80
    // mpp(m-mode previous privilege) = 0b00 = U
    // mpie(m-mode previous interrupt enable) = 0b1
    // mie(m-mode interrupt enable) = 0b0
    // interrupts will be enabled when mret at the end of handle_reset
    // jumps to main (mret does mie = mpie)
    // for machine mode: mstatus = 0x1880
    // mpp = 0b11
    // mpie = 0b1
    // mie = 0b0

    // Qingke V2A, V2C
    // (does not have user mode)
    #[cfg(feature = "v2")]
    unsafe {
        core::arch::asm!(
            "
            li t0, 0x1880
            csrw mstatus, t0
            li t0, 0x3
            csrw 0x804, t0
            "
        );
    }

    // Qingke V3A, V3B, V3C, V3F, V3V
    #[cfg(feature = "v3")]
    unsafe {
        #[cfg(feature = "u-mode")]
        core::arch::asm!(
            "
            li t0, 0x80
            csrs mstatus, t0
            "
        );
        #[cfg(not(feature = "u-mode"))]
        core::arch::asm!(
            "
            li t0, 0x1880
            csrs mstatus, t0
            "
        );
    }

    // corecfgr(0xbc0): 流水线控制位 & 动态预测控制位
    // corecfgr(0xbc0): Pipeline control bit & Dynamic prediction control
    #[cfg(any(
        feature = "v4",
        not(any(feature = "v2", feature = "v3", feature = "v4"))     // Fallback condition
    ))]
    unsafe {
        #[cfg(feature = "u-mode")]
        core::arch::asm!(
            "
            li t0, 0x1f
            csrw 0xbc0, t0
            li t0, 0x3
            csrw 0x804, t0
            li t0, 0x80
            csrs mstatus, t0
            "
        );
        #[cfg(not(feature = "u-mode"))]
        core::arch::asm!(
            "
            li t0, 0x1f
            csrw 0xbc0, t0
            li t0, 0x3
            csrw 0x804, t0
            li t0, 0x1880
            csrs mstatus, t0
            "
        );
        qingke::register::gintenr::set_enable();
    }

    // V3A: no VectoredAddress support, use Direct mode + software dispatch.
    #[cfg(feature = "v3a")]
    unsafe {
        unsafe extern "C" {
            fn _unified_trap_handler();
        }
        mtvec::write(_unified_trap_handler as *const () as usize, TrapMode::Direct);
    }

    // Qingke V2's mtvec must be 1KB aligned.

    #[cfg(not(feature = "v3a"))]
    unsafe {
        #[cfg(feature = "highcode")]
        mtvec::write(0x20000000, TrapMode::VectoredAddress);

        #[cfg(not(feature = "highcode"))]
        mtvec::write(0x00000000, TrapMode::VectoredAddress);
    }

    unsafe {
        qingke::pfic::wfi_to_wfe(true);
    }
}

#[doc(hidden)]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub fn DefaultInterruptHandler() {
    loop {
        // Prevent this from turning into a UDF instruction
        // see rust-lang/rust#28728 for details
        continue;
    }
}

#[doc(hidden)]
#[unsafe(no_mangle)]
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

// V3A software dispatch handler for Direct mode.
// Reads mcause, looks up handler address from the vector table, and jumps to it.
#[cfg(all(feature = "v3a", feature = "highcode"))]
global_asm!(
    r#"
        .section .trap, "ax"
        .global _unified_trap_handler
        .align 2
    _unified_trap_handler:
        csrr t0, mcause
        bgez t0, _exception_handler
        slli t0, t0, 1
        srli t0, t0, 1
        slli t0, t0, 2
        la t1, _highcode_vma_start
        add t0, t0, t1
        lw t0, 0(t0)
        beqz t0, 1f
        jr t0
    1:
        la t0, DefaultInterruptHandler
        jr t0
    "#
);

#[cfg(all(feature = "v3a", not(feature = "highcode")))]
global_asm!(
    r#"
        .section .trap, "ax"
        .global _unified_trap_handler
        .align 2
    _unified_trap_handler:
        csrr t0, mcause
        bgez t0, _exception_handler
        slli t0, t0, 1
        srli t0, t0, 1
        slli t0, t0, 2
        la t1, _start
        add t0, t0, t1
        lw t0, 0(t0)
        beqz t0, 1f
        jr t0
    1:
        j DefaultInterruptHandler
    "#
);

#[doc(hidden)]
#[unsafe(link_section = ".trap.rust")]
#[unsafe(export_name = "_exception_handler_rust")]
pub unsafe extern "C" fn qingke_exception_handler() {
    // jump according to the __EXCEPTIONS table
    unsafe extern "C" {
        fn ExceptionHandler();
    }

    let cause = mcause::read();
    let code = cause.code();

    if cause.is_exception() {
        if code < __EXCEPTIONS.len() {
            let h = &__EXCEPTIONS[code];
            if let Some(handler) = h {
                unsafe { handler() };
            } else {
                unsafe { ExceptionHandler() };
            }
        } else {
            unsafe { ExceptionHandler() };
        }
    } else {
        loop {
            // Prevent this from turning into a UDF instruction
            // see rust-lang/rust#28728 for details
            continue;
        }
    }
}

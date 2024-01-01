#![no_std]
//! # Differences vs the riscv-rt version
//!
//! - The structure of exception handlers is different
//! - The structure of core interrupt handlers is different
//! - Hardware stack push is available, so no need to push manually
use qingke::register::{gintenr, mtvec, mtvec::TrapMode};
use qingke::riscv::register::mcause;

pub use qingke_rt_macros::entry;

#[cfg(feature = "highcode")]
pub use qingke_rt_macros::highcode;

use core::arch::{asm, global_asm};

mod asm;

// Let this crate conflicts with riscv-rt
#[export_name = "error: riscv-rt appears more than once in the dependency graph"]
#[doc(hidden)]
pub static __ONCE__: () = ();

extern "C" {
    fn InstructionMisaligned();
    fn InstructionFault();
    fn IllegalInstruction();
    fn Breakpoint();
    fn LoadMisaligned();
    fn LoadFault();
    fn StoreMisaligned();
    fn StoreFault();
    fn UserEnvCall();
    fn MachineEnvCall();
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
    Some(StoreFault), // 7, Not accurate, async
    Some(UserEnvCall),
    None,
    None,
    Some(MachineEnvCall),
];

extern "C" {
    fn NonMaskableInt();
    fn SysTick();
    fn Software();
}

/// Core interrupts
#[doc(hidden)]
#[no_mangle]
#[link_section = ".vector_table.interrupts"]
pub static __CORE_INTERRUPTS: [Option<unsafe extern "C" fn()>; 16] = [
    None,
    None,
    Some(NonMaskableInt), // 2
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(SysTick), // 12
    None,
    Some(Software), // 14
    None,
];

// bind all the potential device specific interrupts
// to the default handler
#[doc(hidden)]
#[no_mangle]
#[link_section = ".vector_table.interrupts"]
pub static mut __EXTERNAL_INTERRUPTS: [Option<unsafe extern "C" fn()>; 256] = [None; 256];

/// The trap handler, Rust version.
#[link_section = ".trap.rust"]
#[export_name = "_start_trap_rust"]
pub unsafe extern "C" fn qingke_start_trap_rust() {
    extern "C" {
        fn ExceptionHandler();
        fn DefaultHandler();
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
    } else if code < __CORE_INTERRUPTS.len() {
        let h = &__CORE_INTERRUPTS[code];
        if let Some(handler) = h {
            handler();
        } else {
            DefaultHandler();
        }
    } else if code < __EXTERNAL_INTERRUPTS.len() {
        let h = &__EXTERNAL_INTERRUPTS[code];
        if let Some(handler) = h {
            handler();
        } else {
            DefaultHandler();
        }
    } else {
        DefaultHandler();
    }
}

// override _start_trap in riscv-rt
global_asm!(
    r#"
        .section .trap, "ax"
        .global _start_trap
    _start_trap:
        addi sp, sp, -4
        sw ra, 0(sp)
        jal _start_trap_rust
        lw ra, 0(sp)
        addi sp, sp, 4
        mret
    "#
);

#[no_mangle]
#[link_section = ".init.rust"]
#[export_name = "_setup_interrupts"]
unsafe extern "C" fn qingke_setup_interrupts() {
    extern "C" {
        fn _start_trap();
    }

    // corecfgr(0xbc0): 流水线控制位 & 动态预测控制位
    // corecfgr: Pipeline control bit & Dynamic prediction control

    // enable hardware stack push
    // intsyscr: Open nested interrupts and hardware stack functions
    // 0x3 both nested interrupts and hardware stack
    // 0x1 only hardware stack

    // Restore state
    // - use 0x88 to set mpp=0, return to user mode
    // - use 0x1888 to set mpp=3, return to machine mode

    // return to user mode
    asm!(
        "
        li t0, 0x1f
        csrw 0xbc0, t0
        li t0, 0x3
        csrw 0x804, t0
        li t0, 0x88
        csrs mstatus, t0
        "
    );
    mtvec::write(_start_trap as usize, TrapMode::Direct);
    gintenr::set_enable();
}

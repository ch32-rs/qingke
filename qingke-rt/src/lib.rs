#![no_std]
#![allow(unexpected_cfgs)]
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

    // WCH's `handle_reset` reaches `main` through `mret`, so `mstatus.MPP`
    // picks the privilege the application runs in (the `mret` also copies
    // MPIE back into MIE). The default is Machine mode, matching `riscv-rt`,
    // which never leaves M-mode: it jumps straight to `_start_rust` rather
    // than `mret`-ing. The `u-mode` feature reproduces WCH's startup instead,
    // returning to User mode, where Machine-mode CSRs raise an illegal
    // instruction (only the `URW` CSRs such as gintenr/intsyscr stay
    // reachable — measured on CH32H417).
    //
    //   `u-mode`:          MPP = 0b00 (U)  — mirrors startup_ch32h417_*.S
    //   default (machine): MPP = 0b11 (M)  — mirrors riscv-rt
    //
    // The V3F/V5F blocks below also need FS = Dirty (bits 14:13) plus
    // MIE/MPIE, so they use a full `csrw`; their two variants differ only in
    // the MPP field: 0x6088 (U) vs 0x7888 (M).

    // Qingke V2A, V2C
    // (does not have user mode)
    #[cfg(qingke_v2)]
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

    // Qingke V3A, V3B, V3C, V3V — the V3 family minus V3F, which alone among
    // the V3 leaves has `inestcr` (0xBC1) and its own startup sequence
    // (`startup_ch32h417_v3f.S`), so it cannot share this block.
    // Leaves corecfgr / intsyscr / nest-level at reset defaults; only OR's /
    // clear's a couple of bits into mstatus. MPP is written explicitly rather
    // than relying on its reset value.
    #[cfg(all(qingke_v3, not(feature = "v3f")))]
    unsafe {
        #[cfg(feature = "u-mode")]
        core::arch::asm!(
            "
            li t0, 0x1800
            csrc mstatus, t0
            li t0, 0x80
            csrs mstatus, t0
            "
        );
        #[cfg(not(feature = "u-mode"))]
        core::arch::asm!(
            "
            li t0, 0x1800
            csrs mstatus, t0
            li t0, 0x80
            csrs mstatus, t0
            "
        );
    }

    // Qingke V3F (CH32H417 primary core).
    // Unlike V3A/V3B, V3F's WCH startup writes the full set of QingKe
    // control CSRs: pipeline/branch-prediction config (corecfgr 0xBC0),
    // 2-level nest control (inestcr 0xBC1), interrupt nesting +
    // hardware-stack enable (intsyscr 0x804), and a full `csrw` of
    // mstatus selecting the return privilege + FP-Dirty. Values mirror
    // `startup_ch32h417_v3f.S:541-551` when `u-mode` is enabled; otherwise
    // MPP is set to Machine mode (see the note at the top of this function).
    //
    // Note: the post-block FP-enable code below (`#[cfg(any(riscvf,
    // riscvd))]`) will subsequently downgrade FS from Dirty (0b11) to
    // Initial (0b01); the first FP register write bumps it back to
    // Dirty automatically.
    #[cfg(feature = "v3f")]
    unsafe {
        use qingke::register::inestcr::{self, NestLevel};
        inestcr::write(NestLevel::Two as usize);
        core::arch::asm!(
            "
            li t0, 0x123703E1
            csrw 0xBC0, t0
            li t0, 0x07
            csrw 0x804, t0
            "
        );
        // 0x6088 = FS Dirty + MPIE + MIE + MPP = U
        // 0x7888 = the same with MPP = M
        #[cfg(feature = "u-mode")]
        core::arch::asm!(
            "
            li t0, 0x6088
            csrw mstatus, t0
            "
        );
        #[cfg(not(feature = "u-mode"))]
        core::arch::asm!(
            "
            li t0, 0x7888
            csrw mstatus, t0
            "
        );
    }

    // Qingke V5F (CH32H417 secondary core).
    // V5F is a deeper / wider core than V3F: 8-level interrupt nesting
    // (vs V3F's 2), pmtcfg=11 in intsyscr (vs V3F's 01), and extra
    // pipeline tuning bits in corecfgr (NLP_EN + a couple of reserved
    // defaults). Values mirror `startup_ch32h417_v5f.S:481-491`.
    //
    // Note: ICache and PMP setup that the WCH SDK does immediately
    // after this block (using `_cache_beg` / `_cache_end` linker
    // symbols) is intentionally NOT performed here — it requires the
    // user's linker script to define those symbols. ICache support
    // will land as a separate opt-in feature.
    //
    // Same FS-downgrade note as v3f applies: the riscvf/d post-block
    // will reset FS from Dirty to Initial. As for v3f, the mstatus write is
    // gated on `u-mode`; the default keeps MPP = Machine mode.
    #[cfg(feature = "v5f")]
    unsafe {
        use qingke::register::inestcr::{self, NestLevel};
        inestcr::write(NestLevel::Eight as usize);
        core::arch::asm!(
            "
            li t0, 0x1237B3E0
            csrw 0xBC0, t0
            li t0, 0x0F
            csrw 0x804, t0
            "
        );
        // 0x6088 = FS Dirty + MPIE + MIE + MPP = U
        // 0x7888 = the same with MPP = M
        #[cfg(feature = "u-mode")]
        core::arch::asm!(
            "
            li t0, 0x6088
            csrw mstatus, t0
            "
        );
        #[cfg(not(feature = "u-mode"))]
        core::arch::asm!(
            "
            li t0, 0x7888
            csrw mstatus, t0
            "
        );
    }

    // corecfgr (0xBC0): pipeline control and branch prediction
    #[cfg(any(
        qingke_v4,
        // Fallback when no leaf is selected: the build script still picks a
        // core family, so this only matters for generic builds.
        not(any(qingke_v2, qingke_v3, qingke_v4, qingke_v5))
    ))]
    unsafe {
        // As for v3a/v3b: MPP is written explicitly instead of relying on its
        // reset value.
        #[cfg(feature = "u-mode")]
        core::arch::asm!(
            "
            li t0, 0x1f
            csrw 0xbc0, t0
            li t0, 0x3
            csrw 0x804, t0
            li t0, 0x1800
            csrc mstatus, t0
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
            li t0, 0x1800
            csrs mstatus, t0
            li t0, 0x80
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
        {
            unsafe extern "C" {
                static _highcode_vma_start: u8;
            }
            mtvec::write(
                &raw const _highcode_vma_start as usize,
                TrapMode::VectoredAddress,
            );
        }
        #[cfg(not(feature = "highcode"))]
        {
            unsafe extern "C" {
                static _start: u8;
            }
            mtvec::write(&raw const _start as usize, TrapMode::VectoredAddress);
        }
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

use core::arch::global_asm;

macro_rules! cfg_global_asm {
    {@inner, [$($x:tt)*], } => {
        global_asm!{$($x)*}
    };
    (@inner, [$($x:tt)*], #[cfg($meta:meta)] $asm:literal, $($rest:tt)*) => {
        #[cfg($meta)]
        cfg_global_asm!{@inner, [$($x)* $asm,], $($rest)*}
        #[cfg(not($meta))]
        cfg_global_asm!{@inner, [$($x)*], $($rest)*}
    };
    {@inner, [$($x:tt)*], $asm:literal, $($rest:tt)*} => {
        cfg_global_asm!{@inner, [$($x)* $asm,], $($rest)*}
    };
    {$($asms:tt)*} => {
        cfg_global_asm!{@inner, [], $($asms)*}
    };
}
cfg_global_asm! {
    "
    .section    .init,\"ax\"
    .global _start
    .align  1
_start:
    j handle_reset
    ",
    "
    .section    .handle_reset,\"ax\",@progbits
    .weak   handle_reset
    .align  1
handle_reset:
    .option push
    .option norelax
    la gp, __global_pointer$
    .option pop
    la sp, _stack_top
    ",
    // load highcode from flash to ram
    #[cfg(feature = "highcode")]
    "
    la a0, _highcode_lma
    la a1, _highcode_vma_start
    la a2, _highcode_vma_end
    bgeu a1, a2, 2f
1:
    lw t0, (a0)
    sw t0, (a1)
    addi a0, a0, 4
    addi a1, a1, 4
    bltu a1, a2, 1b
2:
    ",
    // load data from flash to ram
    "
    la a0, _data_lma
    la a1, _data_vma
    la a2, _edata
    bgeu a1, a2, 2f
1:
    lw t0, (a0)
    sw t0, (a1)
    addi a0, a0, 4
    addi a1, a1, 4
    bltu a1, a2, 1b
2:
    ",
    // clear bss section
    "
    la a0, _sbss
    la a1, _ebss
    bgeu a0, a1, 2f
1:
    sw zero, (a0)
    addi a0, a0, 4
    bltu a0, a1, 1b
2:
    ",

    "
    jal _setup_interrupts
    ",

    // enable floating point and interrupt
    #[cfg(any(riscvf, riscvd))]
    "
    li t0, 0x4000 // bit 14 is FS most significant bit
    li t2, 0x2000 // bit 13 is FS least significant bit
    csrrc x0, mstatus, t0
    csrrs x0, mstatus, t2
    ",

    "
    la t0, main
    csrw mepc, t0

    mret
    ",
}

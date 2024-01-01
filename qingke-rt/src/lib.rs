use core::arch::global_asm;

// Let this conflicts with riscv-rt
#[export_name = "error: riscv-rt appears more than once in the dependency graph"]
#[doc(hidden)]
pub static __ONCE__: () = ();



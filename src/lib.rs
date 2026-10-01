//! Low level access to WCH's QingKe RISC-V processors
#![no_std]
#![allow(unexpected_cfgs)]

#[macro_use]
mod macros;

#[cfg(any(dm_data_f4, dm_data_380, dm_data_340))]
pub mod dm;
pub mod interrupt;
pub mod pfic;
pub mod register;

// re-export
pub use riscv;

#[cfg(all(
    any(
        target_has_atomic = "8",
        // target_has_atomic = "16",
        // target_has_atomic = "32",
        // target_has_atomic = "64",
        // target_has_atomic = "128",
        // target_has_atomic = "ptr"
    ),
    not(feature = "unsafe-trust-wch-atomics")
))]
compile_error!(
    "As tested on QingKe V4, most likely the atomics are broken, 
please validate the atomic instruction on the hardware using 
something like litmus test suite before trusting them."
);

#[cfg(feature = "critical-section-impl")]
mod critical_section_impl;

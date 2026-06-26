//! Low level access to WCH's QingKe RISC-V processors
#![no_std]

#[macro_use]
mod macros;

pub mod interrupt;
pub mod pfic;
pub mod register;

// re-export
pub use riscv;

// Core family selection is mutually exclusive — picking more than one
// means downstream asm blocks (e.g. `qingke_setup_interrupts`) emit
// conflicting startup sequences and the second one clobbers the first.
#[cfg(any(
    all(feature = "v2", feature = "_v3"),
    all(feature = "v2", feature = "v4"),
    all(feature = "v2", feature = "_v5"),
    all(feature = "_v3", feature = "v4"),
    all(feature = "_v3", feature = "_v5"),
    all(feature = "v4", feature = "_v5"),
))]
compile_error!(
    "qingke: at most one core-family feature (v2 / _v3 / v4 / _v5) may be enabled. \
     Concrete leaves like v3a/v3b/v3f/v5f pull in their umbrella, so e.g. v3f and v5f \
     conflict because v3f → _v3 and v5f → _v5."
);

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

//! Low level access to WCH's QingKe RISC-V processors
#![no_std]

#[macro_use]
mod macros;

pub mod interrupt;
pub mod pfic;
pub mod register;

// re-export
pub use riscv;

#[cfg(feature = "critical-section-impl")]
mod critical_section_impl;

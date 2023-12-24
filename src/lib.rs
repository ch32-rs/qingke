//! Low level access to WCH's QingKe RISC-V processors
#![no_std]

#[macro_use]
mod macros;

pub mod pfic;
pub mod register;

// re-export
pub use riscv;

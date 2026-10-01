//! RISC-V Debug Module: memory-mapped `data` registers used for SDI printf.
//!
//! When `hartinfo.dataaccess` is set, `DATA0`/`DATA1` are at
//! `PPB_BASE + hartinfo.dataaddr` (+4 for `DATA1`). Select a leaf core feature
//! on the `qingke` dependency so `build.rs` enables the matching `dm_dataaddr_*` cfg.

/// PPB base (RISC-V system bus).
pub const PPB_BASE: u32 = 0xE000_0000;

cfg_if::cfg_if! {
    if #[cfg(dm_dataaddr_0f4)] {
        /// Byte offset of `DATA0` from [`PPB_BASE`] (`hartinfo.dataaddr`).
        pub const DATA_OFFSET: u32 = 0x00F4;
        /// Address of DM `DATA0` (SDI printf handshake / first 4 payload bytes).
        pub const DATA0: u32 = PPB_BASE + DATA_OFFSET;
        /// Address of DM `DATA1` (SDI printf second 4 payload bytes).
        pub const DATA1: u32 = DATA0 + 4;
    } else if #[cfg(dm_dataaddr_340)] {
        /// Byte offset of `DATA0` from [`PPB_BASE`] (`hartinfo.dataaddr`).
        pub const DATA_OFFSET: u32 = 0x0340;
        /// Address of DM `DATA0` (SDI printf handshake / first 4 payload bytes).
        pub const DATA0: u32 = PPB_BASE + DATA_OFFSET;
        /// Address of DM `DATA1` (SDI printf second 4 payload bytes).
        pub const DATA1: u32 = DATA0 + 4;
    } else if #[cfg(dm_dataaddr_380)] {
        /// Byte offset of `DATA0` from [`PPB_BASE`] (`hartinfo.dataaddr`).
        pub const DATA_OFFSET: u32 = 0x0380;
        /// Address of DM `DATA0` (SDI printf handshake / first 4 payload bytes).
        pub const DATA0: u32 = PPB_BASE + DATA_OFFSET;
        /// Address of DM `DATA1` (SDI printf second 4 payload bytes).
        pub const DATA1: u32 = DATA0 + 4;
    }
}

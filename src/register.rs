//! QingKe extended CSRs

#[cfg(feature = "_v5")]
pub mod cache_pmp_ovr;
#[cfg(feature = "_v5")]
pub mod cache_strtg_ctlr;
pub mod corecfgr;
pub mod gintenr;
#[cfg(feature = "_inestcr")]
pub mod inestcr;
pub mod intsyscr;
pub mod mtvec;
#[cfg(feature = "_v5")]
pub mod opcache_ctlr;

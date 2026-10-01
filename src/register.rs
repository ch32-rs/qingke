//! QingKe extended CSRs

#[cfg(csr_cache_pmp_ovr)]
pub mod cache_pmp_ovr;
#[cfg(csr_cache_strtg_ctlr)]
pub mod cache_strtg_ctlr;
#[cfg(csr_corecfgr)]
pub mod corecfgr;
#[cfg(csr_gintenr)]
pub mod gintenr;
#[cfg(csr_inestcr)]
pub mod inestcr;
#[cfg(csr_intsyscr)]
pub mod intsyscr;
#[cfg(csr_mtvec)]
pub mod mtvec;
#[cfg(csr_opcache_ctlr)]
pub mod opcache_ctlr;

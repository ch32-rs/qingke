use volatile_register::{RO, RW};

use crate::peripheral::STK;

#[repr(C)]
pub struct RegisterBlock {
    /// Control Register
    pub ctlr: RW<u32>,
    /// State Register
    pub sr: RW<u32>,
    /// Counter Value Low
    pub cntl: RW<u32>,
    /// Counter Value High
    pub cnth: RW<u32>,
    /// Reload Value Low
    pub cmplr: RW<u32>,
    /// Reload Value High
    pub cmphr: RW<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StkClkSource {
    /// Use HCLK/8 as clock source
    HclkDiv8,
    /// Use HCLK as clock source
    Hclk,
}

const STK_CTLR_STE:u32 = 1 << 0;
const STK_CTLR_STIE:u32 = 1 << 1;
const STK_CTLR_STCLK:u32 = 1 << 2;
const STK_CTLR_STRE:u32 = 1 << 3;
const STK_CTLR_MODE:u32 = 1 << 4;
const STK_CTLR_INIT:u32 = 1 << 5;
const STK_CTLR_SWIE:u32 = 1 << 31;
const STK_SR_CNTIF:u32 = 1 << 0;


impl STK {

    /// Clear current counter value's low 32 bits
    #[inline]
    pub fn clear_current_low(&mut self) {
        unsafe{self.cntl.write(0)};
    }

    /// Clear current counter value's high 32 bits
    #[inline]
    pub fn clear_current_high(&mut self) {
        unsafe{self.cnth.write(0)};
    }

    /// Clear the current counter value.
    /// Note: this operation is non-atomic, user should add lock if needed.
    #[inline]
    pub fn clear_current_non_atomic(&mut self) {
        self.clear_current_low();
        self.clear_current_high();
    }


    /// Gets current value's low 32 bits
    #[inline]
    pub fn get_current_low() -> u32 {
        unsafe { (*Self::PTR).cntl.read() }
    }

    /// Gets current value's high 32 bits
    #[inline]
    pub fn get_current_high() -> u32 {
        unsafe { (*Self::PTR).cnth.read() }
    }

    /// Gets current value's high 64 bits
    /// Note: this operation is non-atomic, user should add lock if needed.
    #[inline]
    pub fn get_current_non_atomic() -> u64 {
        (Self::get_current_high() as u64) << 32 | Self::get_current_low() as u64
    }

    /// Disable counter
    #[inline]
    pub fn disable_counter(&mut self) {
        unsafe {self.ctlr.modify(|v| v & !STK_CTLR_STE)}
    }


    /// Disable SysTick interrupt
    #[inline]
    pub fn disable_interrupt(&mut self) {
        unsafe {self.ctlr.modify(|v| v & !STK_CTLR_STIE)}
    }

    /// Enables counter
    #[inline]
    pub fn enable_counter(&mut self) {
        unsafe { self.ctlr.modify(|v| v | STK_CTLR_STE) }
    }

    /// Enables SysTick interrupt
    #[inline]
    pub fn enable_interrupt(&mut self) {
        unsafe { self.ctlr.modify(|v| v | STK_CTLR_STIE) }
    }


    #[inline]
    pub fn set_clock_source(&mut self, clk_source: StkClkSource) {
        match clk_source {
            StkClkSource::Hclk => unsafe{self.ctlr.modify(|v| v | STK_CTLR_STCLK)},
            StkClkSource::HclkDiv8 => unsafe{self.ctlr.modify(|v| v & !STK_CTLR_STCLK)},
        }
    }

    /// Sets reload value's low 32 bit
    #[inline]
    pub fn set_reload_low(&mut self, value: u32) {
        unsafe{self.cmplr.write(value)};
    }

    /// Sets reload value's high 32 bit
    #[inline]
    pub fn set_reload_high(&mut self, value: u32) {
        unsafe{self.cmphr.write(value)};
    }

    /// Sets reload value's low 32 bit, this operation is non-atomic, user should add lock if needed.
    #[inline]
    pub fn set_reload_non_atomic(&mut self, value: u64) {
        self.set_reload_low(value as u32);
        self.set_reload_low((value >> 32) as u32);
    }

    /// Checks if the counter wrapped (underflowed) since the last check
    #[inline]
    pub fn has_wrapped(&self) -> bool{
        self.sr.read() & STK_SR_CNTIF != 0
    }
    /// Clear the counter wrapped (underflowed) flag
    pub fn clear_wrapped_flag(&mut self) {
        unsafe{self.sr.modify(|v| v & !STK_SR_CNTIF)}
    }

    /// Checks if counter is enabled
    #[inline]
    pub fn is_counter_enabled(&self) -> bool {
        self.ctlr.read() & STK_CTLR_STE != 0
    }

    /// Checks if SysTick interrupt is enabled
    #[inline]
    pub fn is_interrupt_enabled(&self) -> bool {
        self.ctlr.read() & STK_CTLR_STIE != 0
    }

}

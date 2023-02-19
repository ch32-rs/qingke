//! intsyscr, interrupt system control register
use bit_field::BitField;
use core::arch::asm;

/// intsyscr register
///
/// Write 0x3 to enable nested and hardware stack
#[derive(Clone, Copy, Debug)]
pub struct Intsyscr {
    bits: usize,
}

impl Intsyscr {
    #[inline]
    pub fn hwstken(&self) -> bool {
        self.bits.get_bit(0)
    }

    #[inline]
    pub fn inesten(&self) -> bool {
        self.bits.get_bit(1)
    }

    #[inline]
    pub fn pmtcfg(&self) -> u8 {
        self.bits.get_bits(2..=3) as u8
    }

    #[inline]
    pub fn hwstkoven(&self) -> bool {
        self.bits.get_bit(4)
    }

    #[inline]
    pub fn gihwstknen(&self) -> bool {
        self.bits.get_bit(5)
    }

    #[inline]
    pub fn pmtsta(&self) -> u8 {
        self.bits.get_bits(8..=15) as u8
    }
}

read_csr_as!(Intsyscr, 0x804);

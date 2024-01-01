const PRIORITY_MASK: u8 = 0xf0;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
#[allow(missing_docs)]
pub enum Priority {
    P0 = 0x0,
    P1 = 0x10,
    P2 = 0x20,
    P3 = 0x30,
    P4 = 0x40,
    P5 = 0x50,
    P6 = 0x60,
    P7 = 0x70,
    P8 = 0x80,
    P9 = 0x90,
    P10 = 0xa0,
    P11 = 0xb0,
    P12 = 0xc0,
    P13 = 0xd0,
    P14 = 0xe0,
    P15 = 0xf0,
}

impl From<u8> for Priority {
    fn from(priority: u8) -> Self {
        unsafe { core::mem::transmute(priority & PRIORITY_MASK) }
    }
}

impl From<Priority> for u8 {
    fn from(p: Priority) -> Self {
        p as u8
    }
}

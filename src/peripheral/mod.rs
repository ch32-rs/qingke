use core::{marker::PhantomData, ops::Deref};

pub mod stk;

pub struct STK {
    _marker: PhantomData<*const()>
}

unsafe impl Send for STK{}

impl STK {
    // Pointer to the register block
    pub const PTR: *const stk::RegisterBlock = 0xE000_F000 as *const _;
}

impl Deref for STK {
    type Target = self::stk::RegisterBlock;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe {&*Self::PTR}
    }
}
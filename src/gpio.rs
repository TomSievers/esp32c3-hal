//! Module to control gpio pins

use core::ptr::{read_volatile, write_volatile};

use embedded_hal::digital::{v2::{InputPin, OutputPin, StatefulOutputPin}};

const GPIO_BASE_ADDR : u32 = 0x6000_4000;
const IO_MUX_BASE_ADDR : u32 = 0x6000_9000;

#[derive(Clone, Copy)]
pub(crate) struct Pin<const S : u32> {
    
}

#[repr(u8)]
#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum GpioFunction {
    Function0 = 0,
    Function1 = 1,
    Function2 = 2,
    Function3 = 3,
}

/// Pin pull direction
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Pull {
    /// pullup resistor
    Up,
    /// pulldown resistor
    Down,
    /// none
    None,
}

/// Output pin drive strength
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DriveStrength {
    /// Drive strenght of ~5mA
    MilliA5 = 0,
    /// Drive strenght of ~10mA
    MilliA10 = 1,
    /// Drive strenght of ~20mA
    MilliA20 = 2,
    /// Drive strenght of ~40mA
    MilliA40 = 3
}

#[allow(dead_code)]
impl<const S : u32> Pin<S> {
    const OUT_REG : u32 = GPIO_BASE_ADDR + 0x4;
    const OUT_SET_REG : u32 = GPIO_BASE_ADDR + 0x8;
    const OUT_CLR_REG : u32 = GPIO_BASE_ADDR + 0xC;
    const OUT_EN_REG : u32 = GPIO_BASE_ADDR + 0x20;
    const OUT_EN_SET_REG : u32 = GPIO_BASE_ADDR + 0x24;
    const OUT_EN_CLR_REG : u32 = GPIO_BASE_ADDR + 0x28;
    const IN_REG : u32 = GPIO_BASE_ADDR + 0x3C;
    const IRQS_REG : u32 = GPIO_BASE_ADDR + 0x44;
    const IRQS_SET_REG : u32 = GPIO_BASE_ADDR + 0x48;
    const IRQS_CLR_REG : u32 = GPIO_BASE_ADDR + 0x4C;
    const CFG_REG : u32 = GPIO_BASE_ADDR + 0x74 + 0x4 * S;
    const OUT_CFG_REG : u32 = GPIO_BASE_ADDR + 0x554 + 0x4 * S;
    const IO_MUX : u32 = IO_MUX_BASE_ADDR + 0x4 + 0x4 * S;

    pub unsafe fn set_function(&self, function : GpioFunction) {
        let mut io_mux = read_volatile(Self::IO_MUX as *const u32);

        io_mux &= !(0b11 << 12);
        io_mux |= (function as u32) << 12;

        write_volatile(Self::IO_MUX as *mut u32, io_mux);
    }

    pub unsafe fn set_pull(&self, pull : Pull) {
        let mut io_mux = read_volatile(Self::IO_MUX as *const u32);

        io_mux &= !(0b11 << 7);

        match pull {
            Pull::Up => io_mux |= 0x1 << 8,
            Pull::Down => io_mux |= 0x1 << 7,
            Pull::None => (),
        }

        write_volatile(Self::IO_MUX as *mut u32, io_mux);
    }

    pub unsafe fn input_enable(&self, enable : bool) {
        let mut io_mux = read_volatile(Self::IO_MUX as *const u32);

        io_mux &= !(0x1 << 9);
        io_mux |= (enable as u32) << 9;

        write_volatile(Self::IO_MUX as *mut u32, io_mux);
    }

    pub unsafe fn get_input(&self) -> bool {
        (read_volatile(Self::IN_REG as *const u32) & (1 << S)) > 0
    }

    pub unsafe fn output_enable(&self, enable : bool) {
        if enable {
            write_volatile(Self::OUT_EN_SET_REG as *mut u32, 1 << S);
        } else {
            write_volatile(Self::OUT_EN_CLR_REG as *mut u32, 1 << S);
        }
    }

    pub unsafe fn output_select(&self, select : u8) {
        let mut cfg = read_volatile(Self::OUT_CFG_REG as *const u32);

        debug_assert!(select < 129);

        cfg &= !(0xFF);
        cfg |= select as u32;

        write_volatile(Self::OUT_CFG_REG as *mut u32, cfg);
    }

    pub unsafe fn set_output(&self, high : bool) {
        if high {
            write_volatile(Self::OUT_SET_REG as *mut u32, 1 << S);
        } else {
            write_volatile(Self::OUT_CLR_REG as *mut u32, 1 << S);
        }
    }

    pub unsafe fn get_output(&self) -> bool {
        (read_volatile(Self::OUT_REG as *const u32) & (1 << S)) > 0
    }

    pub unsafe fn set_drive_strength(&self, drive_strength : DriveStrength) {
        let mut io_mux = read_volatile(Self::IO_MUX as *const u32);

        io_mux &= !(0b11 << 10);
        io_mux |= (drive_strength as u32) << 10;

        write_volatile(Self::IO_MUX as *mut u32, io_mux);
    }
}

/// Structure to control a pin as an input pin.
#[derive(Clone, Copy)]
pub struct Input<const S : u32> {
    pin : Pin<S>
}

impl<const S : u32> Input<S> {
    /// Create a new input pin.
    pub fn new() -> Self {

        debug_assert!(S < 22);

        let pin = Pin{};

        unsafe {
            pin.set_function(GpioFunction::Function1);
            pin.set_pull(Pull::None);
            pin.output_enable(false);
            pin.input_enable(true);
        }

        Input { 
            pin
        }
    }

    /// Set the pull direction of the pin.
    pub fn set_pull(&self, pull : Pull) {
        unsafe{self.pin.set_pull(pull)}
    }
}

impl<const S : u32> Default for Input<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const S : u32> InputPin for Input<S> {
    type Error = ();

    fn is_high(&self) -> Result<bool, Self::Error> {
        Ok(unsafe { self.pin.get_input() })
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(unsafe { !self.pin.get_input() })
    }
}

/// Structure to control a pin as an output pin.
#[derive(Clone, Copy)]
pub struct Output<const S : u32> {
    pin : Pin<S>
}

impl<const S : u32> Output<S> {
    /// Create a new input pin.
    pub fn new() -> Self {

        debug_assert!(S < 22);

        let pin = Pin{};

        unsafe {
            pin.set_function(GpioFunction::Function1);
            pin.output_select(128);
            pin.set_drive_strength(DriveStrength::MilliA20);
            pin.set_output(false);
            pin.output_enable(true);
            pin.input_enable(false);
        }

        Output { 
            pin
        }
    }

    /// Set the drive strength of the output.
    pub fn set_drive_strength(&self, drive_strength : DriveStrength) {
        unsafe{self.pin.set_drive_strength(drive_strength)};
    }
}

impl<const S : u32> Default for Output<S> {
    fn default() -> Self {
        Self::new()
    }
}


impl<const S : u32> OutputPin for Output<S> {
    type Error = ();

    fn set_low(&mut self) -> Result<(), Self::Error> {
        unsafe {self.pin.set_output(false)};
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        unsafe {self.pin.set_output(true)};
        Ok(())
    }
}

impl<const S : u32> StatefulOutputPin for Output<S> {
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        Ok(unsafe{self.pin.get_output()})
    }

    fn is_set_low(&self) -> Result<bool, Self::Error> {
        Ok(unsafe{!self.pin.get_output()})
    }
}
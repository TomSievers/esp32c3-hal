//! Driver for DMA hardware

use core::ptr::{read_volatile, write_volatile};

use esp32c3::{Peripherals, DMA};

/// Item in a list of DMA transfers
#[repr(C, align(4))]
pub struct ListItem {
    state : u32,
    buffer_ptr : u32,
    next_item_ptr : u32,
}

impl ListItem {
    /// Create a new DMA transfer list item.
    pub fn new() -> ListItem {
        ListItem { 
            state: 0b11 << 30, 
            buffer_ptr: 0, 
            next_item_ptr: 0 
        }
    }

    /// Set the buffer this transfer points to.
    pub fn set_buffer<T>(&mut self, buffer : &mut [u8]) {
        let len = buffer.len() as u32;

        debug_assert!(len < 4096);

        self.state |= (len << 12) | len;

        let buffer_ptr = buffer.as_mut_ptr() as usize;
        self.buffer_ptr = buffer_ptr as u32;
    }

    /// Set the DMA tranfer that follows this transfer.
    pub fn set_next(&mut self, next : *mut ListItem) {
        self.state &= !(1 << 30);
        let next_ptr = next as usize;
        self.next_item_ptr = next_ptr as u32;
    }

    /// Check if the error bit was set by hardware.
    pub fn has_error(&self) -> bool {
        (self.state >> 28) != 0
    }

}

/// Peripheral used in DMA transfer.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum Peripheral {
    /// SPI peripheral
    SPI2 = 0,
    /// UART0 / UART1 peripheral
    UHCI0 = 2,
    /// I2S peripheral
    I2S = 3,
    /// AES peripheral
    AES = 6,
    /// SHA peripheral
    SHA = 7,
    /// ADC peripheral
    ADC = 8
}

/// Channel used in DMA transfer
#[derive(Clone, Copy)]
pub enum Channel {
    /// Channel 0
    Channel0,
    /// Channel 1
    Channel1,
    /// Channel 2
    Channel2,
}

impl Channel {

    fn reset(conf0_reg : *mut u32) {
        let mut conf0 = unsafe {read_volatile(conf0_reg)};

        conf0 |= 0b1;

        unsafe { write_volatile(conf0_reg, conf0)};

        let mut conf0 = unsafe {read_volatile(conf0_reg)};

        conf0 &= !0b1;

        unsafe { write_volatile(conf0_reg, conf0)};
    }

    /// Reset transmit side of channel.
    pub fn tx_reset(&self, dma : &DMA) {
        let conf0_reg = match self {
            Channel::Channel0 => dma.out_conf0_ch0.as_ptr(),
            Channel::Channel1 => dma.out_conf0_ch1.as_ptr(),
            Channel::Channel2 => dma.out_conf0_ch2.as_ptr()
        };

        Self::reset(conf0_reg);
    }

    /// Reset receive side of channel.
    pub fn rx_reset(&self, dma : &DMA) {
        let conf0_reg = match self {
            Channel::Channel0 => dma.in_conf0_ch0.as_ptr(),
            Channel::Channel1 => dma.in_conf0_ch1.as_ptr(),
            Channel::Channel2 => dma.in_conf0_ch2.as_ptr()
        };

        Self::reset(conf0_reg);
    }

    /// Set the start linked list for the transmit side of the channel.
    pub fn set_tx_start(&self, dma : &DMA, list : *const ListItem) {
        let out_link_reg = match self {
            Channel::Channel0 => dma.out_link_ch0.as_ptr(),
            Channel::Channel1 => dma.out_link_ch1.as_ptr(),
            Channel::Channel2 => dma.out_link_ch2.as_ptr()
        };

        let list_ptr = list as usize;

        let mut out_link = unsafe {read_volatile(out_link_reg)};

        out_link &= !0xFFFFF;

        out_link |= (list_ptr as u32) & 0xFFFFF;

        unsafe { write_volatile(out_link_reg, out_link)};
    }

    /// Set the start linked list for the receive side of the channel.
    pub fn set_rx_start(&self, dma : &DMA, list : *const ListItem) {
        let in_link_reg = match self {
            Channel::Channel0 => dma.in_link_ch0.as_ptr(),
            Channel::Channel1 => dma.in_link_ch1.as_ptr(),
            Channel::Channel2 => dma.in_link_ch2.as_ptr()
        };

        let list_ptr = list as usize;

        let mut in_link = unsafe {read_volatile(in_link_reg)};

        in_link &= !0xFFFFF;

        in_link |= (list_ptr as u32) & 0xFFFFF;

        unsafe { write_volatile(in_link_reg, in_link)};
    }

    /// Set the peripheral to which the transmit channel will transfer data to.
    pub fn set_tx_peripheral(&self, dma : &DMA, peripheral : Peripheral) {
        let out_peri_reg = match self {
            Channel::Channel0 => dma.out_peri_sel_ch0.as_ptr(),
            Channel::Channel1 => dma.out_peri_sel_ch1.as_ptr(),
            Channel::Channel2 => dma.out_peri_sel_ch2.as_ptr()
        };

        unsafe { write_volatile(out_peri_reg, peripheral as u32)};
    }

    /// Set the peripheral from which the receive channel will transfer data from.
    pub fn set_rx_peripheral(&self, dma : &DMA, peripheral : Peripheral) {
        let out_peri_reg = match self {
            Channel::Channel0 => dma.in_peri_sel_ch0.as_ptr(),
            Channel::Channel1 => dma.in_peri_sel_ch1.as_ptr(),
            Channel::Channel2 => dma.in_peri_sel_ch2.as_ptr()
        };

        unsafe { write_volatile(out_peri_reg, peripheral as u32)};
    }

    /// Enable the receiver channel.
    pub fn rx_enable(&self, dma : &DMA) {
        let in_link_reg = match self {
            Channel::Channel0 => dma.in_link_ch0.as_ptr(),
            Channel::Channel1 => dma.in_link_ch1.as_ptr(),
            Channel::Channel2 => dma.in_link_ch2.as_ptr()
        };

        let mut in_link = unsafe {read_volatile(in_link_reg)};

        in_link |= 1 << 22;

        unsafe { write_volatile(in_link_reg, in_link)};
    }

    /// Enable the transmitter channel
    pub fn tx_enable(&self, dma : &DMA) {
        let out_link_reg = match self {
            Channel::Channel0 => dma.out_link_ch0.as_ptr(),
            Channel::Channel1 => dma.out_link_ch1.as_ptr(),
            Channel::Channel2 => dma.out_link_ch2.as_ptr()
        };

        let mut out_link = unsafe {read_volatile(out_link_reg)};

        out_link |= 1 << 22;

        unsafe { write_volatile(out_link_reg, out_link)};
    }

    /// Enable memory to memory transfer (only possible on rx part of channel)
    pub fn mem_to_mem(&self, dma : &DMA) {
        let conf0_reg = match self {
            Channel::Channel0 => dma.in_conf0_ch0.as_ptr(),
            Channel::Channel1 => dma.in_conf0_ch1.as_ptr(),
            Channel::Channel2 => dma.in_conf0_ch2.as_ptr()
        };

        let mut conf0 = unsafe {read_volatile(conf0_reg)};

        conf0 |= 0b1 << 4;

        unsafe { write_volatile(conf0_reg, conf0)};
    }
}

/// A pipe for DMA transfers
pub struct DMAPipe {
    tx_channel : Channel,
    rx_channel : Channel,
    dma : DMA
}

impl DMAPipe {
    /// Create a pipe between memory and a peripheral
    pub fn memory_n_peripheral(tx_channel : Channel, rx_channel : Channel, peripheral : Peripheral) -> DMAPipe {
        let dp = unsafe {Peripherals::steal()};

        tx_channel.tx_reset(&dp.DMA);
        rx_channel.rx_reset(&dp.DMA);

        tx_channel.set_tx_peripheral(&dp.DMA, peripheral);
        rx_channel.set_rx_peripheral(&dp.DMA, peripheral);

        DMAPipe {  
            dma : dp.DMA,
            tx_channel,
            rx_channel
        }
    }

    /// Create a pipe between two memory locations using dma
    pub fn memory_n_memory(channel : Channel) -> DMAPipe {
        let dp = unsafe {Peripherals::steal()};

        channel.tx_reset(&dp.DMA);
        channel.rx_reset(&dp.DMA);

        channel.mem_to_mem(&dp.DMA);

        DMAPipe {  
            dma : dp.DMA,
            tx_channel : channel,
            rx_channel : channel
        }
    }

    /// Start a transfer using the given transmit and receive linked lists.
    pub fn start_transfer(&mut self, tx_list : *const ListItem, rx_list : *const ListItem) {
        self.tx_channel.set_tx_start(&self.dma, tx_list);
        self.rx_channel.set_rx_start(&self.dma, rx_list);

        self.tx_channel.tx_enable(&self.dma);
        self.rx_channel.rx_enable(&self.dma);
    }

    /// Check if the transmit channel has completed the transfer.
    pub fn get_tx_completion(&self) -> bool {
        match self.tx_channel {
            Channel::Channel0 => self.dma.int_st_ch0.read().out_eof_ch0_int_st().bit_is_set(),
            Channel::Channel1 => self.dma.int_st_ch1.read().out_eof_ch1_int_st().bit_is_set(),
            Channel::Channel2 => self.dma.int_st_ch2.read().out_eof_ch2_int_st().bit_is_set(),
        }
    }

    /// Check if the receive channel has completed the transfer.
    pub fn get_rx_completion(&self) -> bool {
        match self.tx_channel {
            Channel::Channel0 => self.dma.int_st_ch0.read().in_done_ch0_int_st().bit_is_set(),
            Channel::Channel1 => self.dma.int_st_ch1.read().in_done_ch1_int_st().bit_is_set(),
            Channel::Channel2 => self.dma.int_st_ch2.read().in_done_ch2_int_st().bit_is_set(),
        }
    }
}
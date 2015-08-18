use core::mem;
use core::intrinsics;
use pm;
use process::{AppSlice};

use helpers::*;

/// Memory registers for a DMA channel. Section 16.6.1 of the datasheet
#[repr(C, packed)]
#[allow(dead_code)]
struct DMARegisters {
    memory_address:           usize,
    peripheral_select:        usize,
    transfer_counter:         usize,
    memory_address_reload:    usize,
    transfer_counter_reload:  usize,
    control:                  usize,
    mode:                     usize,
    status:                   usize,
    interrupt_enable:         usize,
    interrupt_disable:        usize,
    interrupt_mask:           usize,
    interrupt_status:         usize,
    version:                  usize,
    _unused:                  [usize; 3]
}

/// The PDCA's base addresses in memory (Section 7.1 of manual)
pub const DMA_BASE_ADDR : usize = 0x400A2000;

/// The number of bytes between each memory mapped DMA Channel (Section 16.6.1)
pub const DMA_CHANNEL_SIZE : usize = 0x40;

/// Shared counter that Keeps track of how many DMA channels are currently
/// active.
static mut NUM_ENABLED: usize = 0;

/// The DMA channel number. Each channel transfers data between memory and a
/// particular peripheral function (e.g., SPI read or SPI write, but not both
/// simultaneously). There are 16 available channels (Section 16.7)
#[derive(Copy,Clone)]
pub enum DMAChannelNum {
    // Relies on the fact that assigns values 0-15 to each constructor in order
    DMAChannel00 = 0,
    DMAChannel01 = 1,
    DMAChannel02 = 2,
    DMAChannel03 = 3,
    DMAChannel04 = 4,
    DMAChannel05 = 5,
    DMAChannel06 = 6,
    DMAChannel07 = 7,
    DMAChannel08 = 8,
    DMAChannel09 = 9,
    DMAChannel10 = 10,
    DMAChannel11 = 11,
    DMAChannel12 = 12,
    DMAChannel13 = 13,
    DMAChannel14 = 14,
    DMAChannel15 = 15
}


/// The peripheral function a channel is assigned to (Section 16.7)
/// *_RX means transfer data from peripheral to memory, *_TX means transfer data
/// from memory to peripheral.
#[allow(non_camel_case_types)]
pub enum DMAPeripheral {
    USART0_RX      = 0,
    USART1_RX      = 1,
    USART2_RX      = 2,
    USART3_RX      = 3,
    SPI_RX         = 4,
    TWIM0_RX       = 5,
    TWIM1_RX       = 6,
    TWIM2_RX       = 7,
    TWIM3_RX       = 8,
    TWIS0_RX       = 9,
    TWIS1_RX       = 10,
    ADCIFE_RX      = 11,
    CATB_RX        = 12,
    IISC_CH0_RX    = 14,
    IISC_CH1_RX    = 15,
    PARC_RX        = 16,
    AESA_RX        = 17,
    USART0_TX      = 18,
    USART1_TX      = 19,
    USART2_TX      = 20,
    USART3_TX      = 21,
    SPI_TX         = 22,
    TWIM0_TX       = 23,
    TWIM1_TX       = 24,
    TWIM2_TX       = 25,
    TWIM3_TX       = 26,
    TWIS0_TX       = 27,
    TWIS1_TX       = 28,
    ADCIFE_TX      = 29,
    CATB_TX        = 30,
    ABDACB_SDR0_TX = 31,
    ABDACB_SDR1_TX = 32,
    IISC_CH0_TX    = 33,
    IISC_CH1_TX    = 34,
    DACC_TX        = 35,
    AESA_TX        = 36,
    LCDCA_ACMDR_TX = 37,
    LCDCA_ABMDR_TX = 38
}

pub struct DMAChannel {
    registers: &'static mut DMARegisters,
    client: Option<usize>,
    enabled: bool,
}

impl DMAChannel {
    pub fn new(channel: DMAChannelNum) -> DMAChannel {
        let address = DMA_BASE_ADDR + (channel as usize) * DMA_CHANNEL_SIZE;
        DMAChannel {
            registers: unsafe { mem::transmute(address) },
            client: None,
            enabled: false
        }
    }

    pub fn enable(&mut self) {
        unsafe {
            pm::enable_clock(pm::Clock::HSB(pm::HSBClock::PDCA));
            pm::enable_clock(pm::Clock::PBB(pm::PBBClock::PDCA));
        }
        if !self.enabled {
            unsafe {
                let num_enabled = intrinsics::atomic_xadd(&mut NUM_ENABLED, 1);
                if num_enabled == 1 {
                    pm::enable_clock(pm::Clock::HSB(pm::HSBClock::PDCA));
                    pm::enable_clock(pm::Clock::PBB(pm::PBBClock::PDCA));
                }
            }
            self.enabled = true;
        }
    }

    pub fn disable(&mut self) {
        if self.enabled {
            unsafe {
                let num_enabled = intrinsics::atomic_xsub(&mut NUM_ENABLED, 1);
                if num_enabled == 1 {
                    pm::disable_clock(pm::Clock::HSB(pm::HSBClock::PDCA));
                    pm::disable_clock(pm::Clock::PBB(pm::PBBClock::PDCA));
                }
            }
            volatile_store(&mut self.registers.control, 0x2);
            self.enabled = false;
        }
    }

    #[inline(never)]
    pub fn do_xfer<S>(&mut self, pid: usize, slice: AppSlice<S, u8>) {
        volatile_store(&mut self.registers.peripheral_select, pid);
        volatile_store(&mut self.registers.memory_address,
                       &slice.as_ref()[0] as *const u8 as usize);
        volatile_store(&mut self.registers.transfer_counter, slice.len());
        volatile_store(&mut self.registers.control, 0x1);
    }
}


use crate::memory::{InterruptManager, InterruptType};
use crate::GameboyConfig;
use bitflags::bitflags;
use save_state::impl_savable;
use serde::{Deserialize, Serialize};

pub trait SerialDevice {
    /// A device implemnts this, when receiving a call from this function will
    /// send a bit (return) and get a bit from the sender (`bit` argument)
    fn exchange_bit_external_clock(&mut self, bit: bool) -> bool;
}

bitflags! {
    #[derive(Default, Serialize, Deserialize)]
    struct SerialControl: u8 {
        const IN_TRANSFER  = 1 << 7;
        const CLOCK_SPEED  = 1 << 1;
        const CLOCK_SOURCE = 1 << 0;
    }
}

impl SerialControl {
    fn is_internal_clock(&self) -> bool {
        self.contains(Self::CLOCK_SOURCE)
    }

    fn in_transfer(&self) -> bool {
        self.contains(Self::IN_TRANSFER)
    }

    fn end_transfere(&mut self) {
        self.set(Self::IN_TRANSFER, false);
    }

    /// Which bit in `internal_timer` should be examined to trigger a serial
    /// clock, the clock is given on the falling (negative) edge
    fn clock_bit(&self) -> u8 {
        if self.contains(Self::CLOCK_SPEED) {
            // Fast
            1
        } else {
            // Normal
            6
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Serial {
    serial_control: SerialControl,
    transfere_data: u8,
    bits_remaining: u8,
    pub internal_timer: u8,
    config: GameboyConfig,
}

impl Serial {
    pub fn new(config: GameboyConfig) -> Self {
        Self {
            serial_control: SerialControl::from_bits_truncate(0),
            transfere_data: 0,
            bits_remaining: 0,
            internal_timer: 2,
            config,
        }
    }

    pub fn new_skip_boot_rom(config: GameboyConfig) -> Self {
        Self {
            /// FIXME: the internal_timer is not constant for CGB games
            ///  This is done temporary for testing, as testing properly should
            ///  use the bootrom
            internal_timer: if config.is_dmg { 0xF3 } else { 0 },
            ..Self::new(config)
        }
    }

    pub fn read_data(&self) -> u8 {
        self.transfere_data
    }

    pub fn write_data(&mut self, data: u8) {
        self.transfere_data = data
    }

    pub fn read_control(&self) -> u8 {
        0x7E | self.serial_control.bits()
    }

    pub fn write_control(&mut self, mut data: u8) {
        if self.config.is_dmg {
            // The clock speed parameter is not available in DMG
            data &= 0x81;
        }

        self.serial_control
            .clone_from(&SerialControl::from_bits_truncate(data));
        // should start transfere
        if self.serial_control.in_transfer() {
            self.bits_remaining = 8;
        }
    }

    /// Clocks the serial and will return `Some` if a bit should be sent,
    /// the bus must call `receive_bit` afterwards if there is a device.
    /// 1 is received automatically if `receive_bit` is not called
    pub fn clock_for_bit<I: InterruptManager>(&mut self, interrupt: &mut I) -> Option<bool> {
        let old_bit = (self.internal_timer >> self.serial_control.clock_bit()) & 1 == 1;
        self.internal_timer = self.internal_timer.wrapping_add(1);
        let new_bit = (self.internal_timer >> self.serial_control.clock_bit()) & 1 == 1;
        let can_clock = old_bit && !new_bit;

        if can_clock && self.bits_remaining > 0 && self.serial_control.is_internal_clock() {
            let out = self.transfere_data & 0x80 != 0;
            self.transfere_data = self.transfere_data.wrapping_shl(1);

            // data received from the other side, 1 for now meaning its
            // disconnected
            self.transfere_data |= 1;

            self.bits_remaining -= 1;

            if self.bits_remaining == 0 {
                self.serial_control.end_transfere();
                interrupt.request_interrupt(InterruptType::Serial);
            }

            Some(out)
        } else {
            // transfere should not complete as there is no external clock
            // support for now
            //
            // TODO: implement external transfere using interet or something
            None
        }
    }

    pub fn receive_bit(&mut self, bit: bool) {
        // we cannot receive from this method unless we are the master clock
        assert!(self.serial_control.is_internal_clock());

        // clear lowest bit
        self.transfere_data &= !1;
        self.transfere_data |= bit as u8;
    }
}

impl_savable!(Serial, 64);

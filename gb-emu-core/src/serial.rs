use crate::memory::{InterruptManager, InterruptType};
use bitflags::bitflags;

bitflags! {
    #[derive(Default)]
    struct SerialControl: u8 {
        const IN_TRANSFER  = 1 << 7;
        const CLOCK_SPEED  = 1 << 1;
        const CLOCK_SOURCE = 1 << 0;
    }
}

impl SerialControl {
    #[allow(dead_code)]
    fn is_internal_clock(&self) -> bool {
        self.contains(Self::CLOCK_SOURCE)
    }

    fn in_transfer(&self) -> bool {
        self.contains(Self::IN_TRANSFER)
    }

    fn end_transfere(&mut self) {
        self.set(Self::IN_TRANSFER, false);
    }

    /// Number of cycles to wait before 1-bit transfere
    fn clock_reload(&self) -> u8 {
        if self.contains(Self::CLOCK_SPEED) {
            // Fast
            4
        } else {
            // Normal
            128
        }
    }
}

#[derive(Default)]
pub struct Serial {
    serial_control: SerialControl,
    transfere_data: u8,
    transfere_timer: u8,
    bits_remaining: u8,
}

impl Serial {
    pub fn read_data(&self) -> u8 {
        0
    }

    pub fn write_data(&mut self, data: u8) {
        self.transfere_data = data
    }

    pub fn read_control(&self) -> u8 {
        0x7E | self.serial_control.bits()
    }

    pub fn write_control(&mut self, data: u8) {
        self.serial_control
            .clone_from(&SerialControl::from_bits_truncate(data));
        // should start transfere
        if self.serial_control.in_transfer() {
            self.transfere_timer = self.serial_control.clock_reload();
            self.bits_remaining = 8;
        }
    }

    pub fn clock<I: InterruptManager>(&mut self, interrupt: &mut I) {
        if self.bits_remaining > 0 {
            if self.serial_control.is_internal_clock() {
                self.transfere_timer = self.transfere_timer.saturating_sub(1);

                if self.transfere_timer == 0 {
                    self.transfere_data = self.transfere_data.wrapping_shl(1);

                    // data received from the other side, 1 for now meaning its
                    // disconnected
                    self.transfere_data |= 1;

                    self.bits_remaining -= 1;

                    if self.bits_remaining == 0 {
                        self.serial_control.end_transfere();
                        interrupt.request_interrupt(InterruptType::Serial);
                    }
                }
            } else {
                // transfere should not complete as there is no external clock
                // support for now
                //
                // TODO: implement external transfere using interet or something
            }
        }
    }
}

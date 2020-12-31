use crate::memory::{InterruptManager, InterruptType};
use bitflags::bitflags;

bitflags! {
    #[derive(Default)]
    struct SerialControl: u8 {
        const IN_TRANSFERE = 1 << 7;
        const CLOCK_SOURCE    = 1 << 0;
    }
}

impl SerialControl {
    #[allow(dead_code)]
    fn is_internal_clock(&self) -> bool {
        self.contains(Self::CLOCK_SOURCE)
    }

    fn in_transfere(&self) -> bool {
        self.contains(Self::IN_TRANSFERE)
    }

    fn end_transfere(&mut self) {
        self.set(Self::IN_TRANSFERE, false);
    }
}

/// Number of cycles to wait before 1-bit transfere (DMG only)
const SERIAL_TIMER_RELOAD: u8 = 128;

#[derive(Default)]
pub struct Serial {
    serial_control: SerialControl,
    transfere_data: u8,
    transfere_timer: u8,
    bits_remaining: u8,
}

impl Serial {
    pub fn read_register(&mut self, addr: u16) -> u8 {
        match addr {
            0xFF01 => 0,
            0xFF02 => 0x7E | self.serial_control.bits(),
            _ => unreachable!(),
        }
    }

    pub fn write_register(&mut self, addr: u16, data: u8) {
        match addr {
            0xFF01 => self.transfere_data = data,
            0xFF02 => {
                self.serial_control
                    .clone_from(&SerialControl::from_bits_truncate(data));
                // should start transfere
                if self.serial_control.in_transfere() {
                    self.transfere_timer = SERIAL_TIMER_RELOAD;
                    self.bits_remaining = 8;
                }
            }
            _ => unreachable!(),
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

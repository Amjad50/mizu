use crate::memory::{InterruptManager, InterruptType};
use bitflags::bitflags;

bitflags! {
    #[derive(Default)]
    struct TimerControl: u8 {
        const TIMER_ENABLE = 1 <<  2;
        const FREQ_DIVIDER = 0b11;
    }
}

impl TimerControl {
    fn timer_enabled(&self) -> bool {
        self.intersects(Self::TIMER_ENABLE)
    }

    fn freq_divider(&self) -> u16 {
        match self.bits() & Self::FREQ_DIVIDER.bits {
            0 => 1024,
            1 => 16,
            2 => 64,
            3 => 256,
            _ => unreachable!(),
        }
    }
}

#[derive(Default)]
pub struct Timer {
    timer_counter: u8,
    timer_modulo: u8,
    timer_offset: u16,
    timer_control: TimerControl,
}

impl Timer {
    pub fn read_register(&self, addr: u16) -> u8 {
        match addr {
            0xFF05 => self.timer_counter,
            0xFF06 => self.timer_modulo,
            0xFF07 => self.timer_control.bits(),
            _ => unreachable!(),
        }
    }

    pub fn write_register(&mut self, addr: u16, data: u8) {
        match addr {
            0xFF05 => self.timer_counter = data,
            0xFF06 => self.timer_modulo = data,
            0xFF07 => self
                .timer_control
                .clone_from(&TimerControl::from_bits_truncate(data)),
            _ => unreachable!(),
        }
    }

    pub fn clock_timer<I: InterruptManager>(&mut self, interrupt: &mut I) {
        if self.timer_control.timer_enabled() {
            self.timer_offset += 4;

            if self.timer_offset == self.timer_control.freq_divider() {
                self.timer_offset = 0;
                let (new_counter, overflow) = self.timer_counter.overflowing_add(1);

                if overflow {
                    self.timer_counter = self.timer_modulo;
                    // generate interrupt
                    interrupt.request_interrupt(InterruptType::Timer);
                } else {
                    self.timer_counter = new_counter;
                }
            }
        }
    }
}

#[derive(Default)]
pub struct Divider(u16);

impl Divider {
    pub fn reset(&mut self) {
        self.0 = 0xABCC;
    }
    pub fn clock_divider(&mut self) {
        // because each CPU M-cycle is 4 T-cycles
        self.0 = self.0.wrapping_add(4);
    }

    pub fn read_divider(&mut self) -> u8 {
        (self.0 >> 8) as u8
    }

    /// The value to write to the divider does not matter as
    /// it will be reset to 0
    pub fn write_divider(&mut self) {
        self.0 = 0;
    }
}

use crate::memory::{InterruptManager, InterruptType};
use crate::GameboyConfig;
use bitflags::bitflags;
use serde::{Deserialize, Serialize};

bitflags! {
    #[derive(Default, Serialize, Deserialize)]
    struct TimerControl: u8 {
        const TIMER_ENABLE = 1 <<  2;
        const FREQ_DIVIDER = 0b11;
    }
}

impl TimerControl {
    fn timer_enabled(&self) -> bool {
        self.intersects(Self::TIMER_ENABLE)
    }

    fn freq_divider_selection_bit(&self) -> u16 {
        // which bit to check for falling edge when incrementing
        match self.bits() & Self::FREQ_DIVIDER.bits {
            0 => 9,
            1 => 3,
            2 => 5,
            3 => 7,
            _ => unreachable!(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Timer {
    divider: u16,
    timer_counter: u8,
    timer_reload: u8,
    timer_control: TimerControl,
    interrupt_next: bool,
    during_interrupt: bool,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            divider: 0x0008, // divider value if boot_rom is present
            timer_counter: 0,
            timer_reload: 0,
            timer_control: TimerControl::from_bits_truncate(0),
            interrupt_next: false,
            during_interrupt: false,
        }
    }
}

impl Timer {
    pub fn new_skip_boot_rom(config: GameboyConfig) -> Self {
        Self {
            divider: if config.is_dmg { 0xABCC } else { 0x2678 }, // divider value after the boot_rom finish executing
            ..Self::default()
        }
    }

    pub fn read_div(&self) -> u8 {
        (self.divider >> 8) as u8
    }

    pub fn write_div(&mut self, _data: u8) {
        let old_divider_bit = self.divider_bit();
        self.divider = 0; // reset
        let new_divider_bit = self.divider_bit();

        if old_divider_bit && !new_divider_bit {
            self.increment_timer();
        }
    }

    pub fn read_timer_counter(&self) -> u8 {
        self.timer_counter
    }

    pub fn write_timer_counter(&mut self, data: u8) {
        // ignore timer reload and interrupt if there is an interrupt_next
        self.interrupt_next = false;

        // in the case this is the timer counter(TIMA) is reloaded
        // (and interrupt is triggered), then reload from the (TMA)
        // and ignore `data`
        self.timer_counter = if self.during_interrupt {
            self.timer_reload
        } else {
            data
        };
    }

    pub fn read_timer_reload(&self) -> u8 {
        self.timer_reload
    }

    pub fn write_timer_reload(&mut self, data: u8) {
        self.timer_reload = data;

        // if TMA is written during the same cycle it is reloaded into
        // the timer counter (TIMA), then reload TIMA as well
        if self.during_interrupt {
            self.timer_counter = self.timer_reload;
        }
    }

    pub fn read_control(&self) -> u8 {
        self.timer_control.bits() | 0xF8
    }

    pub fn write_control(&mut self, data: u8) {
        let old_enable = self.timer_control.timer_enabled();
        let old_divider_bit = old_enable && self.divider_bit();

        self.timer_control
            .clone_from(&TimerControl::from_bits_truncate(data));

        let new_enable = self.timer_control.timer_enabled();
        let new_divider_bit = new_enable && self.divider_bit();

        if old_divider_bit && !new_divider_bit {
            self.increment_timer();
        }
    }

    pub fn clock_divider<I: InterruptManager>(&mut self, interrupt: &mut I) {
        self.during_interrupt = false;

        if self.interrupt_next {
            interrupt.request_interrupt(InterruptType::Timer);
            self.interrupt_next = false;
            self.timer_counter = self.timer_reload;
            self.during_interrupt = true;
        }

        let old_divider_bit = self.divider_bit();

        // because each CPU M-cycle is 4 T-cycles
        self.divider = self.divider.wrapping_add(4);

        let new_divider_bit = self.divider_bit();

        if self.timer_control.timer_enabled() && old_divider_bit && !new_divider_bit {
            self.increment_timer();
        }
    }
}

impl Timer {
    fn increment_timer(&mut self) {
        let (new_counter, overflow) = self.timer_counter.overflowing_add(1);

        self.timer_counter = new_counter;
        self.interrupt_next = overflow;
    }

    fn divider_bit(&self) -> bool {
        let bit = self.timer_control.freq_divider_selection_bit();
        (self.divider >> bit) & 1 == 1
    }
}

impl_savable!(Timer, 32);

use bitflags::bitflags;
use save_state::Savable;

use std::convert::From;

use crate::memory::{InterruptManager, InterruptType};

pub enum JoypadButton {
    Start,
    Select,
    B,
    A,
    Down,
    Up,
    Left,
    Right,
}

bitflags! {
    #[derive(Default)]
    struct JoypadButtons:u8 {
        const START  = 1 << 7;
        const SELECT = 1 << 6;
        const B      = 1 << 5;
        const A      = 1 << 4;
        const DOWN   = 1 << 3;
        const UP     = 1 << 2;
        const LEFT   = 1 << 1;
        const RIGHT  = 1 << 0;
    }
}

impl From<JoypadButton> for JoypadButtons {
    fn from(button: JoypadButton) -> Self {
        match button {
            JoypadButton::Start => Self::START,
            JoypadButton::Select => Self::SELECT,
            JoypadButton::B => Self::B,
            JoypadButton::A => Self::A,
            JoypadButton::Down => Self::DOWN,
            JoypadButton::Up => Self::UP,
            JoypadButton::Left => Self::LEFT,
            JoypadButton::Right => Self::RIGHT,
        }
    }
}

#[derive(Savable)]
pub struct Joypad {
    #[savable(skip)]
    buttons: JoypadButtons,
    selecting_directions: bool,
    selecting_start: bool,

    old_p1: u8,
}

impl Default for Joypad {
    fn default() -> Self {
        Self {
            buttons: Default::default(),
            selecting_directions: true,
            selecting_start: true,
            old_p1: 0,
        }
    }
}

impl Joypad {
    /// returns the lower 4 bits of P1 (joypad register)
    pub fn get_keys_pressed(&self) -> u8 {
        let mut result = 0xF;

        if self.selecting_start {
            result &= !self.buttons.bits() >> 4;
        }
        if self.selecting_directions {
            result &= !self.buttons.bits();
        }

        result
    }

    pub fn read_joypad(&self) -> u8 {
        let result = self.get_keys_pressed();

        0xC0 | (((!self.selecting_start) as u8) << 5)
            | ((!self.selecting_directions as u8) << 4)
            | result
    }

    pub fn write_joypad(&mut self, data: u8) {
        self.selecting_start = ((data >> 5) & 1) == 0;
        self.selecting_directions = ((data >> 4) & 1) == 0;
    }

    pub fn update_interrupts<I: InterruptManager>(&mut self, interrupt: &mut I) {
        let new_p1 = self.get_keys_pressed();

        let should_interrupt = (self.old_p1 ^ new_p1) & self.old_p1 != 0;

        if should_interrupt {
            interrupt.request_interrupt(InterruptType::Joypad);
        }

        self.old_p1 = new_p1;
    }

    pub fn press_joypad(&mut self, button: JoypadButton) {
        self.buttons.insert(button.into())
    }

    pub fn release_joypad(&mut self, button: JoypadButton) {
        self.buttons.remove(button.into())
    }
}

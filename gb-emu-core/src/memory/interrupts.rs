use bitflags::bitflags;
use std::convert::From;

const INTERRUPTS_VECTOR: [u8; 5] = [0x40, 0x48, 0x50, 0x58, 0x60];

pub enum InterruptType {
    Vblank,
    LcdStat,
    Timer,
    Serial,
    Joypad,
}

pub trait InterruptManager {
    fn request_interrupt(&mut self, interrupt: InterruptType);
}

bitflags! {
    struct InterruptsFlags: u8 {
        const VBLANK   = 1 << 0;
        const LCD_STAT = 1 << 1;
        const TIMER    = 1 << 2;
        const SERIAL   = 1 << 3;
        const JOYPAD   = 1 << 4;
    }
}

impl From<InterruptType> for InterruptsFlags {
    fn from(interrupt: InterruptType) -> Self {
        match interrupt {
            InterruptType::Vblank => Self::VBLANK,
            InterruptType::LcdStat => Self::LCD_STAT,
            InterruptType::Timer => Self::TIMER,
            InterruptType::Serial => Self::SERIAL,
            InterruptType::Joypad => Self::JOYPAD,
        }
    }
}

pub struct Interrupts {
    enabled: InterruptsFlags,
    requested: InterruptsFlags,
}

impl Default for Interrupts {
    fn default() -> Self {
        Self {
            enabled: InterruptsFlags::from_bits_truncate(0),
            requested: InterruptsFlags::from_bits_truncate(0),
        }
    }
}

impl Interrupts {
    pub fn write_interrupt_enable(&mut self, data: u8) {
        self.enabled
            .clone_from(&InterruptsFlags::from_bits_truncate(data));
    }

    pub fn read_interrupt_enable(&self) -> u8 {
        self.enabled.bits() | 0xE0
    }

    pub fn write_interrupt_flags(&mut self, data: u8) {
        self.requested
            .clone_from(&InterruptsFlags::from_bits_truncate(data));
    }

    pub fn read_interrupt_flags(&self) -> u8 {
        self.requested.bits() | 0xE0
    }

    pub fn is_interrupts_available(&self) -> bool {
        self.requested.bits() & self.enabled.bits() != 0
    }

    pub fn get_highest_interrupt_addr_and_ack(&mut self) -> Option<u8> {
        if self.requested.is_empty() {
            None
        } else {
            let mut bits = self.requested.bits();
            let mut counter = 0;
            while bits != 0 {
                if bits & 1 == 1 {
                    self.requested
                        .remove(InterruptsFlags::from_bits_truncate(1 << counter));

                    return Some(INTERRUPTS_VECTOR[counter]);
                }
                counter += 1;
                bits >>= 1;
            }
            None
        }
    }
}

impl InterruptManager for Interrupts {
    fn request_interrupt(&mut self, interrupt: InterruptType) {
        let interrupt = interrupt.into();

        if self.enabled.intersects(interrupt) {
            self.requested.insert(interrupt);
        }
    }
}

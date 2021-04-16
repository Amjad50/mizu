use bitflags::bitflags;
use save_state::Savable;
use std::convert::{From, TryFrom};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum InterruptType {
    Vblank,
    LcdStat,
    Timer,
    Serial,
    Joypad,
}

impl TryFrom<u8> for InterruptType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Vblank),
            1 => Ok(Self::LcdStat),
            2 => Ok(Self::Timer),
            3 => Ok(Self::Serial),
            4 => Ok(Self::Joypad),
            _ => Err(()),
        }
    }
}

pub trait InterruptManager {
    fn request_interrupt(&mut self, interrupt: InterruptType);
}

bitflags! {
    #[derive(Savable)]
    struct InterruptsFlags: u8 {
        /// This is only used when reading `interrupt_enable` only
        const UNUSED   = 0b111 << 5;
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

#[derive(Savable)]
pub struct Interrupts {
    enabled: InterruptsFlags,
    requested: InterruptsFlags,
}

impl Default for Interrupts {
    fn default() -> Self {
        Self {
            enabled: InterruptsFlags::from_bits_truncate(0),
            requested: InterruptsFlags::from_bits_truncate(1),
        }
    }
}

impl Interrupts {
    pub fn write_interrupt_enable(&mut self, data: u8) {
        self.enabled
            .clone_from(&InterruptsFlags::from_bits_truncate(data));
    }

    pub fn read_interrupt_enable(&self) -> u8 {
        self.enabled.bits()
    }

    pub fn write_interrupt_flags(&mut self, data: u8) {
        self.requested
            .clone_from(&InterruptsFlags::from_bits_truncate(data));
    }

    pub fn read_interrupt_flags(&self) -> u8 {
        0xE0 | self.requested.bits()
    }

    pub fn acknowledge_interrupt(&mut self, interrupt: InterruptType) {
        assert!(self.requested.contains(interrupt.into()));

        self.requested.remove(interrupt.into());
    }

    pub fn get_highest_interrupt(&mut self) -> Option<InterruptType> {
        let interrupts_to_take_bits = self.requested.bits() & self.enabled.bits() & 0x1F;

        if interrupts_to_take_bits == 0 {
            None
        } else {
            let mut bits = interrupts_to_take_bits;
            let mut counter = 0;
            while bits != 0 && counter < 5 {
                if bits & 1 == 1 {
                    return Some(InterruptType::try_from(counter).unwrap());
                }
                counter += 1;
                bits >>= 1;
            }
            unreachable!();
        }
    }
}

impl InterruptManager for Interrupts {
    fn request_interrupt(&mut self, interrupt: InterruptType) {
        let interrupt = interrupt.into();

        self.requested.insert(interrupt);
    }
}

mod cpu;
pub mod instruction;
mod instructions_table;

use crate::memory::InterruptType;
pub use cpu::{Cpu, CpuRegisters, CpuState};

pub trait CpuBusProvider {
    fn read(&mut self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, data: u8);

    fn take_next_interrupt(&mut self) -> Option<InterruptType>;
    fn peek_next_interrupt(&mut self) -> Option<InterruptType>;
    fn check_interrupts(&self) -> bool;
}

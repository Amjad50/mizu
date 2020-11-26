mod cpu;
pub mod instruction;
mod instructions_table;

pub use cpu::{Cpu, CpuRegisters, CpuState};

pub trait CpuBusProvider {
    fn read(&mut self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, data: u8);

    fn get_interrupts(&mut self) -> Option<u8>;
    fn check_interrupts(&self) -> bool;
}

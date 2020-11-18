mod cpu;
pub mod instruction;
mod instructions_table;

pub use cpu::Cpu;

pub trait CpuBusProvider {
    fn read(&mut self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, data: u8);

    fn check_interrupts(&mut self) -> bool;
    fn ack_interrupt(&mut self);
}

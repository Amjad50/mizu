pub mod instruction;
mod instructions_table;

use bitflags::bitflags;
use save_state::Savable;

use crate::memory::InterruptType;
use crate::GameboyConfig;
use instruction::{Condition, Instruction, Opcode, OperandType};

pub trait CpuBusProvider {
    fn read(&mut self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, data: u8);

    fn take_next_interrupt(&mut self) -> Option<InterruptType>;
    fn peek_next_interrupt(&mut self) -> Option<InterruptType>;

    fn is_hdma_running(&mut self) -> bool;

    fn enter_stop_mode(&mut self);
    fn stopped(&self) -> bool;

    /// Triggers oam_bug without clock, this is used for inc/dec instructions
    fn trigger_write_oam_bug(&mut self, addr: u16);
    /// Triggers oam_bug special case read_write, this happens when an
    /// increment/decrement happen in the same cycle as read
    fn trigger_read_write_oam_bug(&mut self, addr: u16);
    /// reads data without triggering oam_bug, this is used in pop
    fn read_no_oam_bug(&mut self, addr: u16) -> u8;
}

const INTERRUPTS_VECTOR: [u16; 5] = [0x40, 0x48, 0x50, 0x58, 0x60];

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CpuRegisters {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CpuState {
    Normal,
    InfiniteLoop,
    Halting,
    RunningHDMA,
    Stopped,
    RunningInterrupt(InterruptType),
    Breakpoint(CpuRegisters),
}

bitflags! {
    #[derive(Savable)]
    struct CpuFlags: u8 {
        const Z = 1 << 7;
        const N = 1 << 6;
        const H = 1 << 5;
        const C = 1 << 4;
    }
}

#[derive(Savable, PartialEq)]
enum HaltMode {
    NotHalting,
    HaltRunInterrupt,
    HaltNoRunInterrupt,
    HaltBug,
}

#[derive(Savable)]
pub struct Cpu {
    reg_a: u8,
    reg_b: u8,
    reg_c: u8,
    reg_d: u8,
    reg_e: u8,
    reg_h: u8,
    reg_l: u8,
    reg_f: CpuFlags,

    reg_sp: u16,

    reg_pc: u16,

    enable_interrupt_next: bool,
    ime: bool,
    halt_mode: HaltMode,

    config: GameboyConfig,
}

impl Cpu {
    pub fn new(config: GameboyConfig) -> Self {
        Self {
            reg_a: 0,
            reg_b: 0,
            reg_c: 0,
            reg_d: 0,
            reg_e: 0,
            reg_h: 0,
            reg_l: 0,
            reg_f: CpuFlags::from_bits_truncate(0),
            reg_sp: 0,
            reg_pc: 0,

            enable_interrupt_next: false,
            ime: false,
            halt_mode: HaltMode::NotHalting,

            config,
        }
    }

    /// create a new cpu, with states that match the ones the CPU would have
    /// if the boot-rom would run (default values for registers)
    pub fn new_without_boot_rom(config: GameboyConfig, is_cart_cgb: bool) -> Self {
        let mut cpu = Self::new(config);

        if cpu.config.is_dmg {
            // initial values of the registers (DMG)
            cpu.reg_af_write(0x01B0);
            cpu.reg_bc_write(0x0013);
            cpu.reg_de_write(0x00D8);
            cpu.reg_hl_write(0x014D);
        } else {
            // initial values of the registers (CGB) for CGB games
            // common
            cpu.reg_af_write(0x1180);
            cpu.reg_bc_write(0x0000);
            if is_cart_cgb {
                cpu.reg_de_write(0xFF56);
                cpu.reg_hl_write(0x000D);
            } else {
                // initial values of the registers (CGB) for DMG games
                cpu.reg_de_write(0x0008);
                cpu.reg_hl_write(0x007C);
            }
        }
        cpu.reg_sp = 0xFFFE;
        cpu.reg_pc = 0x0100;

        cpu
    }

    pub fn next_instruction<P: CpuBusProvider>(&mut self, bus: &mut P) -> CpuState {
        if bus.stopped() {
            self.advance_bus(bus);
            return CpuState::Stopped;
        }

        if bus.is_hdma_running() {
            self.advance_bus(bus);
            return CpuState::RunningHDMA;
        }

        if self.halt_mode == HaltMode::HaltRunInterrupt
            || self.halt_mode == HaltMode::HaltNoRunInterrupt
        {
            self.advance_bus(bus);

            if bus.peek_next_interrupt().is_some() {
                self.halt_mode = HaltMode::NotHalting;

                if !self.config.is_dmg {
                    self.advance_bus(bus);
                }
            } else {
                return CpuState::Halting;
            }
        }

        if self.ime && bus.peek_next_interrupt().is_some() {
            let mut cpu_state = CpuState::Normal;

            let pc = self.reg_pc;

            // Push PC part 1
            // trigger write oam bug because of the increment
            bus.trigger_write_oam_bug(self.reg_sp);
            self.reg_sp = self.reg_sp.wrapping_sub(1);
            bus.write(self.reg_sp, (pc >> 8) as u8);

            if let Some(int_type) = bus.take_next_interrupt() {
                cpu_state = CpuState::RunningInterrupt(int_type);
                self.reg_pc = INTERRUPTS_VECTOR[int_type as usize];
            } else {
                // Interrupt cancelled
                self.reg_pc = 0;
            }

            self.ime = false;

            // Push PC part 2
            self.reg_sp = self.reg_sp.wrapping_sub(1);
            bus.write(self.reg_sp, pc as u8);

            // delay for interrupt
            self.advance_bus(bus);
            self.advance_bus(bus);
            self.advance_bus(bus);
            return cpu_state;
        }

        if self.enable_interrupt_next {
            self.ime = true;
            self.enable_interrupt_next = false;
        }

        let pc = self.reg_pc;
        let mut instruction = Instruction::from_byte(self.fetch_next_pc(bus), pc);

        if self.halt_mode == HaltMode::HaltBug {
            self.halt_mode = HaltMode::NotHalting;
            // do not add pc from the last fetch ^
            self.reg_pc = pc;
        }

        if instruction.opcode == Opcode::Prefix {
            instruction = Instruction::from_prefix(self.fetch_next_pc(bus), pc);
        }

        self.exec_instruction(instruction, bus)
    }
}

impl Cpu {
    #[inline]
    fn reg_af_read(&self) -> u16 {
        (self.reg_a as u16) << 8 | self.reg_f.bits() as u16
    }

    #[inline]
    fn reg_bc_read(&self) -> u16 {
        (self.reg_b as u16) << 8 | self.reg_c as u16
    }

    #[inline]
    fn reg_de_read(&self) -> u16 {
        (self.reg_d as u16) << 8 | self.reg_e as u16
    }

    #[inline]
    fn reg_hl_read(&self) -> u16 {
        (self.reg_h as u16) << 8 | self.reg_l as u16
    }

    #[inline]
    fn reg_af_write(&mut self, data: u16) {
        self.reg_a = (data >> 8) as u8;
        self.reg_f
            .clone_from(&CpuFlags::from_bits_truncate(data as u8));
    }

    #[inline]
    fn reg_bc_write(&mut self, data: u16) {
        self.reg_b = (data >> 8) as u8;
        self.reg_c = data as u8;
    }

    #[inline]
    fn reg_de_write(&mut self, data: u16) {
        self.reg_d = (data >> 8) as u8;
        self.reg_e = data as u8;
    }

    #[inline]
    fn reg_hl_write(&mut self, data: u16) {
        self.reg_h = (data >> 8) as u8;
        self.reg_l = data as u8;
    }

    #[inline]
    fn flag_get(&self, flag: CpuFlags) -> bool {
        self.reg_f.intersects(flag)
    }

    #[inline]
    fn flag_set(&mut self, flag: CpuFlags, value: bool) {
        self.reg_f.set(flag, value);
    }

    fn registers(&self) -> CpuRegisters {
        CpuRegisters {
            a: self.reg_a,
            b: self.reg_b,
            c: self.reg_c,
            d: self.reg_d,
            e: self.reg_e,
            f: self.reg_f.bits(),
            h: self.reg_h,
            l: self.reg_l,
            sp: self.reg_sp,
            pc: self.reg_pc,
        }
    }

    fn fetch_next_pc<P: CpuBusProvider>(&mut self, bus: &mut P) -> u8 {
        let result = bus.read(self.reg_pc);
        bus.trigger_read_write_oam_bug(self.reg_pc);
        self.reg_pc = self.reg_pc.wrapping_add(1);
        result
    }

    fn read_operand<P: CpuBusProvider>(&mut self, ty: OperandType, bus: &mut P) -> u16 {
        match ty {
            OperandType::RegA => self.reg_a as u16,
            OperandType::RegB => self.reg_b as u16,
            OperandType::RegC => self.reg_c as u16,
            OperandType::RegD => self.reg_d as u16,
            OperandType::RegE => self.reg_e as u16,
            OperandType::RegH => self.reg_h as u16,
            OperandType::RegL => self.reg_l as u16,
            OperandType::AddrHL => bus.read(self.reg_hl_read()) as u16,
            OperandType::AddrHLDec => {
                let hl = self.reg_hl_read();
                let result = bus.read(hl) as u16;
                bus.trigger_read_write_oam_bug(hl);
                self.reg_hl_write(hl.wrapping_sub(1));
                result
            }
            OperandType::AddrHLInc => {
                let hl = self.reg_hl_read();
                let result = bus.read(hl) as u16;
                bus.trigger_read_write_oam_bug(hl);
                self.reg_hl_write(hl.wrapping_add(1));
                result
            }
            OperandType::AddrBC => bus.read(self.reg_bc_read()) as u16,
            OperandType::AddrDE => bus.read(self.reg_de_read()) as u16,
            OperandType::RegAF => self.reg_af_read(),
            OperandType::RegBC => self.reg_bc_read(),
            OperandType::RegDE => self.reg_de_read(),
            OperandType::RegHL => self.reg_hl_read(),
            OperandType::RegSP => self.reg_sp,
            OperandType::Imm8 => self.fetch_next_pc(bus) as u16,
            OperandType::Imm8Signed => self.fetch_next_pc(bus) as i8 as i16 as u16,
            OperandType::Imm16 => {
                (self.fetch_next_pc(bus) as u16) | ((self.fetch_next_pc(bus) as u16) << 8)
            }
            OperandType::HighAddr8 => {
                let addr = 0xFF00 | self.fetch_next_pc(bus) as u16;
                bus.read(addr) as u16
            }
            OperandType::HighAddrC => bus.read(0xFF00 | self.reg_c as u16) as u16,
            OperandType::Addr16 => {
                let addr =
                    (self.fetch_next_pc(bus) as u16) | ((self.fetch_next_pc(bus) as u16) << 8);
                bus.read(addr) as u16
            }
            OperandType::Implied => 0,
            OperandType::Addr16Val16 => unreachable!(),
        }
    }

    fn write_operand<P: CpuBusProvider>(&mut self, ty: OperandType, data: u16, bus: &mut P) {
        match ty {
            OperandType::RegA => self.reg_a = data as u8,
            OperandType::RegB => self.reg_b = data as u8,
            OperandType::RegC => self.reg_c = data as u8,
            OperandType::RegD => self.reg_d = data as u8,
            OperandType::RegE => self.reg_e = data as u8,
            OperandType::RegH => self.reg_h = data as u8,
            OperandType::RegL => self.reg_l = data as u8,
            OperandType::AddrHL => bus.write(self.reg_hl_read(), data as u8),
            OperandType::AddrHLDec => {
                let hl = self.reg_hl_read();
                bus.write(hl, data as u8);
                self.reg_hl_write(hl.wrapping_sub(1));
            }
            OperandType::AddrHLInc => {
                let hl = self.reg_hl_read();
                bus.write(hl, data as u8);
                self.reg_hl_write(hl.wrapping_add(1));
            }
            OperandType::AddrBC => bus.write(self.reg_bc_read(), data as u8),
            OperandType::AddrDE => bus.write(self.reg_de_read(), data as u8),
            OperandType::RegAF => self.reg_af_write(data),
            OperandType::RegBC => self.reg_bc_write(data),
            OperandType::RegDE => self.reg_de_write(data),
            OperandType::RegHL => self.reg_hl_write(data),
            OperandType::RegSP => self.reg_sp = data,
            OperandType::HighAddr8 => {
                let addr = 0xFF00 | self.fetch_next_pc(bus) as u16;
                bus.write(addr, data as u8);
            }
            OperandType::HighAddrC => bus.write(0xFF00 | self.reg_c as u16, data as u8),
            OperandType::Addr16 => {
                let addr =
                    (self.fetch_next_pc(bus) as u16) | ((self.fetch_next_pc(bus) as u16) << 8);
                bus.write(addr, data as u8);
            }
            OperandType::Addr16Val16 => {
                let addr =
                    (self.fetch_next_pc(bus) as u16) | ((self.fetch_next_pc(bus) as u16) << 8);
                bus.write(addr, data as u8);
                bus.write(addr.wrapping_add(1), (data >> 8) as u8);
            }
            OperandType::Implied => {}
            OperandType::Imm16 | OperandType::Imm8 | OperandType::Imm8Signed => unreachable!(),
        }
    }

    /// advances the bus and all other components by one machine cycle
    fn advance_bus<P: CpuBusProvider>(&mut self, bus: &mut P) {
        bus.read(0);
    }

    fn stack_push<P: CpuBusProvider>(&mut self, data: u16, bus: &mut P) {
        bus.trigger_write_oam_bug(self.reg_sp);
        self.reg_sp = self.reg_sp.wrapping_sub(1);
        bus.write(self.reg_sp, (data >> 8) as u8);
        self.reg_sp = self.reg_sp.wrapping_sub(1);
        bus.write(self.reg_sp, data as u8);
    }

    fn stack_pop<P: CpuBusProvider>(&mut self, bus: &mut P) -> u16 {
        let low = bus.read_no_oam_bug(self.reg_sp);
        // instead of triggering normal read oam bug, a glitch happen and
        // read and write (because of increment) oam bug happen at the same
        // cycle which produce a strange behaviour (impleented in read_write
        // oam bug)
        bus.trigger_read_write_oam_bug(self.reg_sp);

        self.reg_sp = self.reg_sp.wrapping_add(1);
        let high = bus.read(self.reg_sp);
        self.reg_sp = self.reg_sp.wrapping_add(1);

        ((high as u16) << 8) | low as u16
    }

    fn check_cond(&self, cond: Condition) -> bool {
        match cond {
            Condition::NC => !self.flag_get(CpuFlags::C),
            Condition::C => self.flag_get(CpuFlags::C),
            Condition::NZ => !self.flag_get(CpuFlags::Z),
            Condition::Z => self.flag_get(CpuFlags::Z),
            Condition::Unconditional => true,
        }
    }

    fn exec_instruction<P: CpuBusProvider>(
        &mut self,
        instruction: Instruction,
        bus: &mut P,
    ) -> CpuState {
        let src = self.read_operand(instruction.src, bus);

        let mut cpu_state = CpuState::Normal;

        let result = match instruction.opcode {
            Opcode::Nop => 0,
            Opcode::Ld => src,
            Opcode::LdBB => {
                // self.reg_b = self.reg_b;
                println!("Break point at {:04X} was hit", instruction.pc);

                cpu_state = CpuState::Breakpoint(self.registers());

                0
            }
            Opcode::LdSPHL => {
                self.advance_bus(bus);
                self.reg_sp = self.reg_hl_read();
                0
            }
            Opcode::LdHLSPSigned8 => {
                self.advance_bus(bus);
                let result = self.reg_sp.wrapping_add(src);

                self.flag_set(CpuFlags::Z, false);
                self.flag_set(CpuFlags::N, false);
                self.flag_set(CpuFlags::H, (self.reg_sp & 0xf) + (src & 0xf) > 0xf);
                self.flag_set(CpuFlags::C, (self.reg_sp & 0xff) + (src & 0xff) > 0xff);

                result
            }
            Opcode::Push => {
                self.advance_bus(bus);
                self.stack_push(src, bus);
                0
            }
            Opcode::Pop => self.stack_pop(bus),
            Opcode::Inc16 => {
                self.advance_bus(bus);
                bus.trigger_write_oam_bug(src);
                src.wrapping_add(1)
            }

            Opcode::Inc => {
                let result = src.wrapping_add(1) & 0xff;

                self.flag_set(CpuFlags::Z, result == 0);
                self.flag_set(CpuFlags::N, false);
                self.flag_set(CpuFlags::H, result & 0x0f == 0);

                result
            }
            Opcode::Dec16 => {
                self.advance_bus(bus);
                bus.trigger_write_oam_bug(src);
                src.wrapping_sub(1)
            }
            Opcode::Dec => {
                let result = src.wrapping_sub(1);
                self.flag_set(CpuFlags::Z, result == 0);
                self.flag_set(CpuFlags::N, true);
                self.flag_set(CpuFlags::H, result & 0x0f == 0x0f);
                result
            }
            Opcode::Add => {
                let dest = self.read_operand(instruction.dest, bus);
                let result = dest.wrapping_add(src);

                self.flag_set(CpuFlags::Z, result & 0xFF == 0);
                self.flag_set(CpuFlags::N, false);
                self.flag_set(CpuFlags::H, (dest & 0xf) + (src & 0xf) > 0xf);
                self.flag_set(CpuFlags::C, result & 0xff00 != 0);

                result & 0xFF
            }
            Opcode::Add16 => {
                self.advance_bus(bus);
                let dest = self.read_operand(instruction.dest, bus);
                let result = (dest as u32).wrapping_add(src as u32);

                self.flag_set(CpuFlags::N, false);
                self.flag_set(CpuFlags::H, (dest & 0xfff) + (src & 0xfff) > 0xfff);
                self.flag_set(CpuFlags::C, result & 0xffff0000 != 0);

                result as u16
            }
            Opcode::AddSPSigned8 => {
                self.advance_bus(bus);
                self.advance_bus(bus);
                let dest = self.read_operand(instruction.dest, bus);
                let result = dest.wrapping_add(src);

                self.flag_set(CpuFlags::Z, false);
                self.flag_set(CpuFlags::N, false);
                self.flag_set(CpuFlags::H, (dest & 0xf) + (src & 0xf) > 0xf);
                self.flag_set(CpuFlags::C, (dest & 0xff) + (src & 0xff) > 0xff);

                result
            }
            Opcode::Adc => {
                let dest = self.read_operand(instruction.dest, bus);
                let carry = self.flag_get(CpuFlags::C) as u16;
                let result = dest.wrapping_add(src).wrapping_add(carry);

                self.flag_set(CpuFlags::Z, result & 0xFF == 0);
                self.flag_set(CpuFlags::N, false);
                self.flag_set(CpuFlags::H, (dest & 0xf) + (src & 0xf) + carry > 0xf);
                self.flag_set(CpuFlags::C, result & 0xff00 != 0);

                result
            }
            Opcode::Sub => {
                let dest = self.read_operand(instruction.dest, bus);
                let result = dest.wrapping_sub(src);

                self.flag_set(CpuFlags::Z, result & 0xFF == 0);
                self.flag_set(CpuFlags::N, true);
                self.flag_set(CpuFlags::H, (dest & 0xf) < (src & 0xf));
                self.flag_set(CpuFlags::C, result & 0xff00 != 0);

                result & 0xFF
            }
            Opcode::Cp => {
                let dest = self.reg_a as u16;
                let result = dest.wrapping_sub(src);

                self.flag_set(CpuFlags::Z, result & 0xFF == 0);
                self.flag_set(CpuFlags::N, true);
                self.flag_set(CpuFlags::H, (dest & 0xf) < (src & 0xf));
                self.flag_set(CpuFlags::C, result & 0xff00 != 0);

                // will be ignored
                result & 0xFF
            }
            Opcode::Sbc => {
                let dest = self.read_operand(instruction.dest, bus);
                let carry = self.flag_get(CpuFlags::C) as u16;
                let result = dest.wrapping_sub(src).wrapping_sub(carry);

                self.flag_set(CpuFlags::Z, result & 0xFF == 0);
                self.flag_set(CpuFlags::N, true);
                self.flag_set(
                    CpuFlags::H,
                    (dest & 0xf).wrapping_sub(src & 0xf).wrapping_sub(carry) > 0xf,
                );
                self.flag_set(CpuFlags::C, result & 0xff00 != 0);

                result & 0xFF
            }
            Opcode::And => {
                let dest = self.read_operand(instruction.dest, bus);
                let result = dest & src & 0xff;

                self.flag_set(CpuFlags::Z, result == 0);
                self.flag_set(CpuFlags::N, false);
                self.flag_set(CpuFlags::H, true);
                self.flag_set(CpuFlags::C, false);

                result
            }
            Opcode::Xor => {
                let dest = self.read_operand(instruction.dest, bus);
                let result = (dest ^ src) & 0xff;

                self.flag_set(CpuFlags::Z, result == 0);
                self.flag_set(CpuFlags::N, false);
                self.flag_set(CpuFlags::H, false);
                self.flag_set(CpuFlags::C, false);

                result
            }
            Opcode::Or => {
                let dest = self.read_operand(instruction.dest, bus);
                let result = (dest | src) & 0xff;

                self.flag_set(CpuFlags::Z, result == 0);
                self.flag_set(CpuFlags::N, false);
                self.flag_set(CpuFlags::H, false);
                self.flag_set(CpuFlags::C, false);

                result
            }
            Opcode::Jp(cond) => {
                if self.check_cond(cond) {
                    self.advance_bus(bus);
                    if cond == Condition::Unconditional && src == instruction.pc {
                        cpu_state = CpuState::InfiniteLoop;
                    }

                    self.reg_pc = src;
                }
                0
            }
            Opcode::JpHL => {
                if src == instruction.pc {
                    cpu_state = CpuState::InfiniteLoop;
                }

                self.reg_pc = src;
                0
            }
            Opcode::Jr(cond) => {
                if self.check_cond(cond) {
                    self.advance_bus(bus);
                    let new_pc = self.reg_pc.wrapping_add(src);

                    if cond == Condition::Unconditional && new_pc == instruction.pc {
                        cpu_state = CpuState::InfiniteLoop;
                    }

                    self.reg_pc = new_pc;
                }
                0
            }
            Opcode::Call(cond) => {
                if self.check_cond(cond) {
                    self.advance_bus(bus);
                    self.stack_push(self.reg_pc, bus);
                    self.reg_pc = src;
                }
                0
            }
            Opcode::Ret(cond) => {
                if cond != Condition::Unconditional {
                    self.advance_bus(bus);
                }
                if self.check_cond(cond) {
                    self.reg_pc = self.stack_pop(bus);
                    self.advance_bus(bus);
                }
                0
            }
            Opcode::Reti => {
                self.reg_pc = self.stack_pop(bus);
                self.advance_bus(bus);
                self.ime = true;
                0
            }
            Opcode::Rst(loc) => {
                self.advance_bus(bus);
                self.stack_push(self.reg_pc, bus);
                self.reg_pc = loc as u16;
                0
            }
            Opcode::Di => {
                self.ime = false;
                0
            }
            Opcode::Ei => {
                self.enable_interrupt_next = true;
                0
            }
            Opcode::Ccf => {
                self.flag_set(CpuFlags::N, false);
                self.flag_set(CpuFlags::H, false);
                self.flag_set(CpuFlags::C, !self.flag_get(CpuFlags::C));
                0
            }
            Opcode::Scf => {
                self.flag_set(CpuFlags::N, false);
                self.flag_set(CpuFlags::H, false);
                self.flag_set(CpuFlags::C, true);
                0
            }
            Opcode::Daa => {
                let carry = self.flag_get(CpuFlags::C);
                let halfcarry = self.flag_get(CpuFlags::H);

                if !self.flag_get(CpuFlags::N) {
                    let mut correction = 0;
                    if halfcarry || (self.reg_a & 0xf > 0x9) {
                        correction |= 0x6;
                    }

                    if carry || (self.reg_a > 0x99) {
                        correction |= 0x60;
                        self.flag_set(CpuFlags::C, true);
                    }

                    self.reg_a = self.reg_a.wrapping_add(correction);
                } else if carry {
                    self.flag_set(CpuFlags::C, true);
                    self.reg_a = self.reg_a.wrapping_add(if halfcarry { 0x9a } else { 0xa0 });
                } else if halfcarry {
                    self.reg_a = self.reg_a.wrapping_add(0xfa);
                }

                self.flag_set(CpuFlags::Z, self.reg_a == 0);
                self.flag_set(CpuFlags::H, false);

                0
            }
            Opcode::Cpl => {
                self.reg_a = !self.reg_a;

                self.flag_set(CpuFlags::N, true);
                self.flag_set(CpuFlags::H, true);

                0
            }
            Opcode::Rlca => {
                let carry = (self.reg_a >> 7) & 1;
                self.reg_a = self.reg_a.wrapping_shl(1) | carry;

                self.flag_set(CpuFlags::Z, false);
                self.flag_set(CpuFlags::N, false);
                self.flag_set(CpuFlags::H, false);
                self.flag_set(CpuFlags::C, carry == 1);

                0
            }
            Opcode::Rla => {
                let carry = (self.reg_a >> 7) & 1;
                self.reg_a = self.reg_a.wrapping_shl(1) | self.flag_get(CpuFlags::C) as u8;

                self.flag_set(CpuFlags::Z, false);
                self.flag_set(CpuFlags::N, false);
                self.flag_set(CpuFlags::H, false);
                self.flag_set(CpuFlags::C, carry == 1);

                0
            }
            Opcode::Rrca => {
                let carry = self.reg_a & 1;
                self.reg_a = self.reg_a.wrapping_shr(1) | (carry << 7);

                self.flag_set(CpuFlags::Z, false);
                self.flag_set(CpuFlags::N, false);
                self.flag_set(CpuFlags::H, false);
                self.flag_set(CpuFlags::C, carry == 1);

                0
            }
            Opcode::Rra => {
                let carry = self.reg_a & 1;
                self.reg_a = self.reg_a.wrapping_shr(1) | ((self.flag_get(CpuFlags::C) as u8) << 7);

                self.flag_set(CpuFlags::Z, false);
                self.flag_set(CpuFlags::N, false);
                self.flag_set(CpuFlags::H, false);
                self.flag_set(CpuFlags::C, carry == 1);

                0
            }
            Opcode::Rlc => {
                let carry = (src >> 7) & 1;
                let result = src.wrapping_shl(1) | carry & 0xFF;

                self.flag_set(CpuFlags::Z, result == 0);
                self.flag_set(CpuFlags::N, false);
                self.flag_set(CpuFlags::H, false);
                self.flag_set(CpuFlags::C, carry == 1);

                result
            }
            Opcode::Rrc => {
                let carry = src & 1;
                let result = src.wrapping_shr(1) | (carry << 7) & 0xFF;

                self.flag_set(CpuFlags::Z, result == 0);
                self.flag_set(CpuFlags::N, false);
                self.flag_set(CpuFlags::H, false);
                self.flag_set(CpuFlags::C, carry == 1);

                result
            }
            Opcode::Rl => {
                let carry = (src >> 7) & 1;
                let result = (src.wrapping_shl(1) | self.flag_get(CpuFlags::C) as u16) & 0xFF;

                self.flag_set(CpuFlags::Z, result == 0);
                self.flag_set(CpuFlags::N, false);
                self.flag_set(CpuFlags::H, false);
                self.flag_set(CpuFlags::C, carry == 1);

                result
            }
            Opcode::Rr => {
                let carry = src & 1;
                let result =
                    (src.wrapping_shr(1) | ((self.flag_get(CpuFlags::C) as u16) << 7)) & 0xFF;

                self.flag_set(CpuFlags::Z, result == 0);
                self.flag_set(CpuFlags::N, false);
                self.flag_set(CpuFlags::H, false);
                self.flag_set(CpuFlags::C, carry == 1);

                result
            }
            Opcode::Sla => {
                let carry = (src >> 7) & 1;
                let result = src.wrapping_shl(1) & 0xFF;

                self.flag_set(CpuFlags::Z, result == 0);
                self.flag_set(CpuFlags::N, false);
                self.flag_set(CpuFlags::H, false);
                self.flag_set(CpuFlags::C, carry == 1);

                result
            }
            Opcode::Sra => {
                let carry = src & 1;
                let result = (src.wrapping_shr(1) | (src & 0x80)) & 0xFF;

                self.flag_set(CpuFlags::Z, result == 0);
                self.flag_set(CpuFlags::N, false);
                self.flag_set(CpuFlags::H, false);
                self.flag_set(CpuFlags::C, carry == 1);

                result
            }
            Opcode::Swap => {
                let result = ((src >> 4) & 0xf) | ((src & 0xf) << 4);

                self.flag_set(CpuFlags::Z, result == 0);
                self.flag_set(CpuFlags::N, false);
                self.flag_set(CpuFlags::H, false);
                self.flag_set(CpuFlags::C, false);

                result
            }
            Opcode::Srl => {
                let carry = src & 1;
                let result = src.wrapping_shr(1) & 0xFF;

                self.flag_set(CpuFlags::Z, result == 0);
                self.flag_set(CpuFlags::N, false);
                self.flag_set(CpuFlags::H, false);
                self.flag_set(CpuFlags::C, carry == 1);

                result
            }
            Opcode::Bit(bit) => {
                self.flag_set(CpuFlags::Z, (src >> bit) & 1 == 0);
                self.flag_set(CpuFlags::N, false);
                self.flag_set(CpuFlags::H, true);
                0
            }
            Opcode::Res(bit) => src & !((1 << bit) as u16),
            Opcode::Set(bit) => src | ((1 << bit) as u16),
            Opcode::Halt => {
                // When halt instruction is executed several outcomes might occur:
                // - When IME = 1:
                //      In this case, the halt instruction works normally. It will
                //      stop exection and wait until an interrupt occure (`IF & IE & 0x1F != 0`),
                //      then it will exit halt mode and execute the interrupt normally.
                // - When IME = 0:
                //      - If an interrupt is waiting (`IF & IE & 0x1F != 0`), it
                //        will enter a `Halt bug` state, in this state, the halt
                //        mode is not entered and the PC register is not incremented
                //        on the next instruction.
                //      - If an interrupt is not waiting (`IF & IE & 0x1F == 0`),
                //        the cpu will enter halt mode normally and wait for an interrupt
                //        to occur like in *IME = 1* case but if an interrupt is
                //        requested it will not just to the interrupt vector
                //        and it will continue executing normally, we can think
                //        of it as being stuck in a large array of NOP instructions
                //        until an interrupt is requested.
                self.halt_mode = if self.ime {
                    HaltMode::HaltRunInterrupt
                } else if bus.peek_next_interrupt().is_some() {
                    HaltMode::HaltBug
                } else {
                    HaltMode::HaltNoRunInterrupt
                };

                0
            }
            Opcode::Stop => {
                // TODO: respect wait time for speed switch
                bus.enter_stop_mode();
                0
            }
            Opcode::Illegal => todo!(),
            Opcode::Prefix => unreachable!(),
        };

        // DEBUG
        // println!(
        //     "{:04X}: {}, src={:04X}, result={:04X}",
        //     instruction.pc, instruction, src, result
        // );

        self.write_operand(instruction.dest, result, bus);

        cpu_state
    }
}

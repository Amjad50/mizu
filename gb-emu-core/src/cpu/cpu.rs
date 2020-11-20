use super::instruction::{Condition, Instruction, Opcode, OperandType};
use super::CpuBusProvider;

use bitflags::bitflags;

bitflags! {
    struct CpuFlags: u8 {
        const Z = 1 << 7;
        const N = 1 << 6;
        const H = 1 << 5;
        const C = 1 << 4;
    }
}

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

    ime: bool,
}

impl Cpu {
    pub fn new() -> Self {
        let mut cpu = Self {
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

            ime: false,
        };

        cpu.reset();

        cpu
    }

    pub fn reset(&mut self) {
        // initial values of the registers (DMG)
        self.reg_af_write(0x01B0);
        self.reg_bc_write(0x0013);
        self.reg_de_write(0x00D8);
        self.reg_hl_write(0x014D);
        self.reg_sp = 0xFFFE;
        self.reg_pc = 0x0100;
    }

    pub fn next_instruction<P: CpuBusProvider>(&mut self, bus: &mut P) {
        let mut instruction = Instruction::from_byte(self.fetch_next_pc(bus));
        if instruction.opcode == Opcode::Prefix {
            instruction = Instruction::from_prefix(self.fetch_next_pc(bus));
        }

        self.exec_instruction(instruction, bus);
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

    fn fetch_next_pc<P: CpuBusProvider>(&mut self, bus: &mut P) -> u8 {
        let result = bus.read(self.reg_pc);
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
                self.reg_hl_write(hl.wrapping_sub(1));
                result
            }
            OperandType::AddrHLInc => {
                let hl = self.reg_hl_read();
                let result = bus.read(hl) as u16;
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

    fn stack_push<P: CpuBusProvider>(&mut self, data: u16, bus: &mut P) {
        self.reg_sp = self.reg_sp.wrapping_sub(1);
        bus.write(self.reg_sp, (data >> 8) as u8);
        self.reg_sp = self.reg_sp.wrapping_sub(1);
        bus.write(self.reg_sp, data as u8);
    }

    fn stack_pop<P: CpuBusProvider>(&mut self, bus: &mut P) -> u16 {
        let low = bus.read(self.reg_sp);
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

    fn exec_instruction<P: CpuBusProvider>(&mut self, instruction: Instruction, bus: &mut P) {
        let src = self.read_operand(instruction.operand_types.1, bus);

        let result = match instruction.opcode {
            Opcode::Nop => 0,
            Opcode::Ld => src,
            Opcode::LdHLSPSigned8 => {
                let result = self.reg_sp.wrapping_add(src);

                self.flag_set(CpuFlags::Z, false);
                self.flag_set(CpuFlags::N, false);
                self.flag_set(CpuFlags::H, (self.reg_sp & 0xf) > (result & 0xf));
                self.flag_set(CpuFlags::C, result & 0xff00 != 0);

                result
            }
            Opcode::Push => {
                self.stack_push(src, bus);
                0
            }
            Opcode::Pop => self.stack_pop(bus),
            Opcode::Inc16 => src.wrapping_add(1),
            Opcode::Inc => {
                let result = src.wrapping_add(1) & 0xff;

                self.flag_set(CpuFlags::Z, result == 0);
                self.flag_set(CpuFlags::N, false);
                self.flag_set(CpuFlags::H, result & 0x0f == 0);

                result
            }
            Opcode::Dec16 => src.wrapping_sub(1),
            Opcode::Dec => {
                let result = src.wrapping_sub(1);
                self.flag_set(CpuFlags::Z, result == 0);
                self.flag_set(CpuFlags::N, true);
                self.flag_set(CpuFlags::H, result & 0x0f == 0x0f);
                result
            }
            Opcode::Add => {
                let dest = self.read_operand(instruction.operand_types.0, bus);
                let result = dest.wrapping_add(src);

                self.flag_set(CpuFlags::Z, result & 0xFF == 0);
                self.flag_set(CpuFlags::N, false);
                self.flag_set(CpuFlags::H, (dest & 0xf) + (src & 0xf) > 0xf);
                self.flag_set(CpuFlags::C, result & 0xff00 != 0);

                result & 0xFF
            }
            Opcode::Add16 => {
                let dest = self.read_operand(instruction.operand_types.0, bus);
                let result = (dest as u32).wrapping_add(src as u32);

                self.flag_set(CpuFlags::N, false);
                self.flag_set(CpuFlags::H, (dest & 0xfff) + (src & 0xfff) > 0xfff);
                self.flag_set(CpuFlags::C, result & 0xffff0000 != 0);

                result as u16
            }
            Opcode::AddSPSigned8 => {
                let dest = self.read_operand(instruction.operand_types.0, bus);
                let result = dest.wrapping_add(src);

                self.flag_set(CpuFlags::Z, false);
                self.flag_set(CpuFlags::N, false);
                self.flag_set(CpuFlags::H, result & 0xfff0 != 0);
                self.flag_set(CpuFlags::C, result & 0xff00 != 0);

                result
            }
            Opcode::Adc => {
                let dest = self.read_operand(instruction.operand_types.0, bus);
                let carry = self.flag_get(CpuFlags::C) as u16;
                let result = dest.wrapping_add(src).wrapping_add(carry);

                self.flag_set(CpuFlags::Z, result & 0xFF == 0);
                self.flag_set(CpuFlags::N, false);
                self.flag_set(CpuFlags::H, (dest & 0xf) + (src & 0xf) + carry > 0xf);
                self.flag_set(CpuFlags::C, result & 0xff00 != 0);

                result
            }
            Opcode::Sub => {
                let dest = self.read_operand(instruction.operand_types.0, bus);
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
                let dest = self.read_operand(instruction.operand_types.0, bus);
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
                let dest = self.read_operand(instruction.operand_types.0, bus);
                let result = dest & src & 0xff;

                self.flag_set(CpuFlags::Z, result == 0);
                self.flag_set(CpuFlags::N, false);
                self.flag_set(CpuFlags::H, true);
                self.flag_set(CpuFlags::C, false);

                result
            }
            Opcode::Xor => {
                let dest = self.read_operand(instruction.operand_types.0, bus);
                let result = (dest ^ src) & 0xff;

                self.flag_set(CpuFlags::Z, result == 0);
                self.flag_set(CpuFlags::N, false);
                self.flag_set(CpuFlags::H, false);
                self.flag_set(CpuFlags::C, false);

                result
            }
            Opcode::Or => {
                let dest = self.read_operand(instruction.operand_types.0, bus);
                let result = (dest | src) & 0xff;

                self.flag_set(CpuFlags::Z, result == 0);
                self.flag_set(CpuFlags::N, false);
                self.flag_set(CpuFlags::H, false);
                self.flag_set(CpuFlags::C, false);

                result
            }
            Opcode::Jp(cond) => {
                if self.check_cond(cond) {
                    self.reg_pc = src;
                }
                0
            }
            Opcode::Jr(cond) => {
                if self.check_cond(cond) {
                    self.reg_pc = self.reg_pc.wrapping_add(src);
                }
                0
            }
            Opcode::Call(cond) => {
                if self.check_cond(cond) {
                    self.stack_push(self.reg_pc, bus);
                    self.reg_pc = src;
                }
                0
            }
            Opcode::Ret(cond) => {
                if self.check_cond(cond) {
                    self.reg_pc = self.stack_pop(bus);
                }
                0
            }
            Opcode::Reti => {
                self.reg_pc = self.stack_pop(bus);
                self.ime = true;
                0
            }
            Opcode::Rst(loc) => {
                self.stack_push(self.reg_pc, bus);
                self.reg_pc = loc as u16;
                0
            }
            Opcode::Di => {
                self.ime = false;
                0
            }
            Opcode::Ei => {
                self.ime = true;
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

                    if carry || (self.reg_a & 0xff > 0x99) {
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
                self.flag_set(CpuFlags::H, false);
                0
            }
            Opcode::Res(bit) => src & !((1 << bit) as u16),
            Opcode::Set(bit) => src | ((1 << bit) as u16),
            Opcode::Halt => todo!(),
            Opcode::Stop => todo!(),
            Opcode::Illegal => todo!(),
            Opcode::Prefix => unreachable!(),
        };

        self.write_operand(instruction.operand_types.0, result, bus);
    }
}

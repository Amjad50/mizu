use super::instruction::{Condition, Instruction, Opcode, OperandType};

use bitflags::bitflags;

bitflags! {
    struct CpuFlags: u8 {
        const Z = 1 << 7;
        const N = 1 << 6;
        const H = 1 << 5;
        const C = 1 << 4;
    }
}

struct Cpu {
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

    fn read_bus(&mut self, addr: u16) -> u8 {
        0
    }

    fn write_bus(&mut self, addr: u16, data: u8) {}

    fn fetch_next_pc(&mut self) -> u8 {
        let result = self.read_bus(self.reg_pc);
        self.reg_pc = self.reg_pc.wrapping_add(1);
        result
    }

    fn read_operand(&mut self, ty: OperandType) -> u16 {
        match ty {
            OperandType::RegA => self.reg_a as u16,
            OperandType::RegB => self.reg_b as u16,
            OperandType::RegC => self.reg_c as u16,
            OperandType::RegD => self.reg_d as u16,
            OperandType::RegE => self.reg_e as u16,
            OperandType::RegH => self.reg_h as u16,
            OperandType::RegL => self.reg_l as u16,
            OperandType::AddrHL => self.read_bus(self.reg_hl_read()) as u16,
            OperandType::AddrHLDec => {
                let hl = self.reg_hl_read();
                let result = self.read_bus(hl) as u16;
                self.reg_hl_write(hl.wrapping_sub(1));
                result
            }
            OperandType::AddrHLInc => {
                let hl = self.reg_hl_read();
                let result = self.read_bus(hl) as u16;
                self.reg_hl_write(hl.wrapping_add(1));
                result
            }
            OperandType::AddrBC => self.read_bus(self.reg_bc_read()) as u16,
            OperandType::AddrDE => self.read_bus(self.reg_de_read()) as u16,
            OperandType::RegAF => self.reg_af_read(),
            OperandType::RegBC => self.reg_bc_read(),
            OperandType::RegDE => self.reg_de_read(),
            OperandType::RegHL => self.reg_hl_read(),
            OperandType::RegSP => self.reg_sp,
            OperandType::Imm8 => self.fetch_next_pc() as u16,
            OperandType::Imm8Signed => self.fetch_next_pc() as i8 as i16 as u16,
            OperandType::Imm16 => {
                ((self.fetch_next_pc() as u16) << 8) | self.fetch_next_pc() as u16
            }
            OperandType::HighAddr8 => {
                let addr = 0xFF00 | self.fetch_next_pc() as u16;
                self.read_bus(addr) as u16
            }
            OperandType::HighAddrC => self.read_bus(0xFF00 | self.reg_c as u16) as u16,
            OperandType::Addr16 => {
                let addr = ((self.fetch_next_pc() as u16) << 8) | self.fetch_next_pc() as u16;
                self.read_bus(addr) as u16
            }
            OperandType::RstLoc(location) => location as u16,
            OperandType::Implied => 0,
            OperandType::Addr16Val16 => unreachable!(),
        }
    }

    fn write_operand(&mut self, ty: OperandType, data: u16) {
        match ty {
            OperandType::RegA => self.reg_a = data as u8,
            OperandType::RegB => self.reg_b = data as u8,
            OperandType::RegC => self.reg_c = data as u8,
            OperandType::RegD => self.reg_d = data as u8,
            OperandType::RegE => self.reg_e = data as u8,
            OperandType::RegH => self.reg_h = data as u8,
            OperandType::RegL => self.reg_l = data as u8,
            OperandType::AddrHL => self.write_bus(self.reg_hl_read(), data as u8),
            OperandType::AddrHLDec => {
                let hl = self.reg_hl_read();
                self.write_bus(hl, data as u8);
                self.reg_hl_write(hl.wrapping_sub(1));
            }
            OperandType::AddrHLInc => {
                let hl = self.reg_hl_read();
                self.write_bus(hl, data as u8);
                self.reg_hl_write(hl.wrapping_add(1));
            }
            OperandType::AddrBC => self.write_bus(self.reg_bc_read(), data as u8),
            OperandType::AddrDE => self.write_bus(self.reg_de_read(), data as u8),
            OperandType::RegAF => self.reg_af_write(data),
            OperandType::RegBC => self.reg_bc_write(data),
            OperandType::RegDE => self.reg_de_write(data),
            OperandType::RegHL => self.reg_hl_write(data),
            OperandType::RegSP => self.reg_sp = data,
            OperandType::HighAddr8 => {
                let addr = 0xFF00 | self.fetch_next_pc() as u16;
                self.write_bus(addr, data as u8);
            }
            OperandType::HighAddrC => self.write_bus(0xFF00 | self.reg_c as u16, data as u8),
            OperandType::Addr16 => {
                let addr = ((self.fetch_next_pc() as u16) << 8) | self.fetch_next_pc() as u16;
                self.write_bus(addr, data as u8);
            }
            OperandType::Addr16Val16 => {
                let addr = ((self.fetch_next_pc() as u16) << 8) | self.fetch_next_pc() as u16;
                self.write_bus(addr, data as u8);
                self.write_bus(addr.wrapping_add(1), (data >> 8) as u8);
            }
            OperandType::Implied => {}
            OperandType::Imm16
            | OperandType::Imm8
            | OperandType::RstLoc(_)
            | OperandType::Imm8Signed => unreachable!(),
        }
    }

    fn exec_instruction(&mut self, instruction: Instruction) -> u16 {
        let src = self.read_operand(instruction.operand_types.1);

        let a: u16 = match instruction.opcode {
            Opcode::Nop => 0,
            Opcode::Stop => todo!(),
            Opcode::Ld => src,
            Opcode::LdHLSPSigned8 => {
                let result = self.reg_sp.wrapping_add(src);

                self.flag_set(CpuFlags::Z, false);
                self.flag_set(CpuFlags::N, false);
                self.flag_set(
                    CpuFlags::H,
                    (self.reg_sp & 0xf) + (src & 0xf) & 0x10 == 0x10,
                );
                self.flag_set(
                    CpuFlags::C,
                    (self.reg_sp & 0xff) + (src & 0xff) & 0x100 == 0x100,
                );
                result
            }
            Opcode::Push => todo!(),
            Opcode::Pop => todo!(),
            Opcode::Inc16 => src.wrapping_add(1),
            Opcode::Inc => {
                let result = src.wrapping_add(1);
                self.flag_set(CpuFlags::Z, result == 0);
                self.flag_set(CpuFlags::N, false);
                self.flag_set(CpuFlags::H, result & 0xfff0 != 0);
                result
            }
            Opcode::Dec16 => src.wrapping_sub(1),
            Opcode::Dec => {
                let result = src.wrapping_sub(1);
                self.flag_set(CpuFlags::Z, result == 0);
                self.flag_set(CpuFlags::N, true);
                // FIXME: check if its correct
                self.flag_set(CpuFlags::H, result & 0xfff0 == 0);
                result
            }
            Opcode::Add => {
                let dest = self.read_operand(instruction.operand_types.0);
                let result = dest.wrapping_add(src);

                self.flag_set(CpuFlags::Z, result == 0);
                self.flag_set(CpuFlags::N, false);
                self.flag_set(CpuFlags::H, result & 0xfff0 != 0);
                self.flag_set(CpuFlags::N, result & 0xff00 != 0);

                result
            }
            Opcode::Add16 => {
                let dest = self.read_operand(instruction.operand_types.0) as u32;
                let result = dest.wrapping_add(src as u32);

                self.flag_set(CpuFlags::N, false);
                self.flag_set(CpuFlags::H, result & 0xfffff000 != 0);
                self.flag_set(CpuFlags::N, result & 0xffff0000 != 0);

                result as u16
            }
            Opcode::AddSPSigned8 => {
                let dest = self.read_operand(instruction.operand_types.0);
                let result = dest.wrapping_add(src);

                self.flag_set(CpuFlags::Z, false);
                self.flag_set(CpuFlags::N, false);
                self.flag_set(CpuFlags::H, result & 0xfff0 != 0);
                self.flag_set(CpuFlags::N, result & 0xff00 != 0);

                result
            }
            Opcode::Adc => {
                let dest = self.read_operand(instruction.operand_types.0);
                let result = dest
                    .wrapping_add(src)
                    .wrapping_add(self.flag_get(CpuFlags::C) as u16);

                self.flag_set(CpuFlags::Z, result == 0);
                self.flag_set(CpuFlags::N, false);
                self.flag_set(CpuFlags::H, result & 0xfff0 != 0);
                self.flag_set(CpuFlags::N, result & 0xff00 != 0);

                result
            }
            Opcode::Sub => {
                let dest = self.read_operand(instruction.operand_types.0);
                dest.wrapping_sub(src)
            }
            Opcode::Sbc => {
                let dest = self.read_operand(instruction.operand_types.0);
                dest.wrapping_sub(src)
                    .wrapping_sub(self.flag_get(CpuFlags::C) as u16)
            }
            Opcode::And => todo!(),
            Opcode::Xor => todo!(),
            Opcode::Or => todo!(),
            Opcode::Cp => todo!(),
            Opcode::Jp(_) => todo!(),
            Opcode::Jr(_) => todo!(),
            Opcode::Call(_) => todo!(),
            Opcode::Ret(_) => todo!(),
            Opcode::Reti => todo!(),
            Opcode::Rst => todo!(),
            Opcode::Di => todo!(),
            Opcode::Ei => todo!(),
            Opcode::Ccf => todo!(),
            Opcode::Scf => todo!(),
            Opcode::Daa => todo!(),
            Opcode::Cpl => todo!(),
            Opcode::Rlca => todo!(),
            Opcode::Rla => todo!(),
            Opcode::Rrca => todo!(),
            Opcode::Rra => todo!(),
            Opcode::Prefix => todo!(),
            Opcode::Rlc => todo!(),
            Opcode::Rrc => todo!(),
            Opcode::Rl => todo!(),
            Opcode::Rr => todo!(),
            Opcode::Sla => todo!(),
            Opcode::Sra => todo!(),
            Opcode::Swap => todo!(),
            Opcode::Srl => todo!(),
            Opcode::Bit(_) => todo!(),
            Opcode::Res(_) => todo!(),
            Opcode::Set(_) => todo!(),
            Opcode::Illegal => todo!(),
            Opcode::Halt => todo!(),
        };
        a
    }
}

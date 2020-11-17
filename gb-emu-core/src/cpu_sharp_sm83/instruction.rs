struct Instruction {
    opcode: Opcode,
    operand_types: (OperandType, OperandType),
    operand_data: u16,
}

/// This is the location the operands will come from,
/// a basic usage can be something like this
///
/// ```
/// # use gb_emu_core::cpu_sharp_sm83::instruction::OperandType;
/// struct CPU {
///   A: u8,
/// }
///
/// impl CPU {
///     fn write_operand(&mut self, dest: OperandType, data: u8) {
///         match dest {
///             OperandType::RegA => self.A = data,
///             _ => {}
///         }
///     }
///
///     fn read_operand(&self, src: OperandType) -> u8 {
///         match src {
///             OperandType::RegA => self.A,
///             _ => unreachable!(),
///         }
///     }
///
///     // implementation of the Ld instruction
///     fn ld(&mut self, src: OperandType, dest: OperandType) {
///         self.write_operand(dest, self.read_operand(src));
///     }
/// }
/// ```
#[derive(PartialEq, Copy, Clone)]
pub enum OperandType {
    RegA,
    RegB,
    RegC,
    RegD,
    RegE,
    RegH,
    RegL,

    AddrHL, // this is used in many places with reg8
    AddrHLDec,
    AddrHLInc,
    AddrBC,
    AddrDE,

    RegAF,
    RegBC,
    RegDE,
    RegHL,

    RegSP,
    RegSPImm8,

    Imm8,
    Imm16,

    HighAddr8,
    HighAddrC, // only for the C register
    Addr16,

    Arg(u8),

    // Also for instructions with one operand as a fill
    Implied,

    CondC,
    CondZ,
    CondNC,
    CondNZ,
}

impl OperandType {
    fn get_reg8(id: u8) -> Option<Self> {
        match id {
            0 => Some(Self::RegB),
            1 => Some(Self::RegC),
            2 => Some(Self::RegD),
            3 => Some(Self::RegE),
            4 => Some(Self::RegH),
            5 => Some(Self::RegL),
            6 => Some(Self::AddrHL),
            7 => Some(Self::RegA),
            _ => None,
        }
    }

    /// the forth parameter can change between AF and SP in some cases
    fn get_reg16(id: u8, fourth: Self) -> Option<Self> {
        match id {
            0 => Some(Self::RegBC),
            1 => Some(Self::RegDE),
            2 => Some(Self::RegHL),
            3 => Some(fourth),
            _ => None,
        }
    }

    fn get_cond(id: u8) -> Option<Self> {
        match id {
            0 => Some(Self::CondNZ),
            1 => Some(Self::CondZ),
            2 => Some(Self::CondNC),
            3 => Some(Self::CondC),
            _ => None,
        }
    }
}

enum Opcode {
    Nop,
    Stop,

    Ld,

    Push,
    Pop,

    Inc,
    Dec,

    Add,
    Adc,
    Sub,
    Sbc,
    And,
    Xor,
    Or,
    Cp,

    Jp,
    Jr,

    Call,
    Ret,

    Reti,

    Rst,

    Di,
    Ei,
    Ccf,
    Scf,
    Daa,
    Cpl,

    Prefix,

    Illegal,

    Halt,
}

impl Instruction {
    pub fn from_byte(byte: u8) -> Option<Self> {
        use Opcode::*;
        use OperandType::*;

        // Ld r8, r8
        let (opcode, operand_types) = if byte & 0xc0 == 0x40 {
            let dest = (byte & 0b111000) >> 3;
            let src = byte & 0b111;

            let src = OperandType::get_reg8(src).unwrap();
            let dest = OperandType::get_reg8(dest).unwrap();

            if src == AddrHL && dest == AddrHL {
                (Halt, (Implied, Implied))
            } else {
                (Ld, (dest, src))
            }
        } else if byte & 0xc7 == 0x06 {
            let dest = (byte & 0b111000) >> 3;
            let dest = OperandType::get_reg8(dest).unwrap();

            (Ld, (dest, Imm8))
        } else if byte & 0xc7 == 0x04 {
            let dest = (byte & 0b111000) >> 3;
            let dest = OperandType::get_reg8(dest).unwrap();

            (Inc, (dest, dest))
        } else if byte & 0xc7 == 0x05 {
            let dest = (byte & 0b111000) >> 3;
            let dest = OperandType::get_reg8(dest).unwrap();

            (Dec, (dest, dest))
        } else if byte & 0xcf == 0x1 {
            let dest = (byte & 0b110000) >> 4;
            let dest = OperandType::get_reg16(dest, RegSP).unwrap();

            (Ld, (dest, Imm16))
        } else if byte & 0xcf == 0x3 {
            let dest = (byte & 0b110000) >> 4;
            let dest = OperandType::get_reg16(dest, RegSP).unwrap();

            (Inc, (dest, dest))
        } else if byte & 0xcf == 0xB {
            let dest = (byte & 0b110000) >> 4;
            let dest = OperandType::get_reg16(dest, RegSP).unwrap();

            (Dec, (dest, dest))
        } else if byte & 0xcf == 0xc5 {
            let src = (byte & 0b110000) >> 4;
            let src = OperandType::get_reg16(src, RegAF).unwrap();

            (Push, (Implied, src))
        } else if byte & 0xcf == 0xc1 {
            let dest = (byte & 0b110000) >> 4;
            let dest = OperandType::get_reg16(dest, RegAF).unwrap();

            (Push, (dest, Implied))
        } else if byte >= 0x80 && byte <= 0xbf {
            let src = byte & 0b111;
            let src = OperandType::get_reg8(src).unwrap();

            match (byte >> 3) & 0b111 {
                0 => (Add, (RegA, src)),
                1 => (Adc, (RegA, src)),
                2 => (Sub, (RegA, src)),
                3 => (Sbc, (RegA, src)),
                4 => (And, (RegA, src)),
                5 => (Xor, (RegA, src)),
                6 => (Or, (RegA, src)),
                7 => (Cp, (RegA, src)),
                _ => unreachable!(),
            }
        } else if byte & 0xe7 == 0xc2 {
            let dest = (byte >> 3) & 0b11;
            let dest = OperandType::get_cond(dest).unwrap();

            (Jp, (dest, Imm16))
        } else if byte & 0xe7 == 0x20 {
            let dest = (byte >> 3) & 0b11;
            let dest = OperandType::get_cond(dest).unwrap();

            (Jr, (dest, Imm8))
        } else if byte & 0xe7 == 0xc4 {
            let dest = (byte >> 3) & 0b11;
            let dest = OperandType::get_cond(dest).unwrap();

            (Call, (dest, Imm16))
        } else if byte & 0xe7 == 0xc0 {
            let dest = (byte >> 3) & 0b11;
            let dest = OperandType::get_cond(dest).unwrap();

            (Ret, (dest, Implied))
        } else if byte & 0xc7 == 0xc7 {
            let src = (byte >> 3) & 0b111;

            (Rst, (Implied, Arg(src * 8)))
        } else if byte & 0xcf == 0x9 {
            let src = (byte & 0b110000) >> 4;
            let src = OperandType::get_reg16(src, RegSP).unwrap();

            (Add, (RegHL, src))
        } else if byte & 0xc7 == 0xc6 {
            let opcode = match (byte >> 3) & 0b111 {
                0 => Add,
                1 => Adc,
                2 => Sub,
                3 => Sbc,
                4 => And,
                5 => Xor,
                6 => Or,
                7 => Cp,
                _ => unreachable!(),
            };

            (opcode, (RegA, Imm8))
        } else {
            match byte {
                0x00 => (Nop, (Implied, Implied)),
                0x10 => (Stop, (Implied, Implied)),
                0x0A => (Ld, (RegA, AddrBC)),
                0x1A => (Ld, (RegA, AddrDE)),
                0x02 => (Ld, (AddrBC, RegA)),
                0x12 => (Ld, (AddrDE, RegA)),
                0xFA => (Ld, (RegA, Addr16)),
                0xEA => (Ld, (Addr16, RegA)),
                0xF2 => (Ld, (RegA, HighAddrC)),
                0xE2 => (Ld, (HighAddrC, RegA)),
                0xF0 => (Ld, (RegA, HighAddr8)),
                0xE0 => (Ld, (HighAddr8, RegA)),
                0x3A => (Ld, (RegA, AddrHLDec)),
                0x2A => (Ld, (RegA, AddrHLInc)),
                0x32 => (Ld, (AddrHLDec, RegA)),
                0x22 => (Ld, (AddrHLInc, RegA)),
                0x08 => (Ld, (Addr16, RegSP)),
                0xF9 => (Ld, (RegSP, RegHL)),
                0xC3 => (Jp, (Implied, Imm16)),
                0xE9 => (Jp, (Implied, RegHL)),
                0x18 => (Jr, (Implied, Imm8)),
                0xcd => (Call, (Implied, Imm16)),
                0xC9 => (Ret, (Implied, Implied)),
                0xD9 => (Reti, (Implied, Implied)),
                0xF3 => (Di, (Implied, Implied)),
                0xFB => (Ei, (Implied, Implied)),
                0x3F => (Ccf, (Implied, Implied)),
                0x37 => (Scf, (Implied, Implied)),
                0x27 => (Daa, (Implied, Implied)),
                0x2F => (Cpl, (Implied, Implied)),
                0xCB => (Prefix, (Implied, Implied)),
                0xE8 => (Add, (RegSP, Imm8)),
                0xF8 => (Ld, (RegHL, RegSPImm8)),

                // illegal instructions
                0xD3 => (Illegal, (Implied, Implied)),
                0xDB => (Illegal, (Implied, Implied)),
                0xDD => (Illegal, (Implied, Implied)),
                0xE3 => (Illegal, (Implied, Implied)),
                0xE4 => (Illegal, (Implied, Implied)),
                0xEB => (Illegal, (Implied, Implied)),
                0xEC => (Illegal, (Implied, Implied)),
                0xED => (Illegal, (Implied, Implied)),
                0xF4 => (Illegal, (Implied, Implied)),
                0xFC => (Illegal, (Implied, Implied)),
                0xFD => (Illegal, (Implied, Implied)),
                _ => return None,
            }
        };

        Some(Instruction {
            opcode,
            operand_types,
            operand_data: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Instruction;

    #[rustfmt::skip]
    const INTSTRUCTIONS_PRESENT: [u8;256] =
        [1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 0,
         1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 0,
         1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
         1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
         1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
         1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
         1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
         1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
         1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
         1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
         1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
         1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
         1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
         1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
         1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
         1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,];

    #[test]
    fn available_instructions() {
        for i in 0..=255u8 {
            if i % 16 == 0 {
                println!();
            }
            let v = Instruction::from_byte(i as u8).is_some() as u8;

            print!("{}, ", v);

            assert_eq!(
                v,
                INTSTRUCTIONS_PRESENT[i as usize],
                "Instruction {:02X} it implemented and it shouldn't be or not implemented and it should be",
                i);
        }
    }
}

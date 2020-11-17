use super::instructions_table;

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
#[derive(PartialEq, Copy, Clone, Debug)]
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

#[derive(Clone, Copy, Debug)]
pub enum Opcode {
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

    Rlca,
    Rla,
    Rrca,
    Rra,

    Prefix,

    Rlc,
    Rrc,
    Rl,
    Rr,
    Sla,
    Sra,
    Swap,
    Srl,

    Bit(u8),
    Res(u8),
    Set(u8),

    Illegal,

    Halt,
}

impl Instruction {
    pub fn from_byte(byte: u8) -> Self {
        let (opcode, operand_types) = instructions_table::INSTRUCTIONS[byte as usize];

        Instruction {
            opcode,
            operand_types,
            operand_data: 0,
        }
    }

    fn from_prefix(byte: u8) -> Self {
        let (opcode, operand_types) = instructions_table::PREFIXED_INSTRUCTIONS[byte as usize];

        Instruction {
            opcode,
            operand_types,
            operand_data: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Instruction;

    #[test]
    fn available_instructions() {
        for i in 0..=255u8 {
            Instruction::from_byte(i as u8);
        }
    }

    #[test]
    fn available_instructions_with_prefix_cb() {
        for i in 0..=255u8 {
            Instruction::from_prefix(i as u8);
        }
    }
}

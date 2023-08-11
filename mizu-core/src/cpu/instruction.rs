use super::instructions_table;
use std::fmt::Display;

#[derive(Debug)]
pub(super) struct Instruction {
    pub pc: u16,
    pub opcode: Opcode,
    pub src: OperandType,
    pub dest: OperandType,
}

/// This is the location the operands will come from,
/// a basic usage can be something like this
///
/// ```ignore
/// # use mizu_core::cpu::instruction::OperandType;
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

    Imm8,
    Imm8Signed,
    Imm16,

    HighAddr8,
    HighAddrC, // only for the C register
    Addr16,
    Addr16Val16, // write 16bit value to address

    // Also for instructions with one operand as a fill
    Implied,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Condition {
    NC,
    C,
    NZ,
    Z,
    Unconditional,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Opcode {
    Nop,
    Stop,

    Ld,
    LdSPHL,
    LdHLSPSigned8,
    LdBB, // used for breakpoint

    Push,
    Pop,

    Inc,
    Inc16,
    Dec,
    Dec16,

    Add,
    Add16,
    AddSPSigned8,
    Adc,
    Cp, // = Sub (Implied, Reg8)
    Sub,
    Sbc,
    And,
    Xor,
    Or,
    Jp(Condition),
    JpHL,
    Jr(Condition),

    Call(Condition),
    Ret(Condition),

    Reti,

    Rst(u8),

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
    pub fn from_byte(byte: u8, pc: u16) -> Self {
        let (opcode, operand_types) = instructions_table::INSTRUCTIONS[byte as usize];

        Instruction {
            pc,
            opcode,
            src: operand_types.1,
            dest: operand_types.0,
        }
    }

    pub fn from_prefix(byte: u8, pc: u16) -> Self {
        let (opcode, operand_types) = instructions_table::PREFIXED_INSTRUCTIONS[byte as usize];

        Instruction {
            pc,
            opcode,
            src: operand_types.1,
            dest: operand_types.0,
        }
    }
}

fn operand_str(operand: OperandType) -> String {
    match operand {
        OperandType::RegA => "A".into(),
        OperandType::RegB => "B".into(),
        OperandType::RegC => "C".into(),
        OperandType::RegD => "D".into(),
        OperandType::RegE => "E".into(),
        OperandType::RegH => "H".into(),
        OperandType::RegL => "L".into(),
        OperandType::AddrHL => "(HL)".into(),
        OperandType::AddrHLDec => "(HL-)".into(),
        OperandType::AddrHLInc => "(HL+)".into(),
        OperandType::AddrBC => "(BC)".into(),
        OperandType::AddrDE => "(DE)".into(),
        OperandType::RegAF => "AF".into(),
        OperandType::RegBC => "BC".into(),
        OperandType::RegDE => "DE".into(),
        OperandType::RegHL => "HL".into(),
        OperandType::RegSP => "SP".into(),
        OperandType::Imm8 => "d8".into(),
        OperandType::Imm8Signed => "r8".into(),
        OperandType::Imm16 => "d16".into(),
        OperandType::HighAddr8 => "(a8)".into(),
        OperandType::HighAddrC => "(C)".into(),
        OperandType::Addr16 => "(a16)".into(),
        OperandType::Addr16Val16 => "(a16)".into(),
        OperandType::Implied => "".into(),
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let opcode: String = match self.opcode {
            Opcode::Nop => "NOP".into(),
            Opcode::Stop => "STOP".into(),
            Opcode::Ld => "LD".into(),
            Opcode::LdSPHL => "LD SP, HL".into(),
            Opcode::LdHLSPSigned8 => "LDHLSP".into(),
            Opcode::LdBB => "LD B,B".into(),
            Opcode::Push => "PUSH".into(),
            Opcode::Pop => "POP".into(),
            Opcode::Inc => "INC".into(),
            Opcode::Inc16 => "INC".into(),
            Opcode::Dec => "DEC".into(),
            Opcode::Dec16 => "DEC".into(),
            Opcode::Add => "ADD".into(),
            Opcode::Add16 => "ADD".into(),
            Opcode::AddSPSigned8 => "ADD".into(),
            Opcode::Adc => "ADC".into(),
            Opcode::Cp => "CP".into(),
            Opcode::Sub => "SUB".into(),
            Opcode::Sbc => "SBC".into(),
            Opcode::And => "AND".into(),
            Opcode::Xor => "XOR".into(),
            Opcode::Or => "OR".into(),
            Opcode::Jp(Condition::Unconditional) => "JP".into(),
            Opcode::Jp(cond) => format!("JP {:?},", cond),
            Opcode::JpHL => "JP".into(),
            Opcode::Jr(Condition::Unconditional) => "JR".into(),
            Opcode::Jr(cond) => format!("JR {:?},", cond),
            Opcode::Call(Condition::Unconditional) => "CALL".into(),
            Opcode::Call(cond) => format!("CALL {:?},", cond),
            Opcode::Ret(Condition::Unconditional) => "RET".into(),
            Opcode::Ret(cond) => format!("RET {:?},", cond),
            Opcode::Reti => "RETI".into(),
            Opcode::Rst(loc) => format!("RST {:02X}", loc),
            Opcode::Di => "DI".into(),
            Opcode::Ei => "EI".into(),
            Opcode::Ccf => "CCF".into(),
            Opcode::Scf => "SCF".into(),
            Opcode::Daa => "DAA".into(),
            Opcode::Cpl => "CPL".into(),
            Opcode::Rlca => "RLCA".into(),
            Opcode::Rla => "RLA".into(),
            Opcode::Rrca => "RRCA".into(),
            Opcode::Rra => "RRA".into(),
            Opcode::Prefix => "PREFIX".into(),
            Opcode::Rlc => "RLC".into(),
            Opcode::Rrc => "RRC".into(),
            Opcode::Rl => "RL".into(),
            Opcode::Rr => "RR".into(),
            Opcode::Sla => "SLA".into(),
            Opcode::Sra => "SRA".into(),
            Opcode::Swap => "SWAP".into(),
            Opcode::Srl => "SRL".into(),
            Opcode::Bit(n) => format!("BIT {},", n),
            Opcode::Res(n) => format!("RES {},", n),
            Opcode::Set(n) => format!("SET {},", n),
            Opcode::Illegal => "ILLEGAL".into(),
            Opcode::Halt => "HALT".into(),
        };

        let mut operands = operand_str(self.dest);
        if operands.is_empty() {
            operands = operand_str(self.src);
        } else if !operand_str(self.src).is_empty() {
            operands += &format!(",{}", operand_str(self.src));
        }

        write!(f, "{} {}", opcode, operands)
    }
}

#[cfg(test)]
mod tests {
    use super::Instruction;

    #[test]
    fn available_instructions() {
        for i in 0..=255u8 {
            Instruction::from_byte(i, 0);
        }
    }

    #[test]
    fn available_instructions_with_prefix_cb() {
        for i in 0..=255u8 {
            Instruction::from_prefix(i, 0);
        }
    }
}

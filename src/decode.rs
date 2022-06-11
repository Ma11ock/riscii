// RISC-II decoder.
// (C) Ryan Jeffrey <ryan@ryanmj.xyz>, 2022
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or (at
// your option) any later version.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.
extern crate core;

use core::convert::TryInto;
use std::fmt;

/// Types of conditionals the RISC II supports.
#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Conditional {
    /// Greater than.
    Gt,
    /// Less than or equal to.
    Le,
    /// Greater than or equal to.
    Ge,
    /// Less than.
    Lt,
    /// Higher than.
    Hi,
    /// Lower than or same.
    Los,
    /// Lower than no carry.
    Lonc,
    /// Higher than, carry.
    Hisc,
    /// Plus (test sign).
    Pl,
    /// Minus (test sign).
    Mi,
    /// Not equal.
    Ne,
    /// Equal.
    Eq,
    /// No overflow (signed arithmetic).
    Nv,
    /// Overflow (signed arithmetic).
    V,
    /// Always (constant 1).
    Alw,
}

/// The 'source' of the instruction, which can either be a register name,
/// or a 13 bit immediate (signed or unsigned).
#[derive(PartialEq, Eq, Copy, Clone)]
pub enum ShortSource {
    /// Register name.
    Reg(u8),
    /// Unsigned 13 bit immediate, 0-padded to 32 bits.
    UImm13(u32),
    /// Signed 13 bit immediate, Sign-extended to 32 bits.
    SImm13(i32),
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub struct ShortInstruction {
    scc: bool,
    dest: u8,
    rs1: u8,
    short_source: ShortSource,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub struct LongInstruction {
    scc: bool,
    dest: u8,
    imm19: u32,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub struct ShortConditional {
    scc: bool,
    dest: Conditional,
    rs1: u8,
    short_source: ShortSource,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub struct LongConditional {
    scc: bool,
    dest: Conditional,
    imm19: u32,
}

/// A RISC-II Instruction.
/// A RISC-II instruction is in one of two formats: short source and long immediate.
///
/// Short source has the following members: (SCC: bool, dest: u8, rs1: u8, short source: u16).
/// The short source can either be a 13 bit immediate value or a 5 bit register name.
///
/// The long immediate format has the following members: (SCC: bool, dest: u8, imm: 19).
///
/// For both formats, if SCC is true, then update conditional registers (CC's).
///
/// Shift CC's: shift, logical instructions: V := 0; C := 0;
/// Arithmetic instructions update CC's as follows: Z := [d == 0]; N := d<31>;
/// Arithmetic's V := [32 bit 2's-complement overflow occurred].
/// additions: C := carry<31>to<32> (assuming s1, s2: unsigned).
/// subtractions: C := NOT[borrow<31>to<32>] (for s1, s2: unsigned).
///
/// Load instructions: If the instruction has the letter `r` in the name
/// it is relative to PC (PC + imm19). If it has the letter `x` instead,
/// the load is register indexed.
#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Instruction {
    /// Call interrupt.
    /// Notes:
    /// - PRIVILEGED INSTRUCTION.
    /// - The `RS1` and `RS1` registers are read from the OLD window.
    /// - The PC instruction saved is the `PC` at the `CALLI`.
    /// - The `Rd` refers to the destination register in the NEW window.
    /// - If the change to `CWP` makes it equal to `SWP`: stop execution,
    /// generate a trap, and go to address 0x80000020.
    /// CWP := CWP - 1 MOD 8, rd := LSTPC; CC's have same rules as getipc.
    Calli(ShortInstruction),
    /// Get pointer to window. rd := (-1)<31:13> & PSW<12:0>;
    GetPSW(ShortInstruction),
    /// Get the last Program Counter. rd := LSTPC.
    /// Iff SCC == true, Z := [LSTPC == 0]; N := LSTPC<31>; V,C := garbage.
    /// Notes:
    /// - PRIVILEGED INSTRUCTION.
    GetIPC(ShortInstruction),
    /// Set PSW. PSW := [rs1 + ShortSource2]<12:0>;
    /// Notes:
    /// - PRIVILEGED INSTRUCTION.
    /// - SCC-bit MUST be false.
    /// - The next instruction CANNOT be `CALLX`, `CALLR`, CALLI`, `RET`, `RETI`,
    /// i.e. it cannot modify CWP. It also must not set the CC's.
    /// - Rd is discarded.
    /// - New PSW is not in effect until AFTER the next cycle following execution
    /// of this instruction.
    PutPSW(ShortInstruction),
    /// Call procedure at `shortSource` + `rs1`.
    /// - The `RS1` and `RS1` registers are read from the OLD window.
    /// - The PC instruction saved is the `PC` at the `CALLI`.
    /// - The `Rd` refers to the destination register in the NEW window.
    /// - If the change to `CWP` makes it equal to `SWP`: stop execution,
    /// generate a trap, and go to address 0x80000020.
    /// CWP := CWP - 1 MOD 8, rd := PC; CC's have same rules as getipc.
    Callx(ShortInstruction),
    /// Call procedure at `PC` + `imm19`.
    /// - The `RS1` and `RS1` registers are read from the OLD window.
    /// - The PC instruction saved is the `PC` at the `CALLI`.
    /// - The `Rd` refers to the destination register in the NEW window.
    /// - If the change to `CWP` makes it equal to `SWP`: stop execution,
    /// generate a trap, and go to address 0x80000020.
    /// CWP := CWP - 1 MOD 8, rd := PC; CC's have same rules as getipc.
    Callr(LongInstruction),
    /// If conditional is true: PC := `rs1` + `shortSource`;
    Jmpx(ShortConditional),
    /// If conditional is true: PC += `imm19`;
    /// Test alignment: if newPC<0> == 1 then abort instruction and jump
    /// to 0x80000000.
    Jmpr(LongConditional),
    /// Return from the current procedure if conditional is true.
    /// CWP := CWP + 1 MOD 8.
    /// Notes:
    /// - `rs1` and `rs1` are read from the OLD window.
    /// - The usual use case of this instruction is with target address
    /// `rs1` + 8 (with `rs1`=`rd` of the call).
    Ret(ShortConditional),
    /// Return from interrupt if condition is true.
    /// CWP := CWP + 1 MOD 8.
    /// Notes:
    /// - PRIVILEGED INSTRUCTION.
    /// - `rs1` and `rs1` are read from the OLD window.
    /// - The usual use case of this instruction is with target address
    /// `rs1` + 8 (with `rs1`=`rd` of the call).
    Reti(ShortConditional),

    /// Shift left logical.
    Sll(ShortInstruction),
    /// Shift right logical.
    Srl(ShortInstruction),
    /// Shift right arithmetic.
    Sra(ShortInstruction),

    /// Bitwise OR.
    Or(ShortInstruction),
    /// Bitwise And.
    And(ShortInstruction),
    /// Bitwise Xor.
    Xor(ShortInstruction),

    /// Arithmetic add: d := s1 + s2;
    Add(ShortInstruction),
    /// Arithmetic add with constant: d := s1 + s2 + C;
    Addc(ShortInstruction),
    /// Arithmetic sub: d := s1 - s2; (d := s1 + NOT(s2) + 1)
    Sub(ShortInstruction),
    /// Arithmetic sub with constant: d := s1 - s2 - NOT(C); (d := s1 + NOT(s2) + C)
    Subc(ShortInstruction),
    /// Subtract inverse: d := s2 - s1; (d := s2 + NOT(s1))
    Subi(ShortInstruction),
    /// Subtract inverse with constant: d := s2 - s1 - NOT(C); (d := s2 - s1 - NOT(C))
    Subci(ShortInstruction),

    /// Load high: Load 19 bit immediate into top 19 bits of destination register,
    ///  and set the bottom 13 bits to 0.
    Ldhi(LongInstruction),
    /// Load word, register indexed.
    Ldxw(ShortInstruction),
    /// Load word, long-immediate.
    Ldrw(LongInstruction),

    /// Load half signed, register indexed.
    Ldxhs(ShortInstruction),
    /// Load half signed, long-immediate.
    Ldrhs(LongInstruction),
    /// Load half unsigned, register indexed.
    Ldxhu(ShortInstruction),
    /// Load half unsigned, long-immediate.
    Ldrhu(LongInstruction),

    /// Load byte signed, register indexed.
    Ldxbs(ShortInstruction),
    /// Load byte signed, long-immediate.
    Ldrbs(LongInstruction),
    /// Load byte unsigned, register indexed.
    Ldxbu(ShortInstruction),
    /// Load byte unsigned, long-immediate.
    Ldrbu(LongInstruction),

    /// Store word, register indexed.
    Stxw(ShortInstruction),
    /// Store word, long-immediate.
    Strw(LongInstruction),

    /// Store half, register indexed.
    Stxh(ShortInstruction),
    /// Store half, long-immediate.
    Strh(LongInstruction),

    /// Store byte, register indexed.
    Stxb(ShortInstruction),
    /// Store byte, long-immediate.
    Strb(LongInstruction),
}

/// Opcode errors.
#[derive(PartialEq, Eq, Clone)]
pub enum DecodeError {
    /// Indicates an invalid instruction. The first u32 indicates which bits are invalid,
    /// the final u32 is the whole opcode.
    InvalidInstruction(u32, u32),
    InvalidJumpCondition,

    /// Indicates some bug in this program with a string description.
    CodeError(String),
}

/// Get the RISC-II conditional type from a opcode<22-19>.
/// opcode A RISC-II opcode.
/// return RISC-II conditional, or DecodeError if 0.
fn get_cond_from_opcode(opcode: u32) -> Result<Conditional, DecodeError> {
    type C = Conditional;
    Ok(match (opcode & 0x780000) >> 18 {
        1 => C::Gt,
        2 => C::Le,
        3 => C::Ge,
        4 => C::Lt,
        5 => C::Hi,
        6 => C::Los,
        7 => C::Lonc,
        8 => C::Hisc,
        9 => C::Pl,
        10 => C::Mi,
        11 => C::Ne,
        12 => C::Eq,
        13 => C::Nv,
        14 => C::V,
        15 => C::Alw,
        _ => return Err(DecodeError::InvalidJumpCondition),
    })
}

// Public function declarations.

pub fn decode(opcode: u32) -> Result<Instruction, DecodeError> {
    type I = Instruction;
    // SCC flag (<24>).
    let scc = opcode & 0x1000000 != 0;
    // Destination bits (<23-19>).
    let dest = ((opcode & 0xF80000) >> 18) as u8;
    // Short-immediate RS1 value (<18-14>).
    let rs1 = ((opcode & 0x7C000) >> 13) as u8;
    // Immediate-mode bottom 19 bits <18-0>.
    let imm19 = opcode & 0x7FFFF;
    // Short source immediate-mode bottom 13 bits <12-0> or rs1 <4-0>.
    let short_source = if opcode & 0x2000 != 0 {
        ShortSource::UImm13(opcode & 0x1fff)
    } else {
        ShortSource::Reg((opcode & 0x1f) as u8)
    }; // TODO fix ambiguous sign problem.
       // The opcode itself.
    let op = (opcode & 0xFE000000) >> 25;

    let cond = get_cond_from_opcode(opcode);

    let bottom_nibble = op & 0xf;
    // Match the opcode's prefix.
    Ok(match op >> 5 {
        // Match the bottom four bytes of the opcode's prefix.
        0 => match bottom_nibble {
            0 => return Err(DecodeError::InvalidInstruction(0x0f, opcode)),
            1 => I::Calli(ShortInstruction::new(scc, dest, rs1, short_source)),
            2 => I::GetPSW(ShortInstruction::new(scc, dest, rs1, short_source)),
            3 => I::GetIPC(ShortInstruction::new(scc, dest, rs1, short_source)),
            4 => I::PutPSW(ShortInstruction::new(scc, dest, rs1, short_source)),
            5..=7 => return Err(DecodeError::InvalidInstruction(0x0f, opcode)),
            8 => I::Callx(ShortInstruction::new(scc, dest, rs1, short_source)),
            9 => I::Callr(LongInstruction::new(scc, dest, imm19)),
            10..=11 => return Err(DecodeError::InvalidInstruction(0x0f, opcode)),
            12 => I::Jmpx(ShortConditional::new(scc, cond?, rs1, short_source)),
            13 => I::Jmpr(LongConditional::new(scc, cond?, imm19)),
            14 => I::Ret(ShortConditional::new(scc, cond?, rs1, short_source)),
            15 => I::Reti(ShortConditional::new(scc, cond?, rs1, short_source)),
            // Should never be reached.
            _ => {
                return Err(DecodeError::CodeError(String::from(
                    "Match bottom four bytes of opcode prefix",
                )))
            }
        },
        1 => match bottom_nibble {
            0 => return Err(DecodeError::InvalidInstruction(0x0f, opcode)),
            1 => I::Sll(ShortInstruction::new(scc, dest, rs1, short_source)),
            2 => I::Sra(ShortInstruction::new(scc, dest, rs1, short_source)),
            3 => I::Srl(ShortInstruction::new(scc, dest, rs1, short_source)),
            4 => I::Ldhi(LongInstruction::new(scc, dest, imm19)),
            5 => I::And(ShortInstruction::new(scc, dest, rs1, short_source)),
            6 => I::Or(ShortInstruction::new(scc, dest, rs1, short_source)),
            7 => I::Xor(ShortInstruction::new(scc, dest, rs1, short_source)),
            8 => I::Add(ShortInstruction::new(scc, dest, rs1, short_source)),
            9 => I::Addc(ShortInstruction::new(scc, dest, rs1, short_source)),
            10..=11 => return Err(DecodeError::InvalidInstruction(0x0f, opcode)),
            12 => I::Sub(ShortInstruction::new(scc, dest, rs1, short_source)),
            13 => I::Subc(ShortInstruction::new(scc, dest, rs1, short_source)),
            14 => I::Subi(ShortInstruction::new(scc, dest, rs1, short_source)),
            15 => I::Subci(ShortInstruction::new(scc, dest, rs1, short_source)),
            // Should never be reached.
            _ => {
                return Err(DecodeError::CodeError(String::from(
                    "Match bottom four bytes of opcode prefix",
                )))
            }
        },
        2 => match bottom_nibble {
            0..=5 => return Err(DecodeError::InvalidInstruction(0x0f, opcode)),
            6 => I::Ldxw(ShortInstruction::new(scc, dest, rs1, short_source)),
            7 => I::Ldrw(LongInstruction::new(scc, dest, imm19)),
            8 => I::Ldxhu(ShortInstruction::new(scc, dest, rs1, short_source)),
            9 => I::Ldrhu(LongInstruction::new(scc, dest, imm19)),
            10 => I::Ldxhs(ShortInstruction::new(scc, dest, rs1, short_source)),
            11 => I::Ldrhs(LongInstruction::new(scc, dest, imm19)),
            12 => I::Ldxbu(ShortInstruction::new(scc, dest, rs1, short_source)),
            13 => I::Ldrbu(LongInstruction::new(scc, dest, imm19)),
            14 => I::Ldxbs(ShortInstruction::new(scc, dest, rs1, short_source)),
            15 => I::Ldrbs(LongInstruction::new(scc, dest, imm19)),
            // Should never be reached.
            _ => {
                return Err(DecodeError::CodeError(String::from(
                    "Match bottom four bytes of opcode prefix",
                )))
            }
        },
        3 => match bottom_nibble {
            0..=5 => return Err(DecodeError::InvalidInstruction(0x0f, opcode)),
            6 => I::Stxw(ShortInstruction::new(scc, dest, rs1, short_source)),
            7 => I::Strw(LongInstruction::new(scc, dest, imm19)),
            8..=9 => return Err(DecodeError::InvalidInstruction(0x0f, opcode)),
            10 => I::Stxh(ShortInstruction::new(scc, dest, rs1, short_source)),
            11 => I::Strh(LongInstruction::new(scc, dest, imm19)),
            12..=13 => return Err(DecodeError::InvalidInstruction(0x0f, opcode)),
            14 => I::Stxb(ShortInstruction::new(scc, dest, rs1, short_source)),
            15 => I::Strb(LongInstruction::new(scc, dest, imm19)),
            // Should never be reached.
            _ => {
                return Err(DecodeError::CodeError(String::from(
                    "Match bottom four bytes of opcode prefix",
                )))
            }
        },
        // Top bit is 1, meaning an extension opcode.
        4..=8 => match opcode {
            // TODO
            _ => return Err(DecodeError::CodeError(String::from("Not yet implemented!"))),
        },
        _ => return Err(DecodeError::InvalidInstruction(0x8, opcode)),
    })
}

pub fn decode_file(file: &Vec<u8>, pos: usize) -> Result<(), DecodeError> {
    let result = 0usize;

    for i in (0..file.len()).step_by(4) {
        decode(u32::from_ne_bytes(file[pos..pos + 4].try_into().unwrap()))?;
    }

    Ok(())
}

// Impls.

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidInstruction(i, op) => write!(f, "Invalid: 0x{:x}, opcode: 0x{:x}", i, op),
            Self::InvalidJumpCondition => write!(f, "Invalid jump condition"),
            Self::CodeError(s) => write!(f, "Error in RISC II emulator: {}", s),
        }
    }
}

impl LongInstruction {
    pub fn new(scc: bool, dest: u8, imm19: u32) -> Self {
        Self {
            scc: scc,
            dest: dest,
            imm19: imm19,
        }
    }
}

impl fmt::Display for LongInstruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Scc: {}, dest: {}, imm19: {}",
            self.scc, self.dest, self.imm19
        )
    }
}

impl LongConditional {
    pub fn new(scc: bool, dest: Conditional, imm19: u32) -> Self {
        Self {
            scc: scc,
            dest: dest,
            imm19: imm19,
        }
    }
}

impl fmt::Display for LongConditional {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Scc: {}, cond: {}, imm19: {}",
            self.scc, self.dest, self.imm19
        )
    }
}

impl ShortInstruction {
    pub fn new(scc: bool, dest: u8, rs1: u8, short_source: ShortSource) -> Self {
        Self {
            scc: scc,
            dest: dest,
            rs1: rs1,
            short_source: short_source,
        }
    }
}

impl fmt::Display for ShortInstruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Scc: {}, dest: {}, rs1: {}, short_source: {}",
            self.scc, self.dest, self.rs1, self.short_source
        )
    }
}

impl ShortConditional {
    pub fn new(scc: bool, dest: Conditional, rs1: u8, short_source: ShortSource) -> Self {
        Self {
            scc: scc,
            dest: dest,
            rs1: rs1,
            short_source: short_source,
        }
    }
}

impl fmt::Display for ShortConditional {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Scc: {}, conditional: {}, rs1: {}, short_source: {}",
            self.scc, self.dest, self.rs1, self.short_source
        )
    }
}

impl fmt::Display for ShortSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Reg(r) => write!(f, "Reg {}", r),
            Self::UImm13(u) => write!(f, "U{}", u),
            Self::SImm13(i) => write!(f, "S{}", i),
        }
    }
}

impl fmt::Display for Conditional {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Gt => write!(f, "Greater than"),
            Self::Le => write!(f, "Less than or equal to"),
            Self::Ge => write!(f, "Greater than or equal to"),
            Self::Lt => write!(f, "Less than"),
            Self::Hi => write!(f, "Higher than"),
            Self::Los => write!(f, "Lower than or same"),
            Self::Lonc => write!(f, "Lower than no carry"),
            Self::Hisc => write!(f, "Higher than no carry"),
            Self::Pl => write!(f, "Plus"),
            Self::Mi => write!(f, "Minus"),
            Self::Ne => write!(f, "Not equal"),
            Self::Eq => write!(f, "Equal"),
            Self::Nv => write!(f, "No overflow (signed arithmetic)"),
            Self::V => write!(f, "Overflow (signed arithmetic)"),
            Self::Alw => write!(f, "Always (constant 1)"),
        }
    }
}

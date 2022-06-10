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

/// Types of conditionals the RISC II supports.
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
    /// Now overflow (signed arithmetic).
    Nv,
    /// Overflow (signed arithmetic).
    V,
    /// Always (constant 1).
    Alw,
}

/// The 'source' of the instruction, which can either be a register name,
/// or a 13 bit immediate (signed or unsigned).
pub enum ShortSource {
    /// Register name.
    Reg(u8),
    /// Unsigned 13 bit immediate, 0-padded to 32 bits.
    UImm13(u32),
    /// Signed 13 bit immediate, Sign-extended to 32 bits.
    SImm13(i32),
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
    Calli(bool, u8),
    /// Get pointer to window. rd := (-1)<31:13> & PSW<12:0>;
    GetPSW(bool, u8, u8, ShortSource),
    /// Get the last Program Counter. rd := LSTPC.
    /// Iff SCC == true, Z := [LSTPC == 0]; N := LSTPC<31>; V,C := garbage.
    /// Notes:
    /// - PRIVILEGED INSTRUCTION.
    GetIPC(bool, u8, u8, ShortSource),
    /// Set PSW. PSW := [rs1 + ShortSource2]<12:0>;
    /// Notes:
    /// - PRIVILEGED INSTRUCTION.
    /// - SCC-bit MUST be false.
    /// - The next instruction CANNOT be `CALLX`, `CALLR`, CALLI`, `RET`, `RETI`,
    /// i.e. it cannot modify CWP. It also must not set the CC's.
    /// - Rd is discarded.
    /// - New PSW is not in effect until AFTER the next cycle following execution
    /// of this instruction.
    PutPSW(bool, u8, u8, ShortSource),
    /// Call procedure at `shortSource` + `rs1`.
    /// - The `RS1` and `RS1` registers are read from the OLD window.
    /// - The PC instruction saved is the `PC` at the `CALLI`.
    /// - The `Rd` refers to the destination register in the NEW window.
    /// - If the change to `CWP` makes it equal to `SWP`: stop execution,
    /// generate a trap, and go to address 0x80000020.
    /// CWP := CWP - 1 MOD 8, rd := PC; CC's have same rules as getipc.
    Callx(bool, u8, u8, ShortSource),
    /// Call procedure at `PC` + `imm19`.
    /// - The `RS1` and `RS1` registers are read from the OLD window.
    /// - The PC instruction saved is the `PC` at the `CALLI`.
    /// - The `Rd` refers to the destination register in the NEW window.
    /// - If the change to `CWP` makes it equal to `SWP`: stop execution,
    /// generate a trap, and go to address 0x80000020.
    /// CWP := CWP - 1 MOD 8, rd := PC; CC's have same rules as getipc.
    Callr(bool, u8, u32),
    /// If conditional is true: PC := `rs1` + `shortSource`;
    Jmpx(bool, Conditional, u8, ShortSource),
    /// If conditional is true: PC += `imm19`;
    /// Test alignment: if newPC<0> == 1 then abort instruction and jump
    /// to 0x80000000.
    Jmpr(bool, Conditional, u32),
    /// Return from the current procedure if conditional is true.
    /// CWP := CWP + 1 MOD 8.
    /// Notes:
    /// - `rs1` and `rs1` are read from the OLD window.
    /// - The usual use case of this instruction is with target address
    /// `rs1` + 8 (with `rs1`=`rd` of the call).
    Ret(bool, Conditional, u8, ShortSource),
    /// Return from interrupt if condition is true.
    /// CWP := CWP + 1 MOD 8.
    /// Notes:
    /// - PRIVILEGED INSTRUCTION.
    /// - `rs1` and `rs1` are read from the OLD window.
    /// - The usual use case of this instruction is with target address
    /// `rs1` + 8 (with `rs1`=`rd` of the call).
    Reti(bool, Conditional, u8, ShortSource),

    /// Shift left logical.
    Sll(bool, u8, u8, ShortSource),
    /// Shift right logical.
    Srl(bool, u8, u8, ShortSource),
    /// Shift right arithmetic.
    Sra(bool, u8, u8, ShortSource),

    /// Bitwise OR.
    Or(bool, u8, u8, ShortSource),
    /// Bitwise And.
    And(bool, u8, u8, ShortSource),
    /// Bitwise Xor.
    Xor(bool, u8, u8, ShortSource),

    /// Arithmetic add: d := s1 + s2;
    Add(bool, u8, u8, ShortSource),
    /// Arithmetic add with constant: d := s1 + s2 + C;
    Addc(bool, u8, u8, ShortSource),
    /// Arithmetic sub: d := s1 - s2; (d := s1 + NOT(s2) + 1)
    Sub(bool, u8, u8, ShortSource),
    /// Arithmetic sub with constant: d := s1 - s2 - NOT(C); (d := s1 + NOT(s2) + C)
    Subc(bool, u8, u8, ShortSource),
    /// Subtract inverse: d := s2 - s1; (d := s2 + NOT(s1))
    Subi(bool, u8, u8, ShortSource),
    /// Subtract inverse with constant: d := s2 - s1 - NOT(C); (d := s2 - s1 - NOT(C))
    Subci(bool, u8, u8, ShortSource),

    /// Load high: Load 19 bit immediate into top 19 bits of destination register,
    ///  and set the bottom 13 bits to 0.
    Ldhi(bool, u8, u32),
    /// Load word, register indexed.
    Ldxw(bool, u8, u8, ShortSource),
    /// Load word, long-immediate.
    Ldrw(bool, u8, u32),

    /// Load half signed, register indexed.
    Ldxhs(bool, u8, u8, ShortSource),
    /// Load half signed, long-immediate.
    Ldrhs(bool, u8, u32),
    /// Load half unsigned, register indexed.
    Ldxhu(bool, u8, u8, ShortSource),
    /// Load half unsigned, long-immediate.
    Ldrhu(bool, u8, u32),

    /// Load byte signed, register indexed.
    Ldxbs(bool, u8, u8, ShortSource),
    /// Load byte signed, long-immediate.
    Ldrbs(bool, u8, u32),
    /// Load byte unsigned, register indexed.
    Ldxbu(bool, u8, u8, ShortSource),
    /// Load byte unsigned, long-immediate.
    Ldrbu(bool, u8, u32),

    /// Store word, register indexed.
    Stxw(bool, u8, u8, ShortSource),
    /// Store word, long-immediate.
    Strw(bool, u8, u32),

    /// Store half, register indexed.
    Stxh(bool, u8, u8, ShortSource),
    /// Store half, long-immediate.
    Strh(bool, u8, u32),

    /// Store byte, register indexed.
    Stxb(bool, u8, u8, ShortSource),
    /// Store byte, long-immediate.
    Strb(bool, u8, u32),
}

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
    let op = (opcode & 0xFE) >> 24;

    let cond = get_cond_from_opcode(opcode);

    // Math the opcode's prefix.
    Ok(match op >> 5 {
        // Match the bottom four bytes of the opcode's prefix.
        0 => match op & 0xF {
            0 => return Err(DecodeError::InvalidInstruction(0x0f, opcode)),
            1 => I::Calli(scc, dest),
            2 => I::GetPSW(scc, dest, rs1, short_source),
            3 => I::GetIPC(scc, dest, rs1, short_source),
            4 => I::PutPSW(scc, dest, rs1, short_source),
            5..=7 => return Err(DecodeError::InvalidInstruction(0x0f, opcode)),
            8 => I::Callx(scc, dest, rs1, short_source),
            9 => I::Callr(scc, dest, imm19),
            10..=11 => return Err(DecodeError::InvalidInstruction(0x0f, opcode)),
            12 => I::Jmpx(scc, cond?, rs1, short_source),
            13 => I::Jmpr(scc, cond?, imm19),
            14 => I::Ret(scc, cond?, rs1, short_source),
            15 => I::Reti(scc, cond?, rs1, short_source),
            // Should never be reached.
            _ => {
                return Err(DecodeError::CodeError(String::from(
                    "Match bottom four bytes of opcode prefix",
                )))
            }
        },
        1 => match op & 0xF {
            0 => return Err(DecodeError::InvalidInstruction(0x0f, opcode)),
            1 => I::Sll(scc, dest, rs1, short_source),
            2 => I::Sra(scc, dest, rs1, short_source),
            3 => I::Srl(scc, dest, rs1, short_source),
            4 => I::Ldhi(scc, dest, imm19),
            5 => I::And(scc, dest, rs1, short_source),
            6 => I::Or(scc, dest, rs1, short_source),
            7 => I::Xor(scc, dest, rs1, short_source),
            8 => I::Add(scc, dest, rs1, short_source),
            9 => I::Addc(scc, dest, rs1, short_source),
            10..=11 => return Err(DecodeError::InvalidInstruction(0x0f, opcode)),
            12 => I::Sub(scc, dest, rs1, short_source),
            13 => I::Subc(scc, dest, rs1, short_source),
            14 => I::Subi(scc, dest, rs1, short_source),
            15 => I::Subci(scc, dest, rs1, short_source),
            // Should never be reached.
            _ => {
                return Err(DecodeError::CodeError(String::from(
                    "Match bottom four bytes of opcode prefix",
                )))
            }
        },
        2 => match op & 0xF {
            0..=5 => return Err(DecodeError::InvalidInstruction(0x0f, opcode)),
            6 => I::Ldxw(scc, dest, rs1, short_source),
            7 => I::Ldrw(scc, dest, imm19),
            8 => I::Ldxhu(scc, dest, rs1, short_source),
            9 => I::Ldrhu(scc, dest, imm19),
            10 => I::Ldxhs(scc, dest, rs1, short_source),
            11 => I::Ldrhs(scc, dest, imm19),
            12 => I::Ldxbu(scc, dest, rs1, short_source),
            13 => I::Ldrbu(scc, dest, imm19),
            14 => I::Ldxbs(scc, dest, rs1, short_source),
            15 => I::Ldrbs(scc, dest, imm19),
            // Should never be reached.
            _ => {
                return Err(DecodeError::CodeError(String::from(
                    "Match bottom four bytes of opcode prefix",
                )))
            }
        },
        3 => match op & 0xF {
            0..=5 => return Err(DecodeError::InvalidInstruction(0x0f, opcode)),
            6 => I::Stxw(scc, dest, rs1, short_source),
            7 => I::Strw(scc, dest, imm19),
            8..=9 => return Err(DecodeError::InvalidInstruction(0x0f, opcode)),
            10 => I::Stxh(scc, dest, rs1, short_source),
            11 => I::Strh(scc, dest, imm19),
            12..=13 => return Err(DecodeError::InvalidInstruction(0x0f, opcode)),
            14 => I::Stxb(scc, dest, rs1, short_source),
            15 => I::Strb(scc, dest, imm19),
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

/// An emulator for the RISC-II microprocessor architecture.
/// (C) Ryan Jeffrey <ryan@ryanmj.xyz>, 2022
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
#[cfg(test)]
mod main_test;

enum Conditional {
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
enum ShortSource {
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
enum Instruction {
    /// Privileged instruction: call
    Calli(bool, u8, u8, ShortSource),

    GetPSW(bool, u8, u8, ShortSource),

    GetIPC(bool, u8, u8, ShortSource),

    PutPSW(bool, u8, u8, ShortSource),

    Callx(bool, u8, u8, ShortSource),

    Callr(bool, u8, u32),

    Jmpx(bool, Conditional, u8, ShortSource),
    Jmpr(bool, Conditional, u32),
    Ret(bool, u8, u8, ShortSource),
    Reti(bool, u8, u8, ShortSource),

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
    Subic(bool, u8, u8, ShortSource),

    /// Load high: Load 19 bit immediate into top 19 bits of destination register,
    /// leaving the bottom 13 bits untouched.
    Ldhi(bool, u8, u8, ShortSource),
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

fn main() -> Result<(), String> {
    Ok(())
}

// RISC II cpu instruction info.
// "execute" and then "commit".
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

use std::fmt;
use std::fmt::LowerHex;

/// Types of conditionals the RISC II supports.
#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Conditional {
    /// Greater than (signed >).
    Gt = 1,
    /// Less than or equal to (signed <=).
    Le = 2,
    /// Greater than or equal to (signed >=).
    Ge = 3,
    /// Less than (signed <).
    Lt = 4,
    /// Higher than (unsigned >).
    Hi = 5,
    /// Lower than or same (unsigned <=).
    Los = 6,
    /// Lower than no carry (unsigned <).
    Lonc = 7,
    /// Higher than, carry (unsigned >=).
    Hisc = 8,
    /// Plus (test sign).
    Pl = 9,
    /// Minus (test sign).
    Mi = 10,
    /// Not equal.
    Ne = 11,
    /// Equal.
    Eq = 12,
    /// No overflow (signed arithmetic).
    Nv = 13,
    /// Overflow (signed arithmetic).
    V = 14,
    /// Always (constant 1).
    Alw = 15,
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

/// Short instruction format data.
#[derive(PartialEq, Eq, Copy, Clone)]
pub struct ShortInstruction {
    /// Update CC bit.
    scc: bool,
    /// Destination register.
    dest: u8,
    /// Source register.
    rs1: u8,
    /// Short source data.
    short_source: ShortSource,
}

/// Long instruction format data.
#[derive(PartialEq, Eq, Copy, Clone)]
pub struct LongInstruction {
    /// Update CC bit.
    scc: bool,
    /// Destination register.
    dest: u8,
    /// 19 bit constant.
    imm19: u32,
}

/// Short conditional instruction format data.
#[derive(PartialEq, Eq, Copy, Clone)]
pub struct ShortConditional {
    /// Update CC bit.
    scc: bool,
    /// Destination register.
    dest: Conditional,
    /// Source register.
    rs1: u8,
    /// Short source data.
    short_source: ShortSource,
}

/// Long conditional instruction format data.
#[derive(PartialEq, Eq, Copy, Clone)]
pub struct LongConditional {
    /// Update CC bit.
    scc: bool,
    /// Destination register.
    dest: Conditional,
    /// 19 bit constant.
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

// Impls.

impl ShortSource {
    /// Create a new short source.
    /// # Arguments
    /// * `opcode` - The current opcode being executed.
    /// * `signed` - True if `self` is a 13 bit constant and signed. This
    /// is ignored if `self` is not a constant.
    pub fn new(opcode: u32, signed: bool) -> Self {
        // Short source immediate-mode bottom 13 bits <12-0> or rs1 <4-0>.
        if opcode & 0x2000 != 0 {
            let mut tmp = Self::UImm13(opcode & 0x1fff);
            if signed {
                tmp.uimm_to_simm()
            } else {
                tmp
            }
        } else {
            Self::Reg((opcode & 0x1f) as u8)
        }
    }

    /// Create a new short source. If `self` is an unsigned constant,
    /// convert it a signed constant. Else, return `self`.
    pub fn uimm_to_simm(&self) -> Self {
        match *self {
            Self::UImm13(u) => {
                if u & 0x1000 != 0 {
                    // Sign-extend the 13 bit value to 32 bits.
                    Self::SImm13(-(u as i32))
                } else {
                    Self::SImm13(u as i32)
                }
            }
            Self::SImm13(s) => *self,
            Self::Reg(r) => *self,
        }
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

impl LowerHex for ShortSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Reg(r) => write!(f, "Register {:x}", r),
            Self::UImm13(u) => write!(f, "(UImm) {:x}", u),
            Self::SImm13(i) => write!(f, "(SImm) {:x}", i),
        }
    }
}

impl LongInstruction {
    /// Create a new long instruction.
    /// # Arguments
    /// * `scc` - Should update CC's.
    /// * `dest` - Destination register.
    /// * `imm19` - 19 bit constant.
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
            "Scc: {}, dest: {}, imm19: 0x{:x} ({})",
            self.scc, self.dest, self.imm19, self.imm19
        )
    }
}

impl LongConditional {
    /// Create a new long conditional instruction.
    /// # Arguments
    /// * `scc` - Should update CC's.
    /// * `dest` - Conditional.
    /// * `imm19` - 19 bit constant.
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
            "Scc: {}, cond: {}, imm19: 0x{:x} ({})",
            self.scc, self.dest, self.imm19, self.imm19
        )
    }
}

impl ShortInstruction {
    /// Create a new long conditional instruction.
    /// # Arguments
    /// * `scc` - Should update CC's.
    /// * `dest` - Destination register.
    /// * `rs1` - Source register.
    /// * `short_source` - Short source.
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
            "Scc: {}, dest: {}, rs1: {}, short_source: 0x{:x} ({})",
            self.scc, self.dest, self.rs1, self.short_source, self.short_source
        )
    }
}

impl ShortConditional {
    /// Create a new long conditional instruction.
    /// # Arguments
    /// * `scc` - Should update CC's.
    /// * `dest` - Conditional.
    /// * `rs1` - Source register.
    /// * `short_source` - Short source.
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
            "Scc: {}, conditional: {}, rs1: {}, short_source: 0x{:x} {}",
            self.scc, self.dest, self.rs1, self.short_source, self.short_source
        )
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

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

use clock::Phase;
use data_path::{Control, DataPath};
use std::fmt;
use std::fmt::LowerHex;
use std::ops::Fn;

pub const SCC_LOC: u32 = 0x1000000;
pub const DEST_LOC: u32 = 0x00F80000;
pub const RS1_LOC: u32 = 0x7c000;
pub const RS2_LOC: u32 = 0x1f;
pub const IMM19_LOC: u32 = 0x7FFFF;
pub const SHORT_SOURCE_TYPE_LOC: u32 = 0x2000;
pub const OPCODE_LOC: u32 = 0xFE000000;
pub const SHORT_IMM_SIGN_LOC: u32 = 0x1000;
pub const SHORT_IMM_SIGNEXT_BITS: u32 = 0xFFFFE000;

// Public functions.

pub fn decode_opcode(instruction: u32) -> Control {}

// Enums and structs.

pub struct MicroOperation(fn(data_path: &mut DataPath) -> Self);

pub fn noop(dp: &mut DataPath) -> MicroOperation {
    MicroOperation::new(noop)
}

// Instructions change behavior of ALU, shifter, and for DIMM.
// Also which register is loaded into the ALU (stores load Rd in bi).
// Loads and stores suspend pipeline for 1 cycle.

//pub fn add_begin(dp: &mut DataPath) -> MicroOperation {}

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
    Imm13(u32),
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
    /// - Only meant for use by the interrupt handler.
    /// - The `RS1` and `RS1` registers are read from the OLD window.
    /// - The PC instruction saved is the `PC` at the `CALLI`.
    /// - The `Rd` refers to the destination register in the NEW window.
    /// - If the change to `CWP` makes it equal to `SWP`: stop execution,
    ///   generate a trap, and go to address 0x80000020.
    /// CWP := CWP - 1 MOD 8, rd := LSTPC;
    /// Iff SCC == true, Z := [LSTPC == 0]; N := LSTPC<31>; V,C := garbage.
    Calli(ShortInstruction),
    /// Get information on the current CPU state and store it in the bottom 13
    /// bits of rd. Set the top 19 bits to 1.
    /// Format of PSW:
    /// [0]: Carry bit
    /// [1]: Overflow bit
    /// [2]: Negative bit
    /// [3]: Zero bit
    /// [4]: Previous system mode bit.
    /// [5]: System mode bit.
    /// [6]: Interrupt enable bit.
    /// [7-9]: SWP register mod 8.
    /// [10-12]: CWP register mod 8.
    /// Notes:
    /// - Previous instruction must have its SCC bit off (for timing reasons?).
    /// - shortsource must be a register and r0.
    /// rd := (-1)<31:13> & PSW<12:0>;
    /// Iff SCC == true, Z := [dest == 0]; N := LSTPC<31>; V,C := 0.
    GetPSW(ShortInstruction),
    /// Get the last Program Counter. rd := LSTPC.
    /// Iff SCC == true, Z := [LSTPC == 0]; N := LSTPC<31>; V,C := garbage.
    /// Notes:
    /// - PRIVILEGED INSTRUCTION.
    /// - Not transparent to interrupts.
    /// - rs1 and short_source are discarded.
    GetLPC(ShortInstruction),
    /// Set PSW. PSW := [rs1 + ShortSource2]<12:0>;
    /// Format of PSW.
    /// [0]: Carry bit
    /// [1]: Overflow bit
    /// [2]: Negative bit
    /// [3]: Zero bit
    /// [4]: Previous system mode bit.
    /// [5]: System mode bit.
    /// [6]: Interrupt enable bit.
    /// [7-9]: SWP register.
    /// [10-12]: CWP registerr
    /// Notes:
    /// - PRIVILEGED INSTRUCTION.
    /// - SCC-bit MUST be off.
    /// - The next instruction CANNOT be `CALLX`, `CALLR`, `CALLI`, `RET`, `RETI`,
    /// i.e. it cannot modify CWP/SWP. It also cannot modify the CC's.
    /// - Rd is discarded.
    /// - New PSW is not in effect until AFTER the next cycle following execution
    /// of this instruction.
    PutPSW(ShortInstruction),
    /// Call procedure at `shortSource` + `rs1`.
    /// - The `RS1` and `RS2` registers are read from the OLD window.
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

impl Instruction {
    pub fn encode(&self) -> u32 {
        type I = Instruction;
        match *self {
            I::Calli(s) => s.encode(0b0000001),
            I::GetPSW(s) => s.encode(0b0000010),
            I::PutPSW(s) => s.encode(0b0000100),
            I::GetLPC(s) => s.encode(0b0000011),
            I::Callx(s) => s.encode(0b0001000),
            I::Sll(s) => s.encode(0b0010001),
            I::Srl(s) => s.encode(0b0010011),
            I::Sra(s) => s.encode(0b0010010),
            I::Or(s) => s.encode(0b0010110),
            I::And(s) => s.encode(0b0010101),
            I::Xor(s) => s.encode(0b0010111),
            I::Add(s) => s.encode(0b0011000),
            I::Addc(s) => s.encode(0b0011001),
            I::Sub(s) => s.encode(0b0011100),
            I::Subc(s) => s.encode(0b0011101),
            I::Subi(s) => s.encode(0b0011110),
            I::Subci(s) => s.encode(0b0011111),
            I::Ldxw(s) => s.encode(0b0100110),
            I::Ldxhu(s) => s.encode(0b0101000),
            I::Ldxhs(s) => s.encode(0b0101010),
            I::Ldxbu(s) => s.encode(0b0101100),
            I::Ldxbs(s) => s.encode(0b0101110),
            I::Stxw(s) => s.encode(0b0110110),
            I::Stxh(s) => s.encode(0b0111010),
            I::Stxb(s) => s.encode(0b0111110),
            I::Jmpx(s) => s.encode(0b0001100),
            I::Ret(s) => s.encode(0b0001110),
            I::Reti(s) => s.encode(0b0001111),

            I::Jmpr(l) => l.encode(0b0001101),
            I::Callr(l) => l.encode(0b0001001),
            I::Ldhi(l) => l.encode(0b0010100),
            I::Ldrw(l) => l.encode(0b0100111),
            I::Ldrhu(l) => l.encode(0b0101001),
            I::Ldrhs(l) => l.encode(0b0101011),
            I::Ldrbu(l) => l.encode(0b0101101),
            I::Ldrbs(l) => l.encode(0b0101111),
            I::Strw(l) => l.encode(0b0110111),
            I::Strh(l) => l.encode(0b0111011),
            I::Strb(l) => l.encode(0b0111111),
        }
    }
}

impl ShortSource {
    /// Create a new short source.
    /// # Arguments
    /// * `opcode` - The current opcode being executed.
    /// * `signed` - True if `self` is a 13 bit constant and signed. This
    /// is ignored if `self` is not a constant.
    pub fn new(opcode: u32, signed: bool) -> Self {
        // Short source immediate-mode bottom 13 bits <12-0> or rs1 <4-0>.
        if opcode & 0x2000 != 0 {
            let tmp = Self::Imm13(opcode & 0x1fff);
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
            Self::Imm13(u) => {
                if u & 0x1000 != 0 {
                    // Sign-extend the 13 bit value to 32 bits.
                    Self::Imm13((-(u as i32)) as u32)
                } else {
                    Self::Imm13(u)
                }
            }
            Self::Reg(_) => *self,
        }
    }
}

impl fmt::Display for ShortSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Reg(r) => write!(f, "Reg {}", r),
            Self::Imm13(u) => write!(f, "U{}", u),
        }
    }
}

impl LowerHex for ShortSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Reg(r) => write!(f, "Register {:x}", r),
            Self::Imm13(u) => write!(f, "(UImm) {:x}", u),
        }
    }
}

impl LongInstruction {
    pub fn encode(&self, opcode: u8) -> u32 {
        let scc = if self.scc { SCC_LOC } else { 0 };
        let dest = (self.dest as u32) << 19;
        let imm19 = self.imm19;

        ((opcode as u32) << 25) | scc | dest | imm19
    }
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
    pub fn encode(&self, opcode: u8) -> u32 {
        let scc = if self.scc { SCC_LOC } else { 0 };
        let dest = (get_opdata_from_cond(self.dest) as u32) << 19;
        let imm19 = self.imm19;
        println!("Lol 0x{:x}, 0x{:x}", dest, imm19);
        ((opcode as u32) << 25) | scc | dest | imm19
    }
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
    pub fn encode(&self, opcode: u8) -> u32 {
        let scc = if self.scc { SCC_LOC } else { 0 };
        let dest = (self.dest as u32) << 19;
        let rs1 = (self.rs1 as u32) << 14;
        let (ss, ss_bit) = match self.short_source {
            ShortSource::Reg(r) => (r as u32, 0),
            ShortSource::Imm13(i) => (i, SHORT_SOURCE_TYPE_LOC),
        };
        ((opcode as u32) << 25) | scc | dest | rs1 | ss_bit | ss
    }
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
    pub fn encode(&self, opcode: u8) -> u32 {
        let scc = if self.scc { SCC_LOC } else { 0 };
        let dest = (get_opdata_from_cond(self.dest) as u32) << 19;
        let rs1 = (self.rs1 as u32) << 14;
        let (ss, ss_bit) = match self.short_source {
            ShortSource::Reg(r) => (r as u32, 0),
            ShortSource::Imm13(i) => (i, SHORT_SOURCE_TYPE_LOC),
        };
        ((opcode as u32) << 25) | scc | dest | rs1 | ss_bit | ss
    }
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

impl MicroOperation {
    pub fn new(func: fn(data_path: &mut DataPath) -> Self) -> Self {
        Self { 0: func }
    }

    // TODO temporary until implementing Fn becomes stable.
    pub fn call(&self, data_path: &mut DataPath) -> Self {
        self.0(data_path)
    }
}

// Static functions.

fn get_opdata_from_cond(cond: Conditional) -> u8 {
    type C = Conditional;
    match cond {
        C::Gt => 1,
        C::Le => 2,
        C::Ge => 3,
        C::Lt => 4,
        C::Hi => 5,
        C::Los => 6,
        C::Lonc => 7,
        C::Hisc => 8,
        C::Pl => 9,
        C::Mi => 10,
        C::Ne => 11,
        C::Eq => 12,
        C::Nv => 13,
        C::V => 14,
        C::Alw => 15,
    }
}

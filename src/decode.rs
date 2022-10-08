// RISC-II decoder, the first stage in the pipeline. The next stages are
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
extern crate core;

use core::convert::TryInto;
use std::error::Error;
use std::fmt;
use util::Result;

use instruction::*;

macro_rules! bdeii {
    ( $( $loc:expr, $opcode:expr ),* ) => {
        {
            Err(Box::new($( DecodeError::InvalidInstruction { loc: $loc, opcode: $opcode } )*))
        }
    };
}

macro_rules! bdeij {
    ( $( $code:expr ),* ) => {
        {
            Err(Box::new($( DecodeError::InvalidJumpCondition { code: $code } )*))
        }
    };
}

macro_rules! bdece {
    ( $( $descr:expr ),* ) => {
        {
            Err(Box::new($( DecodeError::CodeError { descr: $descr } )*))
        }
    };
}

// Struct declarations.

/// Opcode errors.
#[derive(PartialEq, Eq, Clone)]
pub enum DecodeError {
    /// Indicates an invalid instruction. The first u32 indicates which bits are invalid,
    /// the final u32 is the whole opcode.
    InvalidInstruction {
        loc: u32,
        opcode: u32,
    },
    InvalidJumpCondition {
        code: u32,
    },

    /// Indicates some bug in this program with a string description.
    CodeError {
        descr: String,
    },
}

// Public function declarations.

impl fmt::Debug for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

pub fn decode(opcode: u32) -> Result<Instruction> {
    type I = Instruction;
    // SCC flag (<24>).
    let scc = opcode & SCC_LOC != 0;
    // Destination bits (<23-19>).
    let dest = ((opcode & DEST_LOC) >> 19) as u8;
    // Short-immediate RS1 value (<18-14>).
    let rs1 = ((opcode & RS1_LOC) >> 14) as u8;
    // Immediate-mode bottom 19 bits <18-0>.
    let imm19 = opcode & IMM19_LOC;
    // Short source immediate-mode bottom 13 bits <12-0> or rs1 <4-0>.
    let short_source = if opcode & SHORT_SOURCE_TYPE_LOC != 0 {
        ShortSource::Imm13(opcode & 0x1fff)
    } else {
        ShortSource::Reg((opcode & 0x1f) as u8)
    };
    // The opcode itself.
    let op = (opcode & 0xFE000000) >> 25;

    let cond = get_cond_from_opcode(opcode);

    let bottom_nibble = op & 0xf;
    // Match the opcode's prefix.
    Ok(match op >> 4 {
        // Match the bottom four bytes of the opcode's prefix.
        0 => match bottom_nibble {
            0 => return bdeii!(0xf, opcode),
            1 => I::Calli(ShortInstruction::new(scc, dest, rs1, short_source)),
            2 => I::GetPSW(ShortInstruction::new(scc, dest, rs1, short_source)),
            3 => I::GetLPC(ShortInstruction::new(scc, dest, rs1, short_source)),
            4 => I::PutPSW(ShortInstruction::new(scc, dest, rs1, short_source)),
            5..=7 => return bdeii!(0xf, opcode),
            8 => I::Callx(ShortInstruction::new(scc, dest, rs1, short_source)),
            9 => I::Callr(LongInstruction::new(scc, dest, imm19)),
            10..=11 => return bdeii!(0xf, opcode),
            12 => I::Jmpx(ShortConditional::new(scc, cond?, rs1, short_source)),
            13 => I::Jmpr(LongConditional::new(scc, cond?, imm19)),
            14 => I::Ret(ShortConditional::new(scc, cond?, rs1, short_source)),
            15 => I::Reti(ShortConditional::new(scc, cond?, rs1, short_source)),
            // Should never be reached.
            _ => return bdece!(format!("Match bottom four bytes of opcode prefix")),
        },
        1 => match bottom_nibble {
            0 => return bdeii!(0xf, opcode),
            1 => I::Sll(ShortInstruction::new(scc, dest, rs1, short_source)),
            2 => I::Sra(ShortInstruction::new(scc, dest, rs1, short_source)),
            3 => I::Srl(ShortInstruction::new(scc, dest, rs1, short_source)),
            4 => I::Ldhi(LongInstruction::new(scc, dest, imm19)),
            5 => I::And(ShortInstruction::new(scc, dest, rs1, short_source)),
            6 => I::Or(ShortInstruction::new(scc, dest, rs1, short_source)),
            7 => I::Xor(ShortInstruction::new(scc, dest, rs1, short_source)),
            8 => I::Add(ShortInstruction::new(scc, dest, rs1, short_source)),
            9 => I::Addc(ShortInstruction::new(scc, dest, rs1, short_source)),
            10..=11 => return bdeii!(0xf, opcode),
            12 => I::Sub(ShortInstruction::new(scc, dest, rs1, short_source)),
            13 => I::Subc(ShortInstruction::new(scc, dest, rs1, short_source)),
            14 => I::Subi(ShortInstruction::new(scc, dest, rs1, short_source)),
            15 => I::Subci(ShortInstruction::new(scc, dest, rs1, short_source)),
            // Should never be reached.
            _ => return bdece!(format!("Match bottom four bytes of opcode prefix")),
        },
        2 => match bottom_nibble {
            0..=5 => return bdeii!(0xf, opcode),
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
            _ => return bdece!(format!("Match bottom four bytes of opcode prefix")),
        },
        3 => match bottom_nibble {
            0..=5 => return bdeii!(0xf, opcode),
            6 => I::Stxw(ShortInstruction::new(scc, dest, rs1, short_source)),
            7 => I::Strw(LongInstruction::new(scc, dest, imm19)),
            8..=9 => return bdeii!(0xf, opcode),
            10 => I::Stxh(ShortInstruction::new(scc, dest, rs1, short_source)),
            11 => I::Strh(LongInstruction::new(scc, dest, imm19)),
            12..=13 => return bdeii!(0xf, opcode),
            14 => I::Stxb(ShortInstruction::new(scc, dest, rs1, short_source)),
            15 => I::Strb(LongInstruction::new(scc, dest, imm19)),
            // Should never be reached.
            _ => return bdece!(format!("Match bottom four bytes of opcode prefix")),
        },
        // Top bit is 1, meaning an extension opcode.
        4..=8 => match opcode {
            // TODO
            _ => return bdece!(format!("Not yet implemented!")),
        },
        _ => return bdeii!(0x8, opcode),
    })
}

pub fn decode_file(file: &Vec<u8>, pos: usize) -> Result<()> {
    let result = 0usize;

    for i in (0..file.len()).step_by(4) {
        decode(u32::from_ne_bytes(file[pos..pos + 4].try_into().unwrap()))?;
    }

    Ok(())
}

// Struct impls.

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Error in decoding instruction: {}",
            match self {
                Self::InvalidInstruction { loc: i, opcode: op } =>
                    format!("Invalid bits: 0x{:x}, opcode: 0x{:x}", i, op),
                Self::InvalidJumpCondition { code: code } =>
                    format!("Invalid jump condition: {} (should be 0-15)", code),
                Self::CodeError { descr: s } => format!("Error in RISC II emulator: {}", s),
            }
        )
    }
}

impl Error for DecodeError {}

// Private functions.

/// Get the RISC-II conditional type from a opcode<22-19>.
/// opcode A RISC-II opcode.
/// return RISC-II conditional, or DecodeError if 0.
fn get_cond_from_opcode(opcode: u32) -> Result<Conditional> {
    type C = Conditional;
    Ok(match (opcode & 0x780000) >> 19 {
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
        code => return bdeij!(code),
    })
}

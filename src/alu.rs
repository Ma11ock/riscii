// RISCII ALU emulator.
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

// Public structs.

use std::fmt;

/// Representation of the Arithmetic Logic Unit of the RISCII.
/// Implements bitwise and arithmetic operations, except for shifts.
#[derive(Clone, Copy)]
pub struct ALU {
    /// Input latch 1 for the ALU (fed by src1 or dest).
    pub ai: u32,
    /// Input latch 2 for the ALU (fed by src2 or dest).
    pub bi: u32,
}

// Impls.

impl ALU {
    /// Create a 0'd out ALU.
    pub fn new() -> Self {
        Self { ai: 0, bi: 0 }
    }

    // Bitwise.

    /// Bitwise AND the values in the input latches.
    pub fn and(&self) -> u32 {
        self.ai & self.bi
    }

    /// Bitwise OR the values in the input latches.
    pub fn or(&self) -> u32 {
        self.ai | self.bi
    }

    /// Bitwise XOR the values in the input latches.
    pub fn xor(&self) -> u32 {
        self.ai ^ self.bi
    }

    // Arithmetics.

    /// Add the values in the input latches.
    pub fn add(&self) -> u32 {
        self.ai + self.bi
    }

    /// Add the values in the input latches with the carry bit.
    pub fn addc(&self, carry: bool) -> u32 {
        self.ai + self.bi + (carry as u32)
    }

    /// Subtract the values in the input latches.
    pub fn sub(&self) -> u32 {
        self.ai - self.bi
    }

    /// Subtract the values in the input latches with the carry bit.
    pub fn subc(&self, carry: bool) -> u32 {
        self.ai - self.bi - (!carry as u32)
    }

    /// Subtract (inverse) the values in the input latches.
    pub fn subi(&self) -> u32 {
        self.bi - self.ai
    }

    /// Subtract (inverse) the values in the input latches with the carry bit.
    pub fn subci(&self, carry: bool) -> u32 {
        self.bi - self.ai - (!carry as u32)
    }
}

impl fmt::Display for ALU {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}, {}", self.ai, self.bi,)
    }
}

impl fmt::Debug for ALU {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Input register: {}, Output register: {}",
            self.ai, self.bi,
        )
    }
}

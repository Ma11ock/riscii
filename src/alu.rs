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

use data_path::SCCBits;
use instruction::SIGN_BIT_LOC;

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

    /// Bitwise AND the values in the input latches and return the result.
    pub fn and(&self) -> u32 {
        self.ai & self.bi
    }

    /// Bitwise AND the values in the input latches, return the result
    /// and SCC values.
    /// For bitwise operators the SCC bits are as follows:
    /// v = false
    /// c = false
    /// z = result == 0
    /// n = result as i32 < 0
    pub fn and_scc(&self) -> (u32, SCCBits) {
        let result = self.and();
        (
            result,
            SCCBits {
                z: result == 0,
                n: result & SIGN_BIT_LOC != 0,
                v: false,
                c: false,
            },
        )
    }

    /// Bitwise OR the values in the input latches and return the result.
    pub fn or(&self) -> u32 {
        self.ai | self.bi
    }

    /// Bitwise OR the values in the input latches, return the result
    /// and the SCC values.
    /// For bitwise operators the SCC bits are as follows:
    /// v = false
    /// c = false
    /// z = result == 0
    /// n = result as i32 < 0
    pub fn or_scc(&self) -> (u32, SCCBits) {
        let result = self.or();
        (
            result,
            SCCBits {
                z: result == 0,
                n: result & SIGN_BIT_LOC != 0,
                v: false,
                c: false,
            },
        )
    }

    /// Bitwise XOR the values in the input latches.
    pub fn xor(&self) -> u32 {
        self.ai ^ self.bi
    }

    /// Bitwise XOR the values in the input latches, return the result
    /// and the SCC values.
    /// For bitwise operators the SCC bits are as follows:
    /// v = false
    /// c = false
    /// z = result == 0
    /// n = result as i32 < 0
    pub fn xor_scc(&self) -> (u32, SCCBits) {
        let result = self.xor();
        (
            result,
            SCCBits {
                z: result == 0,
                n: result & SIGN_BIT_LOC != 0,
                v: false,
                c: false,
            },
        )
    }

    // Arithmetics.

    /// Add the values in the input latches and return the sum.
    pub fn add(&self) -> u32 {
        self.ai + self.bi
    }

    /// Add the values in the input latches, return the sum and SCC values.
    /// For addition the SCC bits are as follows:
    /// v = Signed overflow occurred
    /// c = unsigned overflow occurred
    /// z = result == 0
    /// n = result as i32 < 0
    pub fn add_scc(&self) -> (u32, SCCBits) {
        let (iresult, v) = (self.ai as i32).overflowing_add(self.bi as i32);
        let (result, c) = self.ai.overflowing_add(self.bi);
        let z = result == 0;
        let n = iresult < 0;

        (
            result,
            SCCBits {
                z: z,
                n: n,
                c: c,
                v: v,
            },
        )
    }

    /// Add the values in the input latches with the carry bit.
    pub fn addc(&self, carry: bool) -> u32 {
        self.ai + self.bi + (carry as u32)
    }

    /// Add the values in the input latches with the carry bit, return the
    /// sum and the SCC values.
    /// For addition the SCC bits are as follows:
    /// v = Signed overflow occurred
    /// c = unsigned overflow occurred
    /// z = result == 0
    /// n = result as i32 < 0
    pub fn addc_scc(&self, carry: bool) -> (u32, SCCBits) {
        let (iresult, v) = (self.ai as i32 + carry as i32).overflowing_add(self.bi as i32);
        let (result, c) = (self.ai as u32 + carry as u32).overflowing_add(self.bi);
        let z = result == 0;
        let n = iresult.is_negative();

        (
            result,
            SCCBits {
                z: z,
                n: n,
                c: c,
                v: v,
            },
        )
    }

    /// Subtract the values in the input latches and return the difference.
    /// Use `self.ai` is the minuend and use `self.bi` as the subtrahend.
    pub fn sub(&self) -> u32 {
        self.ai - self.bi
    }

    /// Subtract the values in the input latches, return SCC values. Return
    /// difference and the SCC values.
    /// Use `self.ai` is the minuend and use `self.bi` as the subtrahend.
    /// For subtraction the SCC bits are as follows:
    /// v = Signed overflow occurred
    /// c = unsigned overflow NOT occurred
    /// z = result == 0
    /// n = result as i32 < 0
    pub fn sub_scc(&self) -> (u32, SCCBits) {
        let (iresult, v) = (self.ai as i32).overflowing_sub(self.bi as i32);
        let (result, c) = self.ai.overflowing_sub(self.bi);
        let z = result == 0;
        let n = iresult.is_negative();

        (
            result,
            SCCBits {
                z: z,
                c: !c,
                n: n,
                v: v,
            },
        )
    }

    /// Subtract the values in the input latches and add the carry bit to the difference.
    /// Return the sum.
    /// Use `self.ai` is the minuend and use `self.bi` as the subtrahend and add carry to the difference.
    pub fn subc(&self, carry: bool) -> u32 {
        self.ai - self.bi + carry as u32
    }

    /// Subtract the values in the input latches and add the carry bit.
    /// Return the sum and SCC values.
    /// Use `self.ai` is the minuend and use `self.bi` as the subtrahend and add carry to the difference.
    /// For subtraction the SCC bits are as follows:
    /// v = Signed overflow occurred
    /// c = unsigned overflow NOT occurred
    /// z = result == 0
    /// n = result as i32 < 0
    pub fn subc_scc(&self, carry: bool) -> (u32, SCCBits) {
        let (iresult, v) = (self.ai as i32 - self.bi as i32).overflowing_add(carry as i32);
        let (result, c) = (self.ai - self.bi).overflowing_add(carry as u32);
        let z = result == 0;
        let n = iresult.is_negative();

        (
            result,
            SCCBits {
                z: z,
                c: !c,
                n: n,
                v: v,
            },
        )
    }

    /// Subtract the values in the input latches in the reverse order of `sub`, return
    /// the difference.
    /// Use `self.bi` is the minuend and use `self.ai` as the subtrahend.
    pub fn subi(&self) -> u32 {
        self.bi - self.ai
    }

    /// Subtract the values in the input latches in the reverse order of `sub`, return
    /// the difference and SCC bits.
    /// Use `self.bi` is the minuend and use `self.ai` as the subtrahend.
    /// For subtraction the SCC bits are as follows:
    /// v = Signed overflow occurred
    /// c = unsigned overflow NOT occurred
    /// z = result == 0
    /// n = result as i32 < 0
    pub fn subi_scc(&self) -> (u32, SCCBits) {
        let (iresult, v) = (self.bi as i32).overflowing_sub(self.ai as i32);
        let (result, c) = self.bi.overflowing_sub(self.ai);
        let z = result == 0;
        let n = iresult.is_negative();

        (
            result,
            SCCBits {
                z: z,
                c: c,
                n: n,
                v: v,
            },
        )
    }

    /// Subtract the values in the input latches in the reverse order of `sub`,
    /// and add the carry bit. Return the sum.
    /// Use `self.bi` is the minuend and use `self.ai` as the subtrahend.
    pub fn subci(&self, carry: bool) -> u32 {
        self.bi - self.ai - (!carry as u32)
    }

    /// Subtract the values in the input latches in the reverse order of `sub`,
    /// and add the carry bit. Return the sum and the SCC values.
    /// Use `self.bi` is the minuend and use `self.ai` as the subtrahend.
    /// For subtraction the SCC bits are as follows:
    /// v = Signed overflow occurred
    /// c = unsigned overflow NOT occurred
    /// z = result == 0
    /// n = result as i32 < 0
    pub fn subci_scc(&self, carry: bool) -> (u32, SCCBits) {
        let (iresult, v) = (self.bi as i32 - self.ai as i32).overflowing_add(carry as i32);
        let (result, c) = (self.bi - self.ai).overflowing_add(carry as u32);
        let z = result == 0;
        let n = iresult.is_negative();

        (
            result,
            SCCBits {
                z: z,
                c: c,
                n: n,
                v: v,
            },
        )
    }

    /// Right logical shift of the input latches. Return the result.
    /// Use `self.ai` as the value to shift, and `self.bi` and the amount to shift by.
    pub fn shift_right_logical(&self) -> u32 {
        self.ai >> self.bi
    }

    /// Right logical shift of the input latches. Return the result and the SCC values.
    /// Use `self.ai` as the value to shift, and `self.bi` and the amount to shift by.
    /// For bitwise operators the SCC bits are as follows:
    /// v = false
    /// c = false
    /// z = result == 0
    /// n = result as i32 < 0
    pub fn shift_right_logical_scc(&self) -> (u32, SCCBits) {
        let result = self.shift_right_logical();
        (
            result,
            SCCBits {
                z: result == 0,
                n: result & SIGN_BIT_LOC != 0,
                v: false,
                c: false,
            },
        )
    }

    /// Right arithmetic shift of the input latches. Return the result.
    /// Use `self.ai` as the value to shift, and `self.bi` and the amount to shift by.
    pub fn shift_right_arithmetic(&self) -> u32 {
        ((self.ai as i32) >> self.bi) as u32
    }

    /// Right logical shift of the input latches. Return the result and the SCC values.
    /// Use `self.ai` as the value to shift, and `self.bi` and the amount to shift by.
    /// For bitwise operators the SCC bits are as follows:
    /// v = false
    /// c = false
    /// z = result == 0
    /// n = result as i32 < 0
    pub fn shift_right_arithmetic_scc(&self) -> (u32, SCCBits) {
        let result = self.shift_right_arithmetic();
        (
            result,
            SCCBits {
                z: result == 0,
                n: result & SIGN_BIT_LOC != 0,
                v: false,
                c: false,
            },
        )
    }

    /// Left arithmetic shift of the input latches. Return the result.
    /// Use `self.ai` as the value to shift, and `self.bi` and the amount to shift by.
    pub fn shift_left_logical(&self) -> u32 {
        self.ai << self.bi
    }

    /// Right logical shift of the input latches. Return the result and the SCC values.
    /// Use `self.ai` as the value to shift, and `self.bi` and the amount to shift by.
    /// For bitwise operators the SCC bits are as follows:
    /// v = false
    /// c = false
    /// z = result == 0
    /// n = result as i32 < 0
    pub fn shift_left_arithmetic_scc(&self) -> (u32, SCCBits) {
        let result = self.shift_left_logical();
        (
            result,
            SCCBits {
                z: result == 0,
                n: result & SIGN_BIT_LOC != 0,
                v: false,
                c: false,
            },
        )
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
            "Input register one: {}, Input register two: {}",
            self.ai, self.bi,
        )
    }
}

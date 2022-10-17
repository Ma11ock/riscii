// RISCII Shifter emulator.
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

/// Representation of the Shifter for RISCII. Implements left and right shifting.
#[derive(Clone, Copy)]
pub struct Shifter {
    /// Input latch.
    pub src: u32,
    /// Amount to shift the input.
    pub s_ham: u8,
}

// Impls.

impl Shifter {
    /// Create a 0'd out shifter.
    pub fn new() -> Self {
        Self { src: 0, s_ham: 0 }
    }
    /// Left shift `src` by `s_ham` bits.
    pub fn shift_left(&self) -> u32 {
        self.src << (self.s_ham as u32)
    }

    /// Right shift `src` by `s_ham` bits.
    pub fn shift_right(&self) -> u32 {
        self.src >> (self.s_ham as u32)
    }
}

impl fmt::Display for Shifter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}, {}", self.src, self.s_ham,)
    }
}

impl fmt::Debug for Shifter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Input register: {}, SHam latch: {}",
            self.src, self.s_ham,
        )
    }
}

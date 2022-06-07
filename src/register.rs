/// RISC II register system.
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

/// The number of register windows the RISCII supports.
const NUM_WINDOWS: usize = 6;
/// The number of local registers per window.
const NUM_LOCALS: usize = 10;
/// The number of registers shared with the previous register window (input arguments).
const NUM_SHARED_PREV: usize = 6;
/// The number of registers shared with the next register window (output arguments).
const NUM_SHARED_NEXT: usize = 6;
/// The number of registers per window.
const WINDOW_SIZE: usize = NUM_LOCALS + NUM_SHARED_PREV + NUM_SHARED_NEXT;
/// Number of global registers.
const NUM_GLOBALS: usize = 10;

// Struct definitions.

/// A RISC II 32bit register.
struct Register(u32);

/// The CPU's register state.
struct State {
    /// Current window pointer, index of the currently active window.
    cwp: Register,
    /// Saved window pointer, the index of the youngest window saved in memory.
    swp: Register,
    /// Global registers.
    globals: [Register; NUM_GLOBALS],
    /// Register window stack.
    locals: [Register; NUM_WINDOWS * (NUM_LOCALS + NUM_SHARED_NEXT)],
}

// Struct implementations.

impl State {
    pub fn new() -> State {
        State {
            cwp: 0,
            swp: 0,
            globals: [0; NUM_GLOBALS],
            locals: [0; NUM_WINDOWS * (NUM_LOCALS + NUM_SHARED_NEXT)],
        }
    }

    fn inc_cwp(&self) {
        self.cwp += 1;
        while self.cwp >= self.swp {
            // TODO save the top window into memory.
            self.swp += 1;
        }
    }
}

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

/// The number of register window_regs the RISCII supports.
pub const NUM_WINDOW_REGS: usize = 6;
/// The number of local registers per window.
pub const NUM_LOCALS: usize = 10;
/// The number of registers shared with the previous register window (input arguments).
pub const NUM_SHARED_PREV: usize = 6;
/// The number of registers shared with the next register window (output arguments).
pub const NUM_SHARED_NEXT: usize = 6;
/// The number of registers per window.
pub const WINDOW_SIZE: usize = NUM_LOCALS + NUM_SHARED_PREV + NUM_SHARED_NEXT;
/// Number of global registers.
pub const NUM_GLOBALS: usize = 10;
/// Number of general purpose registers that exist in window_regs.
pub const NUM_WINDOW_REGISTERS: usize = NUM_WINDOW_REGS * (NUM_LOCALS + NUM_SHARED_NEXT);
/// Number of "special" registers (cwp, swp, sp, etc.).
pub const NUM_SPECIAL_REGISTERS: usize = 2;
/// The total number of registers on the system.
pub const TOTAL_NUM_REGISTERS: usize = NUM_SPECIAL_REGISTERS + NUM_GLOBALS + NUM_WINDOW_REGISTERS;
/// The size of a register::State object (in bytes).
pub const SIZEOF_STATE: usize = TOTAL_NUM_REGISTERS * 4;
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
    window_regs: [Register; NUM_WINDOW_REGISTERS],
}

// Struct implementations.

impl State {
    /// Create a 0'd out register window.
    pub fn new() -> State {
        State {
            cwp: 0,
            swp: 0,
            globals: [0; NUM_GLOBALS],
            window_regs: [0; NUM_WINDOW_REGISTERS],
        }
    }

    /// Create a register state from a buffer.
    /// # Arguments
    /// * `buffer` - A byte buffer that is the size of the sum of of register::State's
    /// members (in bytes) (see `SIZEOF_STATE`).
    /// The registers should appear in the following order:
    /// - CWP
    /// - SWP
    /// - Global registers
    /// - Window registers
    pub fn from_buf(buf: [u8; SIZEOF_STATE]) -> Self {
        // Offset used for gloabls and window_regs.
        let mut cur_offset = NUM_SPECIAL_REGISTERS * 4;
        Self {
            cwp: u32::from_be_bytes(buf[..4].try_into().unwrap()),
            swp: u32::from_be_bytes(buf[4..8].try_into().unwrap()),
            globals: {
                let mut result = [0u32; NUM_GLOBALS];
                for _ in NUM_GLOBALS {
                    result[i] =
                        u32::from_be_bytes(buf[cur_offset..cur_offset + 4].try_into().unwrap());
                    cur_offset += 4;
                }
                result
            },
            window_regs: {
                let mut result = [0u32; NUM_WINDOW_REGISTERS];
                for _ in NUM_WINDOW_REGISTERS {
                    result[i] =
                        u32::from_be_bytes(buf[cur_offset..cur_offset + 4].try_into().unwrap());
                    cur_offset += 4;
                }
                result
            },
        }
    }

    fn to_buf(&self) -> [u8; SIZEOF_STATE] {
        let mut result: [u8; SIZEOF_STATE] = [
            self.cwp.to_be_bytes(),
            self.swp.to_be_bytes(),
            {
                let mut tmp = [u8; NUM_GLOBALS * 4];
                for i in NUM_GLOBALS {
                    let bytes = self.globals[i].to_be_bytes();
                    tmp[i] = bytes[0];
                    tmp[i + 1] = bytes[1];
                    tmp[i + 1] = bytes[2];
                    tmp[i + 1] = bytes[3];
                }
                tmp
            },
            {
                let mut tmp = [u8; NUM_WINDOW_REGISTERS * 4];
                for i in NUM_WINDOW_REGISTERS {
                    let bytes = self.window_regs[i].to_be_bytes();
                    tmp[i] = bytes[0];
                    tmp[i + 1] = bytes[1];
                    tmp[i + 1] = bytes[2];
                    tmp[i + 1] = bytes[3];
                }
                tmp
            },
        ];
        result
    }

    fn push_reg_window(&mut self) {
        self.cwp += 1;
        while self.cwp >= self.swp {
            // TODO save the top window_regs into memory.
            self.swp += 1;
        }
    }

    fn pop_reg_window(&mut self) {
        self.cwp -= 1;
        while self.swp >= self.cwp {
            // TODO load window_regs from memory.
            self.swp -= 1;
        }
    }
}

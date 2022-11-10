// RISC II register system.
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
use instruction::ShortSource;
use memory::Memory;
use std::convert::TryInto;
use std::fmt;

use berr;

/// The number of register windows the RISCII supports.
pub const NUM_REG_WINDOWS: usize = 8;
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
/// Number of registers that adding a window adds to the total amount of registers.
pub const NUM_ADDED_PER_WINDOW: usize = NUM_LOCALS + NUM_SHARED_NEXT;
/// Number of general purpose registers that exist in window_regs.
pub const NUM_WINDOW_REGISTERS: usize = NUM_REG_WINDOWS * NUM_ADDED_PER_WINDOW;
/// Number of "special" registers (cwp, swp, sp, etc.).
pub const NUM_SPECIAL_REGISTERS: usize = 3;
/// The total number of registers on the system.
pub const TOTAL_NUM_REGISTERS: usize = NUM_SPECIAL_REGISTERS + NUM_GLOBALS + NUM_WINDOW_REGISTERS;
/// The size of a register::RegisterFile object (in bytes).
pub const SIZEOF_REG_FILE: usize = TOTAL_NUM_REGISTERS * 4;
/// The size of an instruction in bytes. Amount to increment the program counter registers by.
pub const SIZEOF_INSTRUCTION: u32 = 4;
/// Location of the interrupt bit in the PSW.
pub const INTERRUPT_LOC: u16 = 1 << 6;
/// Location of the system mode bit bit in the PSW.
pub const SYSTEM_LOC: u16 = 1 << 5;
/// Location of the previous system mode bit bit in the PSW.
pub const PREV_SYSTEM_LOC: u16 = 1 << 4;
/// Location of the zero bit bit in the PSW.
pub const ZERO_LOC: u16 = 1 << 3;
/// Location of the negative bit bit in the PSW.
pub const NEG_LOC: u16 = 1 << 2;
/// Location of the overflow bit bit in the PSW.
pub const OVERFLOW_LOC: u16 = 1 << 1;
/// Location of the carry bit bit in the PSW.
pub const CARRY_LOC: u16 = 1;
/// Location of the saved window pointer bits in the PSW.
pub const SWP_LOC: u16 = 0x7 << 7;
/// Location of the current window pointer bits in the PSW.
pub const CWP_LOC: u16 = 0x7 << 10;
/// Location of the processor status word in the 16 bit uint it is stored in.
pub const PSW_LOC: u16 = 0x1fff;
// Struct definitions.

// TODO maybe convert this into a u16?
/// PSW. Contains internal state that is usually opaque to the system.
/// [12:10] -> Current window pointer (CWP).
/// [9:7] -> Saved window pointer (SWP).
/// [6] Interrupts enabled bit (I).
/// [5] System mode bit (S).
/// [4] Previous system mode bit (P).
/// [3] Zero bit (Z).
/// [2] Negative bit (N).
/// [1] Overflow bit (V).
/// [0] Carry bit (C).
#[derive(Copy, Clone, PartialEq)]
pub struct ProcessorStatusWord(u16);

/// The CPU's register state.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RegisterFile([u32; NUM_GLOBALS + NUM_WINDOW_REGISTERS]);

/// CPU output pins to memory.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct OutputPins {
    /// 32 bit memory output port. For sending data to memory (address).
    pub address: u32,
    /// 32 bit memory output port. For sending data to memory (data).
    pub data: u32,
    /// If the current memory write is a word.
    pub width_code_word: bool,
    /// If the current memory write is a half word.
    pub width_code_half: bool,
    /// If the current memory operation is a write (true) or read (false).
    pub read_write: bool,
    /// If the system is currently in system mode.
    pub system_mode: bool,
    /// If the read/write  data is an instruction (1) or data (0). For writes,
    /// it is always data (0).
    pub instr_or_data_write: bool,
}

// Struct implementations.

impl RegisterFile {
    /// Create a 0'd out register window.
    pub fn new() -> Self {
        Self {
            0: [0u32; NUM_GLOBALS + NUM_WINDOW_REGISTERS],
        }
    }

    // TODO refactor.
    // /// Create a register state from a buffer.
    // /// # Arguments
    // /// * `buf` - A byte buffer that is the size of the sum of of register::RegisterFile's
    // /// members (in bytes) (see `SIZEOF_REG_FILE`).
    // /// The registers should appear in the following order:
    // /// - NXTPC
    // /// - PC
    // /// - LSTPC
    // /// - Global registers
    // /// - Window registers
    // pub fn from_buf(buf: [u8; SIZEOF_REG_FILE]) -> Self {
    //     // Offset used for gloabls and window_regs.
    //     let mut cur_offset = NUM_SPECIAL_REGISTERS * 4;
    //     Self {
    //         nxtpc: u32::from_be_bytes(buf[..4].try_into().unwrap()),
    //         pc: u32::from_be_bytes(buf[4..8].try_into().unwrap()),
    //         lstpc: u32::from_be_bytes(buf[8..cur_offset].try_into().unwrap()),
    //         globals: {
    //             let mut result = [0u32; NUM_GLOBALS];
    //             for i in 0..result.len() {
    //                 result[i] =
    //                     u32::from_be_bytes(buf[cur_offset..cur_offset + 4].try_into().unwrap());
    //                 cur_offset += 4;
    //             }
    //             // Ensure r0 is 0.
    //             result[0] = 0;
    //             result
    //         },
    //         window_regs: {
    //             let mut result = [0u32; NUM_WINDOW_REGISTERS];
    //             for i in 0..result.len() {
    //                 result[i] =
    //                     u32::from_be_bytes(buf[cur_offset..cur_offset + 4].try_into().unwrap());
    //                 cur_offset += 4;
    //             }
    //             result
    //         },
    //     }
    // }

    //// Convert self to a byte buffer of all of the register values.
    // TODO refactor
    // pub fn to_buf(&self) -> [u8; SIZEOF_REG_FILE] {
    //     let mut result = [0u8; SIZEOF_REG_FILE];
    //     // Offset of the special registers to the general purpose registers (bytes).
    //     const SPECIAL_OFFSET: usize = NUM_SPECIAL_REGISTERS * 4;
    //     result[..4].copy_from_slice(&self.nxtpc.to_be_bytes());
    //     result[4..8].copy_from_slice(&self.pc.to_be_bytes());
    //     result[8..SPECIAL_OFFSET].copy_from_slice(&self.lstpc.to_be_bytes());
    //     let globals = {
    //         let mut tmp = [0u8; NUM_GLOBALS * 4];
    //         for i in 0..NUM_GLOBALS {
    //             tmp[i * SPECIAL_OFFSET..i * SPECIAL_OFFSET + 4]
    //                 .copy_from_slice(&self.globals[i].to_be_bytes());
    //         }
    //         tmp
    //     };
    //     const GLOBAL_OFFSET: usize = NUM_SPECIAL_REGISTERS + NUM_GLOBALS * 4;
    //     result[NUM_SPECIAL_REGISTERS..GLOBAL_OFFSET].copy_from_slice(&globals);

    //     let win_regs = {
    //         let mut tmp = [0u8; NUM_WINDOW_REGISTERS * 4];
    //         for i in 0..NUM_WINDOW_REGISTERS {
    //             tmp[i * SPECIAL_OFFSET..i * SPECIAL_OFFSET + 4]
    //                 .copy_from_slice(&self.window_regs[i].to_be_bytes());
    //         }
    //         tmp
    //     };

    //     result[GLOBAL_OFFSET..].copy_from_slice(&win_regs);
    //     result
    // }

    /// Flush entire register window to memory.
    /// # Arguments
    /// * `mem` - Memory to flush to.
    /// * `addr` - Memory address to flush to.
    pub fn flush_to_mem(&self, mem: &mut Memory, addr: u32) {
        let mut address = addr;
        for i in self.0.iter() {
            mem.set_word(address, *i);
            address += 4;
        }
    }

    /// Get a register's value (unsigned). Return the register's value
    /// on success and a string message on error.
    /// Register mapping: [0-9] -> Globals
    ///                   [10-15] -> Outs
    ///                   [16-25] -> Locals
    ///                   [31-26] -> Ins
    /// Anything outside this [0-31] range is an invalid argument.
    /// # Arguments
    /// * `address` - Which register. [0-31] are the only valid values.
    /// * `cwp` - Current window pointer. Used to determine real address of the register.
    pub fn read(&self, address: u8, cwp: u8) -> u32 {
        let addr = address as usize;
        let ptr = cwp as usize;
        match self.get_real_address(address, cwp) {
            Ok(a) => self.0[a],
            Err(_) => 0, // TODO figure out what to do here.
        }
    }

    /// Get a register's real address in the register window. Returns
    ///  Err(()) if address is out of range.
    /// Register mapping: [0-9] -> Globals
    ///                   [10-15] -> Outs
    ///                   [16-25] -> Locals
    ///                   [31-26] -> Ins
    /// Anything outside this [0-31] range is an invalid argument.
    /// # Arguments
    /// * `address` - Which register. [0-31] are the only valid values.
    /// * `cwp` - Current window pointer. Used to determine real address of the register.
    pub fn get_real_address(&self, address: u8, cwp: u8) -> Result<usize, ()> {
        let addr = address as usize;
        let ptr = cwp as usize;
        Ok(match addr {
            0..=9 => addr,
            10..=31 => NUM_ADDED_PER_WINDOW * ptr + addr + NUM_GLOBALS,
            _ => return Err(()),
        })
    }

    /// Set a register's value (unsigned). Return the register's value on
    /// success and a string message on error.
    /// Register mapping: [0-9] -> Globals
    ///                   [10-15] -> Outs
    ///                   [16-25] -> Locals
    ///                   [31-26] -> Ins
    /// Anything outside this [0-31] range is an invalid argument.
    /// Note: writes to register 0 are a no op.
    /// # Arguments
    /// * `address` - Which register. [0-31] are the only valid values.
    /// * `value` - Value to write into the register.
    /// * `cwp` - Current window pointer. Used to determine the real address of the register.
    pub fn write(&mut self, address: u8, value: u32, cwp: u8) {
        let addr = address as usize;
        let ptr = cwp as usize;
        match self.get_real_address(address, cwp) {
            Ok(a) => self.0[a] = value,
            Err(_) => {} // TODO figure out what to do here.
        }
        // Ensure register is 0.
        self.0[0] = 0;
    }
}

impl fmt::Display for RegisterFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl ProcessorStatusWord {
    /// Create a 0'd out PSW.
    pub fn new() -> Self {
        Self { 0: 0 }
    }

    pub fn from_u16(v: u16) -> Self {
        Self { 0: v }
    }

    pub fn init(
        cwp: u8,
        swp: u8,
        interrupts_enabled: bool,
        previous_system_mode: bool,
        system_mode: bool,
        cc_zero: bool,
        cc_neg: bool,
        cc_overflow: bool,
        cc_carry: bool,
    ) -> Self {
        Self {
            0: (((cwp as u16) & 0x7) << 10)
                | (((swp as u16) & 0x7) << 7)
                | ((interrupts_enabled as u16) << 6)
                | ((system_mode as u16) << 5)
                | ((previous_system_mode as u16) << 4)
                | ((cc_zero as u16) << 3)
                | ((cc_neg as u16) << 2)
                | ((cc_overflow as u16) << 1)
                | (cc_carry as u16),
        }
    }

    pub fn get(&self) -> u16 {
        self.0
    }

    /// Push the register window stack. Set CWP to CWP-1 MOD 8. Push the top
    /// window to memory and increment SWP if necessary.
    pub fn push(&mut self) {
        let cwp = self.get_cwp() - 1;
        let swp = self.get_swp();
        if cwp == swp {
            // TODO save windows to memory.
            self.set_swp(swp + 1);
        }
    }

    /// Pop the register window stack. Set CWP to CWP+1 MOD 8. Pull the bottom
    /// window from memory and decrement SWP if necessary.
    pub fn pop(&mut self) {
        let cwp = self.get_cwp() + 1;
        let swp = self.get_swp();
        if cwp == swp {
            // TODO save windows to memory.
            self.set_swp(swp - 1);
        }
    }

    pub fn set_cwp(&mut self, v: u8) {
        self.0 = ((self.0 & !CWP_LOC) | ((v % NUM_REG_WINDOWS as u8) << 10) as u16) & PSW_LOC;
    }

    pub fn set_swp(&mut self, v: u8) {
        self.0 = ((self.0 & !SWP_LOC) | ((v % NUM_REG_WINDOWS as u8) << 7) as u16) & PSW_LOC;
    }

    pub fn set_cc_overflow(&mut self, value: bool) {
        self.0 = (self.0 & !OVERFLOW_LOC) | ((value as u16) << OVERFLOW_LOC);
    }

    pub fn set_cc_carry(&mut self, value: bool) {
        self.0 = (self.0 & !CARRY_LOC) | ((value as u16) << CARRY_LOC);
    }

    pub fn set_cc_zero(&mut self, value: bool) {
        self.0 = (self.0 & !ZERO_LOC) | ((value as u16) << ZERO_LOC);
    }

    pub fn set_cc_neg(&mut self, value: bool) {
        self.0 = (self.0 & !NEG_LOC) | ((value as u16) << NEG_LOC);
    }

    pub fn set_system_mode(&mut self, value: bool) {
        self.0 = (self.0 & !SYSTEM_LOC) | ((value as u16) << SYSTEM_LOC);
    }

    pub fn set_previous_system_mode(&mut self, value: bool) {
        self.0 = (self.0 & !PREV_SYSTEM_LOC) | ((value as u16) << PREV_SYSTEM_LOC);
    }

    pub fn set_interrupt_enabled(&mut self, value: bool) {
        self.0 = (self.0 & !INTERRUPT_LOC) | ((value as u16) << INTERRUPT_LOC);
    }

    pub fn get_cwp(&self) -> u8 {
        ((self.0 & CWP_LOC) >> 10) as u8
    }

    pub fn get_swp(&self) -> u8 {
        ((self.0 & SWP_LOC) as u8) >> 7
    }

    pub fn get_cc_overflow(&self) -> bool {
        (self.0 & OVERFLOW_LOC) != 0
    }

    pub fn get_cc_carry(&self) -> bool {
        (self.0 & CARRY_LOC) != 0
    }

    pub fn get_cc_zero(&self) -> bool {
        (self.0 & ZERO_LOC) != 0
    }

    pub fn get_cc_neg(&self) -> bool {
        (self.0 & NEG_LOC) != 0
    }

    pub fn get_system_mode(&self) -> bool {
        (self.0 & SYSTEM_LOC) != 0
    }

    pub fn get_previous_system_mode(&self) -> bool {
        (self.0 & PREV_SYSTEM_LOC) != 0
    }

    pub fn get_interrupt_enabled(&self) -> bool {
        (self.0 & INTERRUPT_LOC) != 0
    }
}

impl fmt::Display for ProcessorStatusWord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:03x}", self.0)
    }
}

impl fmt::Debug for ProcessorStatusWord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Current window pointer: {}
Saved window pointer: {}
Interrupts Enabled: {}
System mode: {}
Previous system mode: {}
CC Zero: {}
CC Negative: {}
CC Overflow: {}
CC Carry: {}",
            self.get_cwp(),
            self.get_swp(),
            bool_hl_string(self.get_interrupt_enabled()),
            privilege_string(self.get_system_mode()),
            privilege_string(self.get_previous_system_mode()),
            bool_hl_string(self.get_cc_zero()),
            bool_hl_string(self.get_cc_neg()),
            bool_hl_string(self.get_cc_overflow()),
            bool_hl_string(self.get_cc_carry())
        )
    }
}

impl OutputPins {
    pub fn new() -> Self {
        Self {
            address: 0,
            data: 0,
            width_code_half: false,
            width_code_word: false,
            read_write: false,
            system_mode: false,
            instr_or_data_write: false,
        }
    }

    pub fn phase_two_copy(&self, other: &mut Self) {
        let addr = other.address;
        *other = *self;
        other.address = addr;
    }
}

// Private functions.

/// Create a descriptive string for the system's privilege state bits.
/// # Arguments
/// * `s` - Privilege state bit.
fn privilege_string(s: bool) -> &'static str {
    if s {
        "Privileged"
    } else {
        "Unprivileged"
    }
}

/// Stringify booleans with hardware terminology.
/// # Arguments
/// * `s` - Boolean.
fn bool_hl_string(s: bool) -> &'static str {
    if s {
        "High"
    } else {
        "Low"
    }
}

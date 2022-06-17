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
// Struct definitions.

/// A RISC II 32bit register.
type Register = u32;

/// PSW. Contains internal state that is usually opaque to the system.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ProcessorStatusWord {
    /// Current window pointer (MOD 8).
    cwp: u32,
    /// Saved window pointer (MOD 8).
    swp: u32,
    /// If interrupts are enabled.
    interrupt_enable_bit: bool,
    /// System bit, true if running in privileged state.
    system_mode: bool,
    /// The previous state of the `system_mode` bit the last time it was changed.
    previous_system_mode: bool,
    /// Condition codes zero (Z).
    cc_zero: bool,
    /// Condition code negative (N).
    cc_neg: bool,
    /// Condition code overflow (V).
    cc_overflow: bool,
    /// Condition code carry (C).
    cc_carry: bool,
}

/// The CPU's register state.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RegisterFile {
    /// Next program counter, holds the address of the instruction being
    /// fetched for the next cycle.
    nxtpc: Register,
    /// Program counter, holds the address of current instruction being
    /// executed.
    pc: Register,
    /// The lastpc, holds the address of the last executed instruction
    /// (or last attempted to be executed). When running an interrupt `lstpc`
    /// holds the address of the instruction that was aborted.
    lstpc: Register,
    /// Global registers.
    globals: [Register; NUM_GLOBALS],
    /// Register window stack.
    window_regs: [Register; NUM_WINDOW_REGISTERS], // TODO test, there should be 138 regs.
}

// Struct implementations.

impl RegisterFile {
    /// Create a 0'd out register window.
    pub fn new() -> Self {
        Self {
            nxtpc: 0,
            pc: 0,
            lstpc: 0,
            globals: [0; NUM_GLOBALS],
            window_regs: [0; NUM_WINDOW_REGISTERS],
        }
    }

    /// Create a register state from a buffer.
    /// # Arguments
    /// * `buf` - A byte buffer that is the size of the sum of of register::RegisterFile's
    /// members (in bytes) (see `SIZEOF_REG_FILE`).
    /// The registers should appear in the following order:
    /// - NXTPC
    /// - PC
    /// - LSTPC
    /// - Global registers
    /// - Window registers
    pub fn from_buf(buf: [u8; SIZEOF_REG_FILE]) -> Self {
        // Offset used for gloabls and window_regs.
        let mut cur_offset = NUM_SPECIAL_REGISTERS * 4;
        Self {
            nxtpc: u32::from_be_bytes(buf[..4].try_into().unwrap()),
            pc: u32::from_be_bytes(buf[4..8].try_into().unwrap()),
            lstpc: u32::from_be_bytes(buf[8..cur_offset].try_into().unwrap()),
            globals: {
                let mut result = [0u32; NUM_GLOBALS];
                for i in 0..result.len() {
                    result[i] =
                        u32::from_be_bytes(buf[cur_offset..cur_offset + 4].try_into().unwrap());
                    cur_offset += 4;
                }
                // Ensure r0 is 0.
                result[0] = 0;
                result
            },
            window_regs: {
                let mut result = [0u32; NUM_WINDOW_REGISTERS];
                for i in 0..result.len() {
                    result[i] =
                        u32::from_be_bytes(buf[cur_offset..cur_offset + 4].try_into().unwrap());
                    cur_offset += 4;
                }
                result
            },
        }
    }

    /// Convert self to a byte buffer of all of the register values.
    pub fn to_buf(&self) -> [u8; SIZEOF_REG_FILE] {
        let mut result = [0u8; SIZEOF_REG_FILE];
        // Offset of the special registers to the general purpose registers (bytes).
        const SPECIAL_OFFSET: usize = NUM_SPECIAL_REGISTERS * 4;
        result[..4].copy_from_slice(&self.nxtpc.to_be_bytes());
        result[4..8].copy_from_slice(&self.pc.to_be_bytes());
        result[8..SPECIAL_OFFSET].copy_from_slice(&self.lstpc.to_be_bytes());
        let globals = {
            let mut tmp = [0u8; NUM_GLOBALS * 4];
            for i in 0..NUM_GLOBALS {
                tmp[i * SPECIAL_OFFSET..i * SPECIAL_OFFSET + 4]
                    .copy_from_slice(&self.globals[i].to_be_bytes());
            }
            tmp
        };
        const GLOBAL_OFFSET: usize = NUM_SPECIAL_REGISTERS + NUM_GLOBALS * 4;
        result[NUM_SPECIAL_REGISTERS..GLOBAL_OFFSET].copy_from_slice(&globals);

        let win_regs = {
            let mut tmp = [0u8; NUM_WINDOW_REGISTERS * 4];
            for i in 0..NUM_WINDOW_REGISTERS {
                tmp[i * SPECIAL_OFFSET..i * SPECIAL_OFFSET + 4]
                    .copy_from_slice(&self.window_regs[i].to_be_bytes());
            }
            tmp
        };

        result[GLOBAL_OFFSET..].copy_from_slice(&win_regs);
        result
    }

    /// Flush entire register window to memory.
    /// # Arguments
    /// * `mem` - Memory to flush to.
    /// * `addr` - Memory address to flush to.
    pub fn flush_to_mem(&self, mem: &mut Memory, addr: u32) {
        mem.write_buf(addr, &self.to_buf());
    }

    /// Get a register's value (unsigned). Return the register's value
    /// on success and a string message on error.
    /// Register mapping: [0-9] -> Globals
    ///                   [10-15] -> Outs
    ///                   [16-25] -> Locals
    ///                   [31-26] -> Ins
    /// Anything outside this [0-31] range is an invalid argument.
    /// # Arguments
    /// * `which` - Which register. [0-31] are the only valid values.
    pub fn ru(&self, which: u32, psw: &ProcessorStatusWord) -> Result<u32, String> {
        Ok(match which {
            0..=9 => self.globals[which],
            10..=31 => self.window_regs[NUM_ADDED_PER_WINDOW * psw.get_cwp() + rd],
            _ => return Err(format!("Register {} is out of range", which)),
        })
    }

    /// Get a register's value (signed). Return the register's value
    /// on success and a string message on error.
    /// Register mapping: [0-9] -> Globals
    ///                   [10-15] -> Outs
    ///                   [16-25] -> Locals
    ///                   [31-26] -> Ins
    /// Anything outside this [0-31] range is an invalid argument.
    /// # Arguments
    /// * `which` - Which register. [0-31] are the only valid values.
    pub fn rs(&self, which: u32, psw: &ProcessorStatusWord) -> Result<i32, String> {
        Ok(self.ru(which, psw)? as i32)
    }

    /// Set a register's value (unsigned). Return the register's value on
    /// success and a string message on error.
    /// Register mapping: [0-9] -> Globals
    ///                   [10-15] -> Outs
    ///                   [16-25] -> Locals
    ///                   [31-26] -> Ins
    /// Anything outside this [0-31] range is an invalid argument.
    /// # Arguments
    /// * `which` - Which register. [0-31] are the only valid values.
    pub fn rus(&mut self, which: u32, value: u32, psw: ProcessorStatusWord) -> Result<u32, String> {
        match which {
            0..=9 => self.globals[which] = value,
            10..=31 => self.window_regs[NUM_ADDED_PER_WINDOW * psw.get_cwp() + rd] = value,
            _ => return Err(format!("Register {} is out of range", which)),
        }
        Ok(value)
    }

    pub fn get_last_pc(&self) -> u32 {
        self.lstpc
    }

    pub fn get_pc(&self) -> u32 {
        self.pc
    }

    pub fn get_next_pc(&self) -> u32 {
        self.nxtpc
    }

    pub fn inc_pcs(&mut self) {
        self.lstpc = self.pc;
        self.pc = self.nxtpc;
        self.nxtpc += SIZEOF_INSTRUCTION;
    }

    pub fn branch_to(&mut self, to: u32) {
        self.lstpc = self.pc;
        self.pc = nxtpc;
        self.nxtpc = to;
    }

    pub fn get_ss_val(&self, ss: ShortSource) -> Result<u32, String> {
        type SS = ShortSource;
        Ok(match ss {
            SS::Reg(r) => self.ru(r as u32)?,
            SS::Imm13(u) => u,
        })
    }
}

impl fmt::Display for RegisterFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Next PC: 0x{:x}
PC: 0x{:x}
Last PC: 0x{:x}
Globals: {:?}
Window: {:?}",
            self.nxtpc, self.pc, self.lstpc, self.globals, self.window_regs
        )
    }
}

impl ProcessorStatusWord {
    /// Create a 0'd out PSW.
    pub fn new() -> Self {
        Self {
            cwp: 0,
            swp: 0,
            interrupt_enable_bit: false,
            system_mode: false,
            previous_system_mode: false,
            cc_zero: false,
            cc_neg: false,
            cc_overflow: false,
            cc_carry: false,
        }
    }

    pub fn from_u32(v: u32) -> Self {
        Self {
            cwp: (v & (0x7 << 7)) >> 7,
            swp: (v & (0x7 << 10)) >> 10,
            interrupt_enabled_bit: v & (0x1 << 6) != 0,
            previous_system_bit: v & (0x1 << 5) != 0,
            system_bit: v & (0x1 << 4) != 0,
            cc_zero: v & (0x1 << 3) != 0,
            cc_neg: v & (0x1 << 2) != 0,
            cc_overflow: v & (0x1 << 1) != 0,
            cc_carry: v & 0x1 != 0,
        }
    }

    pub fn init(
        cwp: u32,
        swp: u32,
        interrupt_enable_bit: bool,
        previous_system_bit: bool,
        system_bit: bool,
        cc_zero: bool,
        cc_neg: bool,
        cc_overflow: bool,
        cc_carry: bool,
    ) -> Self {
        Self {
            cwp: cwp % 8,
            swp: swp % 8,
            interrupt_enable_bit: interrupt_enable_bit,
            previous_system_bit: previous_system_bit,
            system_bit: system_bit,
            cc_zero: cc_zero,
            cc_neg: cc_neg,
            cc_overflow: cc_overflow,
            cc_carry: cc_carry,
        }
    }

    /// Get the 13 bit PSW value. PSW is the state of the system's special
    /// registers and CC's. After the 13th bit PSW is 0 padded.
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
    pub fn to_u32(&self) -> u32 {
        (self.cc_carry as u32
            | (self.cc_overflow as u32) << 1
            | (self.cc_neg as u32) << 2
            | (self.cc_zero as u32) << 3
            | (self.previous_system_mode as u32) << 4
            | (self.system_mode as u32) << 5
            | (self.interrupt_enable_bit as u32) << 6
            | (self.cwp) << 7
            | (self.swp) << 10)
            & 0x1fff
    }

    /// Push the register window stack. Set CWP to CWP-1 MOD 8. Push the top
    /// window to memory and increment SWP if necessary.
    pub fn push_reg_window(&mut self) {
        self.cwp = (self.cwp - 1) % NUM_REG_WINDOWS;
        if self.cwp == self.swp {
            // TODO save windows to memory.
            self.swp = (self.swp + 1) % NUM_REG_WINDOWS;
        }
    }

    /// Pop the register window stack. Set CWP to CWP+1 MOD 8. Pull the bottom
    /// window from memory and decrement SWP if necessary.
    pub fn pop_reg_window(&mut self) {
        self.cwp = (self.cwp + 1) % NUM_REG_WINDOWS;
        if self.cwp == self.swp {
            // TODO restore windows from memory.
            self.swp = (self.swp - 1) % NUM_REG_WINDOWS;
        }
    }

    pub fn set_cwp(&mut self, v: u32) {
        self.cwp = v % NUM_REG_WINDOWS;
    }

    pub fn set_swp(&mut self, v: u32) {
        self.swp = v % NUM_REG_WINDOWS;
    }

    pub fn get_cwp(&self) -> u32 {
        self.cwp
    }

    pub fn get_swp(&self) -> u32 {
        self.swp
    }

    pub fn get_cc_overflow(&self) -> bool {
        self.cc_overflow
    }

    pub fn get_cc_carry(&self) -> bool {
        self.cc_carry
    }

    pub fn get_cc_zero(&self) -> bool {
        self.cc_zero
    }

    pub fn get_cc_neg(&self) -> bool {
        self.cc_neg
    }

    pub fn set_cc_overflow(&mut self, value: bool) {
        self.cc_overflow = value;
    }

    pub fn set_cc_carry(&mut self, value: bool) {
        self.cc_carry = value;
    }

    pub fn set_cc_zero(&mut self, value: bool) {
        self.cc_zero = value;
    }

    pub fn set_cc_neg(&mut self, value: bool) {
        self.cc_neg = value;
    }

    pub fn set_system_bit(&mut self, v: bool) {
        self.system_mode = v;
    }

    pub fn set_previous_system_bit(&mut self, v: bool) {
        self.previous_system_mode = v;
    }

    pub fn set_interrupt_enabled(&mut self, v: bool) {
        self.interrupt_enable_bit = v;
    }

    pub fn is_system_mode(&self) -> bool {
        self.system_mode
    }

    pub fn is_previous_system_mode(&self) -> bool {
        self.previous_system_mode
    }

    pub fn is_interrupt_enabled(&self) -> bool {
        self.interrupt_enable_bit
    }
}

impl fmt::Display for ProcessorStatusWord {
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
            self.cwp,
            self.swp,
            privilege_string(self.system_mode),
            privilege_string(self.previous_system_mode),
            bool_hl_string(self.cc_zero),
            bool_hl_string(self.cc_neg),
            bool_hl_string(self.cc_overflow),
            bool_hl_string(self.cc_carry)
        )
    }
}

// Private functions.

/// Create a descriptive string for the system's privilege state bits.
/// # Arguments
/// * `s` - Privilege state bit.
fn privilege_string(s: bool) -> &str {
    if s {
        "Privileged"
    } else {
        "Unprivileged"
    }
}

/// Stringify booleans with hardware terminology.
/// # Arguments
/// * `s` - Boolean.
fn bool_hl_string(s: bool) -> &str {
    if s {
        "High"
    } else {
        "Low"
    }
}

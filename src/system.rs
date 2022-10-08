// RISC II emulated machine state.
// See `decode.rs` for the first step, and `commit.rs` for the third step.
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

use config::Config;
use cpu::{ProcessorStatusWord, RegisterFile};
use instruction::noop;
use memory::Memory;
use std::fmt;
use util::Result;

/// RISC II emulated system.
#[derive(Debug, Clone)]
pub struct System {
    /// RISC II register file.
    regs: RegisterFile,
    /// Processor status.
    psw: ProcessorStatusWord,
    /// Memory state.
    mem: Memory,
    /// Temporary latch for destination register.
    tmp_latch: u32,
    /// Next instruction.
    next_instruction: u32,
}

// Impls.

impl System {
    /// Create a new emulated RISC II system. Return system on success and
    /// a string on error.
    /// # Arguments
    /// * `config` - Emulator configuration.
    pub fn new(config: &Config) -> Result<Self> {
        Ok(Self {
            regs: RegisterFile::new(),
            psw: ProcessorStatusWord::new(),
            mem: Memory::new(config),
            tmp_latch: 0,
            next_instruction: noop(),
        })
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
    pub fn get_psw_as_u32(&self) -> u32 {
        self.psw.to_u32()
    }

    pub fn call(&mut self, addr: u32) {
        self.psw.push_reg_window();
    }

    pub fn ret(&mut self) {
        self.psw.pop_reg_window();
    }

    pub fn get_register_file(&mut self) -> &mut RegisterFile {
        &mut self.regs
    }

    pub fn copy_register_file(&self) -> RegisterFile {
        self.regs
    }

    pub fn get_last_pc(&self) -> u32 {
        self.regs.get_last_pc()
    }

    pub fn get_pc(&self) -> u32 {
        self.regs.get_pc()
    }

    pub fn get_next_pc(&self) -> u32 {
        self.regs.get_next_pc()
    }

    pub fn integrate_system_changes(&mut self, other: &System) {
        self.regs = other.regs;
        self.psw = other.psw;
    }

    pub fn get_psw(&self) -> ProcessorStatusWord {
        self.psw
    }

    pub fn set_psw(&mut self, psw: u32) {
        self.psw = ProcessorStatusWord::from_u32(psw);
    }

    pub fn copy_no_mem(&self) -> Self {
        System {
            regs: self.regs,
            psw: self.psw,
            mem: Memory::from_size(0),
            next_instruction: self.next_instruction,
            tmp_latch: self.tmp_latch,
        }
    }

    pub fn get_mem_ref(&mut self) -> &mut Memory {
        &mut self.mem
    }

    /// Run for a single clock cycle.
    pub fn step(&mut self) {
        self.fetch();
        self.execute();
        self.commit();
    }

    fn fetch(&mut self) -> Result<()> {
        self.next_instruction = self.mem.get_word(self.regs.nxtpc)?;
        Ok(())
    }

    fn execute(&mut self) {}

    fn commit(&mut self) {}
}

impl fmt::Display for System {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "CPU register state:\n{}
Processor Status Word:\n{}",
            self.regs, self.psw,
        )
    }
}

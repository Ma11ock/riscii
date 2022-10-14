// RISC II emulated data path.
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
use cpu::{OutputPins, ProcessorStatusWord, RegisterFile, SIZEOF_INSTRUCTION};
use instruction::noop;
use memory::Memory;
use std::fmt;
use util::Result;

use crate::cpu;

/// RISC II emulated data path.
#[derive(Debug, Clone)]
pub struct DataPath {
    /// RISC II register file.
    regs: RegisterFile,
    /// Processor status.
    psw: ProcessorStatusWord,
    /// Temporary latch for destination register.
    dst_latch: u32,
    /// Source latch for the shifter and ALU.
    src_latch: u32,
    /// Next program counter, holds the address of the instruction being
    /// fetched for the next cycle.
    nxtpc: u32,
    /// Program counter, holds the address of current instruction being
    /// executed.
    pc: u32,
    /// The lastpc, holds the address of the last executed instruction
    /// (or last attempted to be executed). When running an interrupt `lstpc`
    /// holds the address of the instruction that was aborted.
    lstpc: u32,
    /// 32 bit memory input pin. For receiving from main memory.
    pins_in: u32,
    /// Pins for communicating with the outside world (memory).
    output_pins: OutputPins,

    // Control unit latches and registers.
    /// Data from memory.
    dimm: u32,
    /// Immediate register (for instruction being decoded).
    imm1: u32,
    /// Immediate register (for currently executing instruction).
    imm2: u32,
    /// Byte address register, bottom two bits of memory address being accesses.
    bar: u8,
    /// Destination register address (for instruction being decoded).
    rd1: u8,
    /// Destination register address (for currently executing instruction).
    rd2: u8,
    /// Destination register address (for commiting/previous instruction).
    rd3: u8,
    /// Source register one.
    ra: u8,
    /// Source register two.
    rb: u8,
    /// Opcode register (for instruction being decoded).
    op1: u8,
    /// Opcode register (for currently executing instruction).
    op2: u8,
    /// SCC flag of the instruction (for instruction being decoded).
    scc_flag1: bool,
    /// SCC flag of the instruction (for currently executing instruction).
    scc_flag2: bool,
    /// SCC flag of the instruction (for commiting/previous instruction).
    scc_flag3: bool,
    /// Immediate flag of the instruction (for instruction being decoded).
    imm_flag1: bool,
    /// Immediate flag of the instruction (for currently executing instruction).
    imm_flag2: bool,
}

// Impls.

impl DataPath {
    /// Create a new emulated RISC II system. Return system on success and
    /// a string on error.
    /// # Arguments
    /// * `config` - Emulator configuration.
    pub fn new(config: &Config) -> Result<Self> {
        Ok(Self {
            regs: RegisterFile::new(),
            psw: ProcessorStatusWord::new(),
            src_latch: 0,
            dst_latch: 0,
            bar: 0,
            rd1: 0,
            rd2: 0,
            rd3: 0,
            ra: 0,
            rb: 0,
            op1: 0,
            op2: 0,
            dimm: 0,
            imm1: 0,
            imm2: 0,
            nxtpc: 0,
            pc: 0,
            lstpc: 0,
            pins_in: 0,
            output_pins: OutputPins::new(),
            scc_flag1: false,
            scc_flag2: false,
            scc_flag3: false,
            imm_flag1: false,
            imm_flag2: false,
        })
    }

    pub fn commit(&mut self) {
        let dest_value = self.dst_latch;
        let dest_reg = self.rd3;
        let cwp = self.psw.get_cwp();
        self.regs.write(dest_reg, dest_value, cwp);
    }

    /// Decode the next instruction's (in `self.pins_in`) source registers.
    pub fn decode_input_regs(&mut self) {
        let next_instruction = self.pins_in;
    }

    pub fn set_input_pins(&mut self, value: u32) {
        self.pins_in = value;
    }

    pub fn get_out_address(&self) -> u32 {
        self.output_pins.address
    }

    pub fn get_out_data(&self) -> u32 {
        self.output_pins.data
    }

    pub fn get_output_pins_ref(&self) -> &OutputPins {
        &self.output_pins
    }

    fn increment_pcs(&mut self) {
        self.lstpc = self.pc;
        self.pc = self.nxtpc;
        self.nxtpc += cpu::SIZEOF_INSTRUCTION;
    }

    fn branch_to(&mut self, address: u32) {
        self.lstpc = self.pc;
        self.pc = self.nxtpc;
        self.nxtpc = address;
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
        self.psw.get() as u32
    }

    pub fn call(&mut self, addr: u32) {
        self.psw.push();
    }

    pub fn ret(&mut self) {
        self.psw.pop();
    }

    pub fn get_register_file(&mut self) -> &mut RegisterFile {
        &mut self.regs
    }

    pub fn copy_register_file(&self) -> RegisterFile {
        self.regs
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

    pub fn get_psw(&self) -> ProcessorStatusWord {
        self.psw
    }

    pub fn set_psw(&mut self, psw: u16) {
        self.psw = ProcessorStatusWord::from_u16(psw);
    }
}

// Clock notes:
// Reads happen in phase 1
// Register decoding happens in phase 2
// Immediates are routed thru shifter in phase 2
// Loads can use the shifter in phase 4 for aligning data.

// f1: register read and int. forwarding.
// f2: routes sources and imm thru shifter, Reg dec,
// f3: register write, ALU
// f4: register dec, shift alignment (for ld)

impl fmt::Display for DataPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "CPU register state:\n{}
Processor Status Word:\n{}",
            self.regs, self.psw,
        )
    }
}

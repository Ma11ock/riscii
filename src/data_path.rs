// RISC II emulated data path.
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

use alu::ALU;
use config::Config;
use cpu::{OutputPins, ProcessorStatusWord, RegisterFile, SIZEOF_INSTRUCTION};
use instruction::*;
use memory::Memory;
use shifter;
use std::fmt;
use util::Result;

use crate::shifter::Shifter;

pub type MicroOp = fn(dp: &mut DataPath);

#[derive(Debug, Clone)]
pub struct Control {
    pub long: bool,
    pub immediate: bool,
    pub memory: bool,
    pub store: bool,
    pub pc_relative: bool,
    pub signed_load: bool,
    pub conditional: bool,
    pub dest_is_psw: bool,
}

/// RISC II emulated data path.
#[derive(Debug, Clone)]
pub struct DataPath {
    /// RISC II register file.
    regs: RegisterFile,
    /// Processor status.
    psw: ProcessorStatusWord,
    /// Temporary latch for destination register.
    dst_latch: u32,
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
    /// Arithmetic logic unit.
    alu: ALU,
    /// Shifter unit.
    shifter: Shifter,

    // Control unit latches and registers.
    /// Data from memory.
    dimm: u32,
    /// Immediate register (for instruction being decoded).
    imm: u32,
    /// Byte address register, bottom two bits of memory address being accesses.
    bar: u8,
    /// Destination register address (for instruction being decoded).
    rd1: u8,
    /// Destination register address (for currently executing instruction).
    rd2: u8,
    /// Destination register address (for commiting/previous instruction).
    rd3: u8,
    /// Source register one (for instruction being decoded).
    rs1_1: u8,
    /// Source register two (for instruction being decoded).
    rs2_1: u8,
    /// Source register one (for currently executing instruction).
    rs1_2: u8,
    /// Source register two (for currently executing instruction).
    rs2_2: u8,
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

    /// Control bits.
    control1: Control,
    control2: Control,
    control3: Control,
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
            shifter: Shifter::new(),
            dst_latch: 0,
            alu: ALU::new(),
            bar: 0,
            rd1: 0,
            rd2: 0,
            rd3: 0,
            rs1_1: 0,
            rs2_1: 0,
            rs1_2: 0,
            rs2_2: 0,
            op1: 0,
            op2: 0,
            dimm: 0,
            imm: 0,
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
            control1: Control::new(),
            control2: Control::new(),
            control3: Control::new(),
        })
    }

    pub fn commit(&mut self) {
        let dest_value = self.dst_latch;
        let dest_reg = self.rd3;
        let cwp = self.psw.get_cwp();
        self.regs.write(dest_reg, dest_value, cwp);
    }

    pub fn route_regs_to_alu(&mut self) {
        if self.control1.pc_relative {
            self.alu.ai = self.pc;
        } else {
            // TODO investigate interrupts. Should src2 be set no matter what?
            let src1 = self.rs1_2;
            let src2 = self.rs1_2;
            let cwp = self.psw.get_cwp();
            let read1 = self.regs.read(src1, cwp);
            let read2 = self.regs.read(src2, cwp);
            self.alu.ai = read1;
            self.alu.bi = read2;
        }
    }

    /// Decode the next instruction's (in `self.pins_in`) source registers.
    pub fn decode_input_regs(&mut self) {
        let next_instruction = self.pins_in;
    }

    pub fn set_input_pins(&mut self, value: u32) {
        self.pins_in = value;
        // Set other latches hooked up to memory data path.
        self.op1 = ((value & 0xFE000000) >> 25) as u8;
        self.imm_flag1 = value & SHORT_SOURCE_TYPE_LOC != 0;
        self.scc_flag1 = value & SCC_LOC != 0;
        self.rd1 = ((value & DEST_LOC) >> 19) as u8;
        self.rs1_1 = ((value & RS1_LOC) >> 14) as u8;
        self.rs2_1 = (value & RS2_LOC) as u8;
        self.imm = value & IMM19_LOC;
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

    pub fn route_imm_to_alu(&mut self) {
        if self.control1.immediate {
            self.alu.bi = self.dimm;
        }
    }

    pub fn shift_pipeline_latches(&mut self) {
        // Move the control bits.
        self.control3 = self.control2.clone();
        self.control2 = self.control1.clone();
        // Move the destination register.
        self.rd3 = self.rd2;
        self.rd2 = self.rd1;
        // Move the source registers.
        self.rs1_2 = self.rs1_1;
        self.rs2_2 = self.rs2_1;
        // Move the scc flags.
        self.scc_flag3 = self.scc_flag2;
        self.scc_flag2 = self.scc_flag1;
        // Move Imm flag.
        self.imm_flag2 = self.imm_flag1;
        // Move the opcode.
        self.op2 = self.op1;
    }

    fn increment_pcs(&mut self) {
        self.lstpc = self.pc;
        self.pc = self.nxtpc;
        self.nxtpc += SIZEOF_INSTRUCTION;
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

    pub fn decode(&mut self) {
        self.control1 = decode_opcode(self.pins_in);
    }
}

impl Control {
    pub fn new() -> Self {
        Self {
            long: false,
            immediate: false,
            memory: false,
            store: false,
            pc_relative: false,
            signed_load: false,
            conditional: false,
            dest_is_psw: false,
        }
    }

    pub fn init(
        long: bool,
        immediate: bool,
        memory: bool,
        store: bool,
        pc_relative: bool,
        signed_load: bool,
        conditional: bool,
        dest_is_psw: bool,
    ) -> Self {
        Self {
            long: long,
            immediate: immediate,
            memory: memory,
            store: store,
            pc_relative: pc_relative,
            signed_load: signed_load,
            conditional: conditional,
            dest_is_psw: dest_is_psw,
        }
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

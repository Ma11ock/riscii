// RISC II emulated PC.
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

use clock::{Clock, Phase};
use config::Config;
use cpu::OutputPins;
use data_path::{Control, DataPath};
use instruction::{noop, InstructionCycle};
use memory::Memory;
use util::Result;

pub struct System {
    /// RISCII data path.
    data_path: DataPath,
    /// Memory state.
    mem: Memory,
    /// External, four phase clock.
    clock: Clock,
    /// Next micro operation to perform for the currently executing instruction.
    cycle_ops: InstructionCycle,
    /// Current CPU non-overlapping clock phase.
    phase: Phase,
    // TODO move below to an MMU emulator.
    /// CPU's output pins, input pins for memory.
    pins_out: OutputPins,
    /// True if the pipeline is currently suspended as a result of a memory operation.
    pipeline_suspended: bool,
}

impl System {
    pub fn new(config: &Config) -> Result<Self> {
        let dp = DataPath::new();
        Ok(Self {
            data_path: dp,
            mem: Memory::new(config),
            clock: Clock::new(config),
            cycle_ops: InstructionCycle::noop_cycle(),
            phase: Phase::One,
            pins_out: OutputPins::new(),
            pipeline_suspended: false,
        })
    }

    pub fn get_mem_ref(&mut self) -> &mut Memory {
        &mut self.mem
    }

    pub fn tick(&mut self) {
        let cur_phase = self.phase.clone();
        self.clock.tick_and_wait(cur_phase);

        // Fetch
        // Execute.
        // Commit.

        let dp = &mut self.data_path;
        self.phase = match self.phase {
            Phase::One => {
                if !self.pipeline_suspended {
                    // Tell the pipeline we're moving on to the next instruction.
                    dp.shift_pipeline_latches();
                    // Registers are read and then sent to the input latches of the ALU.
                    dp.route_regs_to_alu();
                    // TODO determine when this callback should be run.
                    self.cycle_ops[0](dp);
                }
                Phase::Two
            }
            Phase::Two => {
                // Memory copies output pin data for writing (if any writing is to be done).
                dp.get_output_pins_ref().phase_two_copy(&mut self.pins_out);

                if !self.pipeline_suspended {
                    // Route immediate to ALU.
                    dp.route_imm_to_alu();
                    // TODO determine when this callback should be run.
                    self.cycle_ops[1](dp);
                }

                // Route sources and immediate thru shifter.
                Phase::Three
            }
            Phase::Three => {
                // Finish read from last cycle.
                let mem = &self.mem;
                // TODO check for invalid address from MMU.
                dp.set_input_pins(match mem.get_word(self.pins_out.address) {
                    Ok(v) => v,
                    Err(_) => {
                        eprint!("Bad mem read: {}", self.pins_out.address);
                        0
                    }
                });

                if self.pipeline_suspended {
                    self.pipeline_suspended = false;
                } else if dp.current_instruction_is_memory() {
                    // Commit the result of the last instruction.
                    dp.commit();
                    self.pipeline_suspended = true;
                } else {
                    // Commit the result of the last instruction.
                    dp.commit();
                    // TODO determine when this callback should be run.
                    self.cycle_ops[2](dp);
                }
                Phase::Four
            }
            Phase::Four => {
                // In actual RISCII this is where the source and dest registers are decoded
                // for the next instruction, but that is unnecessary here.
                self.pins_out.address = dp.get_out_address();

                if !self.pipeline_suspended {
                    // TODO determine when this callback should be run.
                    self.cycle_ops[3](dp);
                    dp.decode();
                }
                // If the instruction was a load, shift the result if necessary.
                Phase::One
            }
            Phase::Interrupt => Phase::One,
        };
    }

    pub fn clock(&self) -> &Clock {
        &self.clock
    }

    pub fn data_path(&self) -> &DataPath {
        &self.data_path
    }

    pub fn phase(&self) -> Phase {
        self.phase.clone()
    }
}

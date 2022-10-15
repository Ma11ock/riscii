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

use clock::Clock;
use config::Config;
use cpu::OutputPins;
use data_path::DataPath;
use instruction::{noop, MicroOperation};
use memory::Memory;
use util::Result;

use crate::clock::Phase;

pub struct System {
    /// RISCII data path.
    data_path: DataPath,
    /// Memory state.
    mem: Memory,
    /// External, four phase clock.
    clock: Clock,
    /// Next micro operation to perform for the currently executing instruction.
    op: MicroOperation,
    /// Current CPU non-overlapping clock phase.
    phase: Phase,
    // TODO move below to an MMU emulator.
    /// CPU's output pins, input pins for memory.
    pins_out: OutputPins,
}

impl System {
    pub fn new(config: &Config) -> Result<Self> {
        let mut dp = DataPath::new(config)?;
        let nop = noop(&mut dp);
        Ok(Self {
            data_path: dp,
            mem: Memory::new(config),
            clock: Clock::new(config),
            op: nop,
            phase: Phase::One,
            pins_out: OutputPins::new(),
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
                // Registers are read and then send to the input latches of the ALU.
                dp.route_regs_to_alu();
                Phase::Two
            }
            Phase::Two => {
                // Memory copies output pin data for writing (if any writing is to be done).
                dp.get_output_pins_ref().phase_two_copy(&mut self.pins_out);

                // Route sources and immediate thru shifter.
                Phase::Three
            }
            Phase::Three => {
                // Finish read from last cycle.
                let mem = &self.mem;
                let addr = self.pins_out.address;
                // TODO check for invalid address from MMU.
                dp.set_input_pins(match mem.get_word(addr) {
                    Ok(v) => v,
                    Err(_) => {
                        eprint!("Bad mem read: {}", addr);
                        0
                    }
                });
                self.data_path.commit();
                Phase::Four
            }
            Phase::Four => {
                // In actual RISCII this is where the source and dest registers are decoded
                // for the next instruction, but that is unnecessary here.
                self.pins_out.address = dp.get_out_address();
                Phase::One
            }
            Phase::Interrupt => Phase::One,
        };
    }
}

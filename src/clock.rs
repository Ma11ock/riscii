// Clock emulator for RISCII.
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

/// Phases for RISCII's multi (4) phase clock non-overlapping clock.
pub enum Phase {
    /// Phase one of the RISCII's clock. During this phase the register file
    /// is read and forwarded to the shifter and ALU.
    One,
    /// Phase two of the RISCII's clock. During this phase the immediate value
    /// is routed through the shifter. The destination register for the previous
    /// instruction is decoded.
    Two,
    /// Phase three of the RISCII's clock. During this phase the ALU computes
    /// its result value and the previous instruction's result is written to
    /// the destination register.
    Three,
    /// Phase four of the RISCII's clock. During this phase the source and destination
    /// registers are decoded. Load instructions use the shifter to align data.
    Four,
    /// Special interrupt phase TODO.
    Interrupt,
}

#[derive(PartialEq, Eq, Clone)]
pub struct Clock {
    rate: u64,
    count: u64,
    phase: Phase,
}

impl Clock {
    pub fn tick(clock: &mut Self) {
        // TODO cycle accurate clock.
    }

    pub fn new(config: &Config) -> Self {
        Self {
            rate: config.clock_rate,
            count: 0,
            phase: Phase::One,
        }
    }
}

impl fmt::Debug for Clock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Clock: {}", self)
    }
}

impl fmt::Display for Clock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}, {}", self.rate, self.count)
    }
}

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
use std::fmt;
use std::time::{Duration, Instant};

/// Phases for RISCII's multi (4) phase clock non-overlapping clock.
#[derive(PartialEq, Eq, Clone)]
pub enum Phase {
    /// Phase one of the RISCII's clock. During this phase the register file
    /// is read and forwarded to the shifter and ALOE.
    One = 1,
    /// Phase two of the RISCII's clock. During this phase the immediate value
    /// is routed through the shifter. The destination register for the previous
    /// instruction is decoded.
    Two = 2,
    /// Phase three of the RISCII's clock. During this phase the ALU computes
    /// its result value and the previous instruction's result is written to
    /// the destination register.
    Three = 3,
    /// Phase four of the RISCII's clock. During this phase the source and destination
    /// registers are decoded. Load instructions use the shifter to align data.
    Four = 4,
    /// Special interrupt phase TODO.
    Interrupt = 5,
}

#[derive(PartialEq, Eq, Clone)]
pub struct Clock {
    rate: u64,
    count: u64,
    last_time: Instant,
    seconds_coutner: Duration,
}

impl Clock {
    pub fn tick(&mut self, phase: Phase) {
        match phase {
            Phase::One => {
                self.count += 1;
            }
            _ => {}
        }
    }

    pub fn tick_and_wait(&mut self, phase: Phase) {
        match phase {
            Phase::One => {
                self.count += 1;
                if self.count == self.rate {
                    self.idle_clock();
                }
            }
            _ => {}
        }
    }

    pub fn new(config: &Config) -> Self {
        Self {
            rate: config.get_clock_rate(),
            count: 0,
            last_time: Instant::now(),
            seconds_coutner: Duration::new(0, 0),
        }
    }

    fn idle_clock(&mut self) {
        // Calc curTime - lastTime (in nanoseconds). If less than a second has
        // passed, sleep until we've reached that next second.
        const ONE_SECOND: Duration = Duration::from_secs(1);
        let now = Instant::now();
        let time_passed = now - self.last_time;
        if time_passed < ONE_SECOND {
            std::thread::sleep(time_passed);
            self.last_time = now + time_passed;
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

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
use cpu::RegisterFile;
use memory::Memory;
use std::fmt;

pub struct System {
    /// RISC II register file.
    regs: RegisterFile,
    /// Memory state.
    mem: Memory,
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

// Impls.

impl System {
    pub fn new(config: &Config) -> Result<Self, String> {
        Ok(Self {
            regs: RegisterFile::new(),
            mem: Memory::new(config),
            system_mode: true,
            previous_system_mode: false,
            cc_zero: false,
            cc_neg: false,
            cc_overflow: false,
            cc_carry: false,
        })
    }
}

impl fmt::Display for System {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "CPU register state: \n{}
Privilege level: {}
Previous privilege level: {}
CC Zero: {}
CC Neg: {}
CC Overflow: {}
CC Carry: {}",
            self.regs,
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

fn privilege_string(s: bool) -> String {
    if s {
        "Privileged".to_string()
    } else {
        "Unprivileged".to_string()
    }
}

fn bool_hl_string(s: bool) -> String {
    if s {
        "High".to_string()
    } else {
        "Low".to_string()
    }
}

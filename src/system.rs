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

use config::Config;
use data_path::DataPath;
use memory::Memory;
use util::Result;

pub struct System {
    /// RISCII data path.
    data_path: DataPath,
    /// Memory state.
    mem: Memory,
}

impl System {
    pub fn new(config: &Config) -> Result<Self> {
        Ok(Self {
            data_path: DataPath::new(config)?,
            mem: Memory::new(config),
        })
    }

    pub fn get_mem_ref(&mut self) -> &mut Memory {
        &mut self.mem
    }
}

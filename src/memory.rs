// RISC II memory scheme.
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

// Struct definitions.

use util::File;

use config::Config;

/// The real memory of the RISC II emulator.
pub struct Memory(Vec<u8>);

// Struct impls.

impl Memory {
    /// Create a memory object.
    /// # Arguments
    /// * `config` - A configuration object that determines the size of
    /// the memory object.
    pub fn new(config: &Config) -> Self {
        Self {
            0: vec![0u8; config.get_mem_size() as usize],
        }
    }

    pub fn from_vec(memory: &Vec<u8>) -> Self {
        Self { 0: memory.clone() }
    }

    pub fn write_to_file(&mut self, file: &mut File) -> Result<(), String> {
        file.write_vec(&self.0)?;
        Ok(())
    }
}

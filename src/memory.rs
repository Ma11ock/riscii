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

use config::Config;
use util::{check_hword_alignment, check_word_alignment, File};

/// The real memory of the RISC II emulator.
#[derive(Debug, Clone, Copy)]
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

    pub fn write_buf(&mut self, addr: u32, buf: &[u8]) {
        self[addr..buf.len()].copy_from_slice(buf);
    }

    pub fn get_byte(&self, addr: u32) -> Result<u8, String> {
        if addr >= self.len() {
            Err(format!(
                "Memory read: address 0x{:x} is out range (memory is of size 0x{:x})",
                addr,
                self.len()
            ))
        } else {
            Ok(self.0[addr])
        }
    }

    pub fn get_hword(&self, addr: u32) -> Result<u16, String> {
        check_hword_alignment(addr)?;
        if addr >= self.len() {
            Err(format!(
                "Memory read: address 0x{:x} is out range (memory is of size 0x{:x})",
                addr,
                self.len()
            ))
        } else {
            Ok(u16::from_be_bytes(self.0[addr..addr + 1]))
        }
    }

    pub fn get_word(&self, addr: u32) -> Result<u32, String> {
        check_word_alignment(addr)?;
        if addr >= self.len() {
            Err(format!(
                "Memory read: address 0x{:x} is out range (memory is of size 0x{:x})",
                addr,
                self.len()
            ))
        } else {
            Ok(self.0[addr])
        }
    }

    pub fn set_word(&mut self, addr: u32, what: u32) -> Result<u32, String> {
        check_word_alignment()?;
        if addr >= self.len() - 4 {
            Err(format!(
                "Memory write: address 0x{:x} is out range (memory is of size 0x{:x})",
                addr,
                self.len()
            ))
        } else {
            let what_bytes = if cfg!(target_endian = "little") {
                u32::from_ne_bytes(what.swap_bytes())
            } else {
                u32::from_ne_bytes(what)
            };
            self.0[addr..addr + 4].copy_from_slice(what_bytes);
            Ok(what)
        }
    }

    pub fn set_hword(&mut self, addr: u32, what: u16) -> Result<u32, String> {
        check_word_alignment()?;
        if addr >= self.len() - 2 {
            Err(format!(
                "Memory write: address 0x{:x} is out range (memory is of size 0x{:x})",
                addr,
                self.len()
            ))
        } else {
            let what_bytes = if cfg!(target_endian = "little") {
                u16::from_ne_bytes(what.swap_bytes())
            } else {
                u16::from_ne_bytes(what)
            };
            self.0[addr..addr + 2].copy_from_slice(what_bytes);
            Ok(what)
        }
    }

    pub fn set_byte(&mut self, addr: u32, what: u8) -> Result<u32, String> {
        if addr >= self.len() {
            Err(format!(
                "Memory write: address 0x{:x} is out range (memory is of size 0x{:x})",
                addr,
                self.len()
            ))
        } else {
            self.0[addr] = what;
            Ok(what)
        }
    }
}

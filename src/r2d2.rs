// Binary memdmp format.
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

// Public function definitions.

use memory::Memory;
use register::State;
use util::{concat_paths, get_unix_timestamp, File};

use std::time::{SystemTime, UNIX_EPOCH};

use std::fs::OpenOptions;

/// Write a save file to the cache directory. On success, return the name of
/// the save file. Return an error string on failure.
/// # Arguments
/// * `state` - Register state to save.
/// * `mem` - Memory state to save.
pub fn write(config: &Config, state: &State, mem: &Memory) -> Result<String, String> {
    // TODO human readable dates.
    let output_file = concat_paths(
        &config.cache_path,
        &format!("{:?}.r2d2", get_unix_timestamp()?),
    )?;
    let file = File::open_ops(output_file, &OpenOptions::new().write(true))?;

    file.write_buf(&state.to_buf())?;
    mem.write_to_file(&file)?;
    Ok(output_file)
}

pub fn read(config: &Config, which: &String) -> Result<(State, Memory), String> {
    let mut file = File::open(&which)?;
    let mut register_state = [0u8; register::TOTAL_NUM_REGISTERS];
    if file.read(register_state)? < register_state.len() {
        return Err(format!(
            "Archive {} is not large enough to have a register window.",
            which
        ));
    }

    let metadata = file.get_metadata()?;
    let mut memory = vec![0u8; metadata.size() as usize - register::TOTAL_NUM_REGISTERS];

    file.read_into_vec(&mut memory)?;

    // TODO make sure there is no deep copying of memory happening
    Ok(State::from_buf(register_state), Memory::from_vec(&memory))
}

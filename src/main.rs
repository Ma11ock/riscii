// An emulator for the RISC-II microprocessor architecture.
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

extern crate core;
extern crate sdl2;
#[cfg(test)]
mod main_test;

mod config;
mod decode;
mod sdl;
mod util;

use decode::decode_file;
use std::fs;

fn get_program(path: &String) -> Result<Vec<u8>, String> {
    println!("Opening binary file {}.", path);

    Ok(match fs::read(path) {
        Ok(raw_p) => raw_p.to_vec(),
        Err(raw_e) => return Err(raw_e.to_string()),
    })
}

fn main() -> Result<(), String> {
    let config = config::Config::init()?;
    let context = sdl::Context::new(&config)?;

    println!(
        "Running emulator with the following configuration: \n{}\n",
        config
    );
    //let program = get_program(&String::from("test.bin"))?;

    Ok(())
}

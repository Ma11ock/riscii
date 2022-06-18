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
mod decode_test;
#[cfg(test)]
mod main_test;

mod config;
mod cpu;
mod decode;
mod instruction;
mod memory;
mod sdl;
mod system;
mod util;

use decode::decode_file;
use std::fs;

use config::Config;
use std::boxed::Box;
use std::error::Error;
use system::System;

// Struct/enum declarations.

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::init()?;
    let context = sdl::Context::new(&config)?;

    let system = System::new(&config);
    println!(
        "Running emulator with the following configuration: \n{}\n",
        config
    );
    //println!("Opening binary file {}.", path);
    //let program = fs::read(path)?;

    Ok(())
}

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

#[macro_use]
extern crate assert_hex;
extern crate core;
extern crate sdl2;
#[cfg(test)]
mod decode_test;
#[cfg(test)]
mod encode_test;
#[cfg(test)]
mod main_test;

// Modules declared as pub to shut up rust-analyzer about dead code.
pub mod config;
pub mod cpu;
pub mod decode;
pub mod instruction;
pub mod memory;
pub mod sdl;
pub mod system;
pub mod util;
pub mod windows;

use decode::decode_file;
use std::fs;

use config::Config;
use std::boxed::Box;
use std::error::Error;
use system::System;
use windows::{DebugWindow, Drawable, MainWindow};
// Struct/enum declarations.

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::init()?;

    let system = System::new(&config)?;
    println!(
        "Running emulator with the following configuration: \n{}\n",
        config
    );
    //println!("Opening binary file {}.", path);
    //let program = fs::read(path)?;

    let mut debug_window = Some(DebugWindow::new(&config, &system)?);
    let mut main_window = MainWindow::new(&config, &system)?;

    'running: loop {
        if main_window.handle_events() {
            break 'running;
        }
        debug_window = if let Some(mut win) = debug_window {
            if win.handle_events() {
                None
            } else {
                Some(win)
            }
        } else {
            None
        };

        debug_window = if let Some(mut win) = debug_window {
            win.draw();
            Some(win)
        } else {
            None
        };
        main_window.draw();
    }
    Ok(())
}

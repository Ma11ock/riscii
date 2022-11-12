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
pub mod alu;
pub mod clock;
pub mod config;
pub mod cpu;
pub mod data_path;
pub mod debug_window;
pub mod decode;
pub mod instruction;
pub mod memory;
pub mod sdl;
pub mod shifter;
pub mod system;
pub mod util;

use config::Config;
use debug_window::DebugWindow;
use sdl::{make_font_context, Context, Drawable};
use sdl2::event::{Event, WindowEvent};
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;
use system::System;

// Struct/enum declarations.

enum GlobalAction {
    None,
    QuitProgram,
    CloseDebugWindow,
}

fn handle_events(context: &mut Context, debug_window: &mut DebugWindow) -> GlobalAction {
    let event_pump = &mut context.event_pump;
    let mut result = GlobalAction::None;
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. } => {
                return GlobalAction::QuitProgram;
            }
            Event::Window {
                win_event: WindowEvent::Close,
                ..
            } => {
                return GlobalAction::QuitProgram;
            }
            Event::KeyDown {
                keycode: Some(kc), ..
            } => {
                debug_window.handle_key_down(kc);
            }
            Event::KeyUp {
                keycode: Some(kc), ..
            } => {
                debug_window.handle_key_up(kc);
            }
            _ => {}
        }
    }
    return result;
}

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::init()?;

    println!(
        "Running emulator with the following configuration: \n{}\n",
        config
    );
    let system = Rc::new(RefCell::new(System::new(&config)?));
    //println!("Opening binary file {}.", path);
    //let program = fs::read(path)?;
    let mut sdl_context = Context::new()?;
    let mut font_context = make_font_context()?;

    let mut debug_window = if config.is_debug_mode() {
        Some(DebugWindow::new(
            &config,
            system.clone(),
            &mut sdl_context,
            &mut font_context,
        )?)
    } else {
        None
    };

    'running: loop {
        system.borrow_mut().tick();
        debug_window = if let Some(mut win) = debug_window {
            match { handle_events(&mut sdl_context, &mut win) } {
                GlobalAction::QuitProgram => {
                    break 'running;
                }
                GlobalAction::CloseDebugWindow => {
                    debug_window = None;
                }
                _ => {}
            }
            win.draw(&mut sdl_context)?;
            Some(win)
        } else {
            None
        };
    }
    Ok(())
}

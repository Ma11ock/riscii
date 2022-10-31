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
pub mod window;

use config::Config;
use debug_window::DebugWindow;
use sdl::{make_font_context, Context, Drawable};
use sdl2::event::{Event, WindowEvent};
use std::error::Error;
use system::System;
use window::MainWindow;

// Struct/enum declarations.

enum GlobalAction {
    None,
    QuitProgram,
    CloseDebugWindow,
}

fn handle_events(
    context: &mut Context,
    win: &mut MainWindow,
    debug_window: &mut Option<DebugWindow>,
) -> GlobalAction {
    let event_pump = &mut context.event_pump;
    let mut result = GlobalAction::None;
    let main_win_id = win.get_window_id();
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. } => {
                return GlobalAction::QuitProgram;
            }
            Event::Window {
                win_event: WindowEvent::Close,
                window_id: id,
                ..
            } => {
                if id == main_win_id {
                    return GlobalAction::QuitProgram;
                } else if let Some(dwin) = debug_window {
                    if dwin.get_window_id() == id {
                        result = GlobalAction::CloseDebugWindow;
                        continue;
                    }
                }
                eprintln!("Close for window id {}, but it does not exist!", id);
            }
            Event::KeyDown {
                keycode: Some(kc),
                window_id: id,
                ..
            } => {
                if id == main_win_id {
                    win.handle_key_down(kc);
                } else if let Some(dwin) = debug_window {
                    if dwin.get_window_id() == id {
                        dwin.handle_key_down(kc);
                        continue;
                    }
                }
                eprintln!("Keydown event for window id {}, but it does not exist!", id);
            }
            Event::KeyUp {
                keycode: Some(kc),
                window_id: id,
                ..
            } => {
                if id == main_win_id {
                    win.handle_key_up(kc);
                } else if let Some(dwin) = debug_window {
                    if dwin.get_window_id() == id {
                        dwin.handle_key_up(kc);
                        continue;
                    }
                }
                eprintln!("Keyup event for window id {}, but it does not exist!", id);
            }
            _ => {}
        }
    }
    return result;
}

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::init()?;

    let system = System::new(&config)?;
    println!(
        "Running emulator with the following configuration: \n{}\n",
        config
    );
    //println!("Opening binary file {}.", path);
    //let program = fs::read(path)?;
    let mut sdl_context = Context::new()?;
    let mut font_context = make_font_context()?;

    let mut main_window = MainWindow::new(&config, &system, &mut sdl_context)?;
    let mut debug_window = if config.is_debug_mode() {
        Some(DebugWindow::new(
            &config,
            &system,
            &mut sdl_context,
            &mut font_context,
        )?)
    } else {
        None
    };

    'running: loop {
        match { handle_events(&mut sdl_context, &mut main_window, &mut debug_window) } {
            GlobalAction::QuitProgram => {
                break 'running;
            }
            GlobalAction::CloseDebugWindow => {
                debug_window = None;
            }
            _ => {}
        }
        debug_window = if let Some(mut win) = debug_window {
            win.draw(&mut sdl_context);
            Some(win)
        } else {
            None
        };
        main_window.draw(&mut sdl_context);
    }
    Ok(())
}

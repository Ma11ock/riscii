// RISC II emulator windows.
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
use sdl::Context;
use sdl2::event::Event;
use sdl2::pixels::*;
use system::System;
use util::Result;

pub trait Drawable {
    fn draw(&mut self);
    fn handle_events(&mut self) -> bool;
}

pub struct DebugWindow<'a> {
    context: Context,
    system: &'a System,
    config: &'a Config,
}

pub struct MainWindow<'a> {
    context: Context,
    system: &'a System,
    config: &'a Config,
}

// Struct impls.

impl<'a> Drawable for MainWindow<'a> {
    fn draw(&mut self) {}

    fn handle_events(&mut self) -> bool {
        false
    }
}

impl<'a> MainWindow<'a> {
    pub fn new(config: &'a Config, system: &'a System) -> Result<Self> {
        Ok(Self {
            context: Context::new(
                config.get_win_width(),
                config.get_debug_win_height(),
                format!("RISC II"),
            )?,
            system: system,
            config: config,
        })
    }
}

impl<'a> DebugWindow<'a> {
    pub fn new(config: &'a Config, system: &'a System) -> Result<Self> {
        Ok(Self {
            context: Context::new(
                config.get_debug_win_width(),
                config.get_debug_win_height(),
                format!("Debug"),
            )?,
            system: system,
            config: config,
        })
    }
}

impl<'a> Drawable for DebugWindow<'a> {
    fn draw(&mut self) {
        self.context.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.context.canvas.clear();

        //
        self.context.canvas.present();
    }
    fn handle_events(&mut self) -> bool {
        let event_pump = &mut self.context.event_pump;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    return true;
                }
                Event::KeyDown {
                    keycode: Some(kc), ..
                } => {}
                Event::KeyUp {
                    keycode: Some(kc), ..
                } => {}
                _ => {}
            }
        }
        return false;
    }
}

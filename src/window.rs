// RISC II emulator window.
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
use sdl::{Context, Drawable, Pane};
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::pixels::*;
use system::System;
use util::Result;

pub struct MainWindow<'a> {
    pane: Pane,
    system: &'a System,
    config: &'a Config,
}

// Struct impls.

impl<'a> Drawable for MainWindow<'a> {
    fn draw(&mut self, context: &mut Context) -> Result<()> {
        self.pane.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.pane.canvas.clear();
        //
        self.pane.canvas.present();

        Ok(())
    }

    fn handle_key_down(&mut self, kc: Keycode) {}
    fn handle_key_up(&mut self, kc: Keycode) {}
    fn get_window_id(&self) -> u32 {
        self.pane.get_id()
    }
}

impl<'a> MainWindow<'a> {
    pub fn new(config: &'a Config, system: &'a System, context: &mut Context) -> Result<Self> {
        Ok(Self {
            pane: Pane::new(
                config.get_win_width(),
                config.get_debug_win_height(),
                format!("RISC II"),
                context,
            )?,
            system: system,
            config: config,
        })
    }
}

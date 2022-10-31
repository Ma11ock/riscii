// RISC II emulator debug window.
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

use config::Config;
use sdl::{Context, Drawable, Pane};
use sdl2::keyboard::Keycode;
use sdl2::pixels::*;
use sdl2::rect::Rect;
use sdl2::ttf::{Font, Sdl2TtfContext};
use system::System;
use util::Result;

pub struct DebugWindow<'a> {
    pane: Pane,
    system: &'a System,
    config: &'a Config,
    font: Font<'a, 'static>,
}

impl<'a> DebugWindow<'a> {
    pub fn new(
        config: &'a Config,
        system: &'a System,
        context: &mut Context,
        ttf: &'a mut Sdl2TtfContext,
    ) -> Result<Self> {
        let pane = Pane::new(
            config.get_debug_win_width(),
            config.get_debug_win_height(),
            format!("Debug"),
            context,
        )?;
        let debug_font = { ttf.load_font("debug.otf", 12)? };
        Ok(Self {
            font: debug_font,
            pane: pane,
            system: system,
            config: config,
        })
    }
}

impl<'a> Drawable for DebugWindow<'a> {
    fn draw(&mut self, context: &mut Context) {
        // Clear the window.
        const DRAW_COLOR: Color = Color::RGB(0, 0, 0);
        self.pane.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.pane.canvas.clear();

        const OBJ_DEFAULT_COLOR: Color = Color::RGB(0xFF, 0xFF, 0xFF);
        const OBJ_USE_COLOR: Color = Color::RGB(0xFa, 0x10, 0x10);

        // Register file.
        let mut reg_file = Rect::new(100, 200, 200, 400);

        // Draw register file.
        self.pane.canvas.set_draw_color(OBJ_DEFAULT_COLOR);
        self.pane.canvas.draw_rect(reg_file);

        // Draw the debug window.
        self.pane.canvas.present();
    }

    fn handle_key_down(&mut self, kc: Keycode) {}
    fn handle_key_up(&mut self, kc: Keycode) {}
    fn get_window_id(&self) -> u32 {
        self.pane.get_id()
    }
}

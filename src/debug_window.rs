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
use sdl2::event::{Event, WindowEvent};
use sdl2::pixels::*;
use system::System;
use util::Result;

pub struct DebugWindow<'a> {
    pane: Pane,
    system: &'a System,
    config: &'a Config,
}

impl<'a> DebugWindow<'a> {
    pub fn new(config: &'a Config, system: &'a System, context: &mut Context) -> Result<Self> {
        Ok(Self {
            pane: Pane::new(
                config.get_debug_win_width(),
                config.get_debug_win_height(),
                format!("Debug"),
                context,
            )?,
            system: system,
            config: config,
        })
    }
}

impl<'a> Drawable for DebugWindow<'a> {
    fn draw(&mut self, context: &mut Context) {
        self.pane.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.pane.canvas.clear();

        //
        self.pane.canvas.present();
    }
    fn handle_events(&mut self, context: &mut Context) -> bool {
        let event_pump = &mut context.event_pump;
        let window_id = self.pane.get_id();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    return true;
                }
                Event::Window {
                    win_event: WindowEvent::Close,
                    window_id: id,
                    ..
                } => {
                    if id == window_id {
                        return true;
                    }
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

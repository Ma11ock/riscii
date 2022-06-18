// RISC II emulator window and I/O.
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

extern crate sdl2;

use config::Config;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::EventPump;
use sdl2::Sdl;
use sdl2::VideoSubsystem;
use util::Result;

// Struct definitions.

/// SDL context structs.
pub struct Context {
    /// SDL context.
    context: Sdl,
    /// Video context.
    video_system: VideoSubsystem,
    /// Window canvas.
    canvas: Canvas<Window>,
    /// Event queue.
    event_pump: EventPump,
}

// Struct impls.

impl Context {
    /// Create a new SDL window/context. Return context on success and a
    /// string on error.
    pub fn new(config: &Config) -> Result<Self> {
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;
        let window = video_subsystem
            .window("RISC II", config.get_win_width(), config.get_win_height())
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;

        let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();
        let event_pump = sdl_context.event_pump()?;

        Ok(Self {
            context: sdl_context,
            video_system: video_subsystem,
            canvas: canvas,
            event_pump: event_pump,
        })
    }
}

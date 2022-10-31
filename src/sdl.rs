// RISC II emulator SDL layer.
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
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::ttf::{Font, Sdl2TtfContext};
use sdl2::video::{Window, WindowContext};
use sdl2::EventPump;
use sdl2::Sdl;
use sdl2::VideoSubsystem;
use std::path::Path;
use system::System;
use util::Result;

// Struct definitions.

pub trait Drawable {
    fn draw(&mut self, context: &mut Context);
    fn handle_key_down(&mut self, kc: Keycode);
    fn handle_key_up(&mut self, kc: Keycode);
    fn get_window_id(&self) -> u32;
}

pub struct Context {
    /// SDL context.
    pub context: Sdl,
    /// Video context.
    pub video_system: VideoSubsystem,
    /// Event queue.
    pub event_pump: EventPump,
}

/// SDL context structs.
pub struct Pane {
    /// Window canvas.
    pub canvas: Canvas<Window>,
    /// Id.
    window_id: u32,
    /// Texture creator.
    pub texture_creator: TextureCreator<WindowContext>,
}

pub fn make_font_context() -> std::result::Result<Sdl2TtfContext, String> {
    sdl2::ttf::init().map_err(|e| e.to_string())
}

// Struct impls.

impl Context {
    pub fn new() -> Result<Self> {
        let sdl = sdl2::init()?;
        let event_pump = sdl.event_pump()?;

        Ok(Self {
            video_system: sdl.video()?,
            context: sdl,
            event_pump: event_pump,
        })
    }
}

impl Pane {
    /// Create a new SDL window/context. Return context on success and a
    /// string on error.
    pub fn new(width: u32, height: u32, name: String, context: &mut Context) -> Result<Self> {
        let window = context
            .video_system
            .window(name.as_str(), width, height)
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;

        let id = window.id();
        let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        let texture_creator = canvas.texture_creator();

        Ok(Self {
            canvas: canvas,
            window_id: id,
            texture_creator: texture_creator,
        })
    }

    pub fn get_id(&self) -> u32 {
        self.window_id
    }
}

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
use sdl2::gfx::primitives::DrawRenderer;
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
        let debug_font = { ttf.load_font("debug.otf", 20)? };
        Ok(Self {
            font: debug_font,
            pane: pane,
            system: system,
            config: config,
        })
    }
}

impl<'a> Drawable for DebugWindow<'a> {
    fn draw(&mut self, context: &mut Context) -> Result<()> {
        // Clear the window.
        const CLEAR_COLOR: Color = Color::RGB(0, 0, 0);
        self.pane.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.pane.canvas.clear();

        const OBJ_DEFAULT_COLOR: Color = Color::RGB(0xFF, 0xFF, 0xFF);
        const OBJ_USE_COLOR: Color = Color::RGB(0xFa, 0x10, 0x10);

        // Register file.
        let reg_file = Rect::new(50, 400, 200, 400);
        let reg_file_name = self
            .font
            .render("Register File")
            .blended(OBJ_DEFAULT_COLOR)
            .map_err(|e| e.to_string())?;
        let texture = self
            .pane
            .texture_creator
            .create_texture_from_surface(&reg_file_name)
            .map_err(|e| e.to_string())?;

        // Draw register file.
        self.pane.canvas.set_draw_color(OBJ_DEFAULT_COLOR);
        self.pane.canvas.draw_rect(reg_file)?;

        self.pane
            .canvas
            .copy(&texture, None, Some(Rect::new(60, 400, 180, 50)))?;

        // Draw the latches.
        // Start with DST.
        let latch = Rect::new(350, 500, 50, 300);
        self.pane.canvas.set_draw_color(OBJ_DEFAULT_COLOR);
        self.pane.canvas.draw_rect(latch)?;

        let latch_name = self
            .font
            .render("DST")
            .blended(OBJ_DEFAULT_COLOR)
            .map_err(|e| e.to_string())?;
        let texture = self
            .pane
            .texture_creator
            .create_texture_from_surface(&latch_name)
            .map_err(|e| e.to_string())?;

        self.pane
            .canvas
            .copy(&texture, None, Some(Rect::new(325, 800, 100, 50)))?;

        // Now SRC.
        let latch = Rect::new(475, 500, 50, 300);
        self.pane.canvas.set_draw_color(OBJ_DEFAULT_COLOR);
        self.pane.canvas.draw_rect(latch)?;

        let latch_name = self
            .font
            .render("SRC")
            .blended(OBJ_DEFAULT_COLOR)
            .map_err(|e| e.to_string())?;
        let texture = self
            .pane
            .texture_creator
            .create_texture_from_surface(&latch_name)
            .map_err(|e| e.to_string())?;

        self.pane
            .canvas
            .copy(&texture, None, Some(Rect::new(450, 800, 100, 50)))?;

        // Now NXTPC.
        let latch = Rect::new(1075, 500, 50, 300);
        self.pane.canvas.set_draw_color(OBJ_DEFAULT_COLOR);
        self.pane.canvas.draw_rect(latch)?;

        let latch_name = self
            .font
            .render("NXTPC")
            .blended(OBJ_DEFAULT_COLOR)
            .map_err(|e| e.to_string())?;
        let texture = self
            .pane
            .texture_creator
            .create_texture_from_surface(&latch_name)
            .map_err(|e| e.to_string())?;

        self.pane
            .canvas
            .copy(&texture, None, Some(Rect::new(1050, 800, 100, 50)))?;

        // Now PC.
        let latch = Rect::new(1175, 500, 50, 300);
        self.pane.canvas.set_draw_color(OBJ_DEFAULT_COLOR);
        self.pane.canvas.draw_rect(latch)?;

        let latch_name = self
            .font
            .render("PC")
            .blended(OBJ_DEFAULT_COLOR)
            .map_err(|e| e.to_string())?;
        let texture = self
            .pane
            .texture_creator
            .create_texture_from_surface(&latch_name)
            .map_err(|e| e.to_string())?;

        self.pane
            .canvas
            .copy(&texture, None, Some(Rect::new(1175, 800, 50, 50)))?;

        // Now LSTPC.
        let latch = Rect::new(1275, 500, 50, 300);
        self.pane.canvas.set_draw_color(OBJ_DEFAULT_COLOR);
        self.pane.canvas.draw_rect(latch)?;

        let latch_name = self
            .font
            .render("LSTPC")
            .blended(OBJ_DEFAULT_COLOR)
            .map_err(|e| e.to_string())?;
        let texture = self
            .pane
            .texture_creator
            .create_texture_from_surface(&latch_name)
            .map_err(|e| e.to_string())?;

        self.pane
            .canvas
            .copy(&texture, None, Some(Rect::new(1250, 800, 100, 50)))?;

        // RD
        let latch = Rect::new(100, 75, 100, 50);
        self.pane.canvas.set_draw_color(OBJ_DEFAULT_COLOR);
        self.pane.canvas.draw_rect(latch)?;

        let latch_name = self
            .font
            .render("RD")
            .blended(OBJ_DEFAULT_COLOR)
            .map_err(|e| e.to_string())?;
        let texture = self
            .pane
            .texture_creator
            .create_texture_from_surface(&latch_name)
            .map_err(|e| e.to_string())?;

        self.pane
            .canvas
            .copy(&texture, None, Some(Rect::new(125, 125, 50, 50)))?;

        // RS1
        let latch = Rect::new(50, 200, 100, 50);
        self.pane.canvas.set_draw_color(OBJ_DEFAULT_COLOR);
        self.pane.canvas.draw_rect(latch)?;

        let latch_name = self
            .font
            .render("RS1")
            .blended(OBJ_DEFAULT_COLOR)
            .map_err(|e| e.to_string())?;
        let texture = self
            .pane
            .texture_creator
            .create_texture_from_surface(&latch_name)
            .map_err(|e| e.to_string())?;

        self.pane
            .canvas
            .copy(&texture, None, Some(Rect::new(75, 250, 50, 50)))?;

        // RS2
        let latch = Rect::new(175, 200, 100, 50);
        self.pane.canvas.set_draw_color(OBJ_DEFAULT_COLOR);
        self.pane.canvas.draw_rect(latch)?;

        let latch_name = self
            .font
            .render("RS2")
            .blended(OBJ_DEFAULT_COLOR)
            .map_err(|e| e.to_string())?;
        let texture = self
            .pane
            .texture_creator
            .create_texture_from_surface(&latch_name)
            .map_err(|e| e.to_string())?;

        self.pane
            .canvas
            .copy(&texture, None, Some(Rect::new(200, 250, 50, 50)))?;

        // PSW register
        let latch = Rect::new(350, 200, 125, 75);
        self.pane.canvas.set_draw_color(OBJ_DEFAULT_COLOR);
        self.pane.canvas.draw_rect(latch)?;

        let latch_name = self
            .font
            .render("PSW")
            .blended(OBJ_DEFAULT_COLOR)
            .map_err(|e| e.to_string())?;
        let texture = self
            .pane
            .texture_creator
            .create_texture_from_surface(&latch_name)
            .map_err(|e| e.to_string())?;

        self.pane
            .canvas
            .copy(&texture, None, Some(Rect::new(350, 275, 75, 50)))?;
        // imm
        let latch = Rect::new(800, 100, 100, 50);
        self.pane.canvas.set_draw_color(OBJ_DEFAULT_COLOR);
        self.pane.canvas.draw_rect(latch)?;

        let latch_name = self
            .font
            .render("IMM")
            .blended(OBJ_DEFAULT_COLOR)
            .map_err(|e| e.to_string())?;
        let texture = self
            .pane
            .texture_creator
            .create_texture_from_surface(&latch_name)
            .map_err(|e| e.to_string())?;

        self.pane
            .canvas
            .copy(&texture, None, Some(Rect::new(910, 100, 75, 50)))?;
        // dimm
        let latch = Rect::new(800, 250, 250, 75);
        self.pane.canvas.set_draw_color(OBJ_DEFAULT_COLOR);
        self.pane.canvas.draw_rect(latch)?;

        let latch_name = self
            .font
            .render("DIn/DImm")
            .blended(OBJ_DEFAULT_COLOR)
            .map_err(|e| e.to_string())?;
        let texture = self
            .pane
            .texture_creator
            .create_texture_from_surface(&latch_name)
            .map_err(|e| e.to_string())?;

        self.pane
            .canvas
            .copy(&texture, None, Some(Rect::new(900, 325, 150, 50)))?;
        // op
        let latch = Rect::new(1100, 125, 50, 50);
        self.pane.canvas.set_draw_color(OBJ_DEFAULT_COLOR);
        self.pane.canvas.draw_rect(latch)?;

        let latch_name = self
            .font
            .render("OP")
            .blended(OBJ_DEFAULT_COLOR)
            .map_err(|e| e.to_string())?;
        let texture = self
            .pane
            .texture_creator
            .create_texture_from_surface(&latch_name)
            .map_err(|e| e.to_string())?;

        self.pane
            .canvas
            .copy(&texture, None, Some(Rect::new(1100, 175, 50, 50)))?;
        // Shifter
        let latch = Rect::new(600, 500, 175, 300);
        self.pane.canvas.set_draw_color(OBJ_DEFAULT_COLOR);
        self.pane.canvas.draw_rect(latch)?;

        let latch_name = self
            .font
            .render("Shifter")
            .blended(OBJ_DEFAULT_COLOR)
            .map_err(|e| e.to_string())?;
        let texture = self
            .pane
            .texture_creator
            .create_texture_from_surface(&latch_name)
            .map_err(|e| e.to_string())?;

        self.pane
            .canvas
            .copy(&texture, None, Some(Rect::new(600, 450, 100, 50)))?;

        self.pane.canvas.circle(690, 650, 50, OBJ_DEFAULT_COLOR)?;
        // ALU
        self.pane.canvas.polygon(
            &[900, 1000, 1000, 900, 900, 930, 900, 900],
            &[500, 520, 780, 800, 670, 650, 630, 500],
            OBJ_DEFAULT_COLOR,
        )?;
        let alu_name = self
            .font
            .render("ALU")
            .blended(OBJ_DEFAULT_COLOR)
            .map_err(|e| e.to_string())?;
        let texture = self
            .pane
            .texture_creator
            .create_texture_from_surface(&alu_name)
            .map_err(|e| e.to_string())?;

        self.pane
            .canvas
            .copy(&texture, None, Some(Rect::new(900, 450, 75, 50)))?;
        // AI (ALU input latch)
        let latch = Rect::new(875, 500, 25, 120);
        self.pane.canvas.set_draw_color(OBJ_DEFAULT_COLOR);
        self.pane.canvas.draw_rect(latch)?;

        let latch_name = self
            .font
            .render("AI")
            .blended(OBJ_DEFAULT_COLOR)
            .map_err(|e| e.to_string())?;
        let texture = self
            .pane
            .texture_creator
            .create_texture_from_surface(&latch_name)
            .map_err(|e| e.to_string())?;

        self.pane
            .canvas
            .copy(&texture, None, Some(Rect::new(825, 450, 50, 50)))?;
        // BI (ALU input latch)
        let latch = Rect::new(875, 680, 25, 120);
        self.pane.canvas.set_draw_color(OBJ_DEFAULT_COLOR);
        self.pane.canvas.draw_rect(latch)?;

        let latch_name = self
            .font
            .render("BI")
            .blended(OBJ_DEFAULT_COLOR)
            .map_err(|e| e.to_string())?;
        let texture = self
            .pane
            .texture_creator
            .create_texture_from_surface(&latch_name)
            .map_err(|e| e.to_string())?;

        self.pane
            .canvas
            .copy(&texture, None, Some(Rect::new(825, 800, 50, 50)))?;
        // BAR
        let latch = Rect::new(800, 400, 25, 25);
        self.pane.canvas.set_draw_color(OBJ_DEFAULT_COLOR);
        self.pane.canvas.draw_rect(latch)?;

        let latch_name = self
            .font
            .render("BAR")
            .blended(OBJ_DEFAULT_COLOR)
            .map_err(|e| e.to_string())?;
        let texture = self
            .pane
            .texture_creator
            .create_texture_from_surface(&latch_name)
            .map_err(|e| e.to_string())?;

        self.pane
            .canvas
            .copy(&texture, None, Some(Rect::new(830, 400, 75, 50)))?;

        // Draw the debug window.
        self.pane.canvas.present();
        Ok(())
    }

    fn handle_key_down(&mut self, kc: Keycode) {}
    fn handle_key_up(&mut self, kc: Keycode) {}
    fn get_window_id(&self) -> u32 {
        self.pane.get_id()
    }
}

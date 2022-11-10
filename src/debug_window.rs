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

use clock::Phase;
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

    fn draw_static_str(&mut self, string: &str, location: Rect, color: Color) -> Result<()> {
        let name = self
            .font
            .render(string)
            .blended(color)
            .map_err(|e| e.to_string())?;
        let texture = self
            .pane
            .texture_creator
            .create_texture_from_surface(&name)
            .map_err(|e| e.to_string())?;
        self.pane.canvas.copy(&texture, None, Some(location))?;
        Ok(())
    }

    fn draw_string(&mut self, string: &String, location: Rect, color: Color) -> Result<()> {
        self.draw_static_str(string.as_str(), location, color)
    }

    fn draw_lines(&mut self, lines: &[(i16, i16, i16, i16)], color: Color) -> Result<()> {
        for line in lines.iter() {
            self.draw_line(*line, color)?;
        }
        Ok(())
    }

    fn draw_rects(&mut self, rects: &[Rect], color: Color) -> Result<()> {
        for rect in rects.iter() {
            self.draw_rect(*rect, color)?;
        }
        Ok(())
    }

    fn draw_line(&mut self, line: (i16, i16, i16, i16), color: Color) -> Result<()> {
        let (x1, y1, x2, y2) = line;
        self.pane.canvas.line(x1, y1, x2, y2, color)?;
        Ok(())
    }

    fn draw_rect(&mut self, rect: Rect, color: Color) -> Result<()> {
        self.pane.canvas.set_draw_color(color);
        self.pane.canvas.draw_rect(rect)?;
        Ok(())
    }

    fn draw_circle(&mut self, circle: (i16, i16, i16), color: Color) -> Result<()> {
        self.pane
            .canvas
            .circle(circle.0, circle.1, circle.2, color)?;
        Ok(())
    }

    fn draw_polygon(&mut self, xs: &[i16], ys: &[i16], color: Color) -> Result<()> {
        self.pane.canvas.polygon(xs, ys, color)?;
        Ok(())
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

        let dp = self.system.data_path(); // Data path reference.

        // Describe the phase of the clock.
        self.draw_static_str(
            match self.system.phase() {
                Phase::One => "φ₁",
                Phase::Two => "φ₂",
                Phase::Three => "φ₃",
                Phase::Four => "φ₄",
                Phase::Interrupt => "φᵢ",
            },
            Rect::new(1550, 0, 50, 50),
            OBJ_DEFAULT_COLOR,
        )?;

        // busEXT
        self.draw_line((0, 50, 1450, 50), OBJ_DEFAULT_COLOR)?;
        self.draw_static_str("busEXT", Rect::new(600, 50, 125, 50), OBJ_DEFAULT_COLOR)?;

        // Register file.
        // Draw register file.
        self.draw_static_str(
            "Register File",
            Rect::new(60, 800, 180, 50),
            OBJ_DEFAULT_COLOR,
        )?;
        self.draw_rect(Rect::new(50, 400, 200, 400), OBJ_DEFAULT_COLOR)?;

        // Register file values.
        let (rs1, rs2) = dp.execute_source_registers();

        self.draw_string(
            &format!(
                "R{:02}:{:08x}",
                rs1,
                dp.register_file().read(rs1, dp.psw().get_cwp())
            ),
            Rect::new(60, 400, 180, 50),
            OBJ_DEFAULT_COLOR,
        )?;
        self.draw_string(
            &format!(
                "R{:02}:{:08x}",
                rs2,
                dp.register_file().read(rs2, dp.psw().get_cwp())
            ),
            Rect::new(60, 450, 180, 50),
            OBJ_DEFAULT_COLOR,
        )?;
        // busA
        self.draw_line((60, 600, 475, 600), OBJ_DEFAULT_COLOR)?;
        self.draw_static_str("busA", Rect::new(60, 610, 50, 25), OBJ_DEFAULT_COLOR)?;

        // busB
        self.draw_line((60, 700, 475, 700), OBJ_DEFAULT_COLOR)?;
        self.draw_static_str("busB", Rect::new(60, 710, 50, 25), OBJ_DEFAULT_COLOR)?;

        // Draw the latches.
        // Start with DST.
        self.draw_rect(Rect::new(350, 500, 50, 300), OBJ_DEFAULT_COLOR)?;
        self.draw_static_str("DST", Rect::new(325, 800, 100, 50), OBJ_DEFAULT_COLOR)?;
        // busD
        self.draw_lines(
            &[
                (400, 525, 1275, 525),
                (815, 525, 815, 450),
                (850, 525, 850, 575),
                (850, 575, 875, 575),
            ],
            OBJ_DEFAULT_COLOR,
        )?;
        self.draw_static_str("busD", Rect::new(410, 500, 50, 25), OBJ_DEFAULT_COLOR)?;
        // busR
        self.draw_lines(
            &[
                (400, 640, 850, 640),
                (850, 640, 850, 750),
                (805, 640, 805, 450),
                (850, 750, 875, 750),
            ],
            OBJ_DEFAULT_COLOR,
        )?;
        self.draw_static_str("busR", Rect::new(410, 645, 50, 25), OBJ_DEFAULT_COLOR)?;
        // busL
        self.draw_lines(
            &[
                (400, 760, 600, 760),
                (600, 760, 790, 550),
                (790, 550, 790, 350),
                (790, 350, 825, 350),
                (825, 350, 825, 325),
            ],
            OBJ_DEFAULT_COLOR,
        )?;
        self.draw_static_str("busL", Rect::new(410, 760, 50, 25), OBJ_DEFAULT_COLOR)?;

        // Now SRC.
        self.draw_rect(Rect::new(475, 500, 50, 300), OBJ_DEFAULT_COLOR)?;
        self.draw_static_str("SRC", Rect::new(450, 800, 100, 50), OBJ_DEFAULT_COLOR)?;

        // Now NXTPC.
        self.draw_rect(Rect::new(1075, 500, 50, 300), OBJ_DEFAULT_COLOR)?;
        self.draw_static_str("NXTPC", Rect::new(1050, 800, 100, 50), OBJ_DEFAULT_COLOR)?;
        // Now PC.
        self.draw_rect(Rect::new(1175, 500, 50, 300), OBJ_DEFAULT_COLOR)?;
        self.draw_static_str("PC", Rect::new(1175, 800, 50, 50), OBJ_DEFAULT_COLOR)?;

        // Now LSTPC.
        self.draw_rect(Rect::new(1275, 500, 50, 300), OBJ_DEFAULT_COLOR)?;
        self.draw_static_str("LSTPC", Rect::new(1250, 800, 100, 50), OBJ_DEFAULT_COLOR)?;

        // RD
        self.draw_rect(Rect::new(100, 75, 100, 50), OBJ_DEFAULT_COLOR)?;
        self.draw_static_str("RD", Rect::new(125, 125, 50, 50), OBJ_DEFAULT_COLOR)?;
        // busext to RD
        self.pane.canvas.line(150, 50, 150, 75, OBJ_DEFAULT_COLOR)?;
        self.draw_string(
            &format!("R{:02}", dp.decode_rd()),
            Rect::new(125, 75, 50, 50),
            OBJ_DEFAULT_COLOR,
        )?;

        // Source register latches
        let (rs1, rs2) = dp.decode_source_registers();
        // RS1
        self.draw_rect(Rect::new(50, 200, 100, 50), OBJ_DEFAULT_COLOR)?;
        self.draw_static_str("RS1", Rect::new(75, 250, 50, 50), OBJ_DEFAULT_COLOR)?;
        self.draw_string(
            &format!("R{:02}", rs1),
            Rect::new(75, 200, 50, 50),
            OBJ_DEFAULT_COLOR,
        )?;
        // busext to RS1
        self.draw_line((75, 50, 75, 200), OBJ_DEFAULT_COLOR)?;
        // RD to RS1
        self.draw_line((110, 125, 110, 200), OBJ_DEFAULT_COLOR)?;
        // RS2 to Register file
        self.draw_line((125, 250, 125, 400), OBJ_DEFAULT_COLOR)?;
        // RS2
        self.draw_rect(Rect::new(175, 200, 100, 50), OBJ_DEFAULT_COLOR)?;
        self.draw_static_str("RS2", Rect::new(200, 250, 50, 50), OBJ_DEFAULT_COLOR)?;
        self.draw_string(
            &format!("R{:02}", rs2),
            Rect::new(200, 200, 50, 50),
            OBJ_DEFAULT_COLOR,
        )?;
        // busext to RS2
        self.draw_line((250, 50, 250, 200), OBJ_DEFAULT_COLOR)?;
        // RD to RS2
        self.draw_line((190, 125, 190, 200), OBJ_DEFAULT_COLOR)?;
        // RS2 to Register file
        self.draw_line((190, 250, 190, 400), OBJ_DEFAULT_COLOR)?;

        // PSW register
        self.draw_rect(Rect::new(300, 200, 125, 75), OBJ_DEFAULT_COLOR)?;
        self.draw_static_str("PSW", Rect::new(325, 275, 75, 50), OBJ_DEFAULT_COLOR)?;
        self.draw_string(
            &format!("{}", dp.psw()),
            Rect::new(325, 225, 75, 50),
            OBJ_DEFAULT_COLOR,
        )?;
        // busB to PSW and SHam
        self.draw_lines(
            &[(310, 700, 310, 275), (310, 325, 500, 325)],
            OBJ_DEFAULT_COLOR,
        )?;
        // PSW to register file
        self.draw_line((300, 250, 290, 250), OBJ_DEFAULT_COLOR)?;
        self.draw_line((290, 250, 290, 475), OBJ_DEFAULT_COLOR)?;
        self.draw_line((290, 475, 250, 475), OBJ_DEFAULT_COLOR)?;
        // imm
        self.draw_rect(Rect::new(800, 100, 100, 50), OBJ_DEFAULT_COLOR)?;
        self.draw_static_str("IMM", Rect::new(910, 100, 75, 50), OBJ_DEFAULT_COLOR)?;
        self.draw_string(
            &format!("{:05x}", dp.imm()),
            Rect::new(810, 100, 75, 50),
            OBJ_DEFAULT_COLOR,
        )?;
        // busEXT to imm
        self.draw_line((825, 50, 825, 100), OBJ_DEFAULT_COLOR)?;
        // dimm
        self.draw_rect(Rect::new(800, 250, 250, 75), OBJ_DEFAULT_COLOR)?;
        self.draw_static_str("DIn/DIMM", Rect::new(900, 325, 150, 50), OBJ_DEFAULT_COLOR)?;
        self.draw_string(
            &format!("{:08x}", dp.imm()),
            Rect::new(800, 255, 250, 50),
            OBJ_DEFAULT_COLOR,
        )?;
        // busEXT to dimm
        self.draw_line((1000, 50, 1000, 250), OBJ_DEFAULT_COLOR)?;
        // imm to dimm and SHAM
        self.draw_lines(
            &[
                (825, 150, 825, 250),
                (825, 175, 475, 175),
                (475, 175, 475, 315),
                (475, 315, 500, 315),
            ],
            OBJ_DEFAULT_COLOR,
        )?;
        // op
        self.draw_rect(Rect::new(1100, 125, 50, 50), OBJ_DEFAULT_COLOR)?;
        self.draw_static_str("OP", Rect::new(1100, 175, 50, 50), OBJ_DEFAULT_COLOR)?;
        self.draw_string(
            &format!("{:02x}", dp.execute_op()),
            Rect::new(1100, 125, 50, 50),
            OBJ_DEFAULT_COLOR,
        )?;
        // busext to op
        self.draw_line((1125, 50, 1125, 125), OBJ_DEFAULT_COLOR)?;
        // Shifter
        self.draw_rect(Rect::new(600, 500, 175, 300), OBJ_DEFAULT_COLOR)?;
        self.draw_static_str("Shifter", Rect::new(600, 800, 100, 50), OBJ_DEFAULT_COLOR)?;

        self.draw_circle((690, 650, 50), OBJ_DEFAULT_COLOR)?;
        // ALU
        self.draw_polygon(
            &[900, 1000, 1000, 900, 900, 930, 900, 900],
            &[500, 520, 780, 800, 670, 650, 630, 500],
            OBJ_DEFAULT_COLOR,
        )?;
        self.draw_static_str("ALU", Rect::new(900, 450, 75, 50), OBJ_DEFAULT_COLOR)?;
        // AI (ALU input latch)
        self.draw_rect(Rect::new(875, 500, 25, 120), OBJ_DEFAULT_COLOR)?;
        self.draw_static_str("AI", Rect::new(825, 450, 50, 50), OBJ_DEFAULT_COLOR)?;
        // BI (ALU input latch)
        self.draw_rect(Rect::new(875, 680, 25, 120), OBJ_DEFAULT_COLOR)?;

        self.draw_static_str("BI", Rect::new(825, 800, 50, 50), OBJ_DEFAULT_COLOR)?;
        // BAR
        self.draw_rect(Rect::new(800, 400, 50, 50), OBJ_DEFAULT_COLOR)?;
        self.draw_static_str("BAR", Rect::new(855, 400, 75, 50), OBJ_DEFAULT_COLOR)?;
        self.draw_string(
            &format!("{:02x}", dp.bar()),
            Rect::new(800, 400, 50, 50),
            OBJ_DEFAULT_COLOR,
        )?;
        // Bar to SHam
        self.draw_lines(
            &[
                (810, 400, 810, 380),
                (810, 380, 475, 380),
                (475, 380, 475, 340),
                (475, 340, 500, 340),
            ],
            OBJ_DEFAULT_COLOR,
        )?;
        // Busout
        self.draw_lines(
            &[
                (1050, 750, 1450, 750),
                (1450, 750, 1450, 50),
                (1050, 525, 1050, 750),
            ],
            OBJ_DEFAULT_COLOR,
        )?;
        self.draw_static_str("busOUT", Rect::new(1400, 750, 100, 50), OBJ_DEFAULT_COLOR)?;
        // PADS (pins in/out)
        self.draw_rect(Rect::new(1300, 25, 100, 100), OBJ_DEFAULT_COLOR)?;
        self.draw_static_str("PADS", Rect::new(1300, 125, 100, 50), OBJ_DEFAULT_COLOR)?;

        // SDEC and SHAM
        self.draw_rects(
            &[Rect::new(550, 300, 150, 50), Rect::new(500, 300, 50, 50)],
            OBJ_DEFAULT_COLOR,
        )?;
        self.draw_static_str("SHam", Rect::new(500, 250, 75, 50), OBJ_DEFAULT_COLOR)?;
        self.draw_static_str("SDec", Rect::new(650, 250, 75, 50), OBJ_DEFAULT_COLOR)?;
        self.draw_string(
            &format!("{:02x}", dp.shifter().s_ham),
            Rect::new(500, 300, 50, 50),
            OBJ_DEFAULT_COLOR,
        )?;
        self.draw_string(
            &format!("{:02x}", dp.shifter().s_dec),
            Rect::new(600, 300, 50, 50),
            OBJ_DEFAULT_COLOR,
        )?;
        // Connect SDec to Shifter
        self.draw_line((600, 350, 700, 600), OBJ_DEFAULT_COLOR)?;
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

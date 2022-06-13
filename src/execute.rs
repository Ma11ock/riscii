// Instruction execution. The second step in the three step RISC II pipeline.
// See `decode.rs` for the first step, and `commit.rs` for the third step.
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

use instruction::*;
use system::System;

// Public functions.

pub fn execute(instruction: &Instruction, system: &System) -> Result<(), String> {
    type I = Instruction;
    match *instruction {
        I::Calli(si) => {}
        I::GetPSW(o) => format!("GetPSW {}", o),
        I::GetIPC(o) => format!("GetIPC {}", o),
        I::PutPSW(o) => format!("GetPSW {}", o),
        I::Callx(o) => format!("Callx {}", o),
        I::Callr(o) => format!("Callr {}", o),
        I::Jmpx(o) => format!("Jmpx {}", o),
        I::Jmpr(o) => format!("Jmpr {}", o),
        I::Ret(o) => format!("Ret {}", o),
        I::Reti(o) => format!("Reti {}", o),
        I::Sll(o) => format!("Sll {}", o),
        I::Srl(o) => format!("Srl {}", o),
        I::Sra(o) => format!("Sra {}", o),
        I::Or(o) => format!("Or {}", o),
        I::And(o) => format!("And {}", o),
        I::Xor(o) => format!("Xor {}", o),
        I::Add(o) => format!("Add {}", o),
        I::Addc(o) => format!("Addc {}", o),
        I::Sub(o) => format!("Sub {}", o),
        I::Subc(o) => format!("Subc {}", o),
        I::Subi(o) => format!("Subi {}", o),
        I::Subci(o) => format!("Subci {}", o),
        I::Ldhi(o) => format!("Ldhi {}", o),
        I::Ldxw(o) => format!("Ldxw {}", o),
        I::Ldrw(o) => format!("Ldrw {}", o),
        I::Ldxhs(o) => format!("Ldxhs {}", o),
        I::Ldrhs(o) => format!("Ldrhs {}", o),
        I::Ldxhu(o) => format!("Ldxhu {}", o),
        I::Ldrhu(o) => format!("Ldrhu {}", o),
        I::Ldxbs(o) => format!("Ldxbs {}", o),
        I::Ldrbs(o) => format!("Ldrbs {}", o),
        I::Ldxbu(o) => format!("Ldxbu {}", o),
        I::Ldrbu(o) => format!("Ldxbu {}", o),
        I::Stxw(o) => format!("Stxw {}", o),
        I::Strw(o) => format!("Strw {}", o),
        I::Stxh(o) => format!("Stxh {}", o),
        I::Strh(o) => format!("Strh {}", o),
        I::Stxb(o) => format!("Stxb {}", o),
        I::Strb(o) => format!("Strb {}", o),
    }

    Ok(())
}

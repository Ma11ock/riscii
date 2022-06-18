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

use cpu::ProcessorStatusWord;
use instruction::*;
use system::System;
use util::U32_MSB;

// Public structs.

pub struct ExecResult {
    psw: ProcessorStatusWord,
    regs: RegisterFile,
    was_branch: bool,
    psw_delayed: bool,
}

// Public functions.

// TODO timing and memory reads/writes. Need to emulate the pipeline and cpu clock.
pub fn execute(instruction: &Instruction, system: &mut System) -> Result<ExecResult> {
    type I = Instruction;

    let mut result = ExecResult::from_system(system);
    let mut register_file = result.get_register_file();
    let cur_pc = register_file.get_pc();
    let cur_psw = system.get_psw();
    let mut memory = system.get_mem_ref();

    match *instruction {
        I::Calli {
            scc: scc,
            dest: dest,
            rs1: _,
            short_source: _,
        } => {
            if !system.is_system_mode {
                // TODO error
            }
            system.call();
            let lstpc = system.get_last_pc();
            if scc {
                system.set_cc_zero(lstpc == 0);
                system.set_cc_neg(lstpc & U32_MSB != 0);
            }
            register_file.rus(dest, lstpc)?;
            // TODO maybe handle interrupts.
        }
        I::GetPSW {
            scc: scc,
            dest: dest,
            rs1: _,
            short_source: ss,
        } => {
            let psw = cur_psw & 0xffff7;
            register_file.rus(dest, psw)?;
            if scc {
                let dest_val = register_file.ru(dest)?;
                self.set_cc_neg(dest_val & U32_MSB != 1);
                self.set_cc_zero(dest_val == 0);
                self.set_cc_carry(false);
                self.set_cc_overflow(false);
            }
        }
        I::GetLPC {
            scc: scc,
            dest: dest,
            rs1: _,
            short_source: _,
        } => {
            if !system.is_system_mode {
                // TODO error
            }
            let lstpc = system.get_last_pc();
            register_file.rus(dest, lstpc)?;
            if scc {
                system.set_cc_zero(lstpc == 0);
                system.set_cc_neg(lstpc & U32_MSB != 0);
            }
        }
        I::PutPSW {
            scc: scc,
            dest: _,
            rs1: rs1,
            short_source: ss,
        } => {
            if !system.is_system_mode {
                // TODO error
            }
            if scc {
                // TODO error
            }

            let val = register_file.get_ss_val(ss, cur_psw)?;
            result.set_psw(register_file.ru(rs1)? + val);
        }
        I::Callx {
            scc: _,
            dest: dest,
            rs1: rs1,
            short_source: ss,
        } => {
            // TODO test alignment (addr[0] == 1).
            let rs_val = register_file.ru(rs1)?;
            let addr = register_file.get_ss_val(ss, cur_psw)? + rs_val;
            register_file.push_reg_window();
            register_file.branch_to(addr);
            result.rus(cur_pc)?;
        }
        I::Callr {
            scc: _,
            dest: dest,
            imm19: imm19,
        } => {
            // TODO test alignment (addr[0] == 1).
            result.set_branch(true);
            register_file.push_reg_window();
            register_file.branch_to(cur_pc + imm19);
        }
        I::Jmpx {
            scc: _,
            dest: cond,
            rs1: rs1,
            short_source: ss,
        } => {
            // TODO test alignment (addr[0] == 1).
            if exec_conditional(cond, result.get_psw()) {
                result.set_branch(true);
                let rs_val = register_file.ru(rs1)?;
                let addr = register_file.get_ss_val(ss, cur_psw)? + rs_val;
                register_file.branch_to(addr);
            }
        }
        I::Jmpr {
            scc: _,
            dest: cond,
            imm19: imm19,
        } => {
            if exec_conditional(cond, result.get_psw()) {
                result.set_branch(true);
                register_file.branch_to(cur_pc + imm19);
            }
        }
        I::Ret {
            scc: _,
            dest: cond,
            rs1: rs1,
            short_source: ss,
        } => {
            if exec_conditional(cond, result.get_psw()) {
                result.set_branch(true);
                let rs_val = register_file.ru(rs1)?;
                register_file.branch_to(rs_val + (SIZEOF_INSTRUCTION * 2));
                register_file.pop_reg_window();
            }
        }
        I::Reti {
            scc: _,
            dest: cond,
            rs1: rs1,
            short_source: ss,
        } => {
            if !system.is_system_mode {
                // TODO error
            }
            if exec_conditional(cond, result.get_psw()) {
                result.set_branch(true);
                let rs_val = register_file.ru(rs1)?;
                register_file.branch_to(rs_val + (SIZEOF_INSTRUCTION * 2));
                register_file.pop_reg_window();
            }
        }
        I::Sll {
            scc: scc,
            dest: dest,
            rs1: rs1,
            short_source: ss,
        } => {
            let s1_val = register_file.ru(rs1)?;
            let s2_val = register_file.get_ss_val(ss, cur_psw)?;
            let d = register_file.rus(dest, s1_val << s2_val)?;
            if scc {
                set_shift_cc(result.get_psw_ref(), d);
            }
        }
        I::Srl {
            scc: scc,
            dest: dest,
            rs1: rs1,
            short_source: ss,
        } => {
            let s1_val = register_file.ru(rs1)?;
            let s2_val = register_file.get_ss_val(ss, cur_psw)?;
            let d = register_file.rus(dest, s1_val >> s2_val)?;
            if scc {
                set_shift_cc(scc, result.get_psw_ref(), d);
            }
        }
        I::Sra {
            scc: scc,
            dest: dest,
            rs1: rs1,
            short_source: ss,
        } => {
            let s1_val = register_file.ru(rs1)?;
            let s2_val = register_file.get_ss_val(ss, cur_psw)?;
            let d = register_file.rus(dest, s1_val as i32 >> s2_val)?;
            if scc {
                set_shift_cc(result.get_psw_ref(), d);
            }
        }
        I::Or {
            scc: scc,
            dest: dest,
            rs1: rs1,
            short_source: ss,
        } => {
            let s1_val = register_file.ru(rs1)?;
            let s2_val = register_file.get_ss_val(ss, cur_psw)?;
            let d = register_file.rus(dest, s1_val | s2_val)?;
            if scc {
                set_shift_cc(result.get_psw_ref(), d);
            }
        }
        I::And {
            scc: scc,
            dest: dest,
            rs1: rs1,
            short_source: ss,
        } => {
            let s1_val = register_file.ru(rs1)?;
            let s2_val = register_file.get_ss_val(ss, cur_psw)?;
            let d = register_file.rus(dest, s1_val & s2_val)?;
            if scc {
                set_shift_cc(result.get_psw_ref(), d);
            }
        }
        I::Xor {
            scc: scc,
            dest: dest,
            rs1: rs1,
            short_source: ss,
        } => {
            let s1_val = register_file.ru(rs1)?;
            let s2_val = register_file.get_ss_val(ss, cur_psw)?;
            let d = register_file.rus(dest, s1_val ^ s2_val)?;
            if scc {
                set_shift_cc(result.get_psw_ref(), d);
            }
        }
        I::Add {
            scc: scc,
            dest: dest,
            rs1: rs1,
            short_source: ss,
        } => {
            let s1_val = register_file.ru(rs1)?;
            let s2_val = register_file.get_ss_val(ss, cur_psw)?;
            let (res, o) = s1_val.overflowing_add(s2_val);
            let d = register_file.rus(dest, res)?;
            if scc {
                let mut psw = result.get_psw_ref();
                set_operator_cc(psw, d);
                psw.set_cc_overflow(o);
                psw.set_cc_carry(o);
            }
        }
        I::Addc {
            scc: scc,
            dest: dest,
            rs1: rs1,
            short_source: ss,
        } => {
            let s1_val = register_file.ru(rs1)?;
            let s2_val = register_file.get_ss_val(ss, cur_psw)?;
            let mut psw = result.get_psw_ref();
            let (r1, o1) = s1_val.overflowing_add(s2_val);
            let (res, o2) = r1.overflowing_add(psw.get_cc_carry() as u32);
            let o = o1 || o2;
            let d = register_file.rus(dest, res)?;
            if scc {
                set_operator_cc(psw, d);
                psw.set_cc_overflow(o);
                psw.set_cc_carry(o);
            }
        }
        I::Sub {
            scc: scc,
            dest: dest,
            rs1: rs1,
            short_source: ss,
        } => {
            let s1_val = register_file.ru(rs1)?;
            let s2_val = register_file.get_ss_val(ss, cur_psw)?;
            let (res, o) = s1_val.overflowing_sub(s2_val);
            let d = register_file.rus(dest, res)?;
            if scc {
                let mut psw = result.get_psw_ref();
                set_operator_cc(psw, d);
                psw.set_cc_overflow(o);
                psw.set_cc_carry(!o);
            }
        }
        I::Subc {
            scc: scc,
            dest: dest,
            rs1: rs1,
            short_source: ss,
        } => {
            let s1_val = register_file.ru(rs1)?;
            let s2_val = register_file.get_ss_val(ss, cur_psw)?;
            let mut psw = result.get_psw_ref();
            let (r1, o1) = s1_val.overflowing_sub(s2_val);
            let (res, o2) = r1.overflowing_sub(!psw.get_cc_carry() as u32);
            let o = o2 || o1;
            let d = register_file.rus(dest, res);
            if scc {
                let mut psw = result.get_psw_ref();
                set_operator_cc(psw, d);
                psw.set_cc_overflow(o);
                psw.set_cc_carry(!o);
            }
        }
        I::Subi {
            scc: scc,
            dest: dest,
            rs1: rs1,
            short_source: ss,
        } => {
            let s1_val = register_file.ru(rs1)?;
            let s2_val = register_file.get_ss_val(ss, cur_psw)?;
            let (res, o) = s2_val.overflowing_sub(s1_val);
            let d = register_file.rus(dest, res)?;
            if scc {
                let mut psw = result.get_psw_ref();
                set_operator_cc(psw, d);
                let v = d > s2_val;
                psw.set_cc_overflow(v);
                psw.set_cc_carry(!v);
            }
        }
        I::Subci {
            scc: scc,
            dest: dest,
            rs1: rs1,
            short_source: ss,
        } => {
            let s1_val = register_file.ru(rs1)?;
            let s2_val = register_file.get_ss_val(ss, cur_psw)?;
            let mut psw = result.get_psw_ref();
            let (r1, o1) = s2_val.overflowing_sub(s1_val);
            let (res, o2) = r1.overflowing_sub(!psw.get_cc_carry() as u32);
            let o = o1 || o2;
            let d = register_file.rus(dest, res)?;
            if scc {
                set_operator_cc(psw, d);
                psw.set_cc_overflow(o);
                psw.set_cc_carry(!o);
            }
        }
        I::Ldhi {
            scc: scc,
            dest: dest,
            imm19: imm19,
        } => {
            // TODO Test alignment
            let cur_d = register_file.ru(dest)?;
            let d = register_file.rus(dest, dest & ((imm19 << 13) & 0x1fff))?;
            if scc {
                set_load_cc(result.get_psw_ref(), d);
            }
        }
        I::Ldxw {
            scc: scc,
            dest: dest,
            rs1: rs1,
            short_source: ss,
        } => {
            // TODO Test alignment
            let ss_val = register_file.get_ss_val(ss, cur_psw)?;
            let d = register_file.rus(dest, memory.get_word(ss_val)?)?;
            if scc {
                set_load_cc(result.get_psw_ref(), d);
            }
        }
        I::Ldrw {
            scc: scc,
            dest: dest,
            imm19: imm19,
        } => {
            let addr = imm19 + regs.get_pc();
            let d = register_file.rus(dest, memory.get_word(ss_val)?)?;
            if scc {
                set_load_cc(result.get_psw_ref(), d);
            }
        }
        I::Ldxhs {
            scc: scc,
            dest: dest,
            rs1: rs1,
            short_source: ss,
        } => {
            let ss_val = register_file.get_ss_val(ss, cur_psw)?;
            let d = register_file.rus(dest, memory.get_hword(ss_val)? as i32 as u32)?;
            if scc {
                set_load_cc(result.get_psw_ref(), d);
            }
        }
        I::Ldrhs {
            scc: scc,
            dest: dest,
            imm19: imm19,
        } => {
            let addr = imm19 + regs.get_pc();
            let d = register_file.rus(dest, memory.get_hword(ss_val)? as i32 as u32)?;
            if scc {
                set_load_cc(result.get_psw_ref(), d);
            }
        }
        I::Ldxhu {
            scc: scc,
            dest: dest,
            rs1: rs1,
            short_source: ss,
        } => {
            let ss_val = register_file.get_ss_val(ss, cur_psw)?;
            let d = register_file.rus(dest, memory.get_hword(ss_val)? as u32)?;
            if scc {
                set_load_cc(result.get_psw_ref(), d);
            }
        }
        I::Ldrhu {
            scc: scc,
            dest: dest,
            imm19: imm19,
        } => {
            let addr = imm19 + regs.get_pc();
            let d = register_file.rus(dest, memory.get_hword(ss_val)? as u32)?;
            if scc {
                set_load_cc(result.get_psw_ref(), d);
            }
        }
        I::Ldxbs {
            scc: scc,
            dest: dest,
            rs1: rs1,
            short_source: ss,
        } => {
            let ss_val = register_file.get_ss_val(ss, cur_psw)?;
            let d = register_file.rus(dest, memory.get_byte(ss_val)? as i32 as u32)?;
            if scc {
                set_load_cc(result.get_psw_ref(), d);
            }
        }
        I::Ldrbs {
            scc: scc,
            dest: dest,
            imm19: imm19,
        } => {
            let addr = imm19 + regs.get_pc();
            let d = register_file.rus(dest, memory.get_byte(ss_val)? as i32 as u32)?;
            if scc {
                set_load_cc(result.get_psw_ref(), d);
            }
        }
        I::Ldxbu {
            scc: scc,
            dest: dest,
            rs1: rs1,
            short_source: ss,
        } => {
            let ss_val = register_file.get_ss_val(ss, cur_psw)?;
            let d = register_file.rus(dest, memory.get_byte(ss_val)? as u32)?;
            if scc {
                set_load_cc(result.get_psw_ref(), d);
            }
        }
        I::Ldrbu {
            scc: scc,
            dest: dest,
            imm19: imm19,
        } => {
            let addr = imm19 + regs.get_pc();
            let d = register_file.rus(dest, memory.get_byte(ss_val)? as u32)?;
            if scc {
                set_load_cc(result.get_psw_ref(), d);
            }
        }
        I::Stxw {
            scc: scc,
            dest: dest,
            rs1: rs1,
            short_source: ss,
        } => {
            if short_source == ShortSource::Reg(_) {
                // warn
                // return Err("Store instructions should be immediate only (not registers)");
            }
            let ss_val = register_file.get_ss_val(ss, cur_psw)?;
            let rs1_val = register_file.ru(rs1);
            let dest_val = register_file.ru(dest);
            memory.set_word(ss_val + rs1_val, dest_val);
            if scc {
                set_store_cc(result.get_psw_ref());
            }
        }
        I::Strw {
            scc: scc,
            dest: dest,
            imm19: imm19,
        } => {
            if short_source == ShortSource::Reg(_) {
                // warn
                // return Err("Store instructions should be immediate only (not registers)");
            }
            let dest_val = register_file.ru(dest);
            memory.set_word(register_file.get_pc() + imm19, dest_val);
            if scc {
                set_store_cc(result.get_psw_ref());
            }
        }
        I::Stxh {
            scc: scc,
            dest: dest,
            rs1: rs1,
            short_source: ss,
        } => {
            if short_source == ShortSource::Reg(_) {
                // warn
                // return Err("Store instructions should be immediate only (not registers)");
            }
            let ss_val = register_file.get_ss_val(ss, cur_psw)?;
            let rs1_val = register_file.ru(rs1);
            let dest_val = register_file.ru(dest);
            memory.set_hword(ss_val + rs1_val, dest_val as u16);
            if scc {
                set_store_cc(result.get_psw_ref());
            }
        }
        I::Strh {
            scc: scc,
            dest: dest,
            imm19: imm19,
        } => {
            if short_source == ShortSource::Reg(_) {
                // warn
                // return Err("Store instructions should be immediate only (not registers)");
            }
            let dest_val = register_file.ru(dest);
            memory.set_hword(register_file.get_pc() + imm19, dest_val as u16);
            if scc {
                set_store_cc(result.get_psw_ref());
            }
        }
        I::Stxb {
            scc: scc,
            dest: dest,
            rs1: rs1,
            short_source: ss,
        } => {
            if short_source == ShortSource::Reg(_) {
                // warn
                // return Err("Store instructions should be immediate only (not registers)");
            }
            let ss_val = register_file.get_ss_val(ss, cur_psw)?;
            let rs1_val = register_file.ru(rs1);
            let dest_val = register_file.ru(dest);
            memory.set_byte(ss_val + rs1_val, dest_val as u8);
            if scc {
                set_store_cc(result.get_psw_ref());
            }
        }
        I::Strb {
            scc: scc,
            dest: dest,
            imm19: imm19,
        } => {
            if short_source == ShortSource::Reg(_) {
                // warn
                // return Err("Store instructions should be immediate only (not registers)");
            }
            let dest_val = register_file.ru(dest);
            memory.set_byte(register_file.get_pc() + imm19, dest_val as u8);
            if scc {
                set_store_cc(result.get_psw_ref());
            }
        }
    }

    Ok(result)
}

// Struct impls.

impl ExecResult {
    pub fn from_system(system: &System) -> Self {
        Self {
            psw: system.get_psw(),
            register_file: system.copy_register_file(),
            was_branch: false,
            psw_delayed: false,
        }
    }

    pub fn set_psw(&mut self, psw: u32) {
        self.psw.from_u32(psw);
    }

    pub fn get_psw_ref(&mut self) -> &mut ProcessorStatusWord {
        &mut self.psw
    }

    pub fn get_register_file(&mut self) -> &mut RegisterFile {
        &mut self.regs
    }

    pub fn was_branch(&self) -> bool {
        self.was_branch
    }

    pub fn set_branch(&mut self, v: bool) {
        self.was_branch = v;
    }
}

// Private functions.

fn exec_conditional(what: Conditional, psw: ProcessorStatusWord) -> bool {
    todo!()
}

fn set_operator_cc(psw: &mut ProcessorStatusWord, dest_val: u32) {
    psw.set_cc_zero(register_file.ru(dest)? == 0);
    psw.set_cc_neg(register_file.ru(dest)? & U32_MSB != 0);
}

fn set_shift_cc(psw: &mut ProcessorStatusWord, dest_val: u32) {
    set_operator_cc(psw, dest_val);
    psw.set_cc_overflow(false);
    psw.set_cc_carry(false);
}

fn set_load_cc(psw: &mut ProcessorStatusWord, dest_val: u32) {
    psw.set_cc_carry(false);
    psw.set_cc_overflow(false);
    psw.set_cc_zero(d == 0);
    psw.set_cc_neg(d & U32_MSB != 0);
}

fn set_store_cc(psw: &mut ProcessorStatusWord) {
    psw.set_cc_overflow(false);
    psw.set_cc_carry(false);
}

// Test code for the RISC II encoder.
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

#[cfg(test)]
#[path = "encode.rs"]
mod test {
    extern crate assert_hex;

    use super::super::*;
    use assert_hex::*;
    use decode::*;
    use instruction::*;
    use std::fmt;
    use util::Result;

    type I = Instruction;
    type SS = ShortSource;
    type SI = ShortInstruction;

    #[test]
    fn encode_and_noop_imm() -> Result<()> {
        assert_eq_hex!(
            I::And(SI::new(false, 0, 0, SS::Imm13(0))).encode(),
            0x2a002000
        );
        Ok(())
    }

    #[test]
    fn encode_and_noop_register() -> Result<()> {
        assert_eq_hex!(
            I::And(SI::new(false, 0, 0, SS::Reg(0))).encode(),
            0x2a000000
        );
        Ok(())
    }

    #[test]
    fn encode_calli() -> Result<()> {
        assert_eq_hex!(
            I::Calli(SI::new(true, 5, 7, SS::Imm13(4111))).encode(),
            0x0329f00f
        );
        Ok(())
    }

    #[test]
    fn encode_getpsw() -> Result<()> {
        assert_eq_hex!(
            0x05293fff,
            I::GetPSW(ShortInstruction::new(true, 5, 4, SS::Imm13(0x1fff))).encode()
        );
        Ok(())
    }

    #[test]
    fn encode_getipc() -> Result<()> {
        assert_eq_hex!(
            0x07293f69,
            I::GetLPC(ShortInstruction::new(true, 5, 4, SS::Imm13(0x1f69))).encode()
        );
        Ok(())
    }

    #[test]
    fn encode_putpsw() -> Result<()> {
        assert_eq_hex!(
            0x09293f69,
            I::PutPSW(ShortInstruction::new(true, 5, 4, SS::Imm13(0x1f69))).encode()
        );
        Ok(())
    }

    // (unpriveleged) Call/jump/ret instructions.

    #[test]
    fn encode_callx() -> Result<()> {
        assert_eq_hex!(
            0x11293f69,
            I::Callx(ShortInstruction::new(true, 5, 4, SS::Imm13(0x1f69))).encode()
        );
        Ok(())
    }

    #[test]
    fn encode_callr() -> Result<()> {
        assert_eq_hex!(
            0x132b3420,
            I::Callr(LongInstruction::new(true, 5, 0x33420)).encode()
        );
        Ok(())
    }

    #[test]
    fn encode_jmpx() -> Result<()> {
        assert_eq_hex!(
            0x19293f69,
            I::Jmpx(ShortConditional::new(
                true,
                Conditional::Hi,
                4,
                SS::Imm13(0x1f69)
            ))
            .encode()
        );
        Ok(())
    }

    #[test]
    fn encode_jmpr() -> Result<()> {
        assert_eq_hex!(
            0x1bfb3420,
            I::Jmpr(LongConditional::new(true, Conditional::Alw, 0x33420)).encode()
        );
        Ok(())
    }

    #[test]
    fn encode_ret() -> Result<()> {
        assert_eq_hex!(
            0x1d293f69,
            I::Ret(ShortConditional::new(
                true,
                Conditional::Hi,
                4,
                SS::Imm13(0x1f69)
            ))
            .encode()
        );
        Ok(())
    }

    #[test]
    fn encode_reti() -> Result<()> {
        assert_eq_hex!(
            0x1f293f69,
            I::Reti(ShortConditional::new(
                true,
                Conditional::Hi,
                4,
                SS::Imm13(0x1f69)
            ))
            .encode()
        );
        Ok(())
    }

    // Arithmetic and logic instructions (except ldhi).

    #[test]
    fn encode_sll() -> Result<()> {
        assert_eq_hex!(
            0x23293f69,
            I::Sll(ShortInstruction::new(true, 5, 4, SS::Imm13(0x1f69))).encode()
        );
        Ok(())
    }

    #[test]
    fn encode_sra() -> Result<()> {
        assert_eq_hex!(
            0x25293f69,
            I::Sra(ShortInstruction::new(true, 5, 4, SS::Imm13(0x1f69))).encode()
        );
        Ok(())
    }

    #[test]
    fn encode_srl() -> Result<()> {
        assert_eq_hex!(
            0x27293f69,
            I::Srl(ShortInstruction::new(true, 5, 4, SS::Imm13(0x1f69))).encode()
        );
        Ok(())
    }

    #[test]
    fn encode_ldhi() -> Result<()> {
        assert_eq_hex!(
            0x292b3f69,
            I::Ldhi(LongInstruction::new(true, 5, 0x33f69)).encode()
        );
        Ok(())
    }

    #[test]
    fn encode_and() -> Result<()> {
        assert_eq_hex!(
            0x2b293f69,
            I::And(ShortInstruction::new(true, 5, 4, SS::Imm13(0x1f69))).encode()
        );
        Ok(())
    }

    #[test]
    fn encode_or() -> Result<()> {
        assert_eq_hex!(
            0x2d293f69,
            I::Or(ShortInstruction::new(true, 5, 4, SS::Imm13(0x1f69))).encode()
        );
        Ok(())
    }

    #[test]
    fn encode_xor() -> Result<()> {
        assert_eq_hex!(
            0x2f293f69,
            I::Xor(ShortInstruction::new(true, 5, 4, SS::Imm13(0x1f69))).encode()
        );
        Ok(())
    }

    #[test]
    fn encode_add() -> Result<()> {
        assert_eq_hex!(
            0x31293f69,
            I::Add(ShortInstruction::new(true, 5, 4, SS::Imm13(0x1f69))).encode()
        );
        Ok(())
    }

    #[test]
    fn encode_addc() -> Result<()> {
        assert_eq_hex!(
            0x33293f69,
            I::Addc(ShortInstruction::new(true, 5, 4, SS::Imm13(0x1f69))).encode()
        );
        Ok(())
    }

    #[test]
    fn encode_sub() -> Result<()> {
        assert_eq_hex!(
            0x39293f69,
            I::Sub(ShortInstruction::new(true, 5, 4, SS::Imm13(0x1f69))).encode()
        );
        Ok(())
    }

    #[test]
    fn encode_subc() -> Result<()> {
        assert_eq_hex!(
            0x3b293f69,
            I::Subc(ShortInstruction::new(true, 5, 4, SS::Imm13(0x1f69))).encode()
        );
        Ok(())
    }

    #[test]
    fn encode_subi() -> Result<()> {
        assert_eq_hex!(
            0x3d293f69,
            I::Subi(ShortInstruction::new(true, 5, 4, SS::Imm13(0x1f69))).encode()
        );
        Ok(())
    }

    #[test]
    fn encode_subci() -> Result<()> {
        assert_eq_hex!(
            0x3f293f69,
            I::Subci(ShortInstruction::new(true, 5, 4, SS::Imm13(0x1f69))).encode()
        );
        Ok(())
    }

    // Load instructions.

    #[test]
    fn encode_ldxw() -> Result<()> {
        assert_eq_hex!(
            0x4d293f69,
            I::Ldxw(ShortInstruction::new(true, 5, 4, SS::Imm13(0x1f69))).encode()
        );
        Ok(())
    }

    #[test]
    fn encode_ldrw() -> Result<()> {
        assert_eq_hex!(
            0x4f2b3f69,
            I::Ldrw(LongInstruction::new(true, 5, 0x33f69)).encode()
        );
        Ok(())
    }

    #[test]
    fn encode_ldxhu() -> Result<()> {
        assert_eq_hex!(
            0x51293f69,
            I::Ldxhu(ShortInstruction::new(true, 5, 4, SS::Imm13(0x1f69))).encode()
        );
        Ok(())
    }

    #[test]
    fn encode_ldrhu() -> Result<()> {
        assert_eq_hex!(
            0x532b3f69,
            I::Ldrhu(LongInstruction::new(true, 5, 0x33f69)).encode()
        );
        Ok(())
    }

    #[test]
    fn encode_ldxhs() -> Result<()> {
        assert_eq_hex!(
            0x55293f69,
            I::Ldxhs(ShortInstruction::new(true, 5, 4, SS::Imm13(0x1f69))).encode()
        );
        Ok(())
    }

    #[test]
    fn encode_ldrhs() -> Result<()> {
        assert_eq_hex!(
            0x572b3f69,
            I::Ldrhs(LongInstruction::new(true, 5, 0x33f69)).encode()
        );
        Ok(())
    }

    #[test]
    fn encode_ldxbu() -> Result<()> {
        assert_eq_hex!(
            0x59293f69,
            I::Ldxbu(ShortInstruction::new(true, 5, 4, SS::Imm13(0x1f69))).encode()
        );
        Ok(())
    }

    #[test]
    fn encode_ldrbu() -> Result<()> {
        assert_eq_hex!(
            0x5b2b3f69,
            I::Ldrbu(LongInstruction::new(true, 5, 0x33f69)).encode()
        );
        Ok(())
    }

    #[test]
    fn encode_ldxbs() -> Result<()> {
        assert_eq_hex!(
            0x5d293f69,
            I::Ldxbs(ShortInstruction::new(true, 5, 4, SS::Imm13(0x1f69))).encode()
        );
        Ok(())
    }

    #[test]
    fn encode_ldrbs() -> Result<()> {
        assert_eq_hex!(
            0x5f2b3f69,
            I::Ldrbs(LongInstruction::new(true, 5, 0x33f69)).encode()
        );
        Ok(())
    }

    // Store instructions.

    #[test]
    fn encode_stxw() -> Result<()> {
        assert_eq_hex!(
            0x6d293f69,
            I::Stxw(ShortInstruction::new(true, 5, 4, SS::Imm13(0x1f69))).encode()
        );
        Ok(())
    }

    #[test]
    fn encode_strw() -> Result<()> {
        assert_eq_hex!(
            0x6f2b3f69,
            I::Strw(LongInstruction::new(true, 5, 0x33f69)).encode()
        );
        Ok(())
    }

    #[test]
    fn encode_stxh() -> Result<()> {
        assert_eq_hex!(
            0x75293f69,
            I::Stxh(ShortInstruction::new(true, 5, 4, SS::Imm13(0x1f69))).encode()
        );
        Ok(())
    }

    #[test]
    fn encode_strh() -> Result<()> {
        assert_eq_hex!(
            0x772b3f69,
            I::Strh(LongInstruction::new(true, 5, 0x33f69)).encode()
        );
        Ok(())
    }

    #[test]
    fn encode_stxb() -> Result<()> {
        assert_eq_hex!(
            0x7d293f69,
            I::Stxb(ShortInstruction::new(true, 5, 4, SS::Imm13(0x1f69))).encode()
        );
        Ok(())
    }

    #[test]
    fn encode_strb() -> Result<()> {
        assert_eq_hex!(
            0x7f2b3f69,
            I::Strb(LongInstruction::new(true, 5, 0x33f69)).encode()
        );
        Ok(())
    }
}

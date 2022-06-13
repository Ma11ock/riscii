// Test code for the RISC II decoder.
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
#[path = "decode.rs"]
mod test {
    use super::super::*;

    use decode::*;
    use instruction::*;
    use std::fmt;

    type I = Instruction;
    type SS = ShortSource;

    // Privileged instructions.

    #[test]
    fn decode_calli() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x0329f00f)?,
            I::Calli(ShortInstruction::new(true, 5, 7, SS::UImm13(4111)))
        );
        Ok(())
    }

    #[test]
    fn decode_getpsw() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x05293fff)?,
            I::GetPSW(ShortInstruction::new(true, 5, 4, SS::UImm13(0x1fff)))
        );
        Ok(())
    }

    #[test]
    fn decode_getipc() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x07293f69)?,
            I::GetIPC(ShortInstruction::new(true, 5, 4, SS::UImm13(0x1f69)))
        );
        Ok(())
    }

    #[test]
    fn decode_putpsw() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x09293f69)?,
            I::PutPSW(ShortInstruction::new(true, 5, 4, SS::UImm13(0x1f69)))
        );
        Ok(())
    }

    // (unpriveleged) Call/jump/ret instructions.

    #[test]
    fn decode_callx() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x11293f69)?,
            I::Callx(ShortInstruction::new(true, 5, 4, SS::UImm13(0x1f69)))
        );
        Ok(())
    }

    #[test]
    fn decode_callr() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x132b3420)?,
            I::Callr(LongInstruction::new(true, 5, 0x33420))
        );
        Ok(())
    }

    #[test]
    fn decode_jmpx() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x19293f69)?,
            I::Jmpx(ShortConditional::new(
                true,
                Conditional::Hi,
                4,
                SS::UImm13(0x1f69)
            ))
        );
        Ok(())
    }

    #[test]
    fn decode_jmpr() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x1bfb3420)?,
            I::Jmpr(LongConditional::new(true, Conditional::Alw, 0x33420))
        );
        Ok(())
    }

    #[test]
    fn decode_ret() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x1d293f69)?,
            I::Ret(ShortConditional::new(
                true,
                Conditional::Hi,
                4,
                SS::UImm13(0x1f69)
            ))
        );
        Ok(())
    }

    #[test]
    fn decode_reti() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x1f293f69)?,
            I::Reti(ShortConditional::new(
                true,
                Conditional::Hi,
                4,
                SS::UImm13(0x1f69)
            ))
        );
        Ok(())
    }

    // Arithmetic and logic instructions (except ldhi).

    #[test]
    fn decode_sll() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x23293f69)?,
            I::Sll(ShortInstruction::new(true, 5, 4, SS::UImm13(0x1f69)))
        );
        Ok(())
    }

    #[test]
    fn decode_sra() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x25293f69)?,
            I::Sra(ShortInstruction::new(true, 5, 4, SS::UImm13(0x1f69)))
        );
        Ok(())
    }

    #[test]
    fn decode_srl() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x27293f69)?,
            I::Srl(ShortInstruction::new(true, 5, 4, SS::UImm13(0x1f69)))
        );
        Ok(())
    }

    #[test]
    fn decode_ldhi() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x292b3f69)?,
            I::Ldhi(LongInstruction::new(true, 5, 0x33f69))
        );
        Ok(())
    }

    #[test]
    fn decode_and() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x2b293f69)?,
            I::And(ShortInstruction::new(true, 5, 4, SS::UImm13(0x1f69)))
        );
        Ok(())
    }

    #[test]
    fn decode_or() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x2d293f69)?,
            I::Or(ShortInstruction::new(true, 5, 4, SS::UImm13(0x1f69)))
        );
        Ok(())
    }

    #[test]
    fn decode_xor() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x2f293f69)?,
            I::Xor(ShortInstruction::new(true, 5, 4, SS::UImm13(0x1f69)))
        );
        Ok(())
    }

    #[test]
    fn decode_add() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x31293f69)?,
            I::Add(ShortInstruction::new(true, 5, 4, SS::UImm13(0x1f69)))
        );
        Ok(())
    }

    #[test]
    fn decode_addc() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x33293f69)?,
            I::Addc(ShortInstruction::new(true, 5, 4, SS::UImm13(0x1f69)))
        );
        Ok(())
    }

    #[test]
    fn decode_sub() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x39293f69)?,
            I::Sub(ShortInstruction::new(true, 5, 4, SS::UImm13(0x1f69)))
        );
        Ok(())
    }

    #[test]
    fn decode_subc() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x3b293f69)?,
            I::Subc(ShortInstruction::new(true, 5, 4, SS::UImm13(0x1f69)))
        );
        Ok(())
    }

    #[test]
    fn decode_subi() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x3d293f69)?,
            I::Subi(ShortInstruction::new(true, 5, 4, SS::UImm13(0x1f69)))
        );
        Ok(())
    }

    #[test]
    fn decode_subci() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x3f293f69)?,
            I::Subci(ShortInstruction::new(true, 5, 4, SS::UImm13(0x1f69)))
        );
        Ok(())
    }

    // Load instructions.

    #[test]
    fn decode_ldxw() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x4d293f69)?,
            I::Ldxw(ShortInstruction::new(true, 5, 4, SS::UImm13(0x1f69)))
        );
        Ok(())
    }

    #[test]
    fn decode_ldrw() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x4f2b3f69)?,
            I::Ldrw(LongInstruction::new(true, 5, 0x33f69))
        );
        Ok(())
    }

    #[test]
    fn decode_ldxhu() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x51293f69)?,
            I::Ldxhu(ShortInstruction::new(true, 5, 4, SS::UImm13(0x1f69)))
        );
        Ok(())
    }

    #[test]
    fn decode_ldrhu() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x532b3f69)?,
            I::Ldrhu(LongInstruction::new(true, 5, 0x33f69))
        );
        Ok(())
    }

    #[test]
    fn decode_ldxhs() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x55293f69)?,
            I::Ldxhs(ShortInstruction::new(true, 5, 4, SS::UImm13(0x1f69)))
        );
        Ok(())
    }

    #[test]
    fn decode_ldrhs() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x572b3f69)?,
            I::Ldrhs(LongInstruction::new(true, 5, 0x33f69))
        );
        Ok(())
    }

    #[test]
    fn decode_ldxbu() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x59293f69)?,
            I::Ldxbu(ShortInstruction::new(true, 5, 4, SS::UImm13(0x1f69)))
        );
        Ok(())
    }

    #[test]
    fn decode_ldrbu() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x5b2b3f69)?,
            I::Ldrbu(LongInstruction::new(true, 5, 0x33f69))
        );
        Ok(())
    }

    #[test]
    fn decode_ldxbs() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x5d293f69)?,
            I::Ldxbs(ShortInstruction::new(true, 5, 4, SS::UImm13(0x1f69)))
        );
        Ok(())
    }

    #[test]
    fn decode_ldrbs() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x5f2b3f69)?,
            I::Ldrbs(LongInstruction::new(true, 5, 0x33f69))
        );
        Ok(())
    }

    // Store instructions.

    #[test]
    fn decode_stxw() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x6d293f69)?,
            I::Stxw(ShortInstruction::new(true, 5, 4, SS::UImm13(0x1f69)))
        );
        Ok(())
    }

    #[test]
    fn decode_Strw() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x6f2b3f69)?,
            I::Strw(LongInstruction::new(true, 5, 0x33f69))
        );
        Ok(())
    }

    #[test]
    fn decode_stxh() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x75293f69)?,
            I::Stxh(ShortInstruction::new(true, 5, 4, SS::UImm13(0x1f69)))
        );
        Ok(())
    }

    #[test]
    fn decode_Strh() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x772b3f69)?,
            I::Strh(LongInstruction::new(true, 5, 0x33f69))
        );
        Ok(())
    }

    #[test]
    fn decode_stxb() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x7d293f69)?,
            I::Stxb(ShortInstruction::new(true, 5, 4, SS::UImm13(0x1f69)))
        );
        Ok(())
    }

    #[test]
    fn decode_Strb() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x7f2b3f69)?,
            I::Strb(LongInstruction::new(true, 5, 0x33f69))
        );
        Ok(())
    }

    // Short source tests.

    #[test]
    fn ss_uimm_to_simm1() {
        assert_eq!(SS::new(0xf00f, false).uimm_to_simm(), SS::SImm13(-4111));
    }

    #[test]
    fn ss_uimm_to_simm2() {
        assert_eq!(
            SS::new(0xf0ff, false).uimm_to_simm(),
            SS::SImm13(-(0x10ff as i32))
        );
    }

    impl fmt::Debug for SS {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self)
        }
    }

    impl fmt::Debug for DecodeError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self)
        }
    }

    impl fmt::Debug for I {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            // Instruction to string conversion.
            write!(
                f,
                "{}",
                match *self {
                    I::Calli(o) => format!("Calli {}", o),
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
            )
        }
    }
}

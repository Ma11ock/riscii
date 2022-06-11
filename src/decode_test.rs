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

    type I = decode::Instruction;
    type SS = decode::ShortSource;

    use decode::*;
    use std::fmt;

    #[test]
    fn decode_calli() -> Result<(), DecodeError> {
        assert_eq!(
            decode(0x02D00000)?,
            I::Calli(ShortInstruction::new(true, 5, 0, SS::Reg(0)))
        );
        Ok(())
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

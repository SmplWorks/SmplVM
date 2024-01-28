use smpl_core_common::{Instruction, Register, Width};
use crate::{decompile, utils::Result};

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(non_snake_case)]
pub struct VM {
    registers : [u16; 16],
    ram : Vec<u8>,
}

impl VM {
    pub fn new(ram : Vec<u8>) -> Self {
        Self { registers: [0; 16], ram }
    }

    pub fn reset(&mut self) {
        let low = self.get_mem(0xFFFE) as u16;
        let high = self.get_mem(0xFFFF) as u16;
        self.set_reg(&Register::RIP, low | (high << 8));
        self.set_reg(&Register::Flags, 0x0000);
        // TODO: Set RINFO
    }

    pub fn execute_n(&mut self, n : usize) -> Result<()> {
        for _ in 0..n {
            self.execute_next()?;
        }
        Ok(())
    }

    pub fn execute_next(&mut self) -> Result<()> {
        let inst = self.decompile_next()?;
        Ok(self.execute_instr(&inst))
    }

    pub fn execute_instr(&mut self, inst : &Instruction) {
        use Instruction::*;
        match inst {
            Nop => (),
            DB(_) => unreachable!(),

            MovC2R(value, dest) => self.set_reg(&dest, value.value_word()),

            _ => todo!(),
        }
    }

    pub fn decompile_next(&mut self) -> Result<Instruction> {
        let (inst, skip) = decompile(&self, *self.get_reg(&Register::RIP));
        self.set_reg(&Register::RIP, self.get_reg(&Register::RIP).wrapping_add(skip));
        inst
    }

    pub fn set_reg(&mut self, reg : &Register, value : u16) {
        *self.get_reg_mut(reg) = match reg.width() {
            Width::Byte => (value as u8) as u16,
            Width::Word => value,
        }
    }

    pub fn get_reg_mut(&mut self, reg : &Register) -> &mut u16 {
        let idx = reg.compile_src() as usize;
        self.registers.get_mut(idx).unwrap()
    }

    pub fn get_reg(&self, reg : &Register) -> &u16 {
        let idx = reg.compile_src() as usize;
        self.registers.get(idx).unwrap()
    }

    pub fn get_mem(&self, addr : u16) -> u8 {
        self.ram.get(addr as usize) // None if out of bounds, undefined behaviour
            .map_or(0, |b| *b) // TODO: Return random?
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! case {
        ($ident:ident, $reset:literal, $reps:literal, $code:literal, $regs:expr, $mem:expr) => {
            #[test]
            fn $ident() {
                let mut ram = vec![0; 0x10000];
                ram[0xFFFE] = $reset as u8;
                ram[0xFFFF] = ($reset >> 8) as u8;

                sasm_lib::compile($code).unwrap().into_iter().enumerate()
                    .for_each(|(idx, b)| ram[idx] = b);

                let mut vm = VM::new(ram);
                vm.reset();
                let res = vm.execute_n($reps);

                assert!(res.is_ok());
                assert_eq!(vm.registers, $regs, "registers");
                $mem.into_iter().for_each(|(addr, b)| assert_eq!(vm.get_mem(addr), b));
            }
        };
    }

    case!(reset, 0xF337u16, 0, "", [0, 0xF337u16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], []);
    case!(nop, 0x0000u16, 1, "nop", [0, 0x0002u16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], []);

    case!(movc2r_byte, 0x0000u16, 1, "mov 0xF3, rb0", [0, 0x0004u16, 0, 0, 0x00F3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], []);
    case!(movc2r_word, 0x0000u16, 1, "mov 0xF337, r0", [0, 0x0004u16, 0, 0, 0xF337, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], []);
}

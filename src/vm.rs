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
            MovR2R(src, dest) => self.set_reg(&dest, *self.get_reg(src)),
            MovM2R(src, dest)
                => self.set_reg(&dest, self.get_mem(*self.get_reg(src)) as u16),
            MovR2M(src, dest)
                => self.set_mem(*self.get_reg(dest), *self.get_reg(src) as u8),

            Add(src, dest) => self.execute_add(src, dest),
            Sub(src, dest) => self.execute_sub(src, dest),
            
            Jmp(reg) => self.set_reg(&Register::RIP, *self.get_reg(reg)),
        }
    }

    fn execute_add(&mut self, src : &Register, dest : &Register) {
        // TODO: Repeated code with execute_sub
        let src_value = *self.get_reg(src);
        let dest_value = *self.get_reg(dest);
        let (res, zero, negative,  overflow) = match src.width() {
            Width::Byte => {
                let (res, overflow) = (src_value as u8).overflowing_add(dest_value as u8);
                (res as u16, res == 0, (res & 0x80) == 0x80, overflow)
            },
            Width::Word => {
                let (res, overflow) = src_value.overflowing_add(dest_value);
                (res, res == 0, (res & 0x8000) == 0x8000, overflow)
            }
        };

        self.set_flags(zero, negative, overflow);
        self.set_reg(dest, res);
    }

    fn execute_sub(&mut self, src : &Register, dest : &Register) {
        // TODO: Repeated code with execute_add
        let src_value = *self.get_reg(src);
        let dest_value = *self.get_reg(dest);
        let (res, zero, negative,  overflow) = match src.width() {
            Width::Byte => {
                let (res, overflow) = (src_value as u8).overflowing_sub(dest_value as u8);
                (res as u16, res == 0, (res & 0x80) == 0x80, overflow)
            },
            Width::Word => {
                let (res, overflow) = src_value.overflowing_sub(dest_value);
                (res, res == 0, (res & 0x8000) == 0x8000, overflow)
            }
        };

        self.set_flags(zero, negative, overflow);
        self.set_reg(dest, res);
    }

    pub fn decompile_next(&mut self) -> Result<Instruction> {
        let (inst, skip) = decompile(&self, *self.get_reg(&Register::RIP));
        self.set_reg(&Register::RIP, self.get_reg(&Register::RIP).wrapping_add(skip));
        inst
    }

    pub fn set_flags(&mut self, zero : bool, negative : bool, overflow : bool) {
        self.set_reg(&Register::Flags, Self::calc_flags(zero, negative, overflow))
    }

    pub fn calc_flags(zero : bool, negative : bool, overflow : bool) -> u16 {
        ((zero as u16) << 0) | ((negative as u16) << 1) | ((overflow as u16) << 2)
    }

    pub fn set_reg(&mut self, reg : &Register, value : u16) {
        let prev_value = self.get_reg_mut(reg);
        *self.get_reg_mut(reg) = match reg.width() {
            Width::Byte => ((value as u8) as u16) | (*prev_value & 0xFF00),
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

    pub fn set_mem(&mut self, addr : u16, value : u8) {
        let Some(b) = self.ram.get_mut(addr as usize) else { return };
        *b = value;
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
    case!(movc2r_word, 0x0000u16, 1, "mov 0xF337, r1", [0, 0x0004u16, 0, 0, 0, 0xF337, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], []);
    case!(movr2r_byte, 0x0000u16, 2, "mov 0xF3, rb2\nmov rb2, rb3",
        [0, 0x0006u16, 0, 0, 0, 0, 0x00F3, 0x00F3, 0, 0, 0, 0, 0, 0, 0, 0], []);
    case!(movr2r_word, 0x0000u16, 2, "mov 0xF337, r4\nmov r4, r5",
        [0, 0x0006u16, 0, 0, 0, 0, 0, 0, 0xF337, 0xF337, 0, 0, 0, 0, 0, 0], []);
    case!(movm2r_word, 0x0000u16, 1, "mov [r0], rb6",
        [0, 0x0002u16, 0, 0, 0, 0, 0, 0, 0, 0, Instruction::movm2r(Register::r0(), Register::rb6()).unwrap().opcode() as u16, 0, 0, 0, 0, 0], []);
    case!(movr2m_word, 0x0000u16, 2, "mov 0xF337, r7\nmov rb7, [r0]",
        [0, 0x0006u16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xF337, 0, 0, 0, 0], [(0x0000, 0x37)]);

    case!(add_byte, 0x0000u16, 3, "mov 0xF337, r8\nmov 0x1111, r9\nadd rb8, rb9",
        [0, 0x000Au16, 0, VM::calc_flags(false, false, false), 0, 0, 0, 0, 0, 0, 0, 0, 0xF337, 0x1148, 0, 0], []);
    case!(add_word, 0x0000u16, 3, "mov 0xF337, r8\nmov 0x1111, r9\nadd r8, r9",
        [0, 0x000Au16, 0, VM::calc_flags(false, false, true), 0, 0, 0, 0, 0, 0, 0, 0, 0xF337, 0x0448, 0, 0], []);
    case!(add_byte_overflow_and_zero, 0x0000u16, 3, "mov 0xFF, rb10\nmov 0x01, rb11\nadd rb10, rb11",
        [0, 0x000Au16, 0, VM::calc_flags(true, false, true), 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0x00], []);
    case!(add_byte_negative, 0x0000u16, 3, "mov 0x0F, rb10\nmov 0xF0, rb11\nadd rb10, rb11",
        [0, 0x000Au16, 0, VM::calc_flags(false, true, false), 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x0F, 0xFF], []);
    case!(sub_byte, 0x0000u16, 3, "mov 0xF337, r0\nmov 0x1111, r1\nsub rb0, rb1",
        [0, 0x000Au16, 0, VM::calc_flags(false, false, false), 0xF337, 0x1126, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], []);
    case!(sub_word, 0x0000u16, 3, "mov 0xF337, r0\nmov 0x1111, r1\nsub r0, r1",
        [0, 0x000Au16, 0, VM::calc_flags(false, true, false), 0xF337, 0xE226, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], []);

    case!(jmp, 0x0000u16, 2, "mov 0xF337, r0\njmp r0",
        [0, 0xF337u16, 0, 0, 0xF337, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], []);
}

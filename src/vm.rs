use std::sync::{Arc, Mutex};

use smpl_core_common::{Instruction, Register, Value, Width};
use crate::{decompile, utils::Result};

#[derive(Debug, Clone)]
#[allow(non_snake_case)]
pub struct VM {
    pub registers : [u16; 16],

    pub ram : Vec<u8>,
    pub rom : [u8; 2],
    pub display_buffer : Arc<Mutex<[u8; 64 * 32 * 2]>>,
}

impl VM {
    pub fn new(ram : Vec<u8>, rom : [u8; 2], display_buffer : Arc<Mutex<[u8; 64 * 32 * 2]>>) -> Self {
        Self { registers: [0; 16], ram, rom, display_buffer }
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

            AddC2R(value, dest) => self.execute_add(value, dest, true),
            AddR2R(src, dest) => self.execute_add(&self.get_reg_as_value(src), dest, true),
            SubC2R(value, dest) => self.execute_sub(value, dest, true),
            SubR2R(src, dest) => self.execute_sub(&self.get_reg_as_value(src), dest, true),
            
            AJmp(reg) => self.set_reg(&Register::RIP, *self.get_reg(reg)),
            Jmp(reg) => self.execute_add(&self.get_reg_as_value(reg), &Register::RIP, false),
        }
    }

    fn execute_add(&mut self, src_value : &Value, dest : &Register, flags : bool) {
        // TODO: Repeated code with execute_sub
        let dest_value = *self.get_reg(dest);
        let (res, zero, negative,  overflow) = match src_value.width() {
            Width::Byte => {
                let (res, overflow) = src_value.value_byte(0).overflowing_add(dest_value as u8);
                (res as u16, res == 0, (res & 0x80) == 0x80, overflow)
            },
            Width::Word => {
                let (res, overflow) = src_value.value_word().overflowing_add(dest_value);
                (res, res == 0, (res & 0x8000) == 0x8000, overflow)
            }
        };

        if flags {
            self.set_flags(zero, negative, overflow);
        }
        self.set_reg(dest, res);
    }

    fn execute_sub(&mut self, src_value : &Value, dest : &Register, flags : bool) {
        // TODO: Repeated code with execute_add
        let dest_value = *self.get_reg(dest);
        let (res, zero, negative,  overflow) = match src_value.width() {
            Width::Byte => {
                let (res, overflow) = src_value.value_byte(0).overflowing_sub(dest_value as u8);
                (res as u16, res == 0, (res & 0x80) == 0x80, overflow)
            },
            Width::Word => {
                let (res, overflow) = src_value.value_word().overflowing_sub(dest_value);
                (res, res == 0, (res & 0x8000) == 0x8000, overflow)
            }
        };

        if flags {
            self.set_flags(zero, negative, overflow);
        }
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

    pub fn get_reg_as_value(&self, reg : &Register) -> Value {
        Value::new(reg.width(), *self.get_reg(reg))
    }

    pub fn set_mem(&mut self, addr : u16, value : u8) {
        if addr < 0x8000 { // RAM
            if let Some(b) = self.ram.get_mut(addr as usize) {
                *b = value;
            }
        } else if addr < 0x9000 { // Display
            let mut buffer = self.display_buffer.lock().unwrap();
            if let Some(b) = buffer.get_mut((addr - 0x8000) as usize) {
                *b = value;
            }
        }
    }

    pub fn get_mem(&self, addr : u16) -> u8 {
        let b = if addr < 0x8000 { // RAM
            self.ram.get(addr as usize)
        } else if addr >= 0xFFFE { // ROM
            self.rom.get((addr - 0xFFFE) as usize)
        } else {
            None
        }; 

        b.map_or(0, |b| *b) // None if out of bounds, undefined behaviour // TODO: Return random? 
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! case {
        ($ident:ident, $reset:literal, $reps:literal, $code:expr, $regs:expr, $mem:expr) => {
            #[test]
            fn $ident() {
                let mut ram = vec![0; 0x10000];
                let rom = [$reset as u8, ($reset >> 8) as u8];
                let display_buffer = Arc::new(Mutex::new([0; 64 * 32 * 2]));

                sasm_lib::compile($code).unwrap().into_iter().enumerate()
                    .for_each(|(idx, b)| ram[idx] = b);

                let mut vm = VM::new(ram, rom, display_buffer);
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

    case!(add_c2r_byte, 0x0000u16, 2, "mov 0xF337, r8\nadd 0x11, rb8",
        [0, 0x0008u16, 0, VM::calc_flags(false, false, false), 0, 0, 0, 0, 0, 0, 0, 0, 0xF348, 0, 0, 0], []);
    case!(add_c2r_word, 0x0000u16, 2, "mov 0xF337, r8\nadd 0x1111, r8",
        [0, 0x0008u16, 0, VM::calc_flags(false, false, true), 0, 0, 0, 0, 0, 0, 0, 0, 0x0448, 0, 0, 0], []);
    case!(add_r2r_byte, 0x0000u16, 3, "mov 0xF337, r8\nmov 0x1111, r9\nadd rb8, rb9",
        [0, 0x000Au16, 0, VM::calc_flags(false, false, false), 0, 0, 0, 0, 0, 0, 0, 0, 0xF337, 0x1148, 0, 0], []);
    case!(add_r2r_word, 0x0000u16, 3, "mov 0xF337, r8\nmov 0x1111, r9\nadd r8, r9",
        [0, 0x000Au16, 0, VM::calc_flags(false, false, true), 0, 0, 0, 0, 0, 0, 0, 0, 0xF337, 0x0448, 0, 0], []);
    case!(add_byte_overflow_and_zero, 0x0000u16, 3, "mov 0xFF, rb10\nmov 0x01, rb11\nadd rb10, rb11",
        [0, 0x000Au16, 0, VM::calc_flags(true, false, true), 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0x00], []);
    case!(add_r2r_byte_negative, 0x0000u16, 3, "mov 0x0F, rb10\nmov 0xF0, rb11\nadd rb10, rb11",
        [0, 0x000Au16, 0, VM::calc_flags(false, true, false), 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x0F, 0xFF], []);
    case!(sub_c2r_byte, 0x0000u16, 2, "mov 0xF337, r0\nsub 0x11, rb0",
        [0, 0x0008u16, 0, VM::calc_flags(false, false, false), 0xF326, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], []);
    case!(sub_c2r_word, 0x0000u16, 2, "mov 0xF337, r0\nsub 0x1111, r0",
        [0, 0x0008u16, 0, VM::calc_flags(false, false, true), 0x1DDA, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], []);
    case!(sub_r2r_byte, 0x0000u16, 3, "mov 0xF337, r0\nmov 0x1111, r1\nsub rb0, rb1",
        [0, 0x000Au16, 0, VM::calc_flags(false, false, false), 0xF337, 0x1126, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], []);
    case!(sub_r2r_word, 0x0000u16, 3, "mov 0xF337, r0\nmov 0x1111, r1\nsub r0, r1",
        [0, 0x000Au16, 0, VM::calc_flags(false, true, false), 0xF337, 0xE226, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], []);

    case!(ajmp, 0x0000u16, 2, "mov 0xF337, r0\najmp r0",
        [0, 0xF337, 0, 0, 0xF337, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], []);
    case!(jmp, 0x0000u16, 2, "mov 0xF337, r0\njmp r0",
        [0, 0xF33D, 0, 0, 0xF337, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], []);

    case!(basic, 0x0000, 15, &std::fs::read_to_string(std::path::Path::new("./examples/basic.sasm")).unwrap(),
        [0, 0x001C, 0, 0, 0x0CF3, 0x6000, 0x0CE6, 256, 0xF3, -2i16 as u16, 0, 0, 0, 0, 0, 0], [(256, 0xF3)]);
}

use smpl_core_common::{Instruction, Register};
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

    pub fn execute_next(&mut self) -> Result<()> {
        let inst = self.decompile_next()?;
        Ok(self.execute_instr(&inst))
    }

    pub fn execute_instr(&mut self, inst : &Instruction) {
        use Instruction::*;
        match inst {
            Nop => (),
            DB(_) => unreachable!(),

            _ => todo!(),
        }
    }

    pub fn decompile_next(&mut self) -> Result<Instruction> {
        let (inst, skip) = decompile(&self, *self.get_reg(&Register::RIP));
        self.set_reg(&Register::RIP, self.get_reg(&Register::RIP).wrapping_add(skip));
        inst
    }

    pub fn set_reg(&mut self, reg : &Register, value : u16) {
        let idx = reg.compile_src() as usize;
        let reg = self.registers.get_mut(idx).unwrap();
        *reg = value;
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

    #[test]
    fn reset() {
        let mut ram = vec![0; 0x10000];
        ram[0xFFFE] = 0x37;
        ram[0xFFFF] = 0xF3;

        let mut vm = VM::new(ram);
        vm.reset();

        assert_eq!(*vm.get_reg(&Register::RIP), 0xF337);
    }

    #[test]
    fn nop() {
        let ram = vec![0; 0x10000];
        // TODO: Load "nop" to RAM

        let mut vm = VM::new(ram);
        vm.reset();

        let inst = vm.decompile_next();
        assert_eq!(inst, Ok(Instruction::Nop));

        vm.execute_instr(&inst.unwrap());
        // TODO: Check vm state 
    }
}

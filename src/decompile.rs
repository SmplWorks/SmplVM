use smpl_core_common::Instruction;
use crate::{VM, utils::{Error, Result}};

pub fn decompile(vm : &VM, addr : u16) -> (Result<Instruction>, u16) {
    match vm.get_mem(addr) {
        0x00 => (Ok(Instruction::Nop), 2),

        opcode => (Err(Error::InvalidOpcode(opcode, vm.get_mem(addr + 1))), 2),
    }
}

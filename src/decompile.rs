use smpl_core_common::{Instruction, Register, Value, Width};
use crate::{VM, utils::{Error, Result}};

pub fn decompile(vm : &VM, addr : u16) -> (Result<Instruction>, u16) {
    use Instruction::*;
    let inst = match vm.get_mem(addr) {
        0x00 => Ok(Nop),
        0x01 => Ok(MovC2R(
            Value::byte(vm.get_mem(addr + 2)),
            Register::from_dest(Width::Byte, vm.get_mem(addr + 1))
        )),
        0x02 => Ok(MovC2R(
            Value::word((vm.get_mem(addr + 2) as u16) | ((vm.get_mem(addr + 3) as u16) << 8)),
            Register::from_dest(Width::Word, vm.get_mem(addr + 1))
        )),
        0x03 => Ok(MovR2R(
            Register::from_src(Width::Byte, vm.get_mem(addr + 1)),
            Register::from_dest(Width::Byte, vm.get_mem(addr + 1))
        )),
        0x04 => Ok(MovR2R(
            Register::from_src(Width::Word, vm.get_mem(addr + 1)),
            Register::from_dest(Width::Word, vm.get_mem(addr + 1))
        )),
        0x05 => Ok(MovM2R(
            Register::from_src(Width::Word, vm.get_mem(addr + 1)),
            Register::from_dest(Width::Byte, vm.get_mem(addr + 1))
        )),
        0x06 => Ok(MovR2M(
            Register::from_src(Width::Byte, vm.get_mem(addr + 1)),
            Register::from_dest(Width::Word, vm.get_mem(addr + 1))
        )),

        0x07 => Ok(AddC2R(
            Value::byte(vm.get_mem(addr + 2)),
            Register::from_dest(Width::Byte, vm.get_mem(addr + 1))
        )),
        0x08 => Ok(AddC2R(
            Value::word((vm.get_mem(addr + 2) as u16) | ((vm.get_mem(addr + 3) as u16) << 8)),
            Register::from_dest(Width::Word, vm.get_mem(addr + 1))
        )),
        0x09 => Ok(AddR2R(
            Register::from_src(Width::Byte, vm.get_mem(addr + 1)),
            Register::from_dest(Width::Byte, vm.get_mem(addr + 1))
        )),
        0x0A => Ok(AddR2R(
            Register::from_src(Width::Word, vm.get_mem(addr + 1)),
            Register::from_dest(Width::Word, vm.get_mem(addr + 1))
        )),

        0x0B => Ok(SubC2R(
            Value::byte(vm.get_mem(addr + 2)),
            Register::from_dest(Width::Byte, vm.get_mem(addr + 1))
        )),
        0x0C => Ok(SubC2R(
            Value::word((vm.get_mem(addr + 2) as u16) | ((vm.get_mem(addr + 3) as u16) << 8)),
            Register::from_dest(Width::Word, vm.get_mem(addr + 1))
        )),
        0x0D => Ok(SubR2R(
            Register::from_src(Width::Byte, vm.get_mem(addr + 1)),
            Register::from_dest(Width::Byte, vm.get_mem(addr + 1))
        )),
        0x0E => Ok(SubR2R(
            Register::from_src(Width::Word, vm.get_mem(addr + 1)),
            Register::from_dest(Width::Word, vm.get_mem(addr + 1))
        )),
        
        0x0F => Ok(AJmp(Register::from_src(Width::Word, vm.get_mem(addr + 1)))),
        0x10 => Ok(Jmp(Register::from_src(Width::Word, vm.get_mem(addr + 1)))),

        opcode => Err(Error::InvalidOpcode(opcode, vm.get_mem(addr + 1))),
    };

    let len = inst.as_ref().map_or(2, |inst| inst.len());
    (inst, len)
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! case_gen {
        ($ident:ident, $code:literal, $vm:ident, $res:ident, $expect:ident, $assert:block) => {
            #[test]
            fn $ident() {
								use std::sync::{Arc, Mutex};

                let ram = sasm_lib::compile($code).unwrap();
                let rom = [0, 0];
                let display_buffer = Arc::new(Mutex::new([0; 64 * 32 * 2]));
                let mut $vm = VM::new(ram, rom, display_buffer);
                $vm.set_reg(&smpl_core_common::Register::RIP, 0x0000);

                let $expect = sasm_lib::parse($code).unwrap().0[0];
                let $res = $vm.decompile_next();

                $assert
            }
        };
    }

    macro_rules! case {
        ($ident:ident, $code:literal) => {
            case_gen!($ident, $code, vm, res, expect, { 
                assert_eq!(res, Ok(expect));
                assert_eq!(*vm.get_reg(&smpl_core_common::Register::RIP), expect.len() as u16);
            });
        };
    }

    case!(nop, "nop");
    case_gen!(db, "db 0xF3, 0x37", vm, res, _expect, {
        assert_eq!(res, Err(Error::InvalidOpcode(0xF3, 0x37)));
    });

    case!(movc2r_byte, "mov 0xF3, rb0");
    case!(movc2r_word, "mov 0xF337, r1");
    case!(movr2r_byte, "mov rb2, rb3");
    case!(movr2r_word, "mov r4, r5");
    case!(movm2r, "mov [r6], rb7");
    case!(movr2m, "mov rb8, [r9]");

    case!(addc2r_byte, "add 0xF3, rb11");
    case!(addc2r_word, "add 0xF337, r1");
    case!(addr2r_byte, "add rb10, rb11");
    case!(addr2r_word, "add r0, r1");
    case!(subc2r_byte, "sub 0xF3, rb11");
    case!(subc2r_word, "sub 0xF337, r1");
    case!(subr2r_byte, "sub r0, r1");
    case!(subr2r_word, "sub r0, r1");

    case!(ajmp, "ajmp r0");
    case!(jmp, "jmp r0");
}

use smpl_core_common::Register;
use crate::{VM, Cmd, utils::{Args, Config, Result}};

#[derive(Debug, Clone, PartialEq)]
enum Break {
    Step,
    Point(u16),

    GetAddr(u16, u8),
    GetReg(Register, u16),

    None,
}

pub struct Debugger {
    vm : VM,
    breakpoints : Vec<u16>,
    first_prompt : bool,
}

impl Debugger {
    pub fn new(vm : VM, breakpoints : Vec<u16>, first_prompt : bool) -> Self {
        Self { vm, breakpoints, first_prompt }
    }

    pub fn from_cfg(vm : VM, args : &Args, cfg : &Config) -> Self {
        Self::new(
            vm,
            cfg.breakpoints.clone(),
            args.first_prompt,
        )
    }

    fn step(&mut self, ignore_breakpoint : bool) -> Result<Break> {
        let addr = self.vm.get_reg(&smpl_core_common::Register::RIP);
        if !ignore_breakpoint && self.breakpoints.binary_search(addr).is_ok() {
            Ok(Break::Point(*addr))
        } else {
            self.vm.execute_next()
                .map(|_| Break::Step)
        }
    }

    fn cont(&mut self, ignore_breakpoint : bool) -> Result<Break> {
        let mut res = self.step(ignore_breakpoint)?;
        loop {
            match res {
                Break::Step => (),
                Break::Point(_) => return Ok(res),
                
                Break::None | Break::GetAddr(_, _) | Break::GetReg(_, _)
                    => unreachable!("{res:?}"),
            }

            res = self.step(false)?;
        }
    }

    fn action_cmd(&mut self, cmd : Cmd, ignore_breakpoint : bool) -> Result<Break> {
        match cmd {
            Cmd::Step => self.step(ignore_breakpoint),
            Cmd::Continue => self.cont(ignore_breakpoint),

            Cmd::GetAddr(addr) => Ok(Break::GetAddr(addr, self.vm.get_mem(addr))),
            Cmd::SetAddr(addr, value) => {
                self.vm.set_mem(addr, value);
                Ok(Break::None)
            }

            Cmd::GetReg(reg) => Ok(Break::GetReg(reg, *self.vm.get_reg(&reg))),
            Cmd::SetReg(reg, value) => {
                self.vm.set_reg(&reg, value);
                Ok(Break::None)
            }
        }
    }

    pub fn debug(&mut self) -> Result<()> {
        let mut cmd = if self.first_prompt { Cmd::prompt(Some(Cmd::Continue))? } else { Cmd::Continue };
        let mut ignore_breakpoint = false;
        loop {
            let res = self.action_cmd(cmd.clone(), ignore_breakpoint)?;

            ignore_breakpoint = false;
            match res {
                Break::Step | Break::None => (),

                Break::Point(addr) => {
                    println!("Breakpoint at: {addr}");
                    ignore_breakpoint = true;
                },

                Break::GetAddr(addr, value) =>
                    println!("0x{addr:04X}: 0x{value:02X}"),

                Break::GetReg(reg, value) =>
                    println!("{reg}: 0x{value:04X}"),
            }

            cmd = Cmd::prompt(Some(cmd))?;
        }
    }
}

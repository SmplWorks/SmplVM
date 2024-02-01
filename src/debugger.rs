use crate::{vm::VM, utils::{Args, Config, Result}};

pub struct Debugger {
    vm : VM,
    breakpoints : Vec<u16>,
}

impl Debugger {
    pub fn new(vm : VM, breakpoints : Vec<u16>) -> Self {
        Self { vm, breakpoints }
    }

    pub fn from_cfg(vm : VM, _args : &Args, cfg : &Config) -> Self {
        Self::new(vm, cfg.breakpoints.clone())
    }

    pub fn debug(&mut self) -> Result<()> {
        let mut ignore_breakpoint = false;
        loop {
            let addr = self.vm.get_reg(&smpl_core_common::Register::RIP);
            if !ignore_breakpoint && self.breakpoints.binary_search(addr).is_ok() {
                println!("Breakpoint at: {addr}");
                ignore_breakpoint = true;
                continue;
            }

            self.vm.execute_next()?;

            ignore_breakpoint = false;
        }
    }
}

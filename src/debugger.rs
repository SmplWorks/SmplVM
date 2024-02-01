use crate::{vm::VM, utils::{Args, Config, Error, Result}};

enum Break {
    Step,
    Point(u16),
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
            }

            res = self.step(false)?;
        }
    }

    fn prompt(&self) -> Result<String> {
        inquire::Text::new("").prompt()
            .map_err(|err| Error::External(err.to_string()))
    }

    pub fn debug(&mut self) -> Result<()> {
        let mut cmd = if self.first_prompt { self.prompt()? } else { "c".to_string() };
        let mut ignore_breakpoint = false;
        loop {
            let res = match &*cmd {
                "s" | "step" => self.step(ignore_breakpoint)?,
                "c" | "cont" | "continue" => self.cont(ignore_breakpoint)?,

                cmd => todo!("{cmd}"),
            };

            ignore_breakpoint = false;
            match res {
                Break::Step => (),

                Break::Point(addr) => {
                    println!("Breakpoint at: {addr}");
                    ignore_breakpoint = true;
                },
            }

            cmd = self.prompt()?;
        }
    }
}

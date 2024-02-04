mod vm;
mod decompile;
mod display;
mod debugger;
mod cmd;
pub mod utils;

pub use vm::VM;
pub use decompile::decompile;
pub use debugger::Debugger;
pub use cmd::Cmd;

use std::{path::Path, sync::{Arc, Mutex}};

use display::display;
use utils::{Args, Config, Result};

fn compile_file(fpath : &Path) -> Result<Vec<u8>> {
    Ok(sasm_lib::compile(
        &std::fs::read_to_string(fpath)
        .map_err(|err| utils::Error::External(err.to_string()))?
    )?)
}

fn read_file(fpath : &Path) -> Result<Vec<u8>> {
    std::fs::read(fpath).map_err(|err| utils::Error::External(err.to_string()))
}

fn main_loop(mut vm : VM, args : &Args, cfg : &Config) -> Result<()> {
    if cfg.debug || args.debug {
        let mut dbg = Debugger::from_cfg(vm, args, cfg);
        dbg.debug()
    } else {
        loop {
            vm.execute_next()?
        }
    }
}

fn main() -> Result<()> {
    let args = Args::load();
    let cfg = Config::load(&args)?;

    let in_path = Path::new(&cfg.in_path);
    let ram = if cfg.compile {
        compile_file(in_path)?
    } else {
        read_file(in_path)?
    };

    let display_buffer = Arc::new(Mutex::new([0; 64 * 32 * 2]));

    let vm = VM::new(ram, [0, 0], display_buffer.clone());

    if cfg.display && !args.no_display {
        std::thread::spawn(move || main_loop(vm, &args, &cfg).unwrap()); // TODO: Handle error
        display(display_buffer.clone())
    } else {
        main_loop(vm, &args, &cfg)
    }
}

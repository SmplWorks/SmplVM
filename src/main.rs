mod vm;

pub use vm::VM;

mod decompile;
pub use decompile::decompile;

mod display;

pub mod utils;

use std::{path::Path, sync::{Arc, Mutex}};

use display::display;
use utils::Result;

fn compile_file(fpath : &Path) -> Result<Vec<u8>> {
    Ok(sasm_lib::compile(
        &std::fs::read_to_string(fpath)
        .map_err(|err| utils::Error::External(err.to_string()))?
    )?)
}

fn read_file(fpath : &Path) -> Result<Vec<u8>> {
    std::fs::read(fpath).map_err(|err| utils::Error::External(err.to_string()))
}

fn main_loop(mut vm : VM) -> Result<()> {
    loop {
        vm.execute_next()?;
    }
}

fn main() -> Result<()> {
    let args = utils::Args::load();
    let cfg = utils::Config::load(&args)?;

    let in_path = Path::new(&cfg.in_path);
    let ram = if cfg.compile {
        compile_file(in_path)?
    } else {
        read_file(in_path)?
    };

    let display_buffer = Arc::new(Mutex::new([0; 64 * 32 * 2]));

    let vm = VM::new(ram, [0, 0], display_buffer.clone());

    if cfg.display && !args.no_display {
        std::thread::spawn(move || main_loop(vm).unwrap()); // TODO: Handle error
        display(display_buffer.clone())
    } else {
        main_loop(vm)
    }
}

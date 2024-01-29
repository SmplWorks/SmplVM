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

fn main() -> Result<()> {
    let args = utils::Args::load();

    let in_path = Path::new(&args.in_path);
    let ram = if args.compile {
        compile_file(in_path)?
    } else {
        read_file(in_path)?
    };

    let display_buffer = Arc::new(Mutex::new([0; 64 * 32 * 2]));

    let mut vm = VM::new(ram, [0, 0], display_buffer.clone());

    std::thread::spawn(move || {
        vm.execute_n(args.reps).unwrap(); // TODO: Handle panic
        dbg!(vm.registers);
    });

    display(display_buffer.clone())
}

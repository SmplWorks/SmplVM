mod vm;

pub use vm::VM;

mod decompile;
pub use decompile::decompile;

pub mod utils;

/*
use std::path::Path;
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

    let mut vm = VM::new(ram, [0, 0]);

    vm.execute_n(args.reps)?;
    dbg!(vm.registers);

    Ok(())
}
*/

use std::num::NonZeroU32;
use std::rc::Rc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    dpi::LogicalSize,
};
use utils::{Error, Result};

fn main() -> Result<()> {
    let event_loop = EventLoop::new().map_err(|err| Error::External(err.to_string()))?;

    let builder = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(64 * 8, 32 * 8))
        // .with_position(position) // TODO
        .with_resizable(false)
        .with_title("SmplVM") // TODO: File being executed
        // .with_window_icon(icon) // TODO
        .with_active(true)
        ;
    let window = Rc::new(builder.build(&event_loop).map_err(|err| Error::External(err.to_string()))?);
    let context = softbuffer::Context::new(window.clone()).map_err(|err| Error::External(err.to_string()))?;
    let mut surface = softbuffer::Surface::new(&context, window.clone()).map_err(|err| Error::External(err.to_string()))?;
    surface.resize(NonZeroU32::new(64 * 8).unwrap(), NonZeroU32::new(32 * 8).unwrap()).unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. }
                => elwt.exit(),

            Event::AboutToWait => {
                // window.request_redraw();
            },

            Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
                let (width, height) = {
                    let size = window.inner_size();
                    (size.width, size.height)
                };
                surface
                    .resize(
                        NonZeroU32::new(width).unwrap(),
                        NonZeroU32::new(height).unwrap(),
                    )
                    .unwrap();

                let mut buffer = surface.buffer_mut().unwrap();
                for x in 0..20usize {
                    for y in 0..20usize {
                        let (red, green, blue) = (0xFF, 0xFF, 0xFF);
                        buffer[x + y * width as usize] = (red << 16) | (green << 8) | blue;
                    }
                }

                buffer.present().unwrap();
            }

            _ => (),
        }
    }).map_err(|err| Error::External(err.to_string()))
}

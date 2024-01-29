use std::{num::NonZeroU32, rc::Rc, sync::{Arc, Mutex}};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    dpi::LogicalSize,
};

use crate::utils::{Error, Result};

fn draw_char(c : char, _color : u8, offset : (usize, usize), foo : (usize, usize),(width, height) : (usize, usize), buffer : &mut [u32]) {
    let font = {
        let font_data = include_bytes!("./PxPlus_IBM_VGA_8x16-2x.ttf");
        rusttype::Font::try_from_bytes(font_data).unwrap()
    };

    let scale = rusttype::Scale {
        x: (width / foo.0) as f32,
        y: (height / foo.1) as f32
    };

    let offset = rusttype::point(
        offset.0 as f32 * scale.x,
        offset.1 as f32 * scale.y + font.v_metrics(scale).ascent
    );

    let mapping = [0x00_00_00_00, 0x00_FF_FF_FF]; // TODO: Color mapping
    let mapping_scale = (mapping.len() - 1) as f32;

    let g = font.glyph(c).scaled(scale).positioned(offset);
    if let Some(bb) = g.pixel_bounding_box() {
        g.draw(|x, y, v| {
            // v should be in the range 0.0 to 1.0
            let i = (v * mapping_scale + 0.5) as usize;
            let c = mapping[i];

            let x = x as i32 + bb.min.x;
            let y = y as i32 + bb.min.y;
            // There's still a possibility that the glyph clips the boundaries of the bitmap
            if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                let x = x as usize;
                let y = y as usize;
                buffer[x + y * width as usize] = c
            }
        });
    }
}

fn draw_buffer(in_buffer : &[u8], foo : (usize, usize), out_bb : (usize, usize), out_buffer : &mut [u32]) {
    for x in 0..foo.0 {
        for y in 0..foo.1 {
            let c = in_buffer[(x + y * foo.0) * 2] as char;
            let color = in_buffer[(x + y * foo.0) * 2 + 1];
            draw_char(c, color, (x, y), foo, out_bb, out_buffer);
        }
    }
}

pub fn display(in_buffer : Arc<Mutex<[u8; 64 * 32 * 2]>>) -> Result<()> {
    //let foo = (64u32, 32u32);
    let foo = (64u32, 32u32);

    let event_loop = EventLoop::new().map_err(|err| Error::External(err.to_string()))?;

    let builder = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(foo.0 * 8, foo.1 * 16))
        // .with_position(position) // TODO
        .with_resizable(false)
        .with_title("SmplVM") // TODO: File being executed
        // .with_window_icon(icon) // TODO
        .with_active(true)
        ;
    let window = Rc::new(builder.build(&event_loop).map_err(|err| Error::External(err.to_string()))?);
    let context = softbuffer::Context::new(window.clone()).map_err(|err| Error::External(err.to_string()))?;
    let mut surface = softbuffer::Surface::new(&context, window.clone()).map_err(|err| Error::External(err.to_string()))?;
    let (width, height) = {
        let size = window.inner_size();
        (size.width, size.height)
    };
    surface.resize(NonZeroU32::new(width).unwrap(), NonZeroU32::new(height).unwrap()).unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. }
                => elwt.exit(),

            Event::AboutToWait => {
                window.request_redraw();
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
                for b in buffer.iter_mut() {
                    *b = 0;
                }

                draw_buffer(
                    in_buffer.lock().unwrap().as_ref(),
                    (foo.0 as usize, foo.1 as usize),
                    (width as usize, height as usize),
                    &mut buffer
                );

                buffer.present().unwrap();
            }

            _ => (),
        }
    }).map_err(|err| Error::External(err.to_string()))
}

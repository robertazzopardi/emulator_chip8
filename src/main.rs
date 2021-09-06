pub mod chip8;
pub mod window;

extern crate sdl2;

use chip8::chip_8::Chip8;
use sdl2::{event::Event, keyboard::Keycode};
use std::env;
use window::window_driver::Win;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let path = &args[1];

    let mut window = Win::init()?;

    let mut chip8 = Chip8::init();
    chip8.load(path);

    'running: loop {
        for event in window.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown { keycode: code, .. } => {
                    chip8.set_action(code.unwrap(), 1);
                }
                Event::KeyUp { keycode: code, .. } => {
                    chip8.set_action(code.unwrap(), 0);
                }
                _ => {}
            }
        }

        chip8.emulate();

        if chip8.sound_timer != 0 {
            // play
            window.audio_device.resume();
        } else {
            // pause
            window.audio_device.pause();
        }

        if chip8.draw_flag {
            window.canvas.clear();
            chip8.update_quads(&mut window.canvas);
            window.canvas.present();
            chip8.draw_flag = false;
        }
    }

    Ok(())
}

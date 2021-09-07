pub mod audio;
pub mod processor;
pub mod window;

extern crate sdl2;

use audio::audio_driver::Audio;
use processor::chip_8::Chip8;
use std::env;
use window::window_driver::Win;

fn main() -> Result<(), String> {
    let path = &env::args().collect::<Vec<String>>()[1];

    let sdl_context = sdl2::init()?;

    let mut window = Win::new(&sdl_context)?;

    let mut chip8 = Chip8::new();
    chip8.load(path);

    let audio_device = Audio::new(&sdl_context.audio()?);

    while window.is_running() {
        window.handle_events(&mut chip8);

        chip8.cycle();

        audio_device.play(&mut chip8);

        window.draw(&mut chip8);
    }

    Ok(())
}

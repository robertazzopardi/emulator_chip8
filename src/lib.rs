mod chip8;

pub mod emulator_driver {
    use crate::chip8::{
        audio::audio_driver::Audio, processor::chip::Chip8, window::window_driver::Win,
    };
    use sdl2::Sdl;
    use std::env;

    pub const NAME: &str = "CHIP 8";

    pub fn start(rom_path: Option<&str>, sdl_context: Option<&Sdl>) -> Result<(), String> {
        let audio_device: Audio;
        let mut window: Win;

        if let Some(context) = sdl_context {
            audio_device = Audio::new(&context.audio()?);
            window = Win::new(context)?;
        } else {
            let sdl_context = sdl2::init()?;
            audio_device = Audio::new(&sdl_context.audio()?);
            window = Win::new(&sdl_context)?;
        }

        let mut chip8 = Chip8::new();

        if let Some(path) = rom_path {
            chip8.load(path)?;
        } else {
            let args = &mut env::args().collect::<Vec<String>>();
            if args.len() == 1 {
                return Err("Please provide a rom to run!".to_string());
            }
            chip8.load(&args[1])?
        }

        // TODO: move to window.rs
        while window.is_running() {
            window.handle_events(&mut chip8);

            chip8.cycle();

            audio_device.play(&mut chip8);

            window.draw(&mut chip8);
        }

        Ok(())
    }
}

mod chip8;

pub mod emulator_driver {
    use std::env;

    use crate::chip8::{
        audio::audio_driver::Audio, processor::chip::Chip8, window::window_driver::Win,
    };

    pub fn start() -> Result<(), String> {
        let args = &env::args().collect::<Vec<String>>();
        if args.len() == 1 {
            return Err("Please provide a rom to run!".to_string());
        }

        let sdl_context = sdl2::init()?;

        let audio_device = Audio::new(&sdl_context.audio()?);

        let mut window = Win::new(&sdl_context)?;

        let mut chip8 = Chip8::new();
        chip8.load(&args[1])?;

        while window.is_running() {
            window.handle_events(&mut chip8);

            chip8.cycle();

            audio_device.play(&mut chip8);

            window.draw(&mut chip8);
        }

        Ok(())
    }
}

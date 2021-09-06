pub mod window_driver {
    use sdl2::{
        audio::{AudioCallback, AudioDevice, AudioSpecDesired},
        render::Canvas,
        video::Window,
        EventPump,
    };

    pub struct SquareWave {
        phase_inc: f32,
        phase: f32,
        volume: f32,
    }

    impl AudioCallback for SquareWave {
        type Channel = f32;

        fn callback(&mut self, out: &mut [f32]) {
            // Generate a square wave
            for x in out.iter_mut() {
                *x = self.volume * if self.phase < 0.5 { 1.0 } else { -1.0 };
                self.phase = (self.phase + self.phase_inc) % 1.0;
            }
        }
    }

    pub struct Win {
        pub event_pump: EventPump,
        pub canvas: Canvas<Window>,
        pub audio_device: AudioDevice<SquareWave>,
    }

    impl Win {
        pub fn init() -> Result<Win, String> {
            let sdl_context = sdl2::init()?;
            let video_subsystem = sdl_context.video()?;

            let window = video_subsystem
                .window("Chip8 Emulator", 64 * 20, 32 * 20)
                .position_centered()
                .build()
                .map_err(|e| e.to_string())?;

            let canvas = window
                .into_canvas()
                .present_vsync()
                .accelerated()
                .build()
                .map_err(|e| e.to_string())?;

            // audio
            let audio_subsystem = sdl_context.audio()?;

            let desired_spec = AudioSpecDesired {
                freq: Some(44100),
                channels: Some(1), // mono
                samples: None,     // default sample size
            };

            let device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
                // initialize the audio callback
                SquareWave {
                    phase_inc: 240. / spec.freq as f32,
                    phase: 0.,
                    volume: 0.25,
                }
            })?;

            Ok(Win {
                event_pump: sdl_context.event_pump()?,
                canvas,
                audio_device: device,
            })
        }
    }
}

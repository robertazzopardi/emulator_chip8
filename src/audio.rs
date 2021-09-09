pub mod audio_driver {
    use crate::processor::chip_8::Chip8;
    use sdl2::{
        audio::{AudioCallback, AudioDevice, AudioSpecDesired},
        AudioSubsystem,
    };

    pub struct Audio {
        audio_device: AudioDevice<SquareWave>,
    }

    impl Audio {
        pub fn new(audio_subsystem: &AudioSubsystem) -> Audio {
            let desired_spec = AudioSpecDesired {
                freq: Some(44100),
                channels: Some(1), // mono
                samples: None,     // default sample size
            };

            let device = audio_subsystem
                .open_playback(None, &desired_spec, |spec| {
                    // newialize the audio callback
                    SquareWave {
                        phase_inc: 240. / spec.freq as f32,
                        phase: 0.,
                        volume: 0.25,
                    }
                })
                .unwrap();

            Audio {
                audio_device: device,
            }
        }

        pub fn play(&self, chip8: &mut Chip8) {
            if chip8.should_play_sound() {
                // play
                self.audio_device.resume();
            } else {
                // pause
                self.audio_device.pause();
            }
        }
    }

    struct SquareWave {
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
}

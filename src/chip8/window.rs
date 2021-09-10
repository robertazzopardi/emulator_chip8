pub mod window_driver {
    use crate::chip8::processor::chip::Chip8;
    use sdl2::{event::Event, keyboard::Keycode, render::Canvas, video::Window, EventPump, Sdl};

    pub struct Win {
        event_pump: EventPump,
        canvas: Canvas<Window>,
        running: bool,
    }

    impl Win {
        pub const fn is_running(&self) -> bool {
            self.running
        }

        pub fn handle_events(&mut self, chip8: &mut Chip8) {
            for event in self.event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => self.running = false,
                    Event::KeyDown { keycode: code, .. } => {
                        chip8.set_action(code.unwrap(), 1);
                    }
                    Event::KeyUp { keycode: code, .. } => {
                        chip8.set_action(code.unwrap(), 0);
                    }
                    _ => {}
                }
            }
        }

        pub fn draw(&mut self, chip8: &mut Chip8) {
            if chip8.should_draw() {
                self.canvas.clear();
                chip8.update_quads(&mut self.canvas);
                chip8.draw_done();
                self.canvas.present();
            }
        }

        pub fn new(sdl_context: &Sdl) -> Result<Win, String> {
            let window = sdl_context
                .video()?
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

            Ok(Win {
                event_pump: sdl_context.event_pump()?,
                canvas,
                running: true,
            })
        }
    }
}

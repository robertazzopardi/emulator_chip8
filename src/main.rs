use emulator_chip8::emulator_driver;

fn main() -> Result<(), String> {
    // emulator_driver::start(None)
    emulator_driver::start(Some("./roms/pong2.c8"), None)
}

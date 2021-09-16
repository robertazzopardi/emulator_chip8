use emulator_chip8::emulator_driver;

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() <= 1 {
        Err("Path to rom required!".to_string())
    } else {
        emulator_driver::start(Some(args.last().unwrap().as_str()))
    }
}

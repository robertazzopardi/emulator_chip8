pub mod chip_8 {
    use rand::Rng;
    use sdl2::{keyboard::Keycode, rect::Rect, render::Canvas, video::Window};
    use std::fs;

    const CHIP8_FONTSET: [u8; 80] = [
        0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
        0x20, 0x60, 0x20, 0x20, 0x70, // 1
        0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
        0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
        0x90, 0x90, 0xF0, 0x10, 0x10, // 4
        0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
        0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
        0xF0, 0x10, 0x20, 0x40, 0x40, // 7
        0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
        0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
        0xF0, 0x90, 0xF0, 0x90, 0x90, // A
        0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
        0xF0, 0x80, 0x80, 0x80, 0xF0, // C
        0xE0, 0x90, 0x90, 0x90, 0xE0, // D
        0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
        0xF0, 0x80, 0xF0, 0x80, 0x80, // F
    ];

    // Preserve this order
    const KEY_ORDER: [Keycode; 16] = [
        Keycode::X,
        Keycode::Num1,
        Keycode::Num2,
        Keycode::Num3,
        Keycode::Q,
        Keycode::W,
        Keycode::E,
        Keycode::A,
        Keycode::S,
        Keycode::D,
        Keycode::Z,
        Keycode::C,
        Keycode::Num4,
        Keycode::R,
        Keycode::F,
        Keycode::V,
    ];

    macro_rules! skip_instruction {
        ($self:expr, $op:tt, $sh:expr) => {
            if $self.g_reg[$sh as usize] $op ($self.opcode & 0x00FF) as u8 {
                $self.pc += 4;
            } else {
                $self.pc += 2;
            }
        };
        ($self:expr, $op:tt, $sh8:expr, $sh4:expr) => {
            if $self.g_reg[$sh8 as usize] $op $self.g_reg[$sh4 as usize] {
                $self.pc += 4;
            } else {
                $self.pc += 2;
            }
        };
    }

    macro_rules! skip_instruction_key_press {
        ($self:expr, $op:tt, $sh8:expr) => {
            if $self.key[($self.g_reg[$sh8 as usize]) as usize] $op 0 {
                $self.pc += 4;
            } else {
                $self.pc += 2;
            }
        };
    }

    macro_rules! update_register {
        ($self:expr, $op:tt, $left:expr) => {
            $self.g_reg[$left as usize] $op ($self.opcode & 0x00FF) as u8;
            $self.pc += 2;
        };
        ($self:expr, $op:tt, $left:expr, $right:expr) => {
            $self.g_reg[$left as usize] $op $self.g_reg[$right as usize];
            $self.pc += 2;
        };
    }

    pub struct Chip8 {
        opcode: u16,
        memory: [u8; 4096],
        g_reg: [u8; 16],
        ir: u16,
        pc: u16,
        gfx: [u8; 64 * 32],
        delay_timer: u8,
        sound_timer: u8,
        stack: [u16; 16],
        sp: u16,
        key: [u8; 16],
        draw_flag: bool,
    }

    impl Default for Chip8 {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Chip8 {
        pub fn new() -> Chip8 {
            let mut memory: [u8; 4096] = [0; 4096];

            memory[..80].clone_from_slice(&CHIP8_FONTSET[..80]);

            Chip8 {
                opcode: 0,
                memory,
                g_reg: [0; 16],
                ir: 0,
                pc: 0x200,
                gfx: [0; 64 * 32],
                delay_timer: 0,
                sound_timer: 0,
                stack: [0; 16],
                sp: 0,
                key: [0; 16],
                draw_flag: true,
            }
        }

        pub const fn should_play_sound(&self) -> bool {
            self.sound_timer != 0
        }

        pub const fn should_draw(&self) -> bool {
            self.draw_flag
        }

        pub fn draw_done(&mut self) {
            self.draw_flag = false
        }

        pub fn load(&mut self, path: &str) -> Result<(), String> {
            let data = fs::read(path).expect("Unable to read file");
            if (4096 - 512) > data.len() {
                self.memory[512..512 + data.len()].clone_from_slice(&data[..data.len()]);
                Ok(())
            } else {
                // too large
                Err("Could Not Load Rom, It's too large".to_string())
            }
        }

        fn set_register_on_borrow(&mut self, left: u16, right: u16) {
            if self.g_reg[left as usize] > self.g_reg[right as usize] {
                self.g_reg[0xF] = 0;
            } else {
                self.g_reg[0xF] = 1;
            }
        }

        pub fn cycle(&mut self) {
            // Fetch self.opcode
            self.opcode = (self.memory[self.pc as usize] as u16) << 8
                | (self.memory[(self.pc + 1) as usize] as u16);

            let shr8 = (self.opcode & 0x0F00) >> 8;
            let shr4 = (self.opcode & 0x00F0) >> 4;

            // Process self.opcode
            match self.opcode & 0xF000 {
                0x0000 => {
                    match self.opcode & 0x000F {
                        0x0000 => {
                            // 0x00E0=>{ Clears the scree=>{
                            for i in 0..2048 {
                                self.gfx[i] = 0x0;
                            }
                            self.draw_flag = true;
                            self.pc += 2;
                        }
                        0x000E => {
                            // 0x00EE=>{ Returns from subroutin=>{
                            self.sp -= 1; // 16 levels of stack, decrease stack pointer to prevent overwrite
                            self.pc = self.stack[self.sp as usize]; // Put the stored return address from the stack back into the program counter
                            self.pc += 2; // Don't forget to increase the program counter!
                        }
                        _ => {
                            println!("Unknown opcode [0x0000]=> {}", self.opcode);
                        }
                    }
                }
                0x1000 => {
                    // 0x1NNN=>{ Jumps to address NN=>{
                    self.pc = self.opcode & 0x0FFF;
                }
                0x2000 => {
                    // 0x2NNN=>{ Calls subroutine at NNN=>{
                    self.stack[self.sp as usize] = self.pc; // Store current address in stack
                    self.sp += 1; // Increment stack pointer
                    self.pc = self.opcode & 0x0FFF; // Set the program counter to the address at NNN
                }
                0x3000 => {
                    // 0x3XNN=>{ Skips the next instruction if VX equals N=>{
                    skip_instruction!(self, ==, shr8);
                }
                0x4000 => {
                    // 0x4XNN=>{ Skips the next instruction if VX doesn't equal N=>{
                    skip_instruction!(self, !=, shr8);
                }
                0x5000 => {
                    // 0x5XY0=>{ Skips the next instruction if VX equals VY=>{
                    skip_instruction!(self, ==, shr8, shr4);
                }
                0x6000 => {
                    // 0x6XNN=>{ Sets VX to NN=>{
                    update_register!(self, =, shr8);
                }
                0x7000 => {
                    // 0x7XNN=>{ Adds NN to VX=>{
                    update_register!(self, +=, shr8);
                }
                0x8000 => {
                    match self.opcode & 0x000F {
                        0x0000 => {
                            // 0x8XY0=>{ Sets VX to the value of self.g_reg=>{
                            update_register!(self, =, shr8, shr4);
                        }
                        0x0001 => {
                            // 0x8XY1=>{ Sets VX to "VX OR VY=>{
                            update_register!(self, |=, shr8, shr4);
                        }
                        0x0002 => {
                            // 0x8XY2=>{ Sets VX to "VX AND VY=>{
                            update_register!(self, &=, shr8, shr4);
                        }
                        0x0003 => {
                            // 0x8XY3=>{ Sets VX to "VX XOR VY=>{
                            update_register!(self, ^=, shr8, shr4);
                        }
                        0x0004 => {
                            // 0x8XY4=>{ Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there isn'=>{
                            if self.g_reg[shr4 as usize] > (0xFF - self.g_reg[shr8 as usize]) {
                                self.g_reg[0xF] = 1;
                            } else {
                                self.g_reg[0xF] = 0;
                            }
                            update_register!(self, +=, shr8, shr4);
                        }
                        0x0005 => {
                            // 0x8XY5=>{ VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there isn'=>{
                            self.set_register_on_borrow(shr4, shr8);
                            update_register!(self, -=, shr8, shr4);
                        }
                        0x0006 => {
                            // 0x8XY6=>{ Shifts VX right by one. VF is set to the value of the least significant bit of VX before the shif=>{
                            self.g_reg[0xF] = self.g_reg[shr8 as usize] & 0x1;
                            self.g_reg[shr8 as usize] >>= 1;
                            self.pc += 2;
                        }
                        0x0007 => {
                            // 0x8XY7=>{ Sets VX to VY minus VX. VF is set to 0 when there's a borrow, and 1 when there isn'=>{
                            self.set_register_on_borrow(shr8, shr4);
                            self.g_reg[shr8 as usize] =
                                self.g_reg[shr4 as usize] - self.g_reg[shr8 as usize];
                            self.pc += 2;
                        }
                        0x000E => {
                            // 0x8XYE=>{ Shifts VX left by one. VF is set to the value of the most significant bit of VX before the shif=>{
                            self.g_reg[0xF] = self.g_reg[shr8 as usize] >> 7;
                            self.g_reg[shr8 as usize] <<= 1;
                            self.pc += 2;
                        }
                        _ => {
                            println!("Unknown opcode [0x8000]=> {}", self.opcode);
                        }
                    }
                }
                0x9000 => {
                    // 0x9XY0=>{ Skips the next instruction if VX doesn't equal self.g_reg=>{
                    skip_instruction!(self, !=, shr8, shr4);
                }
                0xA000 => {
                    // ANNN=>{ Sets I to the address NN=>{
                    self.ir = (self.opcode & 0x0FFF) as u16;
                    self.pc += 2;
                }
                0xB000 => {
                    // BNNN=>{ Jumps to the address NNN plus self.g_reg=>{
                    self.pc = (self.opcode & 0x0FFF) + self.g_reg[0] as u16;
                }
                0xC000 => {
                    // CXNN=>{ Sets VX to a random number and N=>{
                    self.g_reg[shr8 as usize] =
                        (rand::thread_rng().gen::<u8>() % 0xFF) & (self.opcode & 0x00FF) as u8;
                    self.pc += 2;
                }
                0xD000 => {
                    // DXYN=>{ Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels and a height of N pixels=>{
                    // Each row of 8 pixels is read as bit-coded starting from memory location I;
                    // I value doesn't change after the execution of this instruction.
                    // VF is set to 1 if any screen pixels are flipped from set to unset when the sprite is drawn,
                    // and to 0 if that doesn't happen

                    let x = self.g_reg[shr8 as usize] as u16;
                    let y = self.g_reg[shr4 as usize] as u16;
                    let height = (self.opcode & 0x000F) as u16;

                    self.g_reg[0xF] = 0;

                    for yline in 0..height {
                        let pixel = self.memory[(self.ir + yline) as usize] as u16;
                        for xline in 0..8 {
                            if (pixel & (0x80 >> xline)) != 0 {
                                let index = (x + xline + ((y + yline) * 64)) as usize;
                                if index < self.gfx.len() {
                                    if self.gfx[index] == 1 {
                                        self.g_reg[0xF] = 1;
                                    }
                                    self.gfx[index] ^= 1;
                                }
                            }
                        }
                    }

                    self.draw_flag = true;
                    self.pc += 2;
                }
                0xE000 => {
                    match self.opcode & 0x00FF {
                        0x009E => {
                            // EX9E=>{ Skips the next instruction if the key stored in VX is presse=>{
                            skip_instruction_key_press!(self, !=, shr8);
                        }
                        0x00A1 => {
                            // EXA1=>{ Skips the next instruction if the key stored in VX isn't presse=>{
                            skip_instruction_key_press!(self, ==, shr8);
                        }
                        _ => {
                            println!("Unknown opcode [0xE000]=> {}", self.opcode);
                        }
                    }
                }
                0xF000 => {
                    match self.opcode & 0x00FF {
                        0x0007 => {
                            // FX07=>{ Sets VX to the value of the delay time=>{
                            self.g_reg[shr8 as usize] = self.delay_timer;
                            self.pc += 2;
                        }
                        0x000A => {
                            // FX0A=>{ A key press is awaited, and then stored in self.g_reg=>{

                            let mut key_press = false;

                            for i in 0..16 {
                                if self.key[i] != 0 {
                                    self.g_reg[shr8 as usize] = i as u8;
                                    key_press = true;
                                }
                            }

                            // If we didn't received a keypress, skip this cycle and try again.
                            if !key_press {
                                return;
                            }

                            self.pc += 2;
                        }

                        0x0015 => {
                            // FX15=>{ Sets the delay timer to self.g_reg=>{
                            self.delay_timer = self.g_reg[shr8 as usize];
                            self.pc += 2;
                        }

                        0x0018 => {
                            // FX18=>{ Sets the sound timer to self.g_reg=>{
                            self.sound_timer = self.g_reg[shr8 as usize];
                            self.pc += 2;
                        }
                        0x001E => {
                            // FX1E=>{ Adds VX to =>{
                            if self.ir + self.g_reg[shr8 as usize] as u16 > 0xFFF {
                                // VF is set to 1 when range overflow (I+VX>0xFFF), and 0 when there isn't.
                                self.g_reg[0xF] = 1;
                            } else {
                                self.g_reg[0xF] = 0;
                            }
                            self.ir += self.g_reg[shr8 as usize] as u16;
                            self.pc += 2;
                        }
                        0x0029 => {
                            // FX29=>{ Sets I to the location of the sprite for the character in VX. Characters 0-F (in hexadecimal) are represented by a 4x5 fon=>{
                            self.ir = (self.g_reg[shr8 as usize] * 0x5) as u16;
                            self.pc += 2;
                        }
                        0x0033 => {
                            // FX33=>{ Stores the Binary-coded decimal representation of VX at the addresses I, I plus 1, and I plus =>{
                            self.memory[self.ir as usize] = self.g_reg[shr8 as usize] / 100;
                            self.memory[(self.ir + 1) as usize] =
                                (self.g_reg[shr8 as usize] / 10) % 10;
                            self.memory[(self.ir + 2) as usize] =
                                (self.g_reg[shr8 as usize] % 100) % 10;
                            self.pc += 2;
                        }
                        0x0055 => {
                            // FX55=>{ Stores V0 to VX in memory starting at address =>{
                            for i in 0..=shr8 {
                                self.memory[(self.ir + i) as usize] = self.g_reg[i as usize];
                            }

                            // On the original interpreter, when the operation is done, self.ir = self.ir + X + 1.
                            self.ir += shr8 + 1u16;
                            self.pc += 2;
                        }
                        0x0065 => {
                            // FX65=>{ Fills V0 to VX with values from memory starting at address =>{
                            for i in 0..=shr8 {
                                self.g_reg[i as usize] = self.memory[(self.ir + i) as usize];
                            }

                            // On the original interpreter, when the operation is done, self.ir = self.ir + X + 1.
                            self.ir += shr8 + 1u16;
                            self.pc += 2;
                        }
                        _ => {
                            println!("Unknown opcode [0xF000]=> {}", self.opcode);
                        }
                    }
                }
                _ => {
                    println!("Unknown opcode: {}", self.opcode);
                }
            }

            // Update timers
            if self.delay_timer > 0 {
                self.delay_timer -= 1;
            }

            if self.sound_timer > 0 {
                if self.sound_timer == 1 {
                    println!("BEEP!");
                }
                self.sound_timer -= 1;
            }
        }

        pub fn update_quads(&self, canvas: &mut Canvas<Window>) {
            canvas.set_draw_color(sdl2::pixels::Color::BLACK);

            let size = canvas.window().size();
            let _background = canvas.fill_rect(Rect::new(0, 0, size.0, size.1));

            for y in 0..32 {
                let y_offset = y * 64;
                for x in 0..64 {
                    if self.gfx[y_offset + x] == 1 {
                        canvas.set_draw_color(sdl2::pixels::Color::WHITE);

                        let _pixel =
                            canvas.fill_rect(Rect::new((x * 20) as i32, (y * 20) as i32, 20, 20));
                    }
                }
            }
        }

        pub fn set_action(&mut self, code: Keycode, on_or_off: u8) {
            for (i, keycode) in KEY_ORDER.into_iter().enumerate() {
                if code == keycode {
                    self.key[i] = on_or_off;
                }
            }
        }
    }
}

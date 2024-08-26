use minifb::{Key, Scale, Window, WindowOptions};
use std::{fs, io, time};
use std::thread::sleep;
use std::time::{Duration, Instant};
use rand::random;

const FONT_SET: [u8; 80] = [
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

const WINDOW_WIDTH: usize = 64;
const WINDOW_HEIGHT: usize = 32;


struct Chip8 {
    i: u16,
    pc: u16,
    sp: u16,
    v: [u8; 16],
    ram: [u8; 4096],
    stack: [u16; 16],
    sound_timer: u8,
    delay_timer: u8,
    vram: [[u8; WINDOW_WIDTH / 8]; WINDOW_HEIGHT],
    window: Window,
    is_dirty: bool,
}

fn main() -> io::Result<()> {
    let rom = fs::read("roms/tetris.rom")?;

    let window_options = WindowOptions {
        scale: Scale::X16,
        ..WindowOptions::default()
    };

    let window = Window::new(
        "Chip8",
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
        window_options
    ).unwrap_or_else(|e| panic!("{}", e));

    let mut chip = Chip8 {
        i: 0,
        pc: 0x200,
        sp: 0,
        v: [0; 16],
        ram: [0; 4096],
        stack: [0; 16],
        sound_timer: 0,
        delay_timer: 0,
        vram: [[0; WINDOW_WIDTH / 8]; WINDOW_HEIGHT],
        window,
        is_dirty: true
    };

    // Load Font into memory
    chip.ram[0..FONT_SET.len()].copy_from_slice(&FONT_SET);

    // Load Rom into memory
    chip.ram[0x200..(rom.len() + 0x200)].copy_from_slice(&rom);

    let instruction_duration = Duration::from_millis(1);
    let timer_duration = Duration::from_millis(16);

    let mut last_instruction_time = Instant::now();
    let mut last_timer_update = Instant::now();

    while chip.window.is_open() && !chip.window.is_key_down(Key::Escape){

        chip.cycle();

        if chip.is_dirty {
            chip.window.update_with_buffer(&chip.vram_to_buffer(), WINDOW_WIDTH, WINDOW_HEIGHT).unwrap();
            chip.is_dirty = false;
        } else {
            chip.window.update();
        }

        let elapsed = last_instruction_time.elapsed();
        if elapsed < instruction_duration {
            sleep(instruction_duration - elapsed);
        }
        last_instruction_time = Instant::now();

        if last_timer_update.elapsed() > timer_duration {
            if chip.delay_timer > 0 {
                chip.delay_timer -= 1;
            }

            if chip.sound_timer > 0 {
                chip.sound_timer -= 1;
            }
            last_timer_update = Instant::now();
        }
    }

    Ok(())
}

impl Chip8 {
    fn cycle(&mut self) {
        let opcode =
            (self.ram[self.pc as usize] as u16) << 8 | self.ram[(self.pc + 1) as usize] as u16;
        self.pc += 2;
        self.exec_opcode(opcode);
        sleep(Duration::from_millis(2))
    }

    fn exec_opcode(&mut self, opcode: u16) {

        let nibbles = (
            ((opcode & 0xF000) >> 12) as u8,
            ((opcode & 0x0F00) >> 8) as u8,
            ((opcode & 0x00F0) >> 4) as u8,
            (opcode & 0x000F) as u8,
        );

        let x = nibbles.1 as usize;
        let y = nibbles.2 as usize;
        let n = nibbles.3;
        let nn = (opcode & 0x00FF) as u8;
        let nnn = opcode & 0x0FFF;

        match nibbles {
            (0x0, 0x0, 0xE, 0x0) => self.op_00e0(),
            (0x0, 0x0, 0xE, 0xE) => self.op_00ee(),
            (0x1, _, _, _) => self.op_1nnn(nnn),
            (0x2,_,_,_) => self.op_2nnn(nnn),
            (0x3,_,_,_) => self.op_3xnn(x, nn),
            (0x4,_,_,_) => self.op_4xnn(x, nn),
            (0x5,_,_,0x0) => self.op_5xy0(x, y),
            (0x6, _, _, _) => self.op_6xnn(x, nn),
            (0x7, _, _, _) => self.op_7xnn(x, nn),
            (0x8,_,_,0x0) => self.op_8xy0(x,y),
            (0x8,_,_,0x1) => self.op_8xy1(x,y),
            (0x8,_,_,0x2) => self.op_8xy2(x,y),
            (0x8,_,_,0x3) => self.op_8xy3(x,y),
            (0x8,_,_,0x4) => self.op_8xy4(x,y),
            (0x8,_,_,0x5) => self.op_8xy5(x,y),
            (0x8,_,_,0x6) => self.op_8xy6(x,y),
            (0x8,_,_,0x7) => self.op_8xy7(x,y),
            (0x8,_,_,0xE) => self.op_8xye(x,y),
            (0x9,_,_,0x0) => self.op_9xy0(x, y),
            (0xA, _, _, _) => self.op_annn(nnn),
            (0xB, _, _, _) => self.op_bnnn(nnn),
            (0xC, _, _, _) => self.op_cxnn(x,nn),
            (0xD, _, _, _) => self.op_dxyn(x, y, n),
            (0xE, _, 0x9, 0xE) => self.op_ex9e(x),
            (0xE, _, 0xA, 0x1) => self.op_exa1(x),
            (0xF, _, 0x0, 0x7) => self.op_fx07(x),
            (0xF, _, 0x1, 0x5) => self.op_fx15(x),
            (0xF, _, 0x1, 0x8) => self.op_fx18(x),
            (0xF, _, 0x1, 0xE) => self.op_fx1e(x),
            (0xF, _, 0x0, 0xA) => self.op_fx0a(x),
            (0xF, _, 0x2, 0x9) => self.op_fx29(x),
            (0xF, _, 0x3, 0x3) => self.op_fx33(x),
            (0xF, _, 0x5, 0x5) => self.op_fx55(x),
            (0xF, _, 0x6, 0x5) => self.op_fx65(x),
            _ => {},
        }
    }

    fn op_00e0(&mut self) {
        self.vram = [[0; 8]; 32];
        self.is_dirty = true;
    }

    fn op_00ee(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
    }

    fn op_1nnn(&mut self, nnn: u16) {
        self.pc = nnn;
    }

    fn op_2nnn(&mut self, nnn: u16){
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        self.pc = nnn;
    }

    fn op_3xnn(&mut self, x: usize, nn: u8){
        if self.v[x] == nn {
            self.pc += 2;
        }
    }

    fn op_4xnn(&mut self, x: usize, nn: u8){
        if self.v[x] != nn {
            self.pc += 2;
        }
    }

    fn op_5xy0(&mut self, x: usize, y: usize){
        if self.v[x] == self.v[y] {
            self.pc += 2;
        }
    }


    fn op_6xnn(&mut self, x: usize, nn: u8) {
        self.v[x] = nn;
    }

    fn op_7xnn(&mut self, x: usize, nn: u8) {
        self.v[x] = self.v[x].wrapping_add(nn);
    }

    fn op_8xy0(&mut self, x: usize, y: usize){
        self.v[x] = self.v[y];
    }
    fn op_8xy1(&mut self, x: usize, y: usize){
        self.v[x] |= self.v[y]
    }
    fn op_8xy2(&mut self, x: usize, y: usize){
        self.v[x] &= self.v[y]
    }
    fn op_8xy3(&mut self, x: usize, y: usize){
        self.v[x] ^= self.v[y]
    }
    fn op_8xy4(&mut self, x: usize, y: usize){
        let (res, overflow) = self.v[x].overflowing_add(self.v[y]);
        self.v[x] = res;
        self.v[0xf] = if overflow {1} else {0};
    }
    fn op_8xy5(&mut self, x: usize, y: usize){
        let (res, overflow) = self.v[x].overflowing_sub(self.v[y]);
        self.v[x] = res;
        self.v[0xf] = !overflow as u8;
    }
    fn op_8xy6(&mut self, x: usize, y: usize){
        self.v[x] = self.v[y];
        let flag = self.v[x] & 0x1;
        self.v[x] >>= 1;
        self.v[0xf] = flag;
    }
    fn op_8xy7(&mut self, x: usize, y: usize){
        let (res, overflow) = self.v[y].overflowing_sub(self.v[x]);
        self.v[x] = res;
        self.v[0xf] = !overflow as u8;
    }
    fn op_8xye(&mut self, x: usize, y: usize){
        self.v[x] = self.v[y];
        let flag = (self.v[x] & 0x80) >> 7;
        self.v[x] <<= 1;
        self.v[0xf] = flag;
    }

    fn op_9xy0(&mut self, x: usize, y: usize){
        if self.v[x] != self.v[y] {
            self.pc += 2;
        }
    }

    fn op_annn(&mut self, nnn: u16) {
        self.i = nnn;
    }

    fn op_bnnn(&mut self, nnn: u16) {
        self.i = nnn + self.v[0] as u16;
    }

    fn op_cxnn(&mut self, x: usize, nn: u8) {
        self.v[x] = random::<u8>() & nn;
    }

    fn op_dxyn(&mut self, x: usize, y: usize, n: u8) {
        let x_pos = self.v[x] as usize % WINDOW_WIDTH;
        let mut y_pos = self.v[y] as usize % WINDOW_HEIGHT;
        self.v[0xf] = 0;

        for offset in 0..n {
            let sprite_data = self.ram[(self.i + offset as u16) as usize];

            for bit in 0..8 {
                let x_bit = (x_pos + bit) % WINDOW_WIDTH;
                let pixel = (sprite_data >> (7 - bit)) & 1;
                let old_pixel = (self.vram[y_pos][x_bit / 8] >> (7 - (x_bit % 8))) & 1;

                if old_pixel == 1 && pixel == 1 {
                    self.v[0xf] = 1;
                }

                self.vram[y_pos][x_bit / 8] ^= pixel << (7 - (x_bit % 8));
            }

            y_pos = (y_pos + 1) % WINDOW_HEIGHT;
        }

        self.is_dirty = true;
    }

    fn op_ex9e(&mut self, x: usize){
        let state = self.poll_keyboard();
        if state[self.v[x] as usize] {self.pc += 2}
    }

    fn op_exa1(&mut self, x: usize){
        let state = self.poll_keyboard();
        if !state[self.v[x] as usize] {self.pc += 2}
    }

    fn op_fx07(&mut self, x: usize){
        self.v[x] = self.delay_timer;
    }
    fn op_fx15(&mut self, x: usize){
        self.delay_timer = self.v[x];
    }
    fn op_fx18(&mut self, x: usize){
        self.sound_timer = self.v[x];
    }
    fn op_fx1e(&mut self, x: usize){
        self.i += self.v[x] as u16;
    }

    fn op_fx0a(&mut self, x: usize){
        let states = self.poll_keyboard();

        if let Some(i) = states.iter().position(|&state| state) {
            self.v[x] = i as u8;
            dbg!(i);
        } else {
            self.pc -= 2;
        }
    }

    fn op_fx29(&mut self, x: usize){
        self.i = self.v[x] as u16 * 5;
    }

    fn op_fx33(&mut self, x: usize){
        let mut remainder = self.v[x];
        let hundredth = remainder / 100;
        remainder %= 100;
        let tens = remainder / 10;
        remainder %= 10;
        let ones = remainder;
        self.ram[self.i as usize] = hundredth;
        self.ram[self.i as usize + 1] = tens;
        self.ram[self.i as usize + 2] = ones;
    }

    fn op_fx55(&mut self, x: usize){
        for offset in 0..=x {
            self.ram[(self.i + offset as u16) as usize] = self.v[offset];
        }
    }

    fn op_fx65(&mut self, x: usize){
        for offset in 0..=x {
            self.v[offset] = self.ram[(self.i + offset as u16) as usize];
        }
    }

    fn poll_keyboard(&self) -> [bool; 16] {
        let mut state = [false; 16];

        if self.window.is_key_down(Key::Key1) { state[0x1] = true};
        if self.window.is_key_down(Key::Key2) { state [0x2] = true};
        if self.window.is_key_down(Key::Key3) { state [0x3] = true};
        if self.window.is_key_down(Key::Key4) { state [0xC] = true};
        if self.window.is_key_down(Key::Q) { state [0x4] = true};
        if self.window.is_key_down(Key::W) { state [0x5] = true};
        if self.window.is_key_down(Key::E) { state [0x6] = true};
        if self.window.is_key_down(Key::R) { state [0xD] = true};
        if self.window.is_key_down(Key::A) { state [0x7] = true};
        if self.window.is_key_down(Key::S) { state [0x8] = true};
        if self.window.is_key_down(Key::D) { state [0x9] = true};
        if self.window.is_key_down(Key::F) { state [0xE] = true};
        if self.window.is_key_down(Key::Z) { state [0xA] = true};
        if self.window.is_key_down(Key::X) { state [0x0] = true};
        if self.window.is_key_down(Key::C) { state [0xB] = true};
        if self.window.is_key_down(Key::V) { state [0xF] = true};

        state
    }

    fn vram_to_buffer(&self) -> Vec<u32> {
        let mut buffer = vec![0; WINDOW_WIDTH * WINDOW_HEIGHT];

        for (y, row) in self.vram.iter().enumerate() {
            for (byte_index, &byte) in row.iter().enumerate() {
                for bit in 0..8 {
                    let x = byte_index * 8 + bit;
                    let pixel = (byte >> (7 - bit)) & 1;
                    buffer[y * WINDOW_WIDTH + x] = if pixel == 1 { 0xFFFFFFFF } else { 0xFF000000 };
                }
            }
        }

        buffer
    }
}

use minifb::{Key, Scale, Window, WindowOptions};
use std::{fs, io};

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
    v: [u8; 16],
    ram: [u8; 4096],
    vram: [[u8; WINDOW_WIDTH / 8]; WINDOW_HEIGHT],
}

fn main() -> io::Result<()> {
    let rom = fs::read("roms/IBM Logo.ch8")?;

    let mut chip = Chip8 {
        i: 0,
        pc: 0x200,
        v: [0; 16],
        ram: [0; 4096],
        vram: [[0; WINDOW_WIDTH / 8]; WINDOW_HEIGHT],
    };

    let window_options = WindowOptions {
        scale: Scale::X8,
        ..WindowOptions::default()
    };

    let mut window = Window::new(
        "Chip8",
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
        window_options
    ).unwrap_or_else(|e| panic!("{}", e));

    window.set_target_fps(60);


    // Load Font into memory
    chip.ram[0..FONT_SET.len()].copy_from_slice(&FONT_SET);

    // Load Rom into memory
    chip.ram[0x200..(rom.len() + 0x200)].copy_from_slice(&rom);

    while window.is_open() && !window.is_key_down(Key::Escape){
        chip.run();
        window.update_with_buffer(&chip.vram_to_buffer(), 64, 32).unwrap()
    }


    Ok(())
}

impl Chip8 {
    fn run(&mut self) {
        let opcode =
            (self.ram[self.pc as usize] as u16) << 8 | self.ram[(self.pc + 1) as usize] as u16;
        self.pc += 2;
        self.parse_instruction(opcode);
    }

    fn parse_instruction(&mut self, opcode: u16) {
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
            (0x0, _, 0xE, _) => self.op_00e0(),
            (0x1, _, _, _) => self.op_1nnn(nnn),
            (0x6, _, _, _) => self.op_6xnn(x, nn),
            (0x7, _, _, _) => self.op_7xnn(x, nn),
            (0xA, _, _, _) => self.op_annn(nnn),
            (0xD, _, _, _) => self.op_dxyn(x, y, n),
            val => panic!("unimplemented instruction {:x?}", val),
        }
    }

    fn op_00e0(&mut self) {
        self.vram = [[0; 8]; 32];
    }

    fn op_1nnn(&mut self, nnn: u16) {
        self.pc = nnn;
    }

    fn op_6xnn(&mut self, x: usize, nn: u8) {
        self.v[x] = nn;
    }

    fn op_7xnn(&mut self, x: usize, nn: u8) {
        self.v[x] += nn;
    }

    fn op_annn(&mut self, nnn: u16) {
        self.i = nnn;
    }

    fn op_dxyn(&mut self, x: usize, y: usize, n: u8) {
        let x_pos = self.v[x] as usize % (WINDOW_WIDTH / 8);
        let mut y_pos = self.v[y] as usize % WINDOW_HEIGHT;
        self.v[0xf] = 0;

        for offset in 0..n {
            let sprite_data = self.ram[(self.i + offset as u16) as usize];

            if (self.vram[y_pos][x_pos] & sprite_data) != 0 {
                self.v[0xf] = 1;
            }

            self.vram[y_pos][x_pos] ^= sprite_data;
            y_pos = (y_pos + 1) % WINDOW_HEIGHT;
        }
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

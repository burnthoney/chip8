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
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

struct Chip8 {
    i: u16,
    pc: u16,
    v: [u8; 16],
    memory: [u8; 4096],
    vram: [[u8; 8]; 32],
}

fn main() -> io::Result<()> {
    let rom = fs::read("roms/IBM Logo.ch8")?;

    let mut chip = Chip8 {
        i: 0,
        pc: 0x200,
        v: [0; 16],
        memory: [0; 4096],
        vram: [[0; 8]; 32],
    };

    // Load Font into memory
    chip.memory[0..FONT_SET.len()].copy_from_slice(&FONT_SET);

    // Load Rom into memory
    chip.memory[0x200..(rom.len() + 0x200)].copy_from_slice(&rom);

    chip.run();

    Ok(())
}

impl Chip8 {
    fn run(&mut self) {
        loop {
            let opcode = (self.memory[self.pc as usize] as u16) << 8
                | self.memory[(self.pc+1) as usize] as u16;
            self.pc += 2;
            if self.parse_instruction(opcode) == 1 {
                break;
            };
        }
    }

    fn parse_instruction(&mut self, opcode: u16) -> u8 {
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

        0
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
        let x_pos = self.v[x] as usize % 8;
        let mut y_pos = self.v[y] as usize % 32;
        self.v[0xf] = 0;

        for offset in 0..n {
            let sprite_data = self.memory[(self.i + offset as u16) as usize];

            if (self.vram[y_pos][x_pos] & sprite_data) != 0 {
                self.v[0xf] = 1;
            }

            self.vram[y_pos][x_pos] ^= sprite_data;
            y_pos = (y_pos + 1) % 32;
        }

        self.print_vram();
    }

    fn print_vram(&self) {
        for row in &self.vram {
            for &byte in row {
                for bit in 0..8 {
                    // Extract each bit from the byte, starting with the most significant bit (leftmost pixel)
                    let pixel = (byte >> (7 - bit)) & 1;
                    if pixel == 1 {
                        print!("▀");  // Represent a set pixel with "#"
                    } else {
                        print!(" ");  // Represent an unset pixel with a space
                    }
                }
            }
            println!(); // Move to the next line after each row
        }
    }
}

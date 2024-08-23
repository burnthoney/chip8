use std::{fs, io};
use std::io::Write;

struct Chip8 {
    i: u16,
    pc: u16,
    v: [u8; 16],
    memory: [u8; 4096],
    frame_buffer: [u8; 64 * 32/ 8],
}

fn main() -> io::Result<()> {
    let rom = fs::read("roms/IBM Logo.ch8")?;

    let mut chip = Chip8 {
        i: 0,
        pc: 64,
        v: [0; 16],
        memory: [0; 4096],
        frame_buffer: [0; 64 * 32 / 8],
    };

    // Load Rom into memory
    chip.memory[64..(rom.len() + 64)].copy_from_slice(&rom[..]);

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

        let x = nibbles.1;
        let y = nibbles.2;
        let n = nibbles.3;
        let nn = (opcode & 0x0FF0) as u8;
        let nnn = opcode & 0x0FFF;

        match nibbles {
            (0x0, _, 0xE, _) => self.op_00e0(),
            (0x1, _, _, _) => self.op_1nnn(nnn),
            (0x6, _, _, _) => self.op_6xnn(x, nn),
            (0x7, _, _, _) => self.op_7xnn(x, nn),
            (0xA, _, _, _) => self.op_annn(nnn),
            (0xD, _, _, _) => self.op_dxyn(x, y, n),
            (0,0,0,0) => return 1,
            val => panic!("unimplemented instruction {:x?}", val),
        }

        0
    }

    fn op_00e0(&mut self) {
        self.frame_buffer = [0; 64 * 32 /8];
    }

    fn op_1nnn(&mut self, nnn: u16) {
        self.pc = nnn;
    }

    fn op_6xnn(&mut self, x: u8, nn: u8) {
        self.v[x as usize] = nn;
    }

    fn op_7xnn(&mut self, x: u8, nn: u8) {
        self.v[x as usize] += nn;
    }

    fn op_annn(&mut self, nnn: u16) {
        self.i = nnn;
    }

    fn op_dxyn(&mut self, x: u8, y: u8, n: u8) {
        let mut x_pos = self.v[x as usize] as usize & 64;
        let mut y_pos = self.v[y as usize] as usize & 31;
        self.v[0xf] = 0;

        for i in 0..n {
            let data = self.memory[(self.i + i as u16) as usize];
            if x_pos + (y_pos * 32) >= 256 {
                continue;
            }
            self.frame_buffer[x_pos + (y_pos * 32)] ^= data;
            x_pos += 1;
            y_pos += 1;
            if y_pos > 32 {
                break;
            }
        }
        self.print_screen();
    }

    fn print_screen(&self){
        for y in 0..=32{
            for x in 0..8 {
                if x + (y * 32) >= 256 {
                    continue;
                }
                let mut builder = String::new();
                let bits = self.frame_buffer[x + (y * 32)];
                // Loop over the bits
                for d in 0..8 {
                    let bit_state = bits & 1 << d as u8;
                    if bit_state == 0 {
                        builder += " ";
                    } else {
                        builder += "█";
                    }
                }
                print!("{}", builder);
            }
            println!();
            io::stdout().flush().unwrap();
        }
    }
}

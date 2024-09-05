use rand::random;

use crate::display::{Display, WINDOW_WIDTH, WINDOW_HEIGHT};
use crate::memory::Memory;
use crate::input::Input;
use crate::audio::Audio;

pub struct Cpu {
    i: u16,
    pc: u16,
    sp: u16,
    v: [u8; 16],
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            i: 0,
            pc: 0x200,
            sp: 0,
            v: [0; 16],
        }
    }

    pub fn reset(&mut self) {
        self.i = 0;
        self.pc = 0x200;
        self.sp = 0;
        self.v = [0; 16];
    }

    pub fn step(&mut self, memory: &mut Memory, display: &mut Display, input: &Input, audio: &mut Audio) {
        let opcode =
            (memory.ram[self.pc as usize] as u16) << 8 | memory.ram[(self.pc + 1) as usize] as u16;

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
            (0x0, 0x0, 0xE, 0x0) => self.op_00e0(display),
            (0x0, 0x0, 0xE, 0xE) => self.op_00ee(memory),
            (0x1, _, _, _) => self.op_1nnn(nnn),
            (0x2, _, _, _) => self.op_2nnn(memory, nnn),
            (0x3, _, _, _) => self.op_3xnn(x, nn),
            (0x4, _, _, _) => self.op_4xnn(x, nn),
            (0x5, _, _, 0x0) => self.op_5xy0(x, y),
            (0x6, _, _, _) => self.op_6xnn(x, nn),
            (0x7, _, _, _) => self.op_7xnn(x, nn),
            (0x8, _, _, 0x0) => self.op_8xy0(x, y),
            (0x8, _, _, 0x1) => self.op_8xy1(x, y),
            (0x8, _, _, 0x2) => self.op_8xy2(x, y),
            (0x8, _, _, 0x3) => self.op_8xy3(x, y),
            (0x8, _, _, 0x4) => self.op_8xy4(x, y),
            (0x8, _, _, 0x5) => self.op_8xy5(x, y),
            (0x8, _, _, 0x6) => self.op_8xy6(x, y),
            (0x8, _, _, 0x7) => self.op_8xy7(x, y),
            (0x8, _, _, 0xE) => self.op_8xye(x, y),
            (0x9, _, _, 0x0) => self.op_9xy0(x, y),
            (0xA, _, _, _) => self.op_annn(nnn),
            (0xB, _, _, _) => self.op_bnnn(nnn),
            (0xC, _, _, _) => self.op_cxnn(x, nn),
            (0xD, _, _, _) => self.op_dxyn(memory, display, x, y, n),
            (0xE, _, 0x9, 0xE) => self.op_ex9e(input, x),
            (0xE, _, 0xA, 0x1) => self.op_exa1(input, x),
            (0xF, _, 0x0, 0x7) => self.op_fx07(audio, x),
            (0xF, _, 0x1, 0x5) => self.op_fx15(audio, x),
            (0xF, _, 0x1, 0x8) => self.op_fx18(audio, x),
            (0xF, _, 0x1, 0xE) => self.op_fx1e(x),
            (0xF, _, 0x0, 0xA) => self.op_fx0a(input, x),
            (0xF, _, 0x2, 0x9) => self.op_fx29(x),
            (0xF, _, 0x3, 0x3) => self.op_fx33(memory, x),
            (0xF, _, 0x5, 0x5) => self.op_fx55(memory, x),
            (0xF, _, 0x6, 0x5) => self.op_fx65(memory, x),
            _ => {}
        }

        self.pc += 2;
    }

    pub fn op_00e0(&self, display: &mut Display) {
        display.reset();
    }

    fn op_00ee(&mut self, memory: &Memory) {
        self.sp -= 1;
        self.pc = memory.stack[self.sp as usize];
    }

    fn op_1nnn(&mut self, nnn: u16) {
        self.pc = nnn;
    }

    fn op_2nnn(&mut self, memory: &mut Memory, nnn: u16) {
        memory.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        self.pc = nnn;
    }

    fn op_3xnn(&mut self, x: usize, nn: u8) {
        if self.v[x] == nn {
            self.pc += 2;
        }
    }

    fn op_4xnn(&mut self, x: usize, nn: u8) {
        if self.v[x] != nn {
            self.pc += 2;
        }
    }

    fn op_5xy0(&mut self, x: usize, y: usize) {
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

    fn op_8xy0(&mut self, x: usize, y: usize) {
        self.v[x] = self.v[y];
    }

    fn op_8xy1(&mut self, x: usize, y: usize) {
        self.v[x] |= self.v[y]
    }

    fn op_8xy2(&mut self, x: usize, y: usize) {
        self.v[x] &= self.v[y];
    }

    fn op_8xy3(&mut self, x: usize, y: usize) {
        self.v[x] ^= self.v[y]
    }

    fn op_8xy4(&mut self, x: usize, y: usize) {
        let (res, overflow) = self.v[x].overflowing_add(self.v[y]);
        self.v[x] = res;
        self.v[0xf] = if overflow { 1 } else { 0 };
    }

    fn op_8xy5(&mut self, x: usize, y: usize) {
        let (res, overflow) = self.v[x].overflowing_sub(self.v[y]);
        self.v[x] = res;
        self.v[0xf] = !overflow as u8;
    }

    fn op_8xy6(&mut self, x: usize, y: usize) {
        self.v[x] = self.v[y];
        let flag = self.v[x] & 0x1;
        self.v[x] >>= 1;
        self.v[0xf] = flag;
    }

    fn op_8xy7(&mut self, x: usize, y: usize) {
        let (res, overflow) = self.v[y].overflowing_sub(self.v[x]);
        self.v[x] = res;
        self.v[0xf] = !overflow as u8;
    }

    fn op_8xye(&mut self, x: usize, y: usize) {
        self.v[x] = self.v[y];
        let flag = (self.v[x] & 0x80) >> 7;
        self.v[x] <<= 1;
        self.v[0xf] = flag;
    }

    fn op_9xy0(&mut self, x: usize, y: usize) {
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

    fn op_dxyn(&mut self, memory: &mut Memory, display: &mut Display, x: usize, y: usize, n: u8) {
        let x_pos = self.v[x] as usize % WINDOW_WIDTH;
        let mut y_pos = self.v[y] as usize % WINDOW_HEIGHT;
        self.v[0xf] = 0;

        for offset in 0..n {
            let sprite_data = memory.ram[(self.i + offset as u16) as usize];

            for bit in 0..8 {
                let x_bit = (x_pos + bit) % WINDOW_WIDTH;
                let pixel = (sprite_data >> (7 - bit)) & 1;
                let old_pixel = (display.vram[(y_pos * WINDOW_WIDTH) + (x_bit / 8)] >> (7 - (x_bit % 8))) & 1;

                if old_pixel == 1 && pixel == 1 {
                    self.v[0xf] = 1;
                }

                display.vram[(y_pos * WINDOW_WIDTH) + (x_bit / 8)] ^= pixel << (7 - (x_bit % 8));
            }

            y_pos = (y_pos + 1) % WINDOW_HEIGHT;
        }

        display.is_dirty = true;
    }

    fn op_ex9e(&mut self, input: &Input,x: usize) {
        if input.state[self.v[x] as usize] {
            self.pc += 2
        }
    }


    fn op_exa1(&mut self, input: &Input,x: usize) {
        if !input.state[self.v[x] as usize] {
            self.pc += 2
        }
    }

    fn op_fx07(&mut self, audio: &Audio, x: usize) {
        self.v[x] = audio.delay_timer;
    }

    fn op_fx15(&self, audio: &mut Audio, x: usize) {
        audio.delay_timer = self.v[x];
    }

    fn op_fx18(&self, audio: &mut Audio, x: usize) {
        audio.sound_timer = self.v[x];
    }

    fn op_fx1e(&mut self, x: usize) {
        self.i += self.v[x] as u16;
    }

    fn op_fx0a(&mut self, input: &Input, x: usize) {
        if let Some(i) = input.state.iter().position(|&state| state) {
            self.v[x] = i as u8;
        } else {
            self.pc -= 2;
        }
    }

    fn op_fx29(&mut self, x: usize) {
        self.i = self.v[x] as u16 * 5;
    }

    fn op_fx33(&self, memory: &mut Memory, x: usize) {
        let value = self.v[x];
        memory.ram[self.i as usize] = value / 100;
        memory.ram[self.i as usize + 1] = (value % 100) / 10;
        memory.ram[self.i as usize + 2] = value % 10;
    }

    fn op_fx55(&self, memory: &mut Memory,x: usize) {
        for offset in 0..=x {
            memory.ram[(self.i + offset as u16) as usize] = self.v[offset];
        }
    }

    fn op_fx65(&mut self, memory: &Memory,x: usize) {
        for offset in 0..=x {
            self.v[offset] = memory.ram[(self.i + offset as u16) as usize];
        }
    }
}

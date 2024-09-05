use crate::cpu::Cpu;
use crate::memory::Memory;
use crate::audio::Audio;
use crate::display::Display;
use crate::input::Input;

pub struct Chip8 {
    pub cpu: Cpu,
    pub memory: Memory,
    pub display: Display,
    pub input: Input,
    pub audio: Audio,
    
    debug: bool,
    running: bool,
}

impl Chip8 {
    pub fn new() -> Self {
        Self {
            cpu: Cpu::new(),
            memory: Memory::new(),
            display: Display::new(),
            input: Input::new(),
            audio: Audio::new(),
            
            debug: false,
            running: true,
        }
    }

    pub fn run(&mut self){
        // Timers

        while self.running {
            self.cpu.step(&mut self.memory, &mut self.display, &self.input, &mut self.audio);
            self.input.poll();
            self.display.update();

            self.audio.tick();
        }
    }
}
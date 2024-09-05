pub const WINDOW_WIDTH: usize = 64;
pub const WINDOW_HEIGHT: usize = 32;

pub struct Display {
    pub vram: [u8; (WINDOW_WIDTH / 8) * WINDOW_HEIGHT],
    pub is_dirty: bool,
}

impl Display {
    pub fn new() -> Self {
        Self {
            vram: [0; (WINDOW_WIDTH / 8) * WINDOW_HEIGHT],
            is_dirty: true,
        }
    }

    pub fn reset(&mut self) {
        self.vram = [0; (WINDOW_WIDTH / 8) * WINDOW_HEIGHT];
        self.is_dirty = true;
    }

    pub fn update(&self){
        if !self.is_dirty {return;}
        
        todo!("Update display")
    }
}

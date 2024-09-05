pub struct Audio {
    pub sound_timer: u8,
    pub delay_timer: u8,
}

impl Audio {
    pub fn new() -> Self {
        Self {
            sound_timer: 0,
            delay_timer: 0,
        }
    }
    
    pub fn reset(&mut self) {
        self.sound_timer = 0;
        self.delay_timer = 0;
    }
    
    pub fn tick(&mut self){
        if self.sound_timer > 0 {
            self.sound_timer -= 1;       
        }
        
        if self.delay_timer > 0 {
            self.delay_timer -= 2;
        }
    }
}

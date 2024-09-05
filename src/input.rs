pub struct Input {
    pub state: [bool; 16],
}

impl Input {
    pub fn new() -> Self {
        Self { state: [false; 16] }
    }
    
    pub fn poll(&mut self){}
}

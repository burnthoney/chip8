use iced::{widget::{button, Column, Text}, Sandbox, };

use crate::chip8;

pub struct App {
    chip: chip8::Chip8,
}

#[derive(Debug, Clone)]
pub enum Message {
    Quit,
}

impl Sandbox for App {
    type Message = Message;
    
    fn new() -> Self {
        Self {
            chip: chip8::Chip8::new(),
        }
    }

    fn title(&self) -> String {
        String::from("Chip8 Emulator")
    }

    fn update(&mut self, message: Message){
        match message {
            Message::Quit => {
                // Do Something
            }
        }
    }

    fn view(&self) -> iced::Element<Message> {
        let button = button::Button::new(Text::new("Quit"))
            .on_press(Message::Quit);

        Column::new().push(button).into()
    }   
}
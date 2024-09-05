use iced::Sandbox;

mod audio;
mod cpu;
mod display;
mod input;
mod memory;
mod chip8;
mod app;

fn main() {
    _ = app::App::run(iced::Settings::default());
}
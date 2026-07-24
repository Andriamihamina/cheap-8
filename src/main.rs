use crate::core::{cpu::CPU, io::{keyboard::MacroquadKeyboard, screen::MacroquadRenderer}};
pub mod core;

#[macroquad::main("Chip-8")]

async fn main() {
    let mut cpu = CPU::new(MacroquadKeyboard::new(), MacroquadRenderer::new());
    let is_load_ok =cpu.load_program("C:/Users/sebas/Documents/dev/cheap-8/IBM Logo.ch8");

    cpu.run().await;
}

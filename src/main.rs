use crate::cpu::CPU;

pub mod cpu;

fn main() {
    let mut cpu = CPU::new();
    let is_load_ok =cpu.load_program("C:/Users/sebas/Documents/dev/cheap-8/IBM Logo.ch8");

    match is_load_ok {
        Ok(_) => println!("Program loaded into memory"),
        Err(_) => println!("Couldn't load the program into memory")
    }
}

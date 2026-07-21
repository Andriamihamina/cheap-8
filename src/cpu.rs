pub mod decoding;
use std::{fs::File, io::{self, BufReader, Read}};

pub struct CPU{
    memory: [u8; 4096],
    v: [u8; 16],
    i: u16,
    sp: u8,
    pc: u16,

    stack: [u16; 16],

    delay_timer: u8,
    sound_timer: u8
    
}

impl CPU {

    pub fn load_program(&mut self, path: &str) -> io::Result<()>{ // TODO error handling
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut address = 0x200;

        for byte_result in reader.bytes() {
            let byte: u8 = byte_result?;
            self.memory[address] = byte;
            address+=1;
            println!("{:x}", byte)
        }

        Ok(())
        
    }

    pub fn new() -> Self {
        Self{
            memory: [0; 4096],
            v: [0; 16],
            i: 0,
            sp: 0,
            stack: [0; 16],
            pc: 0,
            delay_timer: 0,
            sound_timer: 0,
        }
    }

}
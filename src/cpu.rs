use std::{fs::File, io::{self, BufReader, Read}};
use crate::{keyboard::Keyboard, screen::Screen};

pub struct CPU<T: Keyboard>{
    memory: [u8; 4096],
    v: [u8; 16],
    i: u16,
    sp: u8,
    pc: u16,
    screen: Screen,
    keyboard: T,
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
            screen: Screen::new(),
            delay_timer: 0,
            sound_timer: 0,
        }
    }

    fn get_stack_addr(&self, addr: u8) -> u16{
        self.stack[usize::from(addr)]
    }

    fn get_stack_top(&self) -> u16{
        self.get_stack_addr(self.sp)
    }


    fn decode(&mut self, opcode:u16){
        let instruction = (opcode & 0xF000) >> 12;
        let x = ((opcode & 0x0F00) >> 8) as u8;
        let y = ((opcode & 0x00F0) >> 4) as u8;
        let ls = (opcode & 0x000F) as u8;

        let vx = self.get_register(x);
        let vy = self.get_register(y);

        let nnn = opcode & 0x0FFF;
        let kk = opcode & 0x00FF;
        



        match (instruction, x, y, ls) {
            (0, 0, 0xE, 0) => {
                self.screen.clear();
            }
            (0, 0, 0xE, 0xE) => {
                self.ret();
            }
            (1, _, _, _) => {
                self.jp(nnn);
            } 
            
            (2, _, _, _) => {
                self.call(nnn);
            }

            (3, _, _, _) => { // 3xkk skip if Vx == kk
                self.skip_equal(vx as u16, kk);
            }

            (4, _, _, _) => { // skip if vx != kk
                self.skip_not_equal(vx as u16, kk);
            }

            (5, _,  _, _) => {// Skip if vx == vy
                self.skip_equal(self.get_register(x.into()) as u16, self.get_register(y.into()) as u16);

            }

            (6, _, _, _) => {//
                self.set_register(x.into(), kk as u8);
            }

            (7, _, _, _) => {
                self.set_register(x.into(), (kk + vx as u16) as u8);
            }

            (8, _, _, 0) => {
                self.set_register(x.into(), vy);
            }
            (8, _, _, 1) => {
                self.set_register(x.into(), vx | vy);
            }
            (8, _, _, 2) => {
                self.set_register(x.into(), vx & vy);
            }
            (8, _, _, 3) => {
                self.set_register(x.into(), vx ^ vy);
            }
            (8, _, _, 4) => {
                self.set_register(x.into(), vx + vy);
            }
            (8, _, _, 5) => {
                self.set_register(x.into(), vx.wrapping_sub(vy)); 
                self.set_register(0xF, if vx >= vy {1} else {0}); 
            }
            (8, _, _, 6) => {
                self.set_register(0xF, vx & 0x1);
                self.set_register(x.into(), vx >> 1);
            }
            (8, _, _, 7) => {
                self.set_register(x.into(), vy.wrapping_sub(vx)); 
                self.set_register(0xF, if vy >= vx {1} else {0}); 

            }
            (8, _, _, 0xE) => {
                self.set_register(0xF, vx & 0x1);
                self.set_register(x.into(), vx << 1);
            }
            (9, _, _, 0) => {
                self.skip_not_equal(vx as u16, vy as u16);
            }
            (0xA, _, _, _) => {
                self.i = nnn;
            }
            (0xB, _, _, _) => {
                self.jp(nnn + self.get_register(0) as u16);
            }
            (0xC, x, _, _) => {
                let nb = rand::random_range(0..=255);
                self.set_register(x, x & nb);
            }

            (0xD, x, y, n) => {
                self.display(x, y, n);
            }

            _ => (),
        }
    }

    fn cls(){
        //clear display
    }

    fn ret(&mut self){
        self.pc = self.stack[self.sp as usize];
        self.sp-=1;
    }

    fn jp(&mut self, target: u16){
        self.pc = target;
    }
    
    fn call(&mut self, nnn: u16){
        self.sp += 1;
        self.stack[self.sp as usize] = self.pc;
        self.pc = nnn;
    }

    fn skip_equal(&mut self, a: u16, b: u16){//TODO unit test
        if a == b { self.pc += 2} //TODO maybe increment by 1 if it's not same
    }

    //4xkk SNE Vx, byte
    //9xy0 SNE Vx, Vy
    fn skip_not_equal(&mut self, a: u16, b: u16){
        if a != b { self. pc += 2}
    }

    fn get_register(&self, x:u8) -> u8{
        self.v[usize::from(x)]
    }

    //Cxkk - Vx = random AND kkk
    fn set_register(&mut self, x: u8, value: u8){
        self.v[usize::from(x)] = value;
    }
    
    
    fn set_i(&mut self, value: u16){
        self.i = value;
    }

    //Dxyn
    fn display(&mut self, x: u8, y:u8, n:u8){
        let start = usize::from(self.i);
        let end = start + usize::from(n);
        let buffer = &self.memory[start..end]; 

        let x = self.v[x as usize];
        let y = self.v[y as usize];

        for i in y..(y+n){
            let sprite_byte = buffer[usize::from(i)];

            for bit in 0..8 {

                let (x, y) = ((x + bit) % 32, i % 64);


                let sprite_bit = ((sprite_byte >> (7 - bit)) & 1) != 0;
                let screen_bit = self.screen.get_pixel((x,y));

                self.screen.set_pixel((x,y), screen_bit ^ sprite_bit);

            }
        }

    }



    
}

#[cfg(test)]
mod test{

    use std::array;


use super::*;

    #[test]
    fn test_ret(){
        let mut cpu = CPU::new();
        cpu.stack = std::array::from_fn(|i| i as u16); 
        cpu.sp = 5;
        cpu.ret();
        //cpu.pc = cpu.stack[cpu.sp as usize];
        //cpu.sp -= 1;
        assert_eq!(cpu.pc, 5);
        assert_eq!(cpu.sp, 4);
    }

    #[test]
    fn test_jp(){
        let mut cpu = CPU::new();
        let target = 0x0400u16;
        cpu.jp(target);
        assert_eq!(cpu.pc, target)
    }

    #[test]
    fn test_call(){
        let mut cpu = CPU::new();
        let nnn = 7;
        cpu.stack = std::array::from_fn(|i| if  i <= 5 {i as u16} else {0}); //[0, 1, 2, 3, 4, 5, 0, 0, ...]
        cpu.sp = 5;
        assert_eq!(cpu.get_stack_top(), cpu.get_stack_addr(5));
        cpu.call(nnn);

        assert_eq!(cpu.sp, 6);  
        assert_eq!(cpu.get_stack_top(), cpu.get_stack_addr(6));
        assert_eq!(cpu.pc, nnn)

    }

    #[test]
    fn test_dxyn() {//TODO test more cases. Wrapping around etc
        let mut cpu = CPU::new();
        let opcode = 0xD001;
        cpu.memory[512usize] = 0xFF; 
        cpu.v[0] = 0;
        cpu.v[1] = 0;
        cpu.i = 512;

        cpu.decode(opcode);

        let expected_pixels = [true; 8];
        let screen_pixels = array::from_fn(|x| cpu.screen.get_pixel((x as u8,0)));
        assert_eq!(expected_pixels, screen_pixels);
    }

    #[test]
    fn test_se(){
        let mut cpu = CPU::new();
        cpu.set_register(5, 0x66);
        cpu.decode(0x3566);
        assert_eq!(cpu.pc, 2);

        cpu.pc = 0;
        cpu.set_register(5, 0x67);
        cpu.decode(0x3566);
        assert_eq!(cpu.pc, 0);
    }

    #[test]
    fn test_sne() {
        let mut cpu = CPU::new();
        cpu.set_register(5, 0x66);
        cpu.decode(0x4566);
        assert_eq!(cpu.pc, 0);

        cpu.pc = 0;
        cpu.set_register(5, 0x67);
        cpu.decode(0x4566);
        assert_eq!(cpu.pc, 2);
    }

    #[test]
    fn test_6xnn() {
        let mut cpu = CPU::new();
        let opcode = 0x6464;
        cpu.decode(opcode);
        assert_eq!(cpu.get_register(4), 0x64);
    }

    #[test]
    fn test_7xnn() {
        let mut cpu = CPU::new();
        let opcode = 0x7464;

        cpu.set_register(4, 1);
        cpu.decode(opcode);
        assert_eq!(cpu.get_register(4), 0x64 + 1);
    }

    #[test]
    fn test_8xy5_no_underflow() {
        let mut cpu = CPU::new();
        let opcode = 0x8455;

        cpu.set_register(4, 5);
        cpu.set_register(5, 4);

        cpu.decode(opcode);

        assert_eq!(cpu.get_register(4), 1);
        assert_eq!(cpu.get_register(0xF), 1)
    }

    #[test]
    fn test_8xy5_with_underflow() {
        let mut cpu = CPU::new();
        let opcode = 0x8455;

        cpu.set_register(4, 4);
        cpu.set_register(5, 5);

        cpu.decode(opcode);

        assert_eq!(cpu.get_register(4), 255);
        assert_eq!(cpu.get_register(0xF), 0);
    }

    #[test]
    fn test_8xy6(){
        let mut cpu= CPU::new();
        let opcode: u16 = 0x8456;

        cpu.set_register(4, 0xE1);//0xE1 = 1110 0001
        cpu.decode(opcode);//                       1110 0001 >> 1
        assert_eq!(cpu.get_register(4), 0x70);//    0111 0000 = 0x70
        assert_eq!(cpu.get_register(0xF), 1);

        cpu.set_register(4, 0xE0);//0xE0 = 1110 0000
        cpu.decode(opcode);//                       1110 0000 >> 1
        assert_eq!(cpu.get_register(4), 0x70);//    0111 0000 = 0x70
        assert_eq!(cpu.get_register(0xF), 0);
    }

    #[test]
    fn test_8xy7_no_underflow() {
        let mut cpu = CPU::new();
        let opcode = 0x8457;

        cpu.set_register(4, 4);
        cpu.set_register(5, 5);

        cpu.decode(opcode);

        assert_eq!(cpu.get_register(4), 1);
        assert_eq!(cpu.get_register(0xF), 1)
    }

    #[test]
    fn test_8xy7_with_underflow() {
        let mut cpu = CPU::new();
        let opcode = 0x8457;

        cpu.set_register(4, 5);
        cpu.set_register(5, 4);

        cpu.decode(opcode);

        assert_eq!(cpu.get_register(4), 255);
        assert_eq!(cpu.get_register(0xF), 0)
    }

    #[test]
    fn test_8xye(){
        let mut cpu= CPU::new();
        let opcode: u16 = 0x845E;

        cpu.set_register(4, 0x11);// 0x11 = 0001 0001
        cpu.decode(opcode);//                        0001 0001 << 1
        assert_eq!(cpu.get_register(4), 0x22);//     0010 0010
        assert_eq!(cpu.get_register(0xF), 1);

        cpu.set_register(4, 0x10);// 0x10 = 0001 0000
        cpu.decode(opcode);//                        0001 0000 << 1
        assert_eq!(cpu.get_register(4), 0x20);//     0010 0000
        assert_eq!(cpu.get_register(0xF), 0);
    }

    #[test]
    fn test_9xy0(){
        let mut cpu= CPU::new();
        let opcode: u16 = 0x9430;

        cpu.set_register(4, 5);
        cpu.set_register(3, 5);
        cpu.decode(opcode);
        assert_eq!(cpu.pc, 0);

        cpu.set_register(3, 4);
        cpu.decode(opcode);
        assert_eq!(cpu.pc, 2);
    }

    #[test]
    fn test_annn(){
        let mut cpu= CPU::new();
        let opcode: u16 = 0xA245;

        cpu.decode(opcode);
        assert_eq!(cpu.i, 0x245);
    }

    #[test]
    fn test_bnnn(){
        let mut cpu= CPU::new();
        let opcode: u16 = 0xB245;

        cpu.set_register(0, 1);
        cpu.decode(opcode);
        assert_eq!(cpu.pc, 0x246);
    }

    
} 
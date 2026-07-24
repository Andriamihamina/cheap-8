use std::{fs::File, hint::spin_loop, io::{self, BufReader, Read}, time::{Duration, Instant}};
use macroquad::{window::next_frame};

use crate::core::io::{keyboard::Keyboard, screen::Renderer};

pub struct CPU<K: Keyboard, S: Renderer>{
    memory: [u8; 4096],
    v: [u8; 16],
    i: u16,
    sp: u8,
    pc: u16,
    screen: S,
    keyboard: K,
    stack: [u16; 16],

    delay_timer: u8,
    sound_timer: u8
    
}

impl <K: Keyboard, S: Renderer>CPU<K, S> {

    fn load_fonts(&mut self) {
        let fonts: [u8; 16*5] = [
            0xF0, 0x90, 0x90, 0x90, 0xF0,
            0x20, 0x60, 0x20, 0x20, 0x70,
            0xF0, 0x10, 0xF0, 0x80, 0xF0,
            0xF0, 0x10, 0xF0, 0x10, 0xF0,
            0x90, 0x90, 0xF0, 0x10, 0x10,
            0xF0, 0x80, 0xF0, 0x10, 0xF0,
            0xF0, 0x80, 0xF0, 0x90, 0xF0,
            0xF0, 0x10, 0x20, 0x40, 0x40,
            0xF0, 0x90, 0xF0, 0x90, 0xF0,
            0xF0, 0x90, 0xF0, 0x10, 0xF0,
            0xF0, 0x90, 0xF0, 0x90, 0x90,
            0xE0, 0x90, 0xE0, 0x90, 0xE0,
            0xF0, 0x90, 0xF0, 0x90, 0x90,
            0xE0, 0x90, 0x90, 0x90, 0xE0,
            0xF0, 0x80, 0xF0, 0x80, 0xF0,
            0xF0, 0x80, 0xF0, 0x80, 0x80
        ];

        self.memory[0x50..(0x50 + fonts.len())].copy_from_slice(&fonts);
    }

    fn get_instruction(&self, location: u16) -> u16{
        let location: usize = location.into();
        let high_byte: u8 = self.memory[location]; 
        let low_byte: u8 = self.memory[location + 1]; 

        u16::from_be_bytes([high_byte, low_byte])
    }

    pub async fn run(&mut self) {
        let timer_interval = Duration::from_secs_f64(1. / 60.);
        let mut next_tick = Instant::now() + timer_interval;

        loop {
            self.screen.render();

            for _ in 0..10 {
                let opcode =
                ((self.memory[self.pc as usize] as u16) << 8)
                | self.memory[(self.pc + 1) as usize] as u16;

                self.keyboard.update_state();
                if opcode != 0 {println!("{:03X}: {:04X}", self.pc, opcode)};
                self.decode(self.get_instruction(self.pc));
            }

            let now = Instant::now();

            while now >= next_tick {
                if self.delay_timer > 0 {
                    self.delay_timer -= 1;
                }
            }

            if self.sound_timer > 0 {
                self.sound_timer -= 1;
                //TODO play sound
            }

            next_tick += timer_interval;

            next_frame().await;
            
        }
        
    }

    pub fn load_program(&mut self, path: &str) -> io::Result<()>{ // TODO error handling
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut address = 0x0200;

        self.load_fonts();


        for byte_result in reader.bytes() {
            let byte: u8 = byte_result?;
            self.memory[address] = byte;
            address+=1;

        }

        Ok(()) //TODO handle exceptions
        
    }

    pub fn new(keyboard: K, screen: S) -> Self {
        Self{
            memory: [0; 4096],
            v: [0; 16],
            i: 0,
            sp: 0,
            stack: [0; 16],
            pc: 0x200,
            keyboard,
            screen,
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
        let instruction = ((opcode & 0xF000) >> 12) as u8;
        let x = ((opcode & 0x0F00) >> 8) as u8;
        let y = ((opcode & 0x00F0) >> 4) as u8;
        let ls = (opcode & 0x000F) as u8;

        let vx = self.get_register(x);
        let vy = self.get_register(y);

        let nnn = opcode & 0x0FFF;
        let kk = (opcode & 0x00FF) as u8;
        



        match (instruction, x, y, ls) {
            (0, 0, 0xE, 0) => {
                self.screen.clear();
                self.step()
            }
            (0, 0, 0xE, 0xE) => {
                self.ret();
                self.step()
            }
            (1, _, _, _) => {
                self.jp(nnn);
                println!("target: {:X}", nnn)
            } 
            
            (2, _, _, _) => {
                self.call(nnn);
            }

            (3, _, _, _) => { // 3xkk skip if Vx == kk
                self.skip_equal(vx, kk);
                println!("vx: {:?}, kk: {:?}", vx, kk);
                self.step()
            }

            (4, _, _, _) => { // skip if vx != kk
                self.skip_not_equal(vx, kk);
                self.step()
            }

            (5, _,  _, _) => {// Skip if vx == vy
                self.skip_equal(self.get_register(x.into()), self.get_register(y.into()));
                self.step()

            }

            (6, _, _, _) => {//
                self.set_register(x.into(), kk);
                self.step()
            }

            (7, _, _, _) => {
                self.set_register(x.into(), kk + vx);
                self.step()
            }

            (8, _, _, 0) => {
                self.set_register(x.into(), vy);
                self.step()
            }
            (8, _, _, 1) => {
                self.set_register(x.into(), vx | vy);
                self.step()
            }
            (8, _, _, 2) => {
                self.set_register(x.into(), vx & vy);
                self.step()
            }
            (8, _, _, 3) => {
                self.set_register(x.into(), vx ^ vy);
                self.step()
            }
            (8, _, _, 4) => {
                let (result, carry) = vx.overflowing_add(vy);
                self.set_register(x.into(), result);
                self.set_register(0xF, if  carry { 1 } else {0});

                self.step()
            }
            (8, _, _, 5) => {
                self.set_register(x.into(), vx.wrapping_sub(vy)); 
                self.set_register(0xF, if vx >= vy {1} else {0}); 
                self.step()
            }
            (8, _, _, 6) => {
                self.set_register(0xF, vx & 0x1);
                self.set_register(x.into(), vx >> 1);
                self.step()
            }
            (8, _, _, 7) => {
                self.set_register(x.into(), vy.wrapping_sub(vx)); 
                self.set_register(0xF, if vy >= vx {1} else {0}); 
                self.step()

            }
            (8, _, _, 0xE) => {
                self.set_register(0xF, (vx >> 7) & 1);
                self.set_register(x.into(), vx << 1);
                self.step()
            }
            (9, _, _, 0) => {
                self.skip_not_equal(vx, vy);
                self.step()
            }
            (0xA, _, _, _) => {
                self.i = nnn;
                self.step()
            }
            (0xB, _, _, _) => {
                self.jp(nnn + self.get_register(0) as u16);
            }
            (0xC, x, _, _) => {
                let nb = rand::random_range(0..=255);
                self.set_register(x, nb & kk);
                self.step()
            }

            (0xD, x, y, n) => {
                self.display(x, y, n);
                self.step()
            }

            (0xE, _, 0x9, 0x1) => {
                let is_pressed = self.keyboard.is_key_pressed(vx & 0x0F);//only the lowest nibble
                if is_pressed { self.step() }
                self.step()
            }

            (0xE, _, 0xA, 1) => {
                let is_pressed = self.keyboard.is_key_pressed(vx & 0x0F);//only the lowest nibble
                if !is_pressed { self.step()}                
                self.step()
            }

            (0xF, x, 0, 7) => {
                self.set_register(x, self.delay_timer);
                self.step()
            }

            (0xF, x, 0, 0xA) => {
                match self.keyboard.wait_key_press() {
                    Some(key) => { 
                        self.set_register(x, key);
                        self.step();
                    }
                    None => ()
                }
            }

            (0xF, _, 1, 5) => {
                self.delay_timer = vx;
                self.step()
            }

            (0xF, _, 1, 8) => {
                self.sound_timer = vx;
                self.step()
            }

            (0xF, _, 1, 0xE) => {
                self.i += vx as u16;
                self.step()
            }

            (0xF, _, 2, 9) => {
                self.i = (vx as u16) * 5 + 0x50;
                self.step()
            }

            (0xF, _, 3, 3) => {
                self.set_memory(self.i, (vx / 100) % 10);
                self.set_memory(self.i + 1, (vx / 10) % 10);
                self.set_memory(self.i + 2, vx  % 10);
                self.step()
            }

            (0xF, _, 5, 5) => {

                for i in 0..=x {
                    self.set_memory(self.i + i as u16, self.get_register(i));
                }

                self.step()
            }

            (0xF, _, 6, 5) => {

                let mut adress = self.i as u16;
                for i in 0..=x {
                    self.set_register(i, self.get_memory(adress));
                    adress += 1;
                }

                self.step()
            }

            _ => self.step()
        }
    }

    fn step(&mut self) { self.pc += 2}

    fn clear(&mut self){
        self.screen.clear();
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

    fn skip_equal(&mut self, a: u8, b: u8){//TODO unit test
        if a == b { self.step()} //TODO maybe increment by 1 if it's not same
    }

    //4xkk SNE Vx, byte
    //9xy0 SNE Vx, Vy
    fn skip_not_equal(&mut self, a: u8, b: u8){
        if a != b { self. pc += 2}
    }

    fn get_register(&self, x:u8) -> u8{
        self.v[usize::from(x)]
    }

    //Cxkk - Vx = random AND kkk
    fn set_register(&mut self, x: u8, value: u8){
        self.v[usize::from(x)] = value;
    }

    fn set_memory(&mut self, adress: u16, value: u8) {
        self.memory[usize::from(adress)] = value;
    }

    fn get_memory(&self, adress: u16) -> u8 {
        self.memory[usize::from(adress)]
    }
    
    
    fn set_i(&mut self, value: u16){
        self.i = value;
    }

    //Dxyn
    fn display(&mut self, x: u8, y:u8, n:u8){
        let start = usize::from(self.i);
        let end = start + usize::from(n);
        let buffer = &self.memory[start..end].to_vec(); 

        let x = self.get_register(x);
        let y = self.get_register(y);

        self.set_register(0xF, 1);


        for (sprite_row, sprite_byte) in buffer.iter().enumerate() {
            for bit in 0..8 {
                let (x, y) = ((x + bit) % 64, (y + sprite_row as u8) % 32);

                let sprite_bit = ((sprite_byte >> (7 - bit)) & 1) != 0;

                if sprite_bit {
                    let screen_bit = self.screen.get_pixel((x, y));
                    if screen_bit {
                        self.set_register(0xF, 1);
                    }
                    self.screen.set_pixel((x, y), !screen_bit);
                }

            }
        }

    }



    
}

#[cfg(test)]
mod test{

use crate::core::io::{keyboard::MacroquadKeyboard, screen::{MacroquadRenderer}};

use super::*;

    #[test]
    fn test_ret(){
        let mut cpu = CPU::new(MacroquadKeyboard::new(), MacroquadRenderer::new());
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
        let mut cpu = CPU::new(MacroquadKeyboard::new(), MacroquadRenderer::new());
        let target = 0x0400u16;
        cpu.jp(target);
        assert_eq!(cpu.pc, target)
    }

    #[test]
    fn test_call(){
        let mut cpu = CPU::new(MacroquadKeyboard::new(), MacroquadRenderer::new());
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
        let mut cpu = CPU::new(MacroquadKeyboard::new(), MacroquadRenderer::new());
        let opcode = 0xD001;
        cpu.memory[512usize] = 0xFF; 
        cpu.v[0] = 0;
        cpu.v[1] = 0;
        cpu.i = 512;

        cpu.decode(opcode);

        let expected_pixels = [true; 8];
        let screen_pixels = std::array::from_fn(|x| cpu.screen.get_pixel((x as u8,0)));
        assert_eq!(expected_pixels, screen_pixels);
    }

    #[test]
    fn test_se(){
        let mut cpu = CPU::new(MacroquadKeyboard::new(), MacroquadRenderer::new());
        cpu.set_register(5, 0x66);
        cpu.decode(0x3566);
        assert_eq!(cpu.pc, 0x200 + 4);

        cpu.pc = 0x200;
        cpu.set_register(5, 0x67);
        cpu.decode(0x3566);
        assert_eq!(cpu.pc, 0x200 + 2);
    }

    #[test]
    fn test_sne() {
        let mut cpu = CPU::new(MacroquadKeyboard::new(), MacroquadRenderer::new());
        cpu.set_register(5, 0x66);
        cpu.decode(0x4566);
        assert_eq!(cpu.pc, 0x200 + 2);

        cpu.pc = 0x200;
        cpu.set_register(5, 0x67);
        cpu.decode(0x4566);
        assert_eq!(cpu.pc, 0x200 + 4);
    }

    #[test]
    fn test_6xnn() {
        let mut cpu = CPU::new(MacroquadKeyboard::new(), MacroquadRenderer::new());
        let opcode = 0x6464;
        cpu.decode(opcode);
        assert_eq!(cpu.get_register(4), 0x64);
    }

    #[test]
    fn test_7xnn() {
        let mut cpu = CPU::new(MacroquadKeyboard::new(), MacroquadRenderer::new());
        let opcode = 0x7464;

        cpu.set_register(4, 1);
        cpu.decode(opcode);
        assert_eq!(cpu.get_register(4), 0x64 + 1);
    }

    #[test]
    fn test_8xy4() {
        let mut cpu = CPU::new(MacroquadKeyboard::new(), MacroquadRenderer::new());
        let opcode = 0x8454;

        cpu.set_register(4, 0xFF);
        cpu.set_register(5, 1);
        cpu.set_register(0xF, 0);

        cpu.decode(opcode);

        assert_eq!(cpu.get_register(0xF), 1);

        cpu.set_register(4, 1);
        cpu.set_register(5, 1);
        cpu.set_register(0xF, 0);

        assert_eq!(cpu.get_register(0xF), 0);
    }

    #[test]
    fn test_8xy5_no_underflow() {
        let mut cpu = CPU::new(MacroquadKeyboard::new(), MacroquadRenderer::new());
        let opcode = 0x8455;

        cpu.set_register(4, 5);
        cpu.set_register(5, 4);

        cpu.decode(opcode);

        assert_eq!(cpu.get_register(4), 1);
        assert_eq!(cpu.get_register(0xF), 1)
    }

    #[test]
    fn test_8xy5_with_underflow() {
        let mut cpu = CPU::new(MacroquadKeyboard::new(), MacroquadRenderer::new());
        let opcode = 0x8455;

        cpu.set_register(4, 4);
        cpu.set_register(5, 5);

        cpu.decode(opcode);

        assert_eq!(cpu.get_register(4), 255);
        assert_eq!(cpu.get_register(0xF), 0);
    }

    #[test]
    fn test_8xy6(){
        let mut cpu= CPU::new(MacroquadKeyboard::new(), MacroquadRenderer::new());
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
        let mut cpu = CPU::new(MacroquadKeyboard::new(), MacroquadRenderer::new());
        let opcode = 0x8457;

        cpu.set_register(4, 4);
        cpu.set_register(5, 5);

        cpu.decode(opcode);

        assert_eq!(cpu.get_register(4), 1);
        assert_eq!(cpu.get_register(0xF), 1)
    }

    #[test]
    fn test_8xy7_with_underflow() {
        let mut cpu = CPU::new(MacroquadKeyboard::new(), MacroquadRenderer::new());
        let opcode = 0x8457;

        cpu.set_register(4, 5);
        cpu.set_register(5, 4);

        cpu.decode(opcode);

        assert_eq!(cpu.get_register(4), 255);
        assert_eq!(cpu.get_register(0xF), 0)
    }

    #[test]
    fn test_8xye(){
        let mut cpu= CPU::new(MacroquadKeyboard::new(), MacroquadRenderer::new());
        let opcode: u16 = 0x845E;

        cpu.set_register(4, 0x11);// 0x11 = 0001 0001
        cpu.decode(opcode);//                        0001 0001 << 1
        assert_eq!(cpu.get_register(4), 0x22);//     0010 0010
        assert_eq!(cpu.get_register(0xF), 0);

        cpu.set_register(4, 0x80);// 0x10 = 1000 0000
        cpu.decode(opcode);//                        1000 0000 << 1
        assert_eq!(cpu.get_register(4), 0);//        0000 0000
        assert_eq!(cpu.get_register(0xF), 1);
    }

    #[test]
    fn test_9xy0(){
        let mut cpu= CPU::new(MacroquadKeyboard::new(), MacroquadRenderer::new());
        let opcode: u16 = 0x9430;

        cpu.set_register(4, 5);
        cpu.set_register(3, 5);
        cpu.decode(opcode);
        assert_eq!(cpu.pc, 0x0200 + 2);

        cpu.pc = 0x0200;

        cpu.set_register(4, 5);
        cpu.set_register(3, 4);
        cpu.decode(opcode);
        assert_eq!(cpu.pc, 0x0200+4);
    }

    #[test]
    fn test_annn(){
        let mut cpu= CPU::new(MacroquadKeyboard::new(), MacroquadRenderer::new());
        let opcode: u16 = 0xA245;

        cpu.decode(opcode);
        assert_eq!(cpu.i, 0x245);
    }

    #[test]
    fn test_bnnn(){
        let mut cpu= CPU::new(MacroquadKeyboard::new(), MacroquadRenderer::new());
        let opcode: u16 = 0xB245;

        cpu.set_register(0, 1);
        cpu.decode(opcode);
        assert_eq!(cpu.pc, 0x246);
    }

    #[test]
    fn test_fx33(){
        let mut cpu= CPU::new(MacroquadKeyboard::new(), MacroquadRenderer::new());
        let opcode: u16 = 0xF433;

        cpu.set_register(4, 128);
        cpu.i = 0x505;
        cpu.decode(opcode);
        assert_eq!(cpu.get_memory(cpu.i), 1);
        assert_eq!(cpu.get_memory(cpu.i + 1), 2);
        assert_eq!(cpu.get_memory(cpu.i + 2), 8);
    }

    #[test]
    fn test_fx55(){
        let mut cpu= CPU::new(MacroquadKeyboard::new(), MacroquadRenderer::new());
        let opcode: u16 = 0xF355;

        cpu.set_register(0, 0);
        cpu.set_register(1, 1);
        cpu.set_register(2, 2);
        cpu.set_register(3, 3);

        cpu.i = 0x205;
        cpu.decode(opcode);
        for i in 0..4{
            assert_eq!(i, cpu.get_memory(cpu.i + i as u16));
        }
    }


    #[test]
    fn test_fx65(){
        let mut cpu= CPU::new(MacroquadKeyboard::new(), MacroquadRenderer::new());
        let opcode: u16 = 0xF365;

        cpu.i = 0x205;
        let start_adress = usize::from(cpu.i);

        cpu.memory[start_adress] = 0;
        cpu.memory[start_adress + 1] = 1;
        cpu.memory[start_adress + 2] = 2;
        cpu.memory[start_adress + 3] = 3;
        cpu.decode(opcode);
        for i in 0..=3{
            assert_eq!(i, cpu.get_register(i));
        }
    }
    
} 
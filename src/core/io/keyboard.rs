use macroquad::input::{KeyCode, is_key_down};


pub trait Keyboard {
    fn is_key_pressed(&self, key: u8) -> bool;
    fn wait_key_press(&self) -> Option<u8>;
    fn press_key(&mut self, key:u8);
    fn unpress_key(&mut self, key: u8);
    fn update_state(&mut self);
}

pub struct MacroquadKeyboard {
    pressed_keys: Vec<u8>
}

impl MacroquadKeyboard{
    pub fn new() -> Self{
        MacroquadKeyboard{ pressed_keys: vec![] }
    }

    fn map(&self, chip8_key: u8) -> KeyCode {
        match chip8_key {
            0x0 => KeyCode::X,
            0x1 => KeyCode::Key1,
            0x2 => KeyCode::Key2,
            0x3 => KeyCode::Key3,
            0x4 => KeyCode::Q,
            0x5 => KeyCode::W,
            0x6 => KeyCode::E,
            0x7 => KeyCode::A,
            0x8 => KeyCode::S,
            0x9 => KeyCode::D,
            0xA => KeyCode::Y,
            0xB => KeyCode::C,
            0xC => KeyCode::Key4,
            0xD => KeyCode::R,
            0xE => KeyCode::F,
            0xF => KeyCode::V,
            _ => panic!("Invalid CHIP-8 key"),
        }

    }
}

impl Keyboard for MacroquadKeyboard {

    fn is_key_pressed(&self, key: u8) -> bool {
        self.pressed_keys.contains(&key)
    }

    

    fn wait_key_press(&self) -> Option<u8>{ //TODO delay and sound timers should continue processing
        if self.pressed_keys.len() > 0 {
            Option::Some(self.pressed_keys[0])
        } else { None }
    }
    
    fn press_key(&mut self, key:u8) {
        self.pressed_keys.push(key);
    }

    fn unpress_key(&mut self, key: u8) { //TODO maybe this is not needed
        if !self.pressed_keys.contains(&key) {
            panic!("That key is not pressed")
        }
        self.pressed_keys.retain(|x| x != &key);
    }
    
    fn update_state(&mut self) {
        self.pressed_keys = vec![];
        
        for key in 0..=0xF {
            let is_key_pressed = is_key_down(self.map(key));

            if is_key_pressed { self.press_key(key);}
        } 

        if cfg!(debug_assertions) {
            //println!("Pressed keys: {:?}", self.pressed_keys)
        }
    }
}
use macroquad::{color::{BLACK, WHITE}, shapes::draw_rectangle};

pub trait Screen{
    fn get_pixel(&self, coord: (u8, u8)) -> bool;
    fn get_state(&self) -> [[bool;64]; 32];
    fn set_pixel(&mut self, coord: (u8, u8), value: bool);
    fn clear(&mut self);
    fn render(&self);
}

pub struct VirtualScreen {
    pixels: [[bool; 64]; 32]
}

impl VirtualScreen {
    pub fn new() -> Self{
        Self {
            pixels: [[false; 64]; 32]
        }
    }
}

impl Screen for VirtualScreen {
    fn get_pixel(&self, coord: (u8, u8)) -> bool {
        let (x, y) = coord;
        self.pixels[y as usize][x as usize]
    }

    fn set_pixel(&mut self, coord: (u8, u8), value: bool) {
        let (x, y) = coord;
        self.pixels[y as usize][x as usize] = value;

    }

    fn clear(&mut self) {
        self.pixels = [[false; 64]; 32];
    }
    
    fn get_state(&self) -> [[bool;64]; 32] {
        self.pixels
    }
    
    fn render(&self) {
        for (y, pixel_array) in self.get_state().iter().enumerate() {
                for (x, pixel) in pixel_array.iter().enumerate() {
                    draw_rectangle(x as f32 * 10., y as f32 * 10., 10., 10., if *pixel { WHITE } else { BLACK});
            }

        }
    }
}
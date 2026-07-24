use macroquad::{color::{BLACK, WHITE}, shapes::draw_rectangle, window::{screen_height, screen_width}};

pub trait Renderer{
    fn get_pixel(&self, coord: (u8, u8)) -> bool;
    fn get_state(&self) -> [[bool;64]; 32];
    fn set_pixel(&mut self, coord: (u8, u8), value: bool);
    fn clear(&mut self);
    fn render(&self);
}

pub struct MacroquadRenderer {
    pixels: [[bool; 64]; 32]
}

impl MacroquadRenderer {
    pub fn new() -> Self{
        Self {
            pixels: [[false; 64]; 32]
        }
    }

    fn translate_x(&self, x: u8) -> f32 {
        self.x_res() * x as f32
    }

    fn translate_y(&self, x: u8) -> f32 {
        self.y_res() * x as f32
    }

    fn x_res(&self) -> f32 {
        screen_width() / 64.
    }

    fn y_res(&self) -> f32 {
        screen_height() / 64.
    }
}

impl Renderer for MacroquadRenderer {
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
                    draw_rectangle(self.translate_x(x as u8), self.translate_y(y as u8), self.x_res(), self.y_res(), if *pixel { WHITE } else { BLACK});
            }

        }
    }
}
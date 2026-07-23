pub struct Screen{
    pixels: [[bool; 64]; 32]

}

impl Screen {

    pub fn new() -> Self{
        Self { pixels: [[false; 64]; 32] }
    }

    pub fn get_pixel(&self, coord: (u8, u8)) -> bool{
        let (x, y) = coord;
        self.pixels[usize::from(x)][usize::from(y)]
    }

    pub fn set_pixel(&mut self, coord: (u8, u8), value: bool){
        let (x, y) = coord;
        self.pixels[usize::from(x)][usize::from(y)] = value;
    }

    pub fn clear(&mut self){
        self.pixels = [[false; 64]; 32];
    }
    
}
use crate::cpu::CPU;


impl CPU {
    //00e0
    fn cls(){
        //clear display
    }

    //00EE
    fn ret(&mut self){
        self.pc = self.sp as u16;
        self.sp-=1;
    }

    //1nnn Jump to nnn
    fn jp(&mut self, target: u16){
        self.pc = target & 0x0111
    }
}
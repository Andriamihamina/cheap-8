#[cfg(test)]
mod tests {
    use crate::cpu::CPU;


    #[test]
    fn test_ret(){
        let mut cpu = CPU::new();
        cpu.stack = std::array::from_fn(|i| i as u16); 
        cpu.sp = 5;
        cpu.pc = cpu.stack[cpu.sp as usize];
        cpu.sp -= 1;
        assert_eq!(cpu.pc, 5);
        assert_eq!(cpu.sp, 4);

      
    }
}
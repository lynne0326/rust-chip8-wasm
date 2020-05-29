use crate::screen::Screen;
use crate::keyboard::{Keyboard, FONT_SET};
use crate::utils::get_random_buf;

const START_ADDR: u16 = 0x200;

pub struct CPU {
    pc: u16,
    v: [u8; 16],
    i: u16,
    stack: [u16; 16],
    memory: [u8; 4096],
    sp: u8,
    delay_timer: u8,
    sound_timer: u8,
    screen: Screen,
    keyboard: Keyboard,
}

impl Default for CPU {
    fn default() -> Self {
        CPU {
            pc: START_ADDR,
            v: [0; 16],
            i: 0,
            stack: [0; 16],
            memory: [0; 4096],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            screen: Screen::new(),
            keyboard: Keyboard::new(),
        }
    }
}

impl CPU {
    pub fn new() -> CPU {
        let mut cpu: CPU = Default::default();
        cpu.load_fonts();
        cpu
    }

    pub fn get_screen(&mut self) -> &mut Screen {
        &mut self.screen
    }

    pub fn get_keyboard(&mut self) -> &mut Keyboard {
        &mut self.keyboard
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.v = [0; 16];
        self.i = 0;
        self.stack = [0; 16];
        self.memory = [0; 4096];
        self.sp = 0;
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.screen.clear();
        self.keyboard.clear();
        self.load_fonts();
    }

    pub fn load_program(&mut self, program: &[u8]) {
        for idx in 0..program.len() {
            self.memory[self.pc as usize + idx] = program[idx];
        }
    }

    pub fn update_timer(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    fn load_fonts(&mut self) {
        for idx in 0..80 {
            self.memory[idx] = FONT_SET[idx]
        }
    }

    pub fn execute_next(&mut self) {
        let next_op = (self.memory[self.pc as usize] as u16) << 8 | self.memory[self.pc as usize + 1] as u16;
        self.execute(next_op);
    }

    fn execute(&mut self, opcode: u16) {
        self.pc += 2;

        let ops = (
            (opcode & 0xF000) >> 12,
            (opcode & 0x0F00) >> 8,
            (opcode & 0x00F0) >> 4,
            (opcode & 0x000F),
        );
        let nnn: u16 = opcode & 0xFFF;
        let nn: u8 = (opcode & 0x00FF) as u8;
        let n: u8 = ops.3 as u8;

        let x: usize = ops.1 as usize;
        let y: usize = ops.2 as usize;

        match ops {
            // Clear screen
            (0, 0, 0xE, 0) => self.screen.clear(),
            // Returns from a subroutine
            (0, 0, 0xE, 0xE) => {
                self.sp -= 1;
                self.pc = self.stack[self.sp as usize];
            }
            // Jump to address NNN
            (1, _, _, _) => self.pc = nnn,
            // Calls subroutine at NNN
            (2, _, _, _) => {
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = nnn;
            }
            // if(Vx==NN)
            (3, _, _, _) => {
                if self.v[x] == nn {
                    self.pc += 2
                }
            }
            // if(Vx!=NN)
            (4, _, _, _) => {
                if self.v[x] != nn {
                    self.pc += 2
                }
            }
            // 	if(Vx==Vy)
            (5, _, _, 0) => {
                if self.v[x] == self.v[y] {
                    self.pc += 2
                }
            }
            // Vx = NN
            (6, _, _, _) => self.v[x] = nn,
            // Vx += NN
            (7, _, _, _) => self.v[x] += nn,
            // Vx == Vy
            (8, _, _, 0) => self.v[x] = self.v[y],
            // Vx=Vx|Vy
            (8, _, _, 1) => self.v[x] |= self.v[y],
            // Vx=Vx&Vy
            (8, _, _, 2) => self.v[x] &= self.v[y],
            // Vx=Vx^Vy
            (8, _, _, 3) => self.v[x] ^= self.v[y],
            // Vx += Vy
            (8, _, _, 4) => {
                let (res, ov) = self.v[x].overflowing_add(self.v[y]);
                self.v[0x0F] = if ov { 1 } else { 0 };
                self.v[x] = res as u8;
            }
            // Vx -= Vy
            (8, _, _, 5) => {
                let (res, ov) = self.v[x].overflowing_sub(self.v[y]);
                self.v[0x0F] = if ov { 0 } else { 1 };
                self.v[x] = res;
            }
            // Vx>>=1
            (8, _, _, 6) => {
                self.v[0x0F] = self.v[x] & 0x1;
                self.v[x] >>= 1;
            }
            // Vx=Vy-Vx
            (8, _, _, 7) => {
                let (res, ov) = self.v[y].overflowing_sub(self.v[x]);
                self.v[0x0F] = if ov { 0 } else { 1 };
                self.v[x] = res;
            }
            // Vx<<=1
            (8, _, _, 0xE) => {
                self.v[0x0F] = (self.v[x] & 0x80) >> 7;
                self.v[x] <<= 1;
            }
            // if(Vx!=Vy)
            (9, _, _, 0) => {
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
            }
            // ANNN	MEM	I = NNN	Sets I to the address NNN.
            (0xA, _, _, _) => { self.i = nnn as u16; }
            // BNNN	Flow	PC=V0+NNN	Jumps to the address NNN plus V0.
            (0xB, _, _, _) => {
                self.pc = self.v[0] as u16 + nnn;
            }
            // Vx=rand()&NN
            (0xC, _, _, _) => {
                self.v[x] = get_random_buf().unwrap()[0] & nn;
            }
            // draw(Vx,Vy,N)
            (0xD, _, _, _) => {
                let (width, height) = (8, n);
                self.v[0xF] = 0;
                for row in 0..height {
                    let mut sprite = self.memory[self.i as usize + row as usize];
                    for col in 0..width {
                        if sprite & 0x80 > 0 {
                            let c = self.v[x] as usize + col;
                            let r = self.v[y] as usize + row as usize;
                            if self.screen.get_pixel(r, c) {
                                self.v[0xF] = 1;
                            }
                            self.screen.set_pixel(r, c);
                        }
                        sprite <<= 1;
                    }
                }
            }
            // if(key()==Vx)
            (0xE, _, 9, 0xE) => {
                if self.keyboard.is_key_pressed(self.v[x]) {
                    self.pc += 2;
                }
            }
            // if(key()!=Vx)
            (0xE, _, 0xA, 1) => {
                if !self.keyboard.is_key_pressed(self.v[x]) {
                    self.pc += 2;
                }
            }
            // Vx = get_delay()
            (0xF, _, 0, 7) => {
                self.v[x] = self.delay_timer;
            }
            // Vx = get_key()
            (0xF, _, 0, 0xA) => {
                for idx in 0..self.keyboard.pressed_keys.len() {
                    if self.keyboard.pressed_keys[idx] {
                        self.v[x] = idx as u8;
                        self.pc -= 2;
                        break;
                    }
                }
            }
            // delay_timer(Vx)	Sets the delay timer to VX.
            (0xF, _, 1, 5) => {
                self.delay_timer = self.v[x];
            }
            // sound_timer(Vx)	Sets the sound timer to VX.
            (0xF, _, 1, 8) => {
                self.sound_timer = self.v[x];
            }
            // FX1E	MEM	I +=Vx	Adds VX to I. VF is set to 1 when there is a range overflow
            // (I+VX>0xFFF), and to 0 when there isn't.[c]
            (0xF, _, 1, 0xE) => {
                let (res, ov) = self.i.overflowing_add(self.v[x] as u16);
                self.v[0xF] = if ov { 1 } else { 0 };
                self.i = res;
            }
            // FX29	MEM	I=sprite_addr[Vx]	Sets I to the location of the sprite for the character
            // in VX. Characters 0-F (in hexadecimal) are represented by a 4x5 font.
            (0xF, _, 2, 9) => {
                self.i = self.v[x] as u16 * 5;
            }
            // FX33	BCD	set_BCD(Vx);
            // *(I+0)=BCD(3);
            //
            // *(I+1)=BCD(2);
            //
            // *(I+2)=BCD(1);
            (0xF, _, 3, 3) => {
                self.memory[self.i as usize] = self.v[x] / 100;
                self.memory[self.i as usize + 1] = (self.v[x] % 100) / 10;
                self.memory[self.i as usize + 2] = self.v[x] % 10;
            }
            // FX55	MEM	reg_dump(Vx,&I)	Stores V0 to VX (including VX) in memory starting at
            // address I. The offset from I is increased by 1 for each value written,
            // but I itself is left unmodified.[d]
            (0xF, _, 5, 5) => {
                for idx in 0..x + 1 {
                    self.memory[self.i as usize + idx as usize] = self.v[idx];
                }
            }
            // FX65	MEM	reg_load(Vx,&I)	Fills V0 to VX (including VX) with values from memory
            // starting at address I. The offset from I is increased by 1 for each value written,
            // but I itself is left unmodified.[d]
            (0xF, _, 6, 5) => {
                for idx in 0..x + 1 {
                    self.v[idx] = self.memory[self.i as usize + idx as usize];
                }
            }
            (_, _, _, _) => ()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_fonts_when_created() {
        let cpu = CPU::new();
        assert_eq!(cpu.memory[0], FONT_SET[0]);
        assert_eq!(cpu.memory[1], FONT_SET[1]);
        assert_eq!(cpu.memory[79], FONT_SET[79]);
        assert_eq!(cpu.memory[80], 0);
    }

    #[test]
    fn test_load_program() {
        let mut cpu = CPU::new();
        cpu.load_program(&[1, 2, 3]);
        assert_eq!(cpu.memory[0x200], 1);
        assert_eq!(cpu.memory[0x201], 2);
        assert_eq!(cpu.memory[0x202], 3);
    }

    #[test]
    fn test_execute_1xxx() {
        let mut cpu = CPU::new();
        cpu.execute(0x1A2A);
        assert_eq!(cpu.pc, 0x0A2A);
    }

    #[test]
    fn test_execute_2xxx() {
        let mut cpu = CPU::new();
        cpu.pc = 0x20;
        cpu.execute(0x2123);
        assert_eq!(cpu.pc, 0x0123);
        assert_eq!(cpu.sp, 1);
        assert_eq!(cpu.stack[0], 0x20 + 2);
    }

    #[test]
    fn test_execute_3xxx() {
        let mut cpu = CPU::new();
        cpu.v[0] = 0xEE;

        // vx == kk skip 4
        cpu.execute(0x30EE);
        assert_eq!(cpu.pc, START_ADDR + 4);

        // vx != kk
        cpu.execute(0x30EF);
        assert_eq!(cpu.pc, START_ADDR + 6);
    }

    #[test]
    fn test_execute_4xxx() {
        let mut cpu = CPU::new();
        cpu.v[0] = 0xEE;

        // vx == kk
        cpu.execute(0x40EE);
        assert_eq!(cpu.pc, START_ADDR + 2);

        // vx != kk skip 4
        cpu.execute(0x40EF);
        assert_eq!(cpu.pc, START_ADDR + 6);
    }

    #[test]
    fn test_execute_5xxx() {
        let mut cpu = CPU::new();
        cpu.v[0] = 1;
        cpu.v[1] = 1;
        cpu.execute(0x5110);
        assert_eq!(cpu.pc, START_ADDR + 4);
        cpu.execute(0x5210);
        assert_eq!(cpu.pc, START_ADDR + 6);
    }

    #[test]
    fn test_execute_6xxx() {
        let mut cpu = CPU::new();
        cpu.execute(0x60EF);
        assert_eq!(cpu.v[0], 0xEF);
        assert_eq!(cpu.pc, START_ADDR + 2);
    }

    #[test]
    fn test_execute_7xxx() {
        let mut cpu = CPU::new();
        cpu.v[0] = 1;
        cpu.execute(0x70EF);
        assert_eq!(cpu.v[0], 1 + 0xEF);
        assert_eq!(cpu.pc, START_ADDR + 2);
    }

    #[test]
    fn test_execute_8xx0() {
        let mut cpu = CPU::new();
        cpu.v[0] = 1;
        cpu.v[1] = 2;
        cpu.execute(0x8010);
        assert_eq!(cpu.v[0], 2);
        assert_eq!(cpu.pc, START_ADDR + 2);
    }

    // TODO: 8xxx cases

    #[test]
    fn test_execute_axxx() {
        let mut cpu = CPU::new();
        cpu.execute(0xA1EF);
        assert_eq!(cpu.i, 0x1EF);
    }

    #[test]
    fn test_execute_bxxx() {
        let mut cpu = CPU::new();
        cpu.v[0] = 2;
        cpu.execute(0xBEF3);
        assert_eq!(cpu.pc, 0xEF5);
        assert_eq!(cpu.i, 0xEF3)
    }

    #[test]
    fn test_execute_cxxx() {
        let mut cpu = CPU::new();
        cpu.execute(0xC000);
        assert_eq!(cpu.v[0], 0);
        cpu.execute(0xC00F);
        assert_eq!(cpu.v[0] & 0xF0, 0);
    }

    #[test]
    fn test_execute_dxxx() {
        let mut cpu = CPU::new();
        cpu.i = 0;
        cpu.memory[0] = 0b11110011;
        cpu.memory[1] = 0b11001110;
        cpu.execute(0xD002);
        assert_eq!(true, cpu.get_screen().get_pixel(0, 0));
        assert_eq!(true, cpu.get_screen().get_pixel(0, 1));
        assert_eq!(true, cpu.get_screen().get_pixel(0, 2));
        assert_eq!(true, cpu.get_screen().get_pixel(0, 3));
        assert_eq!(false, cpu.get_screen().get_pixel(0, 4));
        assert_eq!(false, cpu.get_screen().get_pixel(0, 5));
        assert_eq!(true, cpu.get_screen().get_pixel(0, 6));
        assert_eq!(true, cpu.get_screen().get_pixel(0, 7));

        assert_eq!(true, cpu.get_screen().get_pixel(1, 0));
        assert_eq!(true, cpu.get_screen().get_pixel(1, 1));
        assert_eq!(false, cpu.get_screen().get_pixel(1, 2));
        assert_eq!(false, cpu.get_screen().get_pixel(1, 3));
        assert_eq!(true, cpu.get_screen().get_pixel(1, 4));
        assert_eq!(true, cpu.get_screen().get_pixel(1, 5));
        assert_eq!(true, cpu.get_screen().get_pixel(1, 6));
        assert_eq!(false, cpu.get_screen().get_pixel(1, 7));
        assert_eq!(cpu.v[0xF], 0);
        assert_eq!(cpu.pc, START_ADDR + 2);

        // test collision
        cpu.memory[0] = 0b11110100;
        cpu.execute(0xD001);
        assert_eq!(false, cpu.get_screen().get_pixel(0, 0));
        assert_eq!(false, cpu.get_screen().get_pixel(0, 1));
        assert_eq!(false, cpu.get_screen().get_pixel(0, 2));
        assert_eq!(false, cpu.get_screen().get_pixel(0, 3));
        assert_eq!(false, cpu.get_screen().get_pixel(0, 4));
        assert_eq!(true, cpu.get_screen().get_pixel(0, 5));
        assert_eq!(true, cpu.get_screen().get_pixel(0, 6));
        assert_eq!(true, cpu.get_screen().get_pixel(0, 7));
        assert_eq!(cpu.v[0xF], 1);
        assert_eq!(cpu.pc, START_ADDR + 4);
    }

    #[test]
    fn test_execute_ex9e() {
        let mut cpu = CPU::new();
        cpu.keyboard.pressed_keys[9] = true;
        cpu.v[0] = 9;
        cpu.execute(0xe09e);
        assert_eq!(cpu.pc, START_ADDR + 4);

        cpu.get_keyboard().clear();
        cpu.execute(0xe09e);
        assert_eq!(cpu.pc, START_ADDR + 6);
    }

    #[test]
    fn test_execute_exa1() {
        let mut cpu = CPU::new();
        cpu.keyboard.pressed_keys[9] = true;
        cpu.v[0] = 9;
        cpu.execute(0xe0a1);
        assert_eq!(cpu.pc, START_ADDR + 2);

        cpu.get_keyboard().clear();
        cpu.execute(0xe0a1);
        assert_eq!(cpu.pc, START_ADDR + 6);
    }

    #[test]
    fn test_execute_fx07() {
        let mut cpu = CPU::new();
        cpu.delay_timer = 20;
        cpu.execute(0xf507);
        assert_eq!(cpu.v[5], 20);
        assert_eq!(cpu.pc, START_ADDR + 2);
    }

    #[test]
    fn test_execute_fx0a() {
        let mut cpu = CPU::new();
        cpu.get_keyboard().key_down(3);
        cpu.execute(0xF30A);
        assert_eq!(cpu.keyboard.pressed_keys[3], true);
        assert_eq!(cpu.v[3], 3);
        assert_eq!(cpu.pc, START_ADDR);
    }

    #[test]
    fn test_execute_fx15() {
        let mut cpu = CPU::new();
        cpu.v[5] = 9;
        cpu.execute(0xf515);
        assert_eq!(cpu.delay_timer, 9);
        assert_eq!(cpu.pc, START_ADDR + 2);
    }

    #[test]
    fn test_execute_fx18() {
        let mut cpu = CPU::new();
        cpu.v[5] = 9;
        cpu.execute(0xf518);
        assert_eq!(cpu.sound_timer, 9);
        assert_eq!(cpu.pc, START_ADDR + 2);
    }

    #[test]
    fn test_execute_fx1e() {
        let mut cpu = CPU::new();
        cpu.v[5] = 9;
        cpu.i = 9;
        cpu.execute(0xf51e);
        assert_eq!(cpu.i, 18);
        assert_eq!(cpu.pc, START_ADDR + 2);
    }

    #[test]
    fn test_execute_fx29() {
        let mut cpu = CPU::new();
        cpu.v[5] = 9;
        cpu.execute(0xf529);
        assert_eq!(cpu.i, 5 * 9);
        assert_eq!(cpu.pc, START_ADDR + 2);
    }

    #[test]
    fn test_execute_fx33() {
        let mut cpu = CPU::new();
        cpu.v[5] = 123;
        cpu.i = 1000;
        cpu.execute(0xf533);
        assert_eq!(cpu.memory[1000], 1);
        assert_eq!(cpu.memory[1001], 2);
        assert_eq!(cpu.memory[1002], 3);
        assert_eq!(cpu.pc, START_ADDR + 2);
    }

    #[test]
    fn test_execute_fx55() {
        let mut cpu = CPU::new();
        cpu.i = 1000;
        cpu.execute(0xff55);
        for i in 0..16 {
            assert_eq!(cpu.memory[1000 + i as usize], cpu.v[i]);
        }
        assert_eq!(cpu.pc, START_ADDR + 2);
    }

    #[test]
    fn test_execute_fx65() {
        let mut cpu = CPU::new();
        for idx in 0..16 as usize {
            cpu.memory[1000 + idx] = idx as u8;
        }
        cpu.i = 1000;
        cpu.execute(0xff65);
        for i in 0..16 as usize {
            assert_eq!(cpu.v[i], cpu.memory[1000 + i]);
        }
        assert_eq!(cpu.pc, START_ADDR + 2);
    }
}

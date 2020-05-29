use crate::cpu::CPU;
use crate::utils::set_panic_hook;
use wasm_bindgen::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub struct Chip8 {
    cpu: CPU,
}

#[wasm_bindgen]
impl Chip8 {
    pub fn new() -> Chip8 {
        set_panic_hook();
        Chip8 {
            cpu : CPU::new()
        }
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
    }

    pub fn load_program(&mut self, program: &[u8]) {
        self.cpu.load_program(program);
    }

    pub fn execute_next(&mut self) {
        self.cpu.execute_next();
    }

    pub fn update_timer(&mut self) {
        self.cpu.update_timer();
    }

    pub fn width(&mut self) -> usize {
        self.cpu.get_screen().width()
    }

    pub fn height(&mut self) -> usize {
        self.cpu.get_screen().height()
    }

    pub fn get_screen_memory(&mut self) -> *const bool {
        self.cpu.get_screen().get_screen_memory()
    }

    pub fn key_down(&mut self, key: u8) {
        self.cpu.get_keyboard().key_down(key);
    }

    pub fn key_up(&mut self, key: u8) {
        self.cpu.get_keyboard().key_up(key);
    }
}

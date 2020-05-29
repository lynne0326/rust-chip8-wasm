use std::fmt;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

pub struct Screen {
    bit_map: [bool; 2048],
    width: usize,
    height: usize,
}

impl Default for Screen {
    fn default() -> Screen {
        Screen {
            bit_map: [false; 2048],
            width: WIDTH,
            height: HEIGHT,
        }
    }
}

impl Screen {
    pub fn new() -> Screen {
        Default::default()
    }
    pub fn set_pixel(&mut self, row: usize, col: usize) {
        self.bit_map[row * WIDTH + col] = !self.bit_map[row * WIDTH + col];
    }

    pub fn get_pixel(&mut self, row: usize, col: usize) -> bool {
        self.bit_map[row * WIDTH + col]
    }

    pub fn clear(&mut self) {
        self.bit_map = [false; 2048];
    }

    pub fn get_screen_memory(&self) -> *const bool {
        return self.bit_map.as_ptr();
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
}

impl fmt::Debug for Screen {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.bit_map.fmt(formatter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_get_dimension() {
        let screen = Screen::new();
        assert_eq!(screen.height(), HEIGHT);
        assert_eq!(screen.width(), WIDTH);
    }

    #[test]
    pub fn test_set_pixel() {
        let mut screen = Screen::new();
        screen.set_pixel(1, 3);
        assert_eq!(screen.get_pixel(1, 3), true);
    }
}
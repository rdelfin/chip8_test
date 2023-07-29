#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Display {
    // Indexed as pixels[x][y]
    pixels: [[bool; 32]; 64],
}

impl Default for Display {
    fn default() -> Display {
        Display {
            pixels: [[false; 32]; 64],
        }
    }
}

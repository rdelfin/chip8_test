use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Display {
    // Indexed as pixels[x][y]
    pub pixels: [[bool; 32]; 64],
}

impl Default for Display {
    fn default() -> Display {
        Display {
            pixels: [[false; 32]; 64],
        }
    }
}

#[allow(dead_code)]
pub struct Coordinates {
    pub x: u8,
    pub y: u8,
}

impl Coordinates {
    #[allow(dead_code)]
    pub fn new(x: u8, y: u8) -> Coordinates {
        Coordinates { x, y }
    }
}

impl Display {
    #[allow(dead_code)]
    fn flip_all(&mut self, start: Coordinates, end: Coordinates) {
        for x in start.x..=end.x {
            let x = x as usize;
            for y in start.y..=end.y {
                let y = y as usize;
                self.pixels[x][y] = !self.pixels[x][y];
            }
        }
    }
}

impl fmt::Display for Display {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Top row cover
        write!(f, ".")?;
        for _ in 0..self.pixels.len() {
            write!(f, "-")?;
        }
        writeln!(f, ".")?;

        // Pixel rows
        for y in 0..self.pixels[0].len() {
            write!(f, "|")?;
            for x in 0..self.pixels.len() {
                if self.pixels[x][y] {
                    write!(f, "█")?;
                } else {
                    write!(f, " ")?;
                }
            }
            writeln!(f, "|")?;
        }

        // Bottom row cover
        write!(f, ".")?;
        for _ in 0..self.pixels.len() {
            write!(f, "-")?;
        }
        write!(f, ".")?;

        Ok(())
    }
}

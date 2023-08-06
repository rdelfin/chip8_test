use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Display {
    // Indexed as pixels[y][x]
    pub pixels: [[bool; 64]; 32],
}

impl Default for Display {
    fn default() -> Display {
        Display {
            pixels: [[false; 64]; 32],
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
    #[cfg(test)]
    pub fn flip_all(&mut self, start: Coordinates, end: Coordinates) {
        for x in start.x..=end.x {
            let x = x as usize;
            for y in start.y..=end.y {
                let y = y as usize;
                self.pixels[y][x] = !self.pixels[y][x];
            }
        }
    }

    pub fn clear(&mut self) {
        self.pixels[..].copy_from_slice(&[[false; 64]; 32]);
    }
}

impl fmt::Display for Display {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Top row cover
        write!(f, ".")?;
        for _ in 0..self.pixels[0].len() {
            write!(f, "-")?;
        }
        writeln!(f, ".")?;

        // Pixel rows
        for y in 0..self.pixels.len() {
            write!(f, "|")?;
            for x in 0..self.pixels[y].len() {
                if self.pixels[y][x] {
                    write!(f, "█")?;
                } else {
                    write!(f, " ")?;
                }
            }
            writeln!(f, "|")?;
        }

        // Bottom row cover
        write!(f, ".")?;
        for _ in 0..self.pixels[0].len() {
            write!(f, "-")?;
        }
        write!(f, ".")?;

        Ok(())
    }
}

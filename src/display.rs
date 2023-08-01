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

use std::{fmt, ops::Add};

pub struct Resolution {
    pub width: usize,
    pub height: usize,
}

pub const SCREEN_RES: Resolution = Resolution {
    width: 64,
    height: 32,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Display {
    // Indexed as pixels[y][x]
    pub pixels: [[bool; SCREEN_RES.width]; SCREEN_RES.height],
}

impl Default for Display {
    fn default() -> Display {
        Display {
            pixels: [[false; SCREEN_RES.width]; SCREEN_RES.height],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Coordinates {
    pub x: u8,
    pub y: u8,
}

impl Coordinates {
    #[allow(dead_code)]
    pub fn new(x: u8, y: u8) -> Coordinates {
        Coordinates {
            x: x % (SCREEN_RES.width as u8),
            y: y % (SCREEN_RES.height as u8),
        }
    }
}

impl Add for Coordinates {
    type Output = Coordinates;
    fn add(self, other: Coordinates) -> Coordinates {
        Coordinates {
            x: self.x + other.x,
            y: self.y + other.y,
        }
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
        self.pixels[..].copy_from_slice(&[[false; SCREEN_RES.width]; SCREEN_RES.height]);
    }

    pub fn apply_sprite(&mut self, sprite: &[u8], coordinates: Coordinates) {
        for (y_offset, byte) in sprite.iter().enumerate() {
            // Truncate y coordinates as soon as possible
            if y_offset + (coordinates.y as usize) >= 32 {
                break;
            }

            let y_offset = y_offset
                .try_into()
                .expect("y offset did not fit in a usize");
            self.apply_row(&[*byte], 8, coordinates + Coordinates::new(0, y_offset));
        }
    }

    fn apply_row(&mut self, row: &[u8], len_bits: u8, coordinates: Coordinates) {
        let full_row: &mut [bool] = &mut self.pixels[coordinates.y as usize];
        let start = coordinates.x;
        let end = (coordinates.x + len_bits).min(
            SCREEN_RES
                .width
                .try_into()
                .expect("screen resolution does not fit in u8"),
        );
        println!("start: {start}; end: {end}; len: {len_bits}");
        // Short-circuit if start and end are equal (or somehow flipped)
        if end <= start {
            return;
        }
        let real_len = end - start;

        for x in 0..real_len {
            let byte: usize = (x / 8).into();
            let bit_in_byte = 7 - (x % 8);
            let val = (row[byte] & (1 << bit_in_byte)) != 0;
            let idx: usize = (start + x).into();
            println!("X: {x} (idx: {idx})");
            if val {
                full_row[idx] = !full_row[idx];
            }
        }
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

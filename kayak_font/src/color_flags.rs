bitflags::bitflags! {
    pub struct ColorFlags: u32 {
        const BLACK = 0b000;
        const RED = 0b001;
        const GREEN = 0b010;
        const BLUE = 0b100;
        const YELLOW = 0b011;
        const MAGENTA = 0b101;
        const CYAN = 0b110;
        const WHITE = 0b111;
    }
}

impl ColorFlags {
    pub(crate) fn switch(self, seed: &mut u64) -> Self {
        match self {
            ColorFlags::WHITE | ColorFlags::BLACK => {
                const START: [ColorFlags; 3] =
                    [ColorFlags::CYAN, ColorFlags::MAGENTA, ColorFlags::YELLOW];
                let tr = START[(*seed % 3) as usize];
                *seed /= 3;
                tr
            }
            ColorFlags::RED | ColorFlags::GREEN | ColorFlags::BLUE => self ^ ColorFlags::WHITE,
            _ => {
                let v = self.bits();
                let v = (v << (1 + (*seed & 1))) & 0b111;
                let v = match v.count_ones() {
                    0 => 0b11,           /* Somehow we lost all the bits. Default to yellow */
                    1 => v | 0b001, /* We just shifted a bit off the left side, add one on the right */
                    2 => v,         /* We already have 2 bits, nothing to do */
                    _ => unreachable!(), /* There should never be 3+ bits set */
                };
                *seed >>= 1;

                Self::from_bits_truncate(v)
            }
        }
    }

    pub(crate) fn switch_banned(self, seed: &mut u64, banned: ColorFlags) -> Self {
        let combined = self & banned;
        match combined {
            ColorFlags::RED | ColorFlags::GREEN | ColorFlags::BLUE => combined ^ ColorFlags::WHITE,
            _ => self.switch(seed),
        }
    }

    pub fn float_color(self) -> [f32; 3] {
        match self {
            ColorFlags::BLACK => [0.0f32, 0.0f32, 0.0f32],
            ColorFlags::RED => [1.0f32, 0.0f32, 0.0f32],
            ColorFlags::GREEN => [0.0f32, 1.0f32, 0.0f32],
            ColorFlags::BLUE => [0.0f32, 0.0f32, 1.0f32],
            ColorFlags::CYAN => [0.0f32, 1.0f32, 1.0f32],
            ColorFlags::MAGENTA => [1.0f32, 0.0f32, 1.0f32],
            ColorFlags::YELLOW => [1.0f32, 1.0f32, 0.0f32],
            ColorFlags::WHITE => [1.0f32, 1.0f32, 1.0f32],
            _ => [0.5, 0.7, 0.5],
        }
    }
}

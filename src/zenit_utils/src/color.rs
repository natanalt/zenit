#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RG8 {
    pub r: u8,
    pub g: u8,
}

impl Into<RGB8> for RG8 {
    #[inline]
    fn into(self) -> RGB8 {
        RGB8 {
            r: self.r,
            g: self.g,
            b: 0,
        }
    }
}

impl Into<RGBA8> for RG8 {
    #[inline]
    fn into(self) -> RGBA8 {
        RGBA8 {
            r: self.r,
            g: self.g,
            b: 0,
            a: 255,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RGB8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Into<RGBA8> for RGB8 {
    #[inline]
    fn into(self) -> RGBA8 {
        RGBA8 {
            r: self.r,
            g: self.g,
            b: self.b,
            a: 255,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RGBA8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

use crate::fontlib::helper::PGFFlags;


pub struct FontStyle{
    pub size: f32,
    pub color: FontColor,
    pub shadow_color: FontColor,
    pub angle: f32,
    pub options: PGFFlags
}

bitflags! {
    pub struct FontColor: u32{
        const BLACK         = 0xFF000000;
        const WHITE         = 0xFFFFFFFF;
        const RED           = 0xFF0000FF;
        const GREEN         = 0xFF00FF00;
        const BLUE          = 0xFFFF0000;
        const LIGHT_GRAY    = 0xFFBFBFBF;
        const GRAY          = 0xFF7F7F7F;
        const DARK_GRAY     = 0xFF3F3F3F;
    }
}

impl FontColor{
    pub fn from_rgba(red:u8,green:u8,blue:u8,alpha:u8) -> Self{
        let color_bits = red as u32 |
            (green as u32) << 2*4 |
            (blue as u32) << 4*4 |
            (alpha as u32) << 6*4;
        Self{ bits: color_bits }
    }
}
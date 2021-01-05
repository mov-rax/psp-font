use crate::fontlib::helper::PGFFlags;

#[derive(Copy, Clone)]
pub( crate) struct Glyph{
    pub(crate) x:u16,
    pub(crate) y:u16,
    pub(crate) width:u8,
    pub(crate) height:u8,
    pub(crate) left:i8,
    pub(crate) top:i8,
    pub(crate) flags: PGFFlags,
    pub(crate) shadow_id: u16,
    pub(crate) advance:i8,
    pub(crate) offset:u32,
}

impl Default for Glyph{
    fn default() -> Self {
        Glyph{
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            left: 0,
            top: 0,
            flags: PGFFlags::NONE,
            shadow_id: 0,
            advance: 0,
            offset: 0
        }
    }
}
#[derive(Copy, Clone)]
pub( crate) struct GlyphBW{
    pub(crate) x:u16,
    pub(crate) y:u16,
    pub(crate) flags: PGFFlags
}

impl Default for GlyphBW{
    fn default() -> Self {
        GlyphBW{
            x: 0,
            y: 0,
            flags: PGFFlags::NONE,
        }
    }
}
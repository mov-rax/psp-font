use crate::fontlib::ccclib::Codepages as CP;
use alloc::alloc::{alloc, dealloc, Layout};
use core::mem::size_of;

#[derive(Eq, PartialEq)]
pub enum FileType{
    PGF,
    BWFON
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) struct UCS2(pub u16);

bitflags!{
        pub struct PGFFlags:u32{
            const ADVANCE_H             = 0x0;
            const ADVANCE_V             = 0x100;
            const ALIGN_LEFT            = 0x0;
            const ALIGN_CENTER          = 0x200;
            const ALIGN_RIGHT           = 0x400;
            const ALIGN_FULL            = 0x600;
            const SCROLL_LEFT           = 0x2000;
            const SCROLL_SEESAW         = 0x2200;
            const SCROLL_RIGHT          = 0x2400;
            const SCROLL_THROUGH        = 0x2600;
            const WIDTH_FIX             = 0x800;
            const ACTIVE                = 0x1000;
            const DIRTY                 = 0x1;
            const CACHE_MED             = 0x0;
            const CACHE_LARGE           = 0x4000;
            const CACHE_ASCII           = 0x8000;
            const CACHE_ALL             = 0xC000;
            const STRING_ASCII          = 0x10000 * CP::ASCII.bits();
            const STRING_US             = 0x10000 * CP::US.bits();
            const STRING_MLATIN         = 0x10000 * CP::MLATIN.bits(); // Multilingual Latin
            const STRING_RUSSIAN        = 0x10000 * CP::RUSSIAN.bits();
            const STRING_S_JIS          = 0x10000 * CP::S_JIS.bits();
            const STRING_GBK            = 0x10000 * CP::GBK.bits();
            const STRING_KOREAN         = 0x10000 * CP::KOREAN.bits();
            const STRING_BIG5           = 0x10000 * CP::BIG5.bits();
            const STRING_CYRILLIC       = 0x10000 * CP::CYRILLIC.bits();
            const STRING_LATIN2         = 0x10000 * CP::LATIN2.bits();
            const STRING_UTF8           = 0x10000 * CP::UTF8.bits();
            const BMP_HORIZONTAL_ROWS   = 0x01;
            const BMP_VERTICAL_ROWS     = 0x02;
            const BMP_OVERLAY           = 0x03;
            const NO_EXTRA1             = 0x04;
            const NO_EXTRA2             = 0x08;
            const NO_EXTRA3             = 0x10;
            const CHAR_GLYPH            = 0x20;
            const SHADOWGLYPH           = 0x40;
            const CACHED                = 0x80;
            const WIDTH_MASK            = 0xFF;
            const OPTIONS_MASK          = 0x3FFF;
            const ALIGN_MASK            = 0x0600;
            const SCROLL_MASK           = 0x2600;
            const CACHE_MASK            = 0xC000;
            const STRING_MASK           = 0xFF0000;
            const NONE                  = 0x0;
        }
    }

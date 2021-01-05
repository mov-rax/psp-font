#![no_std]
#![feature(min_const_generics)]
#[macro_use]
extern crate alloc;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate smart_buffer;

mod fontlib;


#[cfg(test)]
mod tests {
    use crate::fontlib::fontlib::{PGFHeader, Font};
    use alloc::vec::Vec;
    use crate::fontlib::style::{FontStyle, FontColor};
    use crate::fontlib::helper::PGFFlags;

    struct Omega{
        a: u32,
        b: u32,
        c: u32
    }

    #[test]
    fn it_works() {
        // let data = include_bytes!("../ltn0.pgf");
        // let mut font = Font::new(data, PGFFlags::CACHE_ASCII);
        // font.set_style(FontStyle{
        //     size: 23.0,
        //     color: FontColor::RED,
        //     shadow_color: FontColor::BLACK,
        //     angle: 15.0,
        //     options: PGFFlags::NONE,
        // });
        // //font.options.insert(PGFFlags::SCROLL_LEFT);
        // font.print(0.0,0.0,"Hello There");

    }
}

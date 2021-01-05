// #![no_std]
#![feature(min_const_generics)]
extern crate alloc;
#[macro_use]
extern crate bitflags;
use arrayvec;
mod fontlib;

#[cfg(test)]
mod tests {
    use arrayvec::ArrayVec;
    use crate::fontlib::fontlib::PGFHeader;

    struct Omega{
        a: u32,
        b: u32,
        c: u32
    }

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
        let mut x = ArrayVec::<[u8;384]>::new();
        x.push(3);
        println!("{}", std::mem::size_of::<PGFHeader>())
    }
}

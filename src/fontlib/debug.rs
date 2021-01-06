use psp::sys::{sceIoWrite, SceUid};
use bitflags::_core::ffi::c_void;

pub fn io_write(text:&str) -> i32{
    unsafe{
        sceIoWrite(SceUid(1), text.as_bytes().as_ptr() as *const c_void, text.len())
    }
}
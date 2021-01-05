#[no_mangle]
#[repr(C, packed)]
#[derive(Default, Copy, Clone)]
pub( crate) struct FontVertex{
    pub u: f32,
    pub v: f32,
    pub c: u32,
    pub x: f32,
    pub y: f32,
    pub z: f32
}

impl FontVertex{
    pub fn get_mut_ptr(&mut self) -> *mut Self{
        self as *mut Self
    }
    pub unsafe fn from_mut_ptr(ptr: *mut Self) -> Self{
        *ptr
    }
}
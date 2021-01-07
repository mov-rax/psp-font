use alloc::vec::Vec;
use alloc::alloc::{alloc, dealloc, Layout};
use core::mem::size_of;
use aligned_utils::bytes::AlignedBytes;

pub( crate) struct TextureData {
    pub(crate) data: AlignedBytes,
    pub(crate) id: u32,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) x: u16,
    pub(crate) y: u16,
    pub(crate) y_size: u16,
}

impl TextureData{

    /// Convenience function for creating new texture data
    pub fn new(width: u32, height: u32) -> Self{
        Self {
            data: AlignedBytes::new_zeroed(size_of::<u32>()*(width as usize) * (height as usize) >> 1, 16 ),
            id: 0,
            width,
            height,
            x: 1,
            y: 1,
            y_size: 0
        }
    }

    /// safely get the value in an index of texture data
    pub fn get(&self, index:usize) -> Option<u8>{
        if index >=0 && index < self.data.len(){
            return Some((*self.data)[index])
        }
        None
    }


    /// Safely set a value at an index of texture data
    pub fn set_at_index(&mut self, index: usize, value: u8){
        if index >= 0 && index < self.data.len(){
            (*self.data)[index] = value;
        }
    }

    /// Gets a raw pointer to the texture data
    pub unsafe fn get_data_raw_ptr(&mut self) -> *mut u8{
        (*self.data).as_mut_ptr()
    }


}
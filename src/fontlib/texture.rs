use alloc::vec::Vec;
use alloc::alloc::{alloc, dealloc, Layout};
use core::mem::size_of;
pub( crate) struct TextureData {
    data: Option<*mut u8>, // raw bitmap data
    layout: Option<Layout>,
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
        let mut texture_data = Self::default();
        texture_data.width = width;
        texture_data.height = height;
        texture_data.allocate();
        texture_data
    }

    /// MUST BE CALLED AFTER INITIALIZATION
    pub fn allocate(&mut self){
        let size = size_of::<u32>() * self.width as usize * self.height as usize>> 1;
        let layout = Layout::from_size_align(size, 16).unwrap();
        let ptr = unsafe {alloc(layout)};
        self.data = Some(ptr);
        self.layout = Some(layout);
    }

    /// safely get the value in an index of texture data
    pub fn get(&self, index:usize) -> Option<u8>{
        if let Some(data) = self.data{
                if index < self.layout.unwrap().size(){
                     return unsafe {Some(*((data as usize + index) as *mut u8)) }
                }
        }
        None
    }

    /// Unsafely get the value in an index of texture data
    pub unsafe fn get_unchecked(&mut self, index:usize) -> u8{
        unsafe {
            *((self.data.unwrap() as usize + index) as *mut u8)
        }
    }

    /// Safely set a value at an index of texture data
    pub fn set_at_index(&mut self, index: usize, value: u8){
        if let Some(data) = self.data{
            if index < self.layout.unwrap().size(){
                unsafe {*((data as usize + index) as *mut u8) = value}
            }
        }
    }

    /// Swizzles the texture data for PSP usage :)
    pub fn swizzle_texture(&mut self, t_height: usize, t_width:usize,){
        let byte_width = t_width >> 1;
        let texture_size = byte_width * t_height;

        let row_blocks = byte_width >> 4;
        let row_blocks_add = (row_blocks - 1) << 7;
        let mut block_address = 0;

        if let Some(mut data) = self.data{
            if let Ok(layout) = Layout::from_size_align(texture_size, 16){
                let t_data = unsafe {alloc(layout)};

                for i in 0..t_height{
                    let mut block = ((t_data as usize + block_address) as *mut u32);
                    for _ in 0..row_blocks{
                        unsafe {
                            for _ in 0..4{
                                *block = *data as u32;
                                block = (block as usize + 1) as *mut u32;
                                data = (data as usize + 1) as *mut u8;
                            }
                            block = (block as usize + 28) as *mut u32;
                        }
                    }
                    if i & 0x7 == 0x7{
                        block_address += row_blocks_add;
                    }
                    block_address += 16;
                }

                unsafe {dealloc(self.data.unwrap(), self.layout.unwrap());}
                self.data = Some(t_data); // switcheroo
                self.layout = Some(layout); // also switcheroo
            }
        }
    }
    /// Gets a raw pointer to the texture data
    pub unsafe fn get_data_raw_ptr(&self) -> *mut u8{
        self.data.unwrap()
    }

    /// Unsafely set a value at an index of texture data
    ///
    /// If memory beyond the size allocated of the texture data is modified,
    /// segfaults may ensue.
    pub unsafe fn set_at_index_unchecked(&mut self, index: usize, data: u8){
        unsafe {*((self.data.unwrap() as usize + index) as *mut u8) = data}
    }
}

impl Default for TextureData{
    fn default() -> Self {
        Self { // safety is our No. 1 priority :)
            data: None,
            layout: None,
            id: 0,
            width: 0,
            height: 0,
            x: 1,
            y: 1,
            y_size: 0
        }
    }
}

impl Drop for TextureData{
    fn drop(&mut self) {
        unsafe {
            if let Some(data) = self.data{
                dealloc(data, self.layout.unwrap()) // only need for one if let, because layout and data are initialized at the same time.
            }
        };
    }
}
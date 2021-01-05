use alloc::vec::Vec;

pub( crate) struct CharmapData {
    pub(crate) charmap_compression_table: Vec<u16>,
    pub(crate) charmap_compression_table_len: u8,
    pub(crate) charmap: Vec<u16>,
    pub(crate) char_ptr_table: Vec<u16>,
    pub(crate) shadow_charmap: Vec<u16>,
}

impl CharmapData{
    /// Able to extract the character data from the charmap and char_pointer_table in a PGF font file.
    /// It gets the raw data and turns it into a usable vector.
    pub fn extract_char_data(data:&[u8], bits_per_element:u32, number_of_elements:u32) -> Vec<u16>{
        let mut map:Vec<u16> = Vec::with_capacity(data.len()); // Holds the information being extracted from data.
        let mut elem_n = 0; // keeps track of the byte being processed
        for _ in 0..number_of_elements{
            map.push(Self::magical_table_function(bits_per_element, data, &mut elem_n) as u16)
        }
        map
    }

    /// I have no clue on as to what this function does. All I know is that it turns
    /// raw data from a table in a PGF file to something that is readable.
    /// Also, this is used in extracting glyphs from font_data.
    pub fn magical_table_function(bits_per_element:u32, table:&[u8], current_bit:&mut u32) -> u32{
        let mut v = 0;
        for i in 0..bits_per_element as u64 {
            v += ((((table[(*current_bit as usize) / 8] >> ((*current_bit) % 8)) & 1) as u64) << i) as u32;
            *current_bit += 1;
        }
        v
    }
}
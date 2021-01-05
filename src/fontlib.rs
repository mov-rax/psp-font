pub(crate) mod ccclib;
mod texture;
mod char_map;
mod glyph;
mod vertex;
pub mod style;
mod rotation;
pub mod helper;

pub mod fontlib{
    use crate::fontlib::ccclib::Codepages as CP;
    use core::fmt::Error;
    use byteorder::{LittleEndian, ByteOrder};
    use alloc::string::String;
    use alloc::alloc::{alloc, dealloc, Layout}; // unsafe but pretty useful
    use core::mem::size_of;
    use crate::fontlib::texture::TextureData;
    use crate::fontlib::char_map::CharmapData;
    use crate::fontlib::glyph::{Glyph, GlyphBW};
    use crate::fontlib::vertex::FontVertex;
    use crate::fontlib::fontlib::FileType::{PGF, BWFON};
    use crate::fontlib::style::{FontColor, FontStyle};
    use alloc::vec::Vec;
    use alloc::boxed::Box;
    use core::ops::Shl;
    use core::f32::consts::PI;
    use crate::fontlib::rotation::Rotation;
    use crate::fontlib::helper::{PGFFlags, FileType, UCS2};
    use smart_buffer;
    use smart_buffer::SmartBuffer;
    use psp::sys::{sceGuGetMemory, sceGuScissor, sceKernelDcacheWritebackAll, sceGuClutMode, sceGuTexMode, sceGuEnable, sceGuTexImage, sceGuTexFunc, sceGuTexEnvColor, sceGuTexOffset, sceGuTexWrap, sceGuTexFilter, sceGuClutLoad, ClutPixelFormat, GuState, TexturePixelFormat, MipmapLevel, TextureEffect, TextureColorComponent, GuTexWrapMode, TextureFilter, sceKernelDcacheWritebackRange, sceGuDisable, sceGuDrawArray, GuPrimitive, VertexType};
    use psp::sys::{DisplayPixelFormat};
    use psp::Align16;
    use psp::sys::vfpu_context::MatrixSet;


    static mut CLUT: Align16<u16> = Align16(0); // Color Lookup Table

    /// An internal structure that is used when reading a bitmap font file
    /// Similar to the PGF_Header struct in intrafont, however, this structure is not
    /// loaded with the raw data from reading the file, it is safely read.
    pub struct PGFHeader{       // TYPICAL VALUES FOR THE DATA
        header_start:u16,   // 0
        header_len:u16,     // 392 or 412
        pgf_id:[char;4],    // "PGF0"
        revision:u32,       // 2 or 3
        version:u32,        // 6
        charmap_len:u32,    // MAX: 65536   (number of char-glyphs in fontdata)
        charptr_len:u32,    // MAX: 512     (number of elements in char_pointer_table)
        charmap_bpe:u32,    //              (number of bits per element in charmap)
        charptr_bpe:u32,    //              (number of bits per element in char_pointer_table)
        family:[char;64],   // "Comic Sans" (the font name/family)
        style:[char;64],    // "Bold"       (the font type/style)
        charmap_min:u16,    //              (first element in charmap)
        charmap_max:u16,    //              (last element in charmap)
        advance:(u32,u32),  // (max x-advance, max y-advance)
        dimension_table_len:u8,
        adjust_table_len:(u8,u8), // (x-adjust-table-len, y-adjust-table-len)
        advance_table_len:u8,
        shadowmap_len:u32,  // MAX: 512     (number of elements in shadow_charmap (number of shadow-glyphs in fontdata))
        shadowmap_bpe:u32,  // 16           (number of bits per element in shadow_charmap)
        shadowscale:(u32, u32)  // (x-shadowscale,y-shadowscale)
    }

    #[feature(min_const_generics)]
    impl PGFHeader{
        const HEADER_SIZE:usize = 0x184; // 388 bytes in length
        pub fn load_from_bytes(file:&Vec<u8>) -> Result<PGFHeader, Error>{ // Uses fmt::Error due to no std being present
            if file.len() < Self::HEADER_SIZE{ // the file must be invalid if it not large enough for a header
                return Err(Error)
            }
            let header_start= LittleEndian::read_u16(&file[0..=1]);
            let header_len  = LittleEndian::read_u16(&file[0x2..=0x3]);
            let mut pgf_id = {
                let mut arr = [' ';4];
                for i in 0..4{
                    arr[i] = file[0x4+i] as char;
                }
                arr
            };
            let revision    = LittleEndian::read_u32(&file[0x8..=0xB]);
            let version     = LittleEndian::read_u32(&file[0xC..=0xF]);
            let charmap_len = LittleEndian::read_u32(&file[0x10..=0x13]);
            let charptr_len = LittleEndian::read_u32(&file[0x14..=0x17]);
            let charmap_bpe = LittleEndian::read_u32(&file[0x18..=0x1B]);
            let charptr_bpe = LittleEndian::read_u32(&file[0x1C..=0x1F]);
            let family = {
                let mut arr = [' ';64];
                for i in 0..64{
                    arr[i] = file[0x35+i] as char;
                }
                arr
            };
            let style = {
                let mut arr = [' ';64];
                for i in 0..64{
                    arr[i] = file[0x75+i] as char;
                }
                arr
            };
            let charmap_min = LittleEndian::read_u16(&file[0xB6..=0xB7]);
            let charmap_max = LittleEndian::read_u16(&file[0xB8..=0xB9]);
            let advance = (LittleEndian::read_u32(&file[0xEC..=0xEF]), LittleEndian::read_u32(&file[0xF0..=0xF3]));
            let dimension_table_len = file[0x102];
            let adjust_table_len = (file[0x103], file[0x104]);
            let advance_table_len = file[0x105];
            let shadowmap_len = LittleEndian::read_u32(&file[0x16C..=0x16F]);
            let shadowmap_bpe = LittleEndian::read_u32(&file[0x170..=0x173]);
            let shadowscale = (LittleEndian::read_u32(&file[0x178..=0x17B]), LittleEndian::read_u32(&file[0x17C..=0x17F]));

            Ok(PGFHeader{ header_start, header_len, pgf_id, revision, version, charmap_len, charptr_len, charmap_bpe, charptr_bpe, family,
                style, charmap_min, charmap_max, advance, dimension_table_len, adjust_table_len, advance_table_len, shadowmap_len, shadowmap_bpe, shadowscale
            })
        }
    }

    #[feature(min_const_generics)]
    pub struct Font<'a>{
        font_data: &'a [u8],
        texture: TextureData,
        charmap_data: CharmapData,
        glyphs: Vec<Glyph>,
        glyphs_bw: Vec<GlyphBW>,
        shadow_glyphs: Vec<Glyph>,
        alt_font: Box<Option<Font<'a>>>,
        font_vertices: Vec<FontVertex>,
        size: f32,
        color: FontColor,
        shadow_color: FontColor,
        rotation: Rotation,
        pub(crate) options: PGFFlags,
        n_chars: u16,
        n_shadows: u16,
        filetype: FileType,
        advance: (u8,u8),
        advance_table: Vec<i32>,
        shadow_scale: u8,
    }

    impl<'a> Font<'a>{
        pub fn new(data:&Vec<u8>, options: PGFFlags) -> Font{
            let header = PGFHeader::load_from_bytes(data);
            if let Err(_) = &header{
                panic!("PSP-FONT: PGF Header of font file is invalid.");
            }
            let header = header.unwrap();

            psp::dprintln!("P: {:X}", header.pgf_id[0]);
            psp::dprintln!("G: {:X}", header.pgf_id[1]);
            psp::dprintln!("F: {:X}", header.pgf_id[2]);
            psp::dprintln!("0: {:X}", header.pgf_id[3]);


            let filetype = if header.pgf_id == ['P','G','F','0']{ // could probably be more robust
                FileType::PGF
            } else if data.len() == 1023372{
                FileType::BWFON
            } else{
                panic!("PSP-FONT: File provided is an unsupported format -> {:?}", header.pgf_id); // now prints what the ID is
            };

            let mut font = match filetype{
                FileType::PGF => {
                    let n_chars = header.charptr_len as u16;
                    let charmap_compression_table_len = if header.revision == 3{
                        7
                    } else{
                        1
                    };
                    let n_shadows = header.shadowmap_len as u16;
                    let advance = (header.advance.0 as u8, header.advance.1 as u8);
                    let shadow_scale = header.shadowscale.0 as u8;
                    let glyphs = Vec::with_capacity(n_chars as usize);
                    let shadow_glyphs = vec![Glyph::default(); n_shadows as usize]; // pre-initialized with 0s
                    let size = 1.0f32;
                    let color = FontColor::WHITE;
                    let shadow_color = FontColor::BLACK;
                    // texture initialization
                    let width:u32 = if options.contains(PGFFlags::CACHE_LARGE){
                        512
                    } else{
                        256
                    };
                    let height = width;
                    let texture = TextureData::new(width,height);

                    // This block gets pertinent information from the PGF file
                    // offset is used to find the beginning of a block of data in a PGF file
                    // end is used to prevent calculations occurring twice
                    let mut offset = header.header_len as usize + (header.dimension_table_len as usize + header.adjust_table_len.0 as usize + header.adjust_table_len.1 as usize)*8;
                    let end = offset + header.advance_table_len as usize * 8;
                    let mut advance_table:Vec<i32> = (&data[offset..end]).to_vec()
                        .chunks_exact(4)
                        .map(|c| i32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                        .collect();

                    offset += end - offset;

                    let end = offset + header.shadowmap_len as usize * 2;
                    let mut shadow_charmap:Vec<u16> = (&data[offset..end]).to_vec()
                        .chunks_exact(2)
                        .map(|c| u16::from_le_bytes([c[0], c[1]]))
                        .collect(); // shadow_charmap holds UCS2 values of chars that have a shadow glyph in font data (it could be 0 if there is no shadow)
                    offset += end - offset;

                    let mut charmap_compression_table = if header.revision == 3{ // compression table only exists in revision 3 pgf files
                        let end = offset + 7 * size_of::<u16>() * 2;
                        let val = (&data[offset..end]).to_vec()
                            .chunks_exact(2)
                            .map(|c| u16::from_le_bytes([c[0], c[1]]))
                            .collect::<Vec<u16>>();
                        offset += end - offset; // adds to the offset so that everything ahead gets read correctly
                        val
                    } else{
                        let mut temp = Vec::new();
                        temp.push(header.charmap_min);
                        temp.push(header.charmap_max);
                        temp
                    };

                    let end = offset + ((header.charmap_len as usize * header.charmap_bpe as usize + 31)/32) * 4;
                    let charmap = CharmapData::extract_char_data(&data[offset..end], header.charmap_bpe, header.charmap_len).iter()
                        .map(|element| if *element < n_chars{
                            *element
                        } else {
                            65535
                        }).collect();
                    offset += end - offset;

                    let end = offset + ((header.charptr_len as usize * header.charptr_bpe as usize + 31)/32) * 4;
                    let char_ptr_table = CharmapData::extract_char_data(&data[offset..end], header.charptr_bpe, header.charptr_len);
                    offset += end - offset;
                    let mut font_data = &data[offset..]; // font data is the rest of the file.

                    let charmap_data = CharmapData{
                        charmap_compression_table,
                        charmap_compression_table_len,
                        charmap,
                        char_ptr_table,
                        shadow_charmap};

                    let mut font = Font {
                        font_data,
                        texture,
                        charmap_data,
                        glyphs,
                        glyphs_bw: vec![GlyphBW::default(); n_chars as usize],
                        shadow_glyphs,
                        alt_font: Box::new(None),
                        font_vertices: Vec::new(),
                        size,
                        color,
                        shadow_color,
                        rotation: Rotation::default(),
                        options,
                        n_chars,
                        n_shadows,
                        filetype,
                        advance,
                        advance_table,
                        shadow_scale
                    };

                    // All the data has been extracted from the file. Now, calculations must be done :)

                    if options.contains(PGFFlags::CACHE_ASCII){
                        font.n_chars = 1; // assume there's at least one char
                        for i in 0..128{
                            if font.get_char_id(i) < 65535{
                                font.n_chars += 1;
                            }
                        }
                        font.n_chars -= 1; // correct assumption
                    }
                    font.extract_font_glyphs();
                    font
                },
                FileType::BWFON => { // Not yet implemented
                    panic!("BWFON File types are not implemented yet")
                }
            };

            let omega = font;
            omega
        }

        /// Gets the character id for a given index from the charmap
        fn get_char_id(&self, index:u16) -> u16{
            let mut j:usize= 0;
            let mut id:u16 = 0;
            let mut found = false;
            while j < self.charmap_data.charmap_compression_table_len as usize && !found{
                if (index >= self.charmap_data.charmap_compression_table[j * 2]) &&
                    (index < self.charmap_data.charmap_compression_table[j * 2] + self.charmap_data.charmap_compression_table[j * 2 + 1]){
                    id += index - self.charmap_data.charmap_compression_table[j * 2];
                    found = true;
                } else {
                    id += self.charmap_data.charmap_compression_table[j * 2 + 1];
                }
                j += 1;
            }
            if !found {
                return 65535; // character is not in charmap
            }
            if self.filetype == FileType::PGF{
                id = self.charmap_data.charmap[id as usize]; // BWFON has right id already
            }
            if id >= self.n_chars{
                return 65535; // character is not in font_data or is not in ASCII-cache
            }
            return id;
        }

        /// Extracts the glyphs present in the font data
        fn extract_font_glyphs(&mut self){
            let mut x:u32 = 1;
            let mut y:u32 = 1;
            let mut y_size:u32 = 0;
            let mut j:u32 = 0;
            let mut count = 0;

            // Normal character glyphs
            for i in 0..self.n_chars as usize{
                j = self.charmap_data.char_ptr_table[i] as u32 * 4 * 8;
                unsafe { self.extract_glyph(&mut j, None)}
                let temp_data = self.glyphs[i];
                if !self.glyphs[i].flags.contains(PGFFlags::BMP_HORIZONTAL_ROWS) != !self.glyphs[i].flags.contains(PGFFlags::BMP_VERTICAL_ROWS){ // H_ROWS xor V_ROWS
                    count += 1;
                    // if they are not the same value, this will execute
                    if self.glyphs[i].height as u16 > self.texture.y_size{
                        self.texture.y_size = self.glyphs[i].height as u16; // find max glyph height
                    }
                    if (x + self.glyphs[i].width as u32) > self.texture.width{
                        y += y_size + 1;
                        x = 1;
                        y_size = 0;
                    }
                    if self.glyphs[i].height as u32 > y_size{
                        y_size = self.glyphs[i].height as u32;
                    }
                    x += self.glyphs[i].width as u32 + 1;
                }
            }

            // shadow glyphs
            let mut char_id:u16 = 0;
            let mut shadow_id:usize = 0;
            for i in 0..self.n_chars as usize{
                shadow_id = self.glyphs[i].shadow_id as usize;
                if self.charmap_data.shadow_charmap.len() == 0{
                    char_id = 65535; // char not in charmap
                } else {
                    char_id = self.get_char_id(self.charmap_data.shadow_charmap[shadow_id]);
                }
                if (char_id < self.n_chars) && (self.shadow_glyphs[shadow_id].shadow_id == 0){
                    // valid char and shadow glyph not yet loaded
                    j = self.charmap_data.char_ptr_table[char_id as usize] as u32 * 4 * 8;
                    unsafe { self.extract_glyph(&mut j, Some(shadow_id))}

                    if !self.shadow_glyphs[shadow_id].flags.contains(PGFFlags::BMP_HORIZONTAL_ROWS) != !self.shadow_glyphs[shadow_id].flags.contains(PGFFlags::BMP_VERTICAL_ROWS){
                        // H_ROWS xor V_ROWS (if they are not the same value then this will execute)
                        if self.shadow_glyphs[shadow_id].height as u16 > self.texture.y_size{
                            self.texture.y_size = self.shadow_glyphs[shadow_id].height as u16; // find max glyph height
                        }
                        if x + self.shadow_glyphs[shadow_id].width as u32 > self.texture.width{
                            y += y_size + 1;
                            x = 1;
                            y_size = 0;
                        }
                        if self.shadow_glyphs[shadow_id].height as u32 > y_size{
                            y_size = self.shadow_glyphs[shadow_id].height as u32;
                        }
                        x += self.shadow_glyphs[shadow_id].width as u32 + 1;
                    }
                }

            }

            // Clear unneeded tables to lower memory usage
            self.advance_table.clear();
            self.charmap_data.char_ptr_table.clear();
            self.charmap_data.shadow_charmap.clear();

            // sceKernelDcacheWWritebackAll()

            if self.options.contains(PGFFlags::CACHE_ASCII) && (y + y_size + 1 <= self.texture.height){ // work-around for cache (not CACHE_ASCII like in C)
                // cache it!!!
                self.options.remove(PGFFlags::CACHE_ASCII);
                self.pre_cache(PGFFlags::CACHE_ASCII);
            }

        }

        /// Extracts glyph from the offset in `font_data`.
        ///
        /// - `offset` is the offset utilized in reading data from the font metric of a glyph.
        /// Every time this function is called, a glyph will be attempted to be read.
        /// This function should only ever be called from `extract_font_glyphs()`.
        ///
        /// - If this function is successfully called, a glyph will be added to self.glyphs
        ///
        /// - If this function is called too many times the currently running program WILL panic
        /// once it reaches beyond the boundaries of `font_data`.
        ///
        /// - For extraction of a shadow glyph, set the `shadow_id` to `Some(shadow_id)`.
        ///
        /// - For extraction of a normal glyph, set the `shadow_id` to `None`.
        unsafe fn extract_glyph(&mut self, offset:&mut u32, shadow_id: Option<usize>){
            if shadow_id == None{
                *offset += 14; //skip the offset pos of the shadow metric
            } else {
                *offset += CharmapData::magical_table_function(14, self.font_data, offset)*8; // skip to shadow
            }

            // Because none of these values are normal sizes, i.e., u8, u16, u32... They will have to be read using the magical function
            let width = CharmapData::magical_table_function(7, self.font_data, offset) as u8;
            let height = CharmapData::magical_table_function(7, self.font_data, offset) as u8;
            let mut left = CharmapData::magical_table_function(7, self.font_data, offset) as i8;
            let mut top = CharmapData::magical_table_function(7, self.font_data, offset) as i8;
            let flags = PGFFlags::from_bits(CharmapData::magical_table_function(6, self.font_data, offset)).unwrap();
            if left >= 64{
                let mut temp = left as i16;
                temp -= 128;
                left = temp as i8;
            }
            if top >= 64{
                let mut temp = top as i16;
                temp -= 128;
                top = temp as i8;
            }
            let mut glyph = Glyph {
                x: 0, // will change
                y: 0, // will change
                width,
                height,
                left,
                top,
                flags,
                shadow_id: 0, // will change
                advance: 0, // will change
                offset: 0 // will change
            };
            /// Extended Metric
            if shadow_id == None{
                *offset += 7; //skip magic number
                glyph.shadow_id = CharmapData::magical_table_function(9, self.font_data, offset) as u16;
                *offset += {
                    let mut x = 24;
                    if !flags.contains(PGFFlags::NO_EXTRA1){
                        x += 56;
                    }
                    if !flags.contains(PGFFlags::NO_EXTRA2){
                        x += 56;
                    }
                    if !flags.contains(PGFFlags::NO_EXTRA3){
                        x += 56;
                    }
                    x
                }; //offsets by certain amounts
                //glyph.advance = (*self.advance_table.get(CharmapData::magical_table_function(8, self.font_data, offset) as usize * 2).unwrap_or(&0) / 16) as i8;
                glyph.advance = (self.advance_table[CharmapData::magical_table_function(8, self.font_data, offset) as usize * 2] / 16) as i8;
                glyph.offset = *offset / 8;
                self.glyphs.push(glyph);
            } else {
                glyph.shadow_id = 65535;
                glyph.advance = 0;
                glyph.offset = *offset / 8;
                self.shadow_glyphs[shadow_id.unwrap()] = glyph;
            }

        }


        fn pre_cache(&mut self, options: PGFFlags){
            let mut ac = false;
            if !options.contains(PGFFlags::CACHE_ASCII){
                return; // no pre-cache requested
            }
            if self.options.contains(PGFFlags::CACHE_ASCII){
                return; // already pre-cached
            } else {
                ac = true;
            }

            // cache all glyphs

            let mut y = 0;
            self.texture.x = 1;
            self.texture.y = 1;
            self.texture.y_size = 0;
            for i in 0..self.n_chars as usize{
                y = self.texture.y;
                self.get_bmp(i, PGFFlags::CHAR_GLYPH);
                if (self.texture.y > y) || (self.texture.y_size < self.glyphs[i].height as u16){
                    self.texture.y_size = self.glyphs[i].height as u16; // minimize ysize after newline in cache ( only valid for pre-cached glyphs)
                }
                if self.texture.y < y{
                    return; // char did not fit into cache -> abort precache (should reset cache and glyph.flags)
                }
            }
            for i in 0..self.n_shadows as usize{
                y = self.texture.y;
                self.get_bmp(i, PGFFlags::SHADOWGLYPH);
                if self.texture.y > y || self.texture.y_size < self.shadow_glyphs[i].height as u16{
                    self.texture.y_size = self.shadow_glyphs[i].height as u16;
                }
                if self.texture.y < y{
                    return; // char did not into cache -> abort precache (should reset cache and glyph.flags)
                }
            }

            self.texture.height = (self.texture.y as u32+ self.texture.y_size as u32 + 7) & !7;
            if self.texture.height > self.texture.width{
                self.texture.height = self.texture.width;
            }

            // reduce fontdata [ NOT IMPLEMENTED DUE TO FONTDATA BEING AN IMMUTABLE REFERENCE TO THE FILE ]

            // Swizzle texture
            unsafe {sceKernelDcacheWritebackAll();}
            self.swizzle();
            unsafe {sceKernelDcacheWritebackAll()};

            self.activate();
            if ac{
                self.options.insert(PGFFlags::CACHE_ASCII);
            }
        }

        /// Does the sce function calls to activate the fonts
        fn activate(&self){
            unsafe {

                sceGuClutMode(ClutPixelFormat::Psm8888, 0, 255, 0);
                sceGuClutLoad(2, CLUT.0 as *mut u16 as *mut _);
                sceGuEnable(GuState::Texture2D);
                sceGuTexMode(TexturePixelFormat::PsmT4, 0, 0, if self.options.contains(PGFFlags::CACHE_ASCII) { 1 } else { 0 });
                sceGuTexImage(MipmapLevel::None, self.texture.width as i32, self.texture.width as i32, self.texture.width as i32, self.texture.get_data_raw_ptr() as *mut _);
                sceGuTexFunc(TextureEffect::Modulate, TextureColorComponent::Rgba);
                sceGuTexEnvColor(0x0);
                sceGuTexOffset(0.0, 0.0);
                sceGuTexWrap(GuTexWrapMode::Clamp, GuTexWrapMode::Clamp);
                sceGuTexFilter(TextureFilter::Linear, TextureFilter::Linear);
            }

        }

        /// Gets the bitmap data for a character with a given ID and glyph_type
        fn get_bmp(&mut self, id: usize, glyph_type: PGFFlags) -> bool{
            let mut glyph_flags = PGFFlags::NONE; // will be read at the end to modify a glyph

            if self.options.contains(PGFFlags::CACHE_ASCII){
                return false // swizzled texture
            }

            let mut glyph = Glyph::default(); // will be modified

            //let mut glyph = &mut self.glyphs[0]; // will probably be pointing to a different glyph below.
            if self.filetype == FileType::PGF{

                if glyph_type.contains(PGFFlags::CHAR_GLYPH){
                    glyph = self.glyphs[id];
                    glyph_flags.insert(PGFFlags::CHAR_GLYPH);
                } else {
                    glyph = self.shadow_glyphs[id];
                    glyph_flags.insert(PGFFlags::SHADOWGLYPH);
                }

            } else { // Filetype BWFON

               if glyph_type.contains(PGFFlags::CHAR_GLYPH){
                   glyph = self.glyphs[0];
                   glyph.flags = self.glyphs_bw[id].flags | PGFFlags::BMP_HORIZONTAL_ROWS;
                   glyph_flags.insert(PGFFlags::CHAR_GLYPH);
               } else {
                   glyph = self.shadow_glyphs[0];
                   glyph_flags.insert(PGFFlags::SHADOWGLYPH);
               }
                glyph.offset = id as u32 * 36; // 36 bytes/char
            }

            if glyph.flags.contains(PGFFlags::CACHED){ // nothing needed to be cached (it has already been cached)
                return true
            }

            let mut b = glyph.offset * 8; // location of the texture?

            if (glyph.width > 0) && (glyph.height > 0){
                if !glyph.flags.contains(PGFFlags::BMP_HORIZONTAL_ROWS) != !glyph.flags.contains(PGFFlags::BMP_VERTICAL_ROWS){
                    // H_ROWS xor V_ROWS
                    if self.texture.x as u32 + glyph.width as u32 + 1 > self.texture.width as u32{
                        self.texture.y += self.texture.y_size + 1;
                        self.texture.x = 1;
                    }
                    if self.texture.y as u32 + glyph.height as u32 + 1 > self.texture.height{
                        self.texture.y = 1;
                        self.texture.x = 1;
                    }
                    glyph.x = self.texture.x;
                    glyph.y = self.texture.y;

                    // draw bmp!! :)


                    let nib_calc = |nibble:u8| if nibble < 8 { nibble } else { 15 - nibble }; // no ternary here :)

                    let mut i = 0;
                    //let mut j = 0;
                    let mut xx = 0;
                    let mut yy = 0;
                    let mut nibble = 0;
                    let mut value = 0;
                    if self.filetype == FileType::PGF{
                        // for compressed PGF format
                        while i < ((glyph.width as u32 * glyph.height as u32) as u8) as u32{
                            nibble = CharmapData::magical_table_function(4, self.font_data, &mut b) as u8;
                            if nibble < 8 {
                                value = CharmapData::magical_table_function(4, self.font_data, &mut b);
                            }

                            for _ in 0..nib_calc(nibble){
                                if i >= ((glyph.width as u32 * glyph.height as u32) as u8) as u32{ break; }
                                if nibble >= 8{
                                    value = CharmapData::magical_table_function(4, self.font_data, &mut b);
                                }
                                if glyph.flags.contains(PGFFlags::BMP_HORIZONTAL_ROWS){
                                    xx = i % glyph.width as u32;
                                    yy = i / glyph.width as u32;
                                } else{
                                    xx = i / glyph.height as u32;
                                    yy = i % glyph.height as u32;
                                }
                                let index = (self.texture.x as u32 + xx + (self.texture.y as u32 + yy) * self.texture.width >> 1) as usize;
                                if ((self.texture.x as u32 + xx) & 1) != 0 {
                                    self.texture.set_at_index(index, self.texture.get(index).unwrap() & 0x0F);
                                    self.texture.set_at_index(index, self.texture.get(index).unwrap() | (value << 4) as u8);
                                } else {
                                    self.texture.set_at_index(index, self.texture.get(index).unwrap() & 0xF0);
                                    self.texture.set_at_index(index, self.texture.get(index).unwrap() | value as u8);
                                }
                                i += 1;
                            }
                        }
                    }
                    else { // NOT PGF ( uncompressed BWFON)
                        yy = 0;
                        while yy < glyph.height as u32{
                            xx = 0;
                            while xx < glyph.width as u32{
                                if glyph_type.contains(PGFFlags::CHAR_GLYPH){ // Getting character BMP
                                    value = CharmapData::magical_table_function(1, self.font_data, &mut b) * 0x0F; // scale 1 bit/pix to 4 bit/pix
                                    let index = ((self.texture.x as u32 + (7 - (xx & 7) + (xx & 248)) + (self.texture.y as u32 + yy) * self.texture.width) >> 1) as usize;
                                    if (self.texture.x as u32 + (7 - (xx & 7) + (xx & 248))) & 1 != 0{
                                        self.texture.set_at_index(index, self.texture.get(index).unwrap() & 0x0F);
                                        self.texture.set_at_index(index, self.texture.get(index).unwrap() | (value << 4) as u8);
                                    } else {
                                        self.texture.set_at_index(index, self.texture.get(index).unwrap() & 0xF0);
                                        self.texture.set_at_index(index, self.texture.get(index).unwrap() | value as u8);
                                    }
                                } else { // Getting shadow BMP
                                    value = CharmapData::magical_table_function(4, self.font_data, &mut b);
                                    let index = ((self.texture.x as u32 + xx + (self.texture.y as u32 + yy) * self.texture.width) >> 1) as usize;
                                    if (self.texture.x as u32 + xx) & 1 != 0{
                                        self.texture.set_at_index(index, self.texture.get(index).unwrap() & 0x0F);
                                        self.texture.set_at_index(index, self.texture.get(index).unwrap() | (value << 4) as u8);
                                    } else {
                                        self.texture.set_at_index(index, self.texture.get(index).unwrap() & 0xF0);
                                        self.texture.set_at_index(index, self.texture.get(index).unwrap() | value as u8);
                                    }
                                }
                            }
                            yy += 1;
                        }
                    }
                    // Time to erase the border around the glyphs
                    for i in (self.texture.x/2) as usize..((self.texture.x + glyph.width as u16 + 1)/2) as usize{
                        self.texture.set_at_index(i + (self.texture.y - 1) as usize * (self.texture.width / 2) as usize, 0);
                        self.texture.set_at_index(i + (self.texture.y + glyph.height as u16) as usize * (self.texture.width / 2) as usize, 0);
                    }
                    for i in (self.texture.y - 1) as usize..(self.texture.y + glyph.height as u16 + 1) as usize{
                        self.texture.set_at_index((self.texture.x as usize - 1 + i * self.texture.width as usize) >> 1, if self.texture.x & 1 != 0 { 0xF0 } else { 0x0F } );
                        let index = (self.texture.x as usize + glyph.width as usize + i * self.texture.width as usize) >> 1;
                        self.texture.set_at_index(index, if (self.texture.x + glyph.width as u16) & 1 != 0{
                            self.texture.get(index).unwrap() & 0xF0
                        } else {
                            self.texture.get(index).unwrap() & 0x0F
                        } );
                    }

                    self.texture.x += glyph.width as u16; // add empty gap to prevent interpolation artifacts from showing

                    if glyph_flags.contains(PGFFlags::CHAR_GLYPH){ // modifies the glyphs at this location
                        self.glyphs[id] = glyph;
                    } else if glyph_flags.contains(PGFFlags::SHADOWGLYPH){ // just to be sure it still is not NONE
                        self.shadow_glyphs[id] = glyph;
                    }

                    // mark dirty glyphs as uncached
                    if self.filetype == PGF{
                        // for PGF glyphs
                        for i in 0..self.n_chars as usize{
                            if self.glyphs[i].flags.contains(PGFFlags::CACHE_MASK) && (self.glyphs[i].y == glyph.y){
                                if (self.glyphs[i].x + self.glyphs[i].width as u16 + 1 > glyph.x) && (self.glyphs[i].x < glyph.x + glyph.width as u16 + 1){
                                    self.glyphs[i].flags.remove(PGFFlags::CACHED);
                                }
                            }
                        }
                    } else {
                        // for BWFON glyphs
                        for i in 0..self.n_chars as usize{
                            if self.glyphs_bw[i].flags.contains(PGFFlags::CACHE_MASK) && (self.glyphs_bw[i].y == glyph.y){
                                if (self.glyphs_bw[i].x + self.glyphs[i].width as u16 + 1 > glyph.x) && (self.glyphs_bw[i].x < glyph.x + glyph.width as u16 + 1){
                                    self.glyphs_bw[i].flags.remove(PGFFlags::CACHED);
                                }
                            }
                        }
                    }
                    // Shadow Glyphs
                    for i in 0..self.n_chars as usize{
                        if self.shadow_glyphs[i].flags.contains(PGFFlags::CACHE_MASK) && (self.shadow_glyphs[i].y == glyph.y){
                            if (self.shadow_glyphs[i].x + self.shadow_glyphs[i].width as u16 + 1 > glyph.x) && (self.glyphs_bw[i].x < glyph.x + glyph.width as u16 + 1){
                                self.shadow_glyphs[i].flags.remove(PGFFlags::CACHED);
                            }
                        }
                    }
                }
                else {
                    // H_ROWS xor V_ROWS was false. (in other words, they were the same value)
                    return false // transposition = 0 or overlay glyph
                }
            } else {
                // (glyph.width > 0) && (glyph.height > 0) is false
                if glyph_flags.contains(PGFFlags::CHAR_GLYPH){ // modifications must be done
                    self.glyphs[id].x = 0;
                    self.glyphs[id].y = 0;
                } else if glyph_flags.contains(PGFFlags::SHADOWGLYPH){
                    self.shadow_glyphs[id].x = 0;
                    self.shadow_glyphs[id].y = 0;
                }
            }
            glyph.flags.insert(PGFFlags::CACHED);

            if self.filetype == BWFON {
                if glyph.flags.contains(PGFFlags::CHAR_GLYPH){
                    self.glyphs_bw[id].x = glyph.x;
                    self.glyphs_bw[id].y = glyph.y;
                    self.glyphs_bw[id].flags = glyph.flags;
                    self.glyphs[0] = glyph;
                } else {
                    self.shadow_glyphs[0] = glyph;
                }
            } else {
                if glyph.flags.contains(PGFFlags::CHAR_GLYPH){
                    self.glyphs[id] = glyph;
                } else {
                    self.shadow_glyphs[id] = glyph;
                }
            }

            true // returns true if all went swell and the bmp was cached
        }

        /// Swizzles the font textures for PSP usage :)
        fn swizzle(&mut self){
            self.texture.swizzle_texture(self.texture.height as usize, self.texture.width as usize);
            self.options.insert(PGFFlags::CACHE_ASCII);
        }

        pub fn set_style(&mut self, style: FontStyle){
            self.size = style.size;
            self.color = style.color;
            self.shadow_color = style.shadow_color;
            let tolerance =  0.0078125f32; // 1/(2^7)
            let diff = if self.rotation.angle > style.angle { self.rotation.angle - style.angle } else { style.angle - self.rotation.angle };
            if diff > tolerance{ // avoid recomputations
                self.rotation.angle = style.angle;
                if self.rotation.angle == 0.0{
                    self.rotation.sin = 0.0;
                    self.rotation.cos = 1.0;
                } else {
                    self.rotation.sin = unsafe { cosf32(style.angle * PI / 180.0 + PI)};
                    self.rotation.cos = unsafe { cosf32(style.angle * PI / 180.0)};
                }
                self.rotation.is_rotated = !(self.rotation.sin == 0.0 && self.rotation.cos == 1.0);
            }
            self.options = (style.options & PGFFlags::OPTIONS_MASK) | (style.options & PGFFlags::STRING_MASK) | (style.options & PGFFlags::CACHE_MASK);
            if (self.options & PGFFlags::WIDTH_MASK).bits() == 0{
                self.options.insert(PGFFlags::from_bits((self.advance.0 as u32/ 8) & PGFFlags::WIDTH_MASK.bits()).unwrap());
            }
        }

        pub fn print(&mut self, x: f32, y:f32, text: &str){
            self.print_column_ex(x,y,0.0,text);
        }

        pub fn print_column_ex(&mut self, x: f32, y: f32, column: f32, text: &str){
            if text.len() <= 0 { return }

            let mut buffer = buf!(0, 64, text.len()); // A hybrid stack/heap buffer
            Self::encode(text, &mut buffer); // Encodes UTF-8 text to UCS2

            if self.options.contains(PGFFlags::SCROLL_LEFT){
                for i in 0..text.len(){
                    if buffer[i] == '\n' as u16{
                        buffer[i] = ' ' as u16;
                    }
                }
            }

            if column >= 0.0{
                let length = buffer.get_size();
                self.print_column_ucs2_ex(x,y,column,buffer, 0, length);
            } else {

            }

        }

        fn measure_text_ucs2_ex<const M: usize>(&mut self, text: &SmartBuffer<u16, M>, offset:usize,  length: i32) -> f32{
            if length <= 0 || text.get_size() == 0{
                return 0.0;
            }
            let mut x = 0.0f32;

            for i in 0..length{
                if text[i as usize +offset] == '\n' as u16{
                    break;
                }
                let char_id = self.get_char_id(text[i as usize +offset]);
                if char_id < self.n_chars{
                    let glyph_ptr = if self.filetype == FileType::PGF { char_id } else { 0 } as usize;
                    x += if self.options.contains(PGFFlags::WIDTH_FIX) {
                        (self.options & PGFFlags::WIDTH_MASK).bits() as f32 * self.size
                    } else {
                        self.glyphs[glyph_ptr].advance as f32 * self.size * 0.25
                    };
                } else {
                    if let Some(font) = &mut *self.alt_font{
                        //x += font.measure_text_ucs2_ex() -- CANNOT CURRENTLY IMPLEMENT THIS C CODE IN RUST :( --TODO: Implement this somehow
                    }
                }
            }
            return x;
        }


        fn print_column_ucs2_ex<const M: usize>(&mut self, mut x:f32, y:f32, column:f32, mut text: SmartBuffer<u16, M>, offset: usize, length:usize) -> f32{
            const VERTEX_PER_QUAD:usize = 6;

            if length <= 0{
                return x;
            }

            let exist_add = |shadow_id| if shadow_id != 0 { 1u16 } else { 0u16 };

            if self.options.contains(PGFFlags::SCROLL_LEFT){
                for i in 0..length{
                    if text[i+offset] == '\n' as u16{
                        text.map(|c| if c == '\n' as u16 { ' ' as u16 } else { c }); // Modifying the initial buffer and then returning it is faster and more efficient than what was done in C
                        return self.print_column_ucs2_ex(x,y,column, text, 0, length)
                    }
                }
            }
            let mut color = self.color;
            let mut shadow_color = self.shadow_color;
            let glyph_scale = self.size;
            let (mut width, mut height) = (0.0f32, self.advance.1 as f32 * glyph_scale / 4.0);
            let (mut left, top) = (x, y - 2.0 * height);
            let (mut eol, mut n_spaces, mut scroll, mut text_width) = (-1i32, -1i32, 0, 0);
            let mut fill = 0.0f32;
            let (mut xl, mut xr, mut yu, mut yd, mut ul, mut ur, mut vu, mut vd) = (0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32);
            let v_buffer:Option<FontVertex> = None;

            let (mut v0, mut v1, mut v2, mut v3, mut v4, mut v5) =  (None, None, None, None, None, None);
            let (mut s0, mut s1, mut s2, mut s3, mut s4, mut s5) =  (None, None, None, None, None, None);
            let (mut j, mut n_glyphs, mut last_n_glyphs, mut n_sglyphs, mut changed, mut count) = (0,0,0,0,false,0);
            let (mut char_id, mut subucs2, mut glyph_id, mut glyph_ptr, mut shadow_glyph_ptr) = (0,0u16,0,0,0);

            // count number of glyphs to draw and cache BMPs
            loop {
                changed = false;
                n_glyphs = 0;
                n_sglyphs = 0;
                last_n_glyphs = 0;
                for i in 0..length{
                    char_id = self.get_char_id(text[i+offset]) as usize; // char
                    if (char_id as u16) < self.n_chars{
                        if self.filetype == PGF{
                            // PGF FILE
                            if self.glyphs[char_id].flags.contains(PGFFlags::BMP_OVERLAY){
                                // overlay glyph?
                                for j in 0..3{
                                    subucs2 = (self.font_data[(self.glyphs[char_id].offset + j * 2) as usize] as u32 +
                                        (self.font_data[(self.glyphs[char_id].offset + j * 2 + 1) as usize] as u32 * 256) as u32) as u16;

                                    if subucs2 != 0{
                                        glyph_id = self.get_char_id(subucs2);
                                        if glyph_id < self.n_chars{
                                            n_glyphs += 1;
                                            if !self.glyphs[glyph_id as usize].flags.contains(PGFFlags::CACHED){
                                                if self.get_bmp(glyph_id as usize, PGFFlags::CHAR_GLYPH){
                                                    changed = true; // something changed :o
                                                }
                                            }
                                        }
                                    }
                                }
                            } else { // Not an overlay glyph
                                n_glyphs += 1;
                                if !self.glyphs[char_id].flags.contains(PGFFlags::CACHED){
                                    if self.get_bmp(char_id, PGFFlags::CHAR_GLYPH){
                                        changed = true; // something changed!!!
                                    }
                                }
                            }

                            if n_glyphs > last_n_glyphs{
                                // Only add shadows if they exist!
                                n_sglyphs += exist_add(self.n_shadows);
                                let shadow_id = self.glyphs[char_id].shadow_id as usize;
                                if !self.shadow_glyphs[shadow_id].flags.contains(PGFFlags::CACHED){
                                    if self.get_bmp(shadow_id, PGFFlags::SHADOWGLYPH){
                                        changed = true; // ChAnGEd
                                    }
                                }
                                last_n_glyphs = n_glyphs;
                            }

                        } else {
                            // BWFON file

                            n_glyphs += 1;
                            if !self.glyphs_bw[char_id].flags.contains(PGFFlags::CACHED){
                                if self.get_bmp(char_id, PGFFlags::CHAR_GLYPH){
                                    changed = true;
                                }
                            }
                            // Only add shadows if they exist!
                            n_sglyphs += exist_add(self.n_shadows);
                            if !self.shadow_glyphs[0].flags.contains(PGFFlags::CACHED){
                                if self.get_bmp(self.glyphs[0].shadow_id as usize, PGFFlags::SHADOWGLYPH){
                                    changed = true;
                                }
                            }
                        }
                    }

                }
                if changed{
                    self.options.insert(PGFFlags::DIRTY)
                }


                count += 1;
                // We want to break out of this loop if the conditions below are true
                // The opposite of the conditional statement that would be in a do-while loop is required
                // Therefore, simply take the inverse of what is written in the C version of Intrafont
                if !changed || count > length { // (AB)' -> (A' + B')
                    break;
                } // No do-while loops in rust, but we can do loop-if loops :)
            }
            // Now comes a lot of psp-specific code.
            // This is a pointer to GPU memory. It is very nice for storing data that has to be displayed on screens :).
            let mut v = unsafe { (sceGuGetMemory(if self.rotation.is_rotated { 6 } else { 2 } * (n_glyphs as i32 + n_sglyphs as i32) * size_of::<FontVertex>() as i32) as *mut FontVertex)};

            let mut s_index = 0;
            let mut c_index = n_sglyphs;
            let mut last_c_index = n_sglyphs; // index for shadow and character/overlay glyphs
            for i in 0..length{
                // calculate left, height and possibly fill for character placement
                if (i == 0) || (text[i+offset] == '\n' as u16) || ((column > 0.0) && (i >= eol as usize) && (text[i+offset] != 32)){
                    //newline

                    if column > 0.0{
                        if self.options.contains(PGFFlags::SCROLL_LEFT){
                            eol = length as i32;
                            scroll = 1;
                            left = (x as i32) as f32;

                            union Union { i: i32, f: f32, };
                            let mut ux = Union { i: 0};
                            let mut uleft = Union {i: 0};
                            ux.f = x;
                            uleft.f = left;
                            count = unsafe {ux.i - uleft.i} as usize;
                            text_width = self.measure_text_ucs2_ex(&text,i+offset, length as i32 - i as i32) as i32;
                            if text_width as f32 > column{
                                match self.options & PGFFlags::SCROLL_MASK{
                                    PGFFlags::SCROLL_LEFT => {
                                        unsafe {sceGuScissor((left - 2.0) as i32, 0, (left + column + 4.0) as i32 , 274)};
                                        if count < 60 {
                                            // show initial text for 1s
                                        } else if count < (text_width + 90) as usize{
                                            left -= count as f32 - 60.0;
                                        } else if count < (text_width + 120) as usize{
                                            color = FontColor::from_bits((color.bits() & 0x00FFFFFF) | ((((color.bits() >> 24) * (count as u32 - text_width as u32 - 90)) / 30) << 24)).unwrap();
                                            shadow_color = FontColor::from_bits((shadow_color.bits() & 0x00FFFFFF) | ((((shadow_color.bits() >> 24) * (count as u32 - text_width as u32 - 90)) / 30) << 24)).unwrap();
                                        } else {
                                            ux.f = left; // reset counter
                                        }
                                    },
                                    PGFFlags::SCROLL_SEESAW => {
                                        unsafe {sceGuScissor((left - column/2.0 - 2.0) as i32, 0, (left + column + 4.0) as i32, 272)};
                                        text_width -= column as i32;
                                        if count < 60{
                                            left -= column/2.0; // show initial text (left side) for 1s
                                        } else if count < (text_width + 60) as usize{
                                            left -= column/2.0 + (count as i32 - 60) as f32 // scroll left
                                        } else if count < (text_width + 120) as usize{
                                            left -= column/2.0 + text_width as f32; // show right side for 1s
                                        } else if count < (2 * text_width + 120) as usize{
                                            left -= column/2.0 + 2.0 * text_width as f32 - count as f32 + 120.0; //scroll right
                                        } else {
                                            ux.f = left; // reset counter
                                            left -= column / 2.0;
                                        }
                                    },
                                    PGFFlags::SCROLL_RIGHT => {
                                    },
                                    PGFFlags::SCROLL_THROUGH => {
                                    },
                                    _ => {}
                                }
                                // NEXT
                                unsafe {
                                    ux.i += 1;
                                    x = ux.f;
                                    sceGuEnable(GuState::ScissorTest);
                                }
                            }

                        } else { // automatic line-break required
                            n_spaces = -1;
                            eol = -1;
                            fill = 0.0;
                            for j in i..length{
                                if text[j+offset] == '\n' as u16{
                                    // newline reached -> no auto-line break
                                    eol = j as i32;
                                    break;
                                }
                                if text[j+offset] == ' ' as u16{
                                    // space found for padding or eol
                                    n_spaces += 1;
                                    eol = j as i32;
                                }
                                if self.measure_text_ucs2_ex(&text, i+offset, (j + 1 - i) as i32) > column{
                                    // line too long -> line break
                                    if eol < 0{
                                        eol = j as i32; // line break in the middle of the word
                                    }
                                    if n_spaces > 0{
                                        fill = (column - self.measure_text_ucs2_ex(&text,i+offset, eol - i as i32)) / n_spaces as f32;
                                        break;
                                    }
                                }
                            }
                            if i == length{
                                eol = length as i32; // last line
                                while (text[(eol - 1) as usize + offset] == ' ' as u16) && (eol > 1){
                                    eol -= 1;
                                }
                            }

                            left = x;
                            if (self.options & PGFFlags::ALIGN_MASK) == PGFFlags::ALIGN_RIGHT{
                                left -= self.measure_text_ucs2_ex(&text, i+offset, eol - 1);
                            }
                            if (self.options & PGFFlags::ALIGN_MASK) == PGFFlags::ALIGN_CENTER{
                                left -= self.measure_text_ucs2_ex(&text, i+offset, eol - i as i32) / 2.0;
                            }
                        }

                    } else {
                        // No column boundary -> display everything
                        left = x;
                        if text[i] == '\n' as u16{
                            if (self.options & PGFFlags::ALIGN_MASK) == PGFFlags::ALIGN_RIGHT{
                                left -= self.measure_text_ucs2_ex(&text, i+1+offset, (length - i - 1) as i32)
                            }
                            if (self.options & PGFFlags::ALIGN_MASK) == PGFFlags::ALIGN_CENTER{
                                left -= self.measure_text_ucs2_ex(&text, i+1+offset, (length - i - 1) as i32) / 2.0;
                            }
                        } else {
                            if (self.options & PGFFlags::ALIGN_MASK) == PGFFlags::ALIGN_RIGHT {
                                left -= self.measure_text_ucs2_ex(&text, i+offset, (length - i) as i32);
                            }
                            if (self.options & PGFFlags::ALIGN_MASK) == PGFFlags::ALIGN_CENTER{
                                left -= self.measure_text_ucs2_ex(&text, i+offset, (length - i ) as i32) / 2.0;
                            }
                        }
                    }

                    width = 0.0;
                    height += self.advance.1 as f32 * glyph_scale * 0.25;
                }

                char_id = self.get_char_id(text[i+offset].clone()) as usize;
                if char_id < self.n_chars as usize{
                    glyph_ptr = if self.filetype == FileType::PGF { char_id } else { 0 };
                    shadow_glyph_ptr = if self.filetype == FileType::PGF { self.glyphs[glyph_ptr].shadow_id} else { 0 };

                    // center glyphs for monospace
                    if self.options.contains(PGFFlags::WIDTH_FIX){
                        width += ((self.options & PGFFlags::WIDTH_MASK).bits() as f32 / 2.0 - self.glyphs[glyph_ptr].advance as f32 / 8.0) * glyph_scale;
                    }

                    // add vertices for sub glyphs
                    for mut j in 0..3{
                        if self.filetype == FileType::PGF{
                            if (self.glyphs[char_id].flags & PGFFlags::BMP_OVERLAY) == PGFFlags::BMP_OVERLAY{
                                subucs2 = (self.font_data[(self.glyphs[char_id].offset + j as u32 * 2) as usize] as u32 + self.font_data[(self.glyphs[char_id].offset + j as u32 * 2 + 1) as usize] as u32 * 256) as u16;
                                glyph_id = self.get_char_id(subucs2 as u16)
                            } else {
                                glyph_id = char_id as u16;
                                j = 2;
                            }
                        } else {
                            // FILETYPE BWFON
                            glyph_id = 0;
                            j = 2;
                        }

                        if glyph_id < self.n_chars{
                            if self.filetype == FileType::BWFON{
                                self.glyphs[glyph_id as usize].x = self.glyphs_bw[char_id].x;
                                self.glyphs[glyph_id as usize].y = self.glyphs_bw[char_id].y;
                            }

                            // screen coords
                            xl = left + width + self.glyphs[glyph_id as usize].left as f32 * glyph_scale;
                            xr = xl + self.glyphs[glyph_id as usize].width as f32 * glyph_scale;
                            yu = top + height - self.glyphs[glyph_id as usize].top as f32 * glyph_scale;
                            yd = yu + self.glyphs[glyph_id as usize].height as f32 * glyph_scale;
                            // Tex coords
                            ul = self.glyphs[glyph_id as usize].x as f32 - 0.25;
                            ur = self.glyphs[glyph_id as usize].x as f32 + self.glyphs[glyph_id as usize].width as f32 + 0.25;
                            vu = self.glyphs[glyph_id as usize].y as f32 - 0.25;
                            vd = self.glyphs[glyph_id as usize].y as f32 + self.glyphs[glyph_id as usize].height as f32 + 0.25;

                            if self.rotation.is_rotated{
                                unsafe { // Unsafety is our no. 1 priority
                                    v0 = Some(v.offset((c_index * 6) as isize));

                                    v1 = Some(v0.unwrap().offset(1));
                                    v2 = Some(v1.unwrap().offset(1));
                                    v3 = Some(v2.unwrap().offset(1));
                                    v4 = Some(v3.unwrap().offset(1));
                                    v5 = Some(v4.unwrap().offset(1));
                                }

                                // But lots of times, safety is pretty nice.
                                if let Some(v0) = v0{
                                    if let Some(v1) = v1{
                                        if let Some(v2) = v2{
                                            if let Some(v3) = v3{
                                                if let Some(v4) = v4{
                                                    if let Some(v5) = v5{
                                                        unsafe {
                                                            // Up-left
                                                            (*v0).u = ul; (*v0).v = vu;
                                                            (*v0).c = color.bits();
                                                            (*v0).x = xl; (*v0).y = yu;

                                                            // Up-right
                                                            (*v1).u = ur; (*v1).v = vu;
                                                            (*v1).c = color.bits();
                                                            (*v1).x = xr; (*v1).y = yu;

                                                            // Down-right
                                                            (*v2).u = ur; (*v2).v = vd;
                                                            (*v2).c = color.bits();
                                                            (*v2).x = xr; (*v2).y = yd;

                                                            // Down-left
                                                            (*v3).u = ul; (*v3).v = vd;
                                                            (*v3).c = color.bits();
                                                            (*v3).x = xl; (*v3).y = yd;

                                                            // Apply rotation to each vertex
                                                            // x' = x cos θ - y sin θ
                                                            // y' = x sin θ + y cos θ

                                                            let (mut vx, mut vy) = (0.0f32, 0.0f32);

                                                            vx = x + ((*v0).x - x) * self.rotation.cos - ((*v0).y - y) * self.rotation.sin;
                                                            vy = y + ((*v0).x - x) * self.rotation.sin + ((*v0).y - y) * self.rotation.cos;
                                                            (*v0).x = vx; (*v0).y = vy;
                                                            vx = x + ((*v1).x - x) * self.rotation.cos - ((*v1).y - y) * self.rotation.sin;
                                                            vy = y + ((*v1).x - x) * self.rotation.sin + ((*v1).y - y) * self.rotation.cos;
                                                            (*v1).x = vx; (*v1).y = vy;
                                                            vx = x + ((*v2).x - x) * self.rotation.cos - ((*v2).y - y) * self.rotation.sin;
                                                            vy = y + ((*v2).x - x) * self.rotation.sin + ((*v2).y - y) * self.rotation.cos;
                                                            (*v2).x = vx; (*v2).y = vy;
                                                            vx = x + ((*v3).x - x) * self.rotation.cos - ((*v3).y - y) * self.rotation.sin;
                                                            vy = y + ((*v3).x - x) * self.rotation.sin + ((*v3).y - y) * self.rotation.cos;
                                                            (*v3).x = vx; (*v3).y = vy;

                                                            *v4 = *v0;
                                                            *v5 = *v2;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                // Font is not rotated :)
                                unsafe {
                                    v0 = Some(v.offset((c_index as isize) << 1));
                                    v1 = Some(v0.unwrap().offset(1));

                                    if let Some(v0) = v0{
                                        if let Some(v1) = v1{

                                            // Up-Left
                                            (*v0).u = ul; (*v0).v = vu;
                                            (*v0).c = color.bits();
                                            (*v0).x = xl; (*v0).y = yu;

                                            // Down-Right
                                            (*v1).u = ur; (*v1).v = vd;
                                            (*v1).c = color.bits();
                                            (*v1).x = xr; (*v1).y = yd;
                                        }
                                    }
                                }
                            }

                            c_index += 1;
                        }
                    }

                    /// add verticies for shadow
                    if c_index > last_c_index{

                        // Screen coords
                        xl = left + width + self.shadow_glyphs[shadow_glyph_ptr as usize].left as f32 * glyph_scale * 64.0 / (self.shadow_scale as f32);
                        xr = xl + self.shadow_glyphs[shadow_glyph_ptr as usize].width as f32 * 64.0 / (self.shadow_scale as f32);
                        yu = top + height - self.shadow_glyphs[shadow_glyph_ptr as usize].top as f32 * glyph_scale * 64.0 / (self.shadow_scale as f32);
                        yd = yu + self.shadow_glyphs[shadow_glyph_ptr as usize].height as f32 * glyph_scale * 64.0 / (self.shadow_scale as f32);
                        // Tex coords
                        ul = self.shadow_glyphs[shadow_glyph_ptr as usize].x as f32 - 0.25;
                        ur = self.shadow_glyphs[shadow_glyph_ptr as usize].x as f32 + self.shadow_glyphs[shadow_glyph_ptr as usize].width as f32 + 0.25;
                        vu = self.shadow_glyphs[shadow_glyph_ptr as usize].y as f32 - 0.25;
                        vd = self.shadow_glyphs[shadow_glyph_ptr as usize].y as f32 + self.shadow_glyphs[shadow_glyph_ptr as usize].height as f32 + 0.25;

                        if self.rotation.is_rotated{
                            // time to get some pointer :)
                            unsafe {
                                s0 = Some(v.offset((s_index * 6) as isize));
                                s1 = Some(s0.unwrap().offset(1));
                                s2 = Some(s1.unwrap().offset(1));
                                s3 = Some(s2.unwrap().offset(1));
                                s4 = Some(s3.unwrap().offset(1));
                                s5 = Some(s4.unwrap().offset(1));
                            }

                            if let Some(s0) = s0{
                                if let Some(s1)  = s1{
                                    if let Some(s2) = s2{
                                        if let Some(s3) = s3{
                                            if let Some(s4) = s4{
                                                if let Some(s5) = s5{

                                                    unsafe {
                                                        // Up-left
                                                        (*s0).u = ul; (*s0).v = vu;
                                                        (*s0).c = shadow_color.bits();
                                                        (*s0).x = xl; (*s0).y = yu;

                                                        // Up-right
                                                        (*s1).u = ur; (*s1).v = vu;
                                                        (*s1).c = shadow_color.bits();
                                                        (*s1).x = xr; (*s1).y = yu;

                                                        // Down-right
                                                        (*s2).u = ur; (*s2).v = vd;
                                                        (*s2).c = shadow_color.bits();
                                                        (*s2).x = xr; (*s2).y = yd;

                                                        // Down-left
                                                        (*s3).u = ul; (*s3).v = vd;
                                                        (*s3).c = shadow_color.bits();
                                                        (*s3).x = xl; (*s3).y = yd;

                                                        // Rotate time.
                                                        let (mut sx, mut sy) = (0.0f32, 0.0f32);
                                                        sx = x + ((*s0).x - x) * self.rotation.cos - ((*s0).y - y) * self.rotation.sin;
                                                        sy = y + ((*s0).x - x) * self.rotation.sin + ((*s0).y - y) * self.rotation.cos;
                                                        (*s0).x = sx; (*s0).y = sy;
                                                        sx = x + ((*s1).x - x) * self.rotation.cos - ((*s1).y - y) * self.rotation.sin;
                                                        sy = y + ((*s1).x - x) * self.rotation.sin + ((*s1).y - y) * self.rotation.cos;
                                                        (*s1).x = sx; (*s1).y = sy;
                                                        sx = x + ((*s2).x - x) * self.rotation.cos - ((*s2).y - y) * self.rotation.sin;
                                                        sy = y + ((*s2).x - x) * self.rotation.sin + ((*s2).y - y) * self.rotation.cos;
                                                        (*s2).x = sx; (*s2).y = sy;
                                                        sx = x + ((*s3).x - x) * self.rotation.cos - ((*s3).y - y) * self.rotation.sin;
                                                        sy = y + ((*s3).x - x) * self.rotation.sin + ((*s3).y - y) * self.rotation.cos;
                                                        (*s3).x = sx; (*s3).y = sy;

                                                        *s4 = *s0;
                                                        *s5 = *s2;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                        } else {
                            // Not rotated :)
                            unsafe {
                                s0 = Some(v.offset((s_index << 1) as isize));
                                s1 = Some(s0.unwrap().offset(1));

                                if let Some(s0) = s0{
                                    if let Some(s1) = s1{
                                        // Up-left
                                        (*s0).u = ul; (*s0).v = vu;
                                        (*s0).c = shadow_color.bits();
                                        (*s0).x = xl; (*s0).y = yu;

                                        // Down-right
                                        (*s1).u = ur; (*s1).v = vd;
                                        (*s1).c = shadow_color.bits();
                                        (*s1).x = xr; (*s1).y = yd;
                                    }
                                }
                            }
                        }
                        s_index += 1;
                        last_c_index = c_index;
                    }

                    // advance
                    if self.options.contains(PGFFlags::WIDTH_FIX){
                        width += ((self.options & PGFFlags::WIDTH_MASK).bits() as f32 / 2.0 + self.glyphs[glyph_ptr].advance as f32 / 8.0) * glyph_scale;
                    } else {
                        width += self.glyphs[glyph_ptr].advance as f32 * glyph_scale * 0.25;
                    }

                    if (text[i+offset] == 32) && ((self.options & PGFFlags::ALIGN_FULL) == PGFFlags::ALIGN_FULL){
                        width += fill;
                    }
                } else {
                    // char_id requested is not available :o

                    if let Some(alt_font) = &mut *self.alt_font{
                        let alt_options = alt_font.options;
                        alt_font.options = alt_options & PGFFlags::from_bits(PGFFlags::WIDTH_MASK.bits() + PGFFlags::WIDTH_MASK.bits()).unwrap();
                        let text_clone = text.clone();
                        width += alt_font.print_column_ucs2_ex(left + width, top + height, 0.0, text_clone, i + offset, 1) - (left + width);
                        alt_font.options = alt_options;
                    }
                }
            }

            // finalize and activate texture (if not already active or ahs been changed)
            unsafe {
                sceKernelDcacheWritebackRange(v as *mut _, (n_glyphs + n_sglyphs) as u32 * size_of::<FontVertex>() as u32); // SAKYA, mrneo240 <-- from C version Intrafont
                if !self.options.contains(PGFFlags::ACTIVE){
                    self.activate(); // And then, there was light...
                }

                sceGuDisable(GuState::DepthTest);
                sceGuDrawArray(if self.rotation.is_rotated { GuPrimitive::Triangles } else { GuPrimitive::Sprites },
                               VertexType::TEXTURE_32BITF | VertexType::COLOR_8888 | VertexType::TRANSFORM_2D,
                               n_glyphs as i32 * if self.rotation.is_rotated { 6 } else { 2 },
                               core::ptr::null(),
                               v.offset((n_sglyphs as u32 * if self.rotation.is_rotated { 6 } else { 2 }) as isize) as *mut _ );
                sceGuEnable(GuState::DepthTest);
            }

            if scroll == 1{
                unsafe {sceGuScissor(0,0,480,272)};
                return x;
            }
            return left + width // done deal fam.
        }

        /// Encode a UTF8 string to a UCS2 string.
        pub fn encode<const M: usize>(input: &str, output: &mut SmartBuffer<u16, M>) {
            let bytes = input.as_bytes();
            let len = bytes.len();
            let mut i = 0;

            while i < len {
                let ch;

                if bytes[i] & 0b1000_0000 == 0b0000_0000 {
                    ch = u16::from(bytes[i]);
                    i += 1;
                } else if bytes[i] & 0b1110_0000 == 0b1100_0000 {
                    // 2 byte codepoint
                    let a = u16::from(bytes[i] & 0b0001_1111);
                    let b = u16::from(bytes[i + 1] & 0b0011_1111);
                    ch = a << 6 | b;
                    i += 2;
                } else if bytes[i] & 0b1111_0000 == 0b1110_0000 {
                    // 3 byte codepoint
                    let a = u16::from(bytes[i] & 0b0000_1111);
                    let b = u16::from(bytes[i + 1] & 0b0011_1111);
                    let c = u16::from(bytes[i + 2] & 0b0011_1111);
                    ch = a << 12 | b << 6 | c;
                    i += 3;
                } else {
                    // safe: impossible utf-8 string.
                    unsafe { core::hint::unreachable_unchecked() }
                }

                output.push(ch);
            }
        }

    }

    // TODO: cosf vs cosf32? which makes intrinsics::cosf32 work?
    #[allow(non_snake_case)]
    pub unsafe fn cosf32(rad: f32) -> f32 {
        let out;

        vfpu_asm!(
        .mips "mfc1 $$t0, $1";

        mfv t1, S000;
        mfv t2, S001;

        mtv t0, S000;
        vcst_s S001, VFPU_2_PI;
        vmul_s S000, S000, S001;
        vcos_s S000, S000;
        mfv t0, S000;

        mtv t1, S000;
        mtv t2, S001;

        .mips "mtc1 $$t0, $0";

        : "=f"(out) : "f"(rad) : "$8", "$9", "$10", "memory" : "volatile"
    );

        out
    }
}
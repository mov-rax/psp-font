bitflags!{
    pub(crate) struct Codepages: u32 {
        const ASCII     = 0x00;
        const US        = 0x01;
        const MLATIN    = 0x05;
        const RUSSIAN   = 0x0B;
        const S_JIS     = 0x0D;
        const GBK       = 0x0E;
        const KOREAN    = 0x0F;
        const BIG5      = 0x10;
        const CYRILLIC  = 0x12;
        const LATIN2    = 0x13;
        const UTF8      = 0xFF;
        const N_CP      = 0x14; // Number of codepages
    }
}


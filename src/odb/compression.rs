#[repr(u8)]
#[derive(Copy, Clone)]
pub enum ObjectCompression {
    None = 0,
    LZMA = 1,
}

pub fn get(byte: u8, pos: usize) -> bool {
    (byte & (1 << pos)) != 0
}

pub fn set(byte: u8, pos: usize) -> u8 {
    byte | (1 << pos)
}

pub fn clear(byte: u8, pos: usize) -> u8 {
    byte & !(1 << pos)
}
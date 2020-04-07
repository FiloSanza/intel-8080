// This struct represents the intel 8080 memory
// the processor was able to access to 64KB of memory
pub struct Memory{
    data: Vec<u8>,
}

impl Memory {
    pub fn new() -> Self {
        Self{
            data: vec![0x00; 0x10000]
        }
    }

    pub fn get(&self, idx: usize) -> u8 {
        self.data[idx]
    }

    pub fn set(&mut self, idx: usize, value: u8) {
        self.data[idx] = value;
    }

    pub fn get_word(&self, idx: usize) -> u16 {
        (self.get(idx) as u16) | ((self.get(idx + 1) as u16) << 8)
    }

    pub fn set_word(&mut self, idx: usize, value: u16) {
        self.set(idx, (value & 0x00ff) as u8);
        self.set(idx + 1, (value >> 8) as u8);
    }
}

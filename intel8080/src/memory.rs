// This struct represents the intel 8080 memory
// the processor was able to access to 64KB of memory

pub trait Memory{
    fn get(&self, idx: usize) -> u8;
    fn set(&mut self, idx: usize, value: u8);

    fn get_word(&self, idx: usize) -> u16 {
        u16::from(self.get(idx)) | (u16::from(self.get(idx + 1)) << 8)
    }

    fn set_word(&mut self, idx: usize, value: u16) {
        self.set(idx, (value & 0xff) as u8);
        self.set(idx + 1, (value >> 8) as u8);
    }
}

#[derive(Default)]
pub struct Linear {
    pub data: Vec<u8>,
}

impl Memory for Linear {
    fn get(&self, idx: usize) -> u8 {
        self.data[idx]
    }

    fn set(&mut self, idx: usize, value: u8) {
        self.data[idx] = value;
    }
}

impl Linear{
    pub fn new() -> Self {
        Self{
            data: vec![0x00; 0x10000]
        }
    }    
}

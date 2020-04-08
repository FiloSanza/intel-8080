use super::bit;

// This struct represents the i8080's registers
// A, B, C, D, E, H and L are 8 bits registers
// F contains the flags
// SP is the stack pointer
// PC is the program counter
// See: https://en.wikipedia.org/wiki/Intel_8080#Registers
pub struct Register {
    pub a: u8,
    pub f: u8,      //Flags
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,  
    pub l: u8,
    pub sp: u16,    //Stack Pointer    
    pub pc: u16,    //Program Counter
}

// B, C, D, E, H, L can be used as 8 bit registers or
// they can be paired  16 bit registers (BC, DE, HL)
// the following implementation allows to use BD, DE and HL registers
impl Register {
    pub fn get_bc(&self) -> u16 {
        ((self.b as u16) << 8) | (self.c as u16)
    }

    pub fn get_de(&self) -> u16 {
        ((self.d as u16) << 8) | (self.e as u16)
    }

    pub fn get_hl(&self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }

    pub fn get_af(&self) -> u16 {
        ((self.a as u16) << 8) | (self.f as u16)
    }

    pub fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = (value & 0x00ff) as u8;
    }

    pub fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = (value & 0x00ff) as u8;
    }

    pub fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = (value & 0x00ff) as u8;
    }

    pub fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.f = (value & 0x00d5 | 0x0002) as u8;
    }
}

// This enum represents the flags that the F register
// contains
// Sign: 1 if result is negative
// Zero: 1 if result is 0
// Parity: 1 if the number of 1 bit  the result is even
// Carry: 1 if the last addition/subtraction had a carry/borrow
// AC aka Auxiliary Carry: used for binary-coded decimal arithmetic
// See: https://en.wikipedia.org/wiki/Intel_8080#Flags
pub enum Flags {
    Sign = 7,
    Zero = 6,
    AC = 4,
    Parity = 2,
    Carry = 0
}

// This impl allows to use and set the flags inside the F register
impl Register {
    pub fn get_flag(&self, flag: Flags) -> bool {
        bit::get(self.f, flag as usize)
    }

    pub fn set_flag(&mut self, flag: Flags, value: bool) {
        if value {
            self.f = bit::set(self.f, flag as usize)
        }
        else{
            self.f = bit::clear(self.f, flag as usize)
        }
    }
}


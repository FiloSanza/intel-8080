use std::rc::Rc;
use std::cell::RefCell;

use super::bit;
use super::register::Register;
use super::register::Flags;
use super::memory::Memory;

 struct Cpu {
    register: Register,
    memory: Rc<RefCell<Memory>>,
}

// This impl block implements Arithmetic Group operations
impl Cpu {
    // Add to the accumulator: A = A + value
    // Instructions:
    //  ADD register
    //  ADD memory
    //  ADI data
    fn alu_add(&mut self, value: u8) {
        let a = self.register.a;
        let result = self.register.a.wrapping_add(value);
        self.register.set_flag(Flags::Zero, result == 0x00);
        self.register.set_flag(Flags::Sign, bit::get(result, 7));
        self.register.set_flag(Flags::AC, (a & 0x0f) + (value & 0x0f) > 0x0f);
        self.register.set_flag(Flags::Parity, result.count_ones() & 0x01 == 0);
        self.register.set_flag(Flags::Carry, (a as u16) + (value as u16) > 0xff);
        self.register.a = result;
    }

    // Add to the accumulator with carry: A = A + value + Carry
    // Instructions:
    //  ADC register
    //  ADC memory
    //  ADC data
    fn alu_adc(&mut self, value: u8) {
        let a = self.register.a;
        let c = self.register.get_flag(Flags::Carry) as u8;
        let result = self.register.a.wrapping_add(value).wrapping_add(c);
        self.register.set_flag(Flags::Zero, result == 0x00);
        self.register.set_flag(Flags::Sign, bit::get(result, 7));
        self.register.set_flag(Flags::AC, (a & 0x0f) + (value & 0x0f) + c > 0x0f);
        self.register.set_flag(Flags::Parity, result.count_ones() & 0x01 == 0);
        self.register.set_flag(Flags::Carry, (a as u16) + (value as u16) + (c as u16) > 0xff);
        self.register.a = result;
    }

    //Subtract from accumulator: A = A - value
    // Instructions:
    //  SUB register
    //  SUB memory
    //  SUI data
    fn alu_sub(&mut self, value: u8) {
        let a = self.register.a;
        let result = a.wrapping_sub(value);
        self.register.set_flag(Flags::Zero, result == 0x00);
        self.register.set_flag(Flags::Sign, bit::get(result, 7));
        self.register.set_flag(Flags::AC, (a as i8 & 0x0f) - (value as i8 & 0x0f) >= 0);
        self.register.set_flag(Flags::Parity, result.count_ones() & 0x01 == 0);
        self.register.set_flag(Flags::Carry, (a as u16) < (value as u16));
        self.register.a = result;
    }

    //Subtract from accumulator with borrow: A = A - value - Carry
    //Instructions:
    //  SBB register
    //  SBB memory
    //  SBI value
    fn alu_sbb(&mut self, value: u8) {
        let a = self.register.a;
        let c = self.register.get_flag(Flags::Carry) as u8;
        let result = a.wrapping_sub(value).wrapping_sub(c);
        self.register.set_flag(Flags::Zero, result == 0x00);
        self.register.set_flag(Flags::Sign, bit::get(result, 7));
        self.register.set_flag(Flags::AC, (a as i8 & 0x0f) - (value as i8 & 0x0f) - (c as i8) >= 0);
        self.register.set_flag(Flags::Parity, result.count_ones() & 0x01 == 0);
        self.register.set_flag(Flags::Carry, (a as u16) < ((value as u16) + (c as u16)));
        self.register.a = result;
    }

    //Increment register or memory: X = X + 1
    //Instructions:
    //  INR register
    //  INR memory
    //CARRY FLAG IS NOT AFFECTED
    fn alu_inr(&mut self, value: u8) -> u8 {
        let result = value.wrapping_add(1);
        self.register.set_flag(Flags::Zero, result == 0x00);
        self.register.set_flag(Flags::Sign, bit::get(result, 7));
        self.register.set_flag(Flags::AC, (value & 0x0f) + 0x01 > 0x0f);
        self.register.set_flag(Flags::Parity, result.count_ones() & 0x01 == 0);
        result
    }

    //Decrement register or memory: X = X - 1
    //Instructions:
    //  DCR register
    //  DCR memory
    //CARRY FLAG IS NOT AFFECTED
    fn alu_dcr(&mut self, value: u8) -> u8 {
        let result = value.wrapping_sub(1);
        self.register.set_flag(Flags::Zero, result == 0x00);
        self.register.set_flag(Flags::Sign, bit::get(result, 7));
        self.register.set_flag(Flags::AC, (result & 0x0f) != 0x0f);
        self.register.set_flag(Flags::Parity, result.count_ones() & 0x01 == 0);
        result
    }

    //ADD register pair to HL: HL = HL + (R1 + R2)
    //Instructions:
    // DAD registers
    //ONLY CARRY FLAG IS AFFECTED
    fn alu_dad(&mut self, value: u16) {
        let hl = self.register.get_hl();
        let result = hl.wrapping_add(value);
        self.register.set_flag(Flags::Carry, hl > 0xffff - value);
        self.register.set_hl(result);
    }

    //Decimal Adjust Accumulator
    //Instructions:
    // DAA
    //The eight-bit number in the accumulator is adjusted to form two
    //four-bit BCD digits
    fn alu_daa(&mut self) {
        let mut to_add: u8 = 0;
        let mut carry = self.register.get_flag(Flags::Carry);
        let low = self.register.a & 0x0f;
        let high = self.register.a >> 4;
        
        //if 4 LSB have a value > 9 or the AC flag is set add 6 to the 4 LSB
        if (low > 9) || self.register.get_flag(Flags::AC) {
            to_add += 0x06; 
        }
        
        //if 4 MSB have a value > 9 after the previous operation or the C flag is set add 6 to the 4 MSB
        if (high > 9) || self.register.get_flag(Flags::Carry) || ((high >= 9) && (low > 9)) {
            to_add += 0x60;
            carry = true;       //there will be an overflow
        }

        self.alu_add(to_add);
        self.register.set_flag(Flags::Carry, carry);
    }
}

// This impl block implements Logical Group operations
impl Cpu {
    //AND the accumulator: A = A & value
    //Instructions:
    //  ANA register
    //  ANA memory
    //  ANI data
    //CARRY IS CLEARED
    fn alu_ana(&mut self, value: u8) {
        let a = self.register.a;
        let result = a & value;
        self.register.set_flag(Flags::Zero, result == 0x00);
        self.register.set_flag(Flags::Sign, bit::get(result, 7));
        self.register.set_flag(Flags::AC, ((a | value) & 0x08) != 0x0);
        self.register.set_flag(Flags::Parity, result.count_ones() & 0x01 == 0);
        self.register.set_flag(Flags::Carry, false);
        self.register.a = result;
    }

    //XOR the accumulator: A = A ^ value
    //Instructions:
    //  XRA register
    //  XRA memory
    //  XRI data
    //AC AND CARRY ARE CLEARED
    fn alu_xra(&mut self, value: u8) {
        let a = self.register.a;
        let result = a ^ value;
        self.register.set_flag(Flags::Zero, result == 0x00);
        self.register.set_flag(Flags::Sign, bit::get(result, 7));
        self.register.set_flag(Flags::AC, false);
        self.register.set_flag(Flags::Parity, result.count_ones() & 0x01 == 0);
        self.register.set_flag(Flags::Carry, false);
        self.register.a = result;
    }

    //OR the accumulator: A = A | value
    //Instructions:
    //  ORA register
    //  ORA memory
    //  ORI data
    //AC AND CARRY ARE CLEARED
    fn alu_ora(&mut self, value: u8) {
        let a = self.register.a;
        let result = a | value;
        self.register.set_flag(Flags::Zero, result == 0x00);
        self.register.set_flag(Flags::Sign, bit::get(result, 7));
        self.register.set_flag(Flags::AC, false);
        self.register.set_flag(Flags::Parity, result.count_ones() & 0x01 == 0);
        self.register.set_flag(Flags::Carry, false);
        self.register.a = result;
    }

    //Compare register: A - value is calculated, the flags are set
    //Instructions:
    //  CMP register
    //  CMP memory
    //  CPI data
    fn alu_cmp(&mut self, value: u8) {
        let a = self.register.a;
        self.alu_sub(value);
        self.register.a = a;        
    }

    //Rotate accumulator left: A(N+1) = A(N); A(0) = A(7)
    //Instructions:
    //  RLC
    //ONLY CARRY IS AFFECTED
    fn alu_rlc(&mut self) {
        let msb = bit::get(self.register.a, 7);
        self.register.set_flag(Flags::Carry, msb);
        self.register.a = (self.register.a << 1) | (msb as u8); 
    }

    //Rotate accumulator right: A(N-1) = A(N); A(7) = A(0)
    //Instructions:
    //  RRC
    //ONLY CARRY IS AFFECTED
    fn alu_rrc(&mut self) {
        let lsb = bit::get(self.register.a, 0);
        self.register.set_flag(Flags::Carry, lsb);
        self.register.a = if lsb {
            (self.register.a >> 1) | 0x80
        }
        else{
            self.register.a >> 1
        };
    }

    //Rotate accumulator left through carry: A(N+1) = A(N); A(0) = Carry
    //Instructions:
    //  RAL
    //ONLY CARRY IS AFFECTED
    fn alu_ral(&mut self) {
        let msb = bit::get(self.register.a, 7);
        self.register.a = (self.register.a << 1) | (self.register.get_flag(Flags::Carry) as u8);
        self.register.set_flag(Flags::Carry, msb);
    }

    //Rotate accumulator right through carry: A(N-1) = A(N); A(7) = Carry
    //Instructions:
    //  RAR
    //ONLY CARRY IS AFFECTED
    fn alu_rar(&mut self) {
        let lsb = bit::get(self.register.a, 0);
        self.register.a = if self.register.get_flag(Flags::Carry) {
            (self.register.a >> 1) & 0x80
        }
        else{
            self.register.a >> 1
        };
        self.register.set_flag(Flags::Carry, lsb);
    }

    //Complement accumulator: A = !A
    //Instructions:
    // CMA
    //NO FLAGS ARE AFFECTED
    fn alu_cma(&mut self) {
        self.register.a = !self.register.a;
    }

    //Complement carry: Carry = !Carry
    //Instructions:
    // CMC
    //ONLY CARRY IS AFFECTED
    fn alu_cmc(&mut self) {
        self.register.set_flag(Flags::Carry, !self.register.get_flag(Flags::Carry));
    }

    //Set carry: Carry = 1
    //Instructions:
    // STC
    //ONLY CARRY IS AFFECTED
    fn alu_stc(&mut self) {
        self.register.set_flag(Flags::Carry, true);
    }
}

// This impl block implements how the Cpu will be used and will handle the opcodes
impl Cpu {
    fn set_m(&self, value: u8) {
        let index = self.register.get_hl();
        self.memory.borrow_mut().set(index as usize, value);
    }

    fn get_m(&self) -> u8 {
        let index = self.register.get_hl();
        self.memory.borrow().get(index as usize)
    }

    fn get_next_byte(&mut self) -> u8 {
        let value = self.memory.borrow().get(self.register.pc as usize);
        self.register.pc += 1;
        value
    }

    fn get_next_word(&mut self) -> u16 {
        let value = self.memory.borrow().get_word(self.register.pc as usize);
        self.register.pc += 2;
        value
    }

    pub fn next(&mut self) {
        let opcode = 1;

        match opcode {
            0x00 => { },                                                                //NOP
            0x01 => (format!("LXI   B,#${:x}{:x}", input.2, input.1), 3),
            0x02 => (format!("STAX  B"), 1),
            0x03 => self.register.set_bc(self.register.get_bc().wrapping_add(1)),       //INX   B   INCREMENT REGISTER PAIR BC
            0x04 => self.register.b = self.alu_inr(self.register.b),                    //INR   B   INCREMENT REGISTER B
            0x05 => self.register.b = self.alu_dcr(self.register.b),                    //DCR   B   DECREMENT REGISTER B
            0x06 => (format!("MVI   B,#${:x}", input.1), 2),
            0x07 => self.alu_rlc(),                                                     //RLC       ROTATE ACCUMULATOR LEFT
            0x09 => self.alu_dad(self.register.get_bc()),                               //DAD   B   ADD REGISTER PAIR BC TO HL
            0x0a => (format!("LDAX  B"), 1),
            0x0b => self.register.set_bc(self.register.get_bc().wrapping_sub(1)),       //DCX   B   DECREMENT REGISTER PAIR BC
            0x0c => self.register.c = self.alu_inr(self.register.c),                    //INR   C   INCREMENT REGISTER C
            0x0d => self.register.c = self.alu_dcr(self.register.c),                    //DCR   C   DECREMENT REGISTER C
            0x0e => (format!("MVI   C,#${:x}", input.1), 2),
            0x0f => self.alu_rrc(),                                                     //RRC       ROTATE ACCUMULATOR RIGHT 
            0x11 => (format!("LXI   D,#${:x}{:x}", input.2, input.1), 3),
            0x12 => (format!("STAX  D"), 1),
            0x13 => self.register.set_de(self.register.get_de().wrapping_add(1)),       //INX   D   INCREMENT REGISTER PAIR DE
            0x14 => self.register.d = self.alu_inr(self.register.d),                    //INR   D   INCREMENT REGISTER D
            0x15 => self.register.d = self.alu_dcr(self.register.d),                    //DCR   D   DECREMENT REGISTER D
            0x16 => (format!("MVI   D,#${:x}", input.1), 2),
            0x17 => self.alu_ral(),                                                     //RAL       ROTATE ACCUMULATOR LEFT THROUGH CARRY
            0x19 => self.alu_dad(self.register.get_de()),                               //DAD   D   ADD REGISTER PAIR DE TO HL
            0x1a => (format!("LDAX  D"), 1),
            0x1b => self.register.set_de(self.register.get_de().wrapping_sub(1)),       //DCX   D   DECREMENT REGISTER PAIR DE
            0x1c => self.register.e = self.alu_inr(self.register.e),                    //INR   E   INCREMENT REGISTER E
            0x1d => self.register.e = self.alu_dcr(self.register.e),                    //DCR   E   DECREMENT REGISTER E
            0x1e => (format!("MVI   E,#${:x}", input.1), 2),
            0x1f => self.alu_rar(),                                                     //RAR       ROTATE ACCUMULATOR RIGHT THROUGH CARRY
            0x20 => (format!("RIM"), 1),
            0x21 => (format!("LXI   H,#${:x}{:x}", input.2, input.1), 3),
            0x22 => (format!("SHLD  ${:x}{:x}", input.2, input.1), 3),
            0x23 => self.register.set_hl(self.register.get_hl().wrapping_add(1)),       //INX   H   INCREMENT REGISTER PAIR HL
            0x24 => self.register.h = self.alu_inr(self.register.h),                    //INR   H   INCREMENT REGISTER H
            0x25 => self.register.h = self.alu_dcr(self.register.h),                    //DCR   H   DECREMENT REGISTER H
            0x26 => (format!("MVI   H,#${:x}", input.1), 2),
            0x27 => self.alu_daa(),                                                     //DAA       DECIMAL ADJUST ACCUMULATION
            0x29 => self.alu_dad(self.register.get_hl()),                               //DAD   H   ADD REGISTER PAIR HL TO HL
            0x2a => (format!("LHLD  ${:x}{:x}", input.2, input.1), 3),
            0x2b => self.register.set_hl(self.register.get_hl().wrapping_sub(1)),       //DCX   H   DECREMENT REGISTER PAIR HL
            0x2c => self.register.l = self.alu_inr(self.register.l),                    //INR   L   INCREMENT REGISTER L
            0x2d => self.register.l = self.alu_dcr(self.register.l),                    //DCR   L   DECREMENT REGISTER L
            0x2e => (format!("MVI   L,#${:x}", input.1), 2),
            0x2f => self.alu_cma(),                                                     //CMA       COMPLEMENT ACCUMULATOR
            0x30 => (format!("SIM"), 1),
            0x31 => (format!("LXI   SP,#${:x}{:x}", input.2, input.1), 3),
            0x32 => (format!("STA   ${:x}{:x}", input.2, input.1), 3),
            0x33 => self.register.sp = self.register.sp.wrapping_add(1),                //INX   SP  INCREMENT REGISTER PAIR SP
            0x34 => self.set_m(self.alu_inr(self.get_m())),                             //INR   M   INCREMENT memory 
            0x35 => self.set_m(self.alu_dcr(self.get_m())),                             //DCR   M   DECREMENT memory
            0x36 => (format!("MVI   M,#${:x}", input.1), 2),
            0x37 => self.alu_stc(),                                                     //STC       SET CARRY 
            0x39 => self.alu_dad(self.register.sp),                                     //DAD   SP  ADD STACK POINTER TO HL
            0x3a => (format!("LDA   ${:x}{:x}", input.2, input.1), 3),
            0x3b => self.register.sp = self.register.sp.wrapping_sub(1),                //DCX   SP  DECREMENT REGISTER PAIR SP
            0x3c => self.register.a = self.alu_inr(self.register.a),                    //INR   A   INCREMENT REGISTER A
            0x3d => self.register.a = self.alu_dcr(self.register.a),                    //DCR   A   DECREMENT REGISTER A
            0x3e => (format!("MVI   A,#${:x}", input.1), 2),
            0x3f => self.alu_cmc(),                                                     //CMC       COMPLEMENT CARRY
            0x40 => (format!("MOV   B,B"), 1),
            0x41 => (format!("MOV   B,C"), 1),
            0x42 => (format!("MOV   B,D"), 1),
            0x43 => (format!("MOV   B,E"), 1),
            0x44 => (format!("MOV   B,H"), 1),
            0x45 => (format!("MOV   B,L"), 1),
            0x46 => (format!("MOV   B,M"), 1),
            0x47 => (format!("MOV   B,A"), 1),
            0x48 => (format!("MOV   C,B"), 1),
            0x49 => (format!("MOV   C,C"), 1),
            0x4a => (format!("MOV   C,D"), 1),
            0x4b => (format!("MOV   C,E"), 1),
            0x4c => (format!("MOV   C,H"), 1),
            0x4d => (format!("MOV   C,L"), 1),
            0x4e => (format!("MOV   C,M"), 1),
            0x4f => (format!("MOV   C,A"), 1),
            0x50 => (format!("MOV   D,B"), 1),
            0x51 => (format!("MOV   D,C"), 1),
            0x52 => (format!("MOV   D,D"), 1),
            0x53 => (format!("MOV   D,E"), 1),
            0x54 => (format!("MOV   D,H"), 1),
            0x55 => (format!("MOV   D,L"), 1),
            0x56 => (format!("MOV   D,M"), 1),
            0x57 => (format!("MOV   D,A"), 1),
            0x58 => (format!("MOV   E,B"), 1),
            0x59 => (format!("MOV   E,C"), 1),
            0x5a => (format!("MOV   E,D"), 1),
            0x5b => (format!("MOV   E,E"), 1),
            0x5c => (format!("MOV   E,H"), 1),
            0x5d => (format!("MOV   E,L"), 1),
            0x5e => (format!("MOV   E,M"), 1),
            0x5f => (format!("MOV   E,A"), 1),
            0x60 => (format!("MOV   H,B"), 1),
            0x61 => (format!("MOV   H,C"), 1),
            0x62 => (format!("MOV   H,D"), 1),
            0x63 => (format!("MOV   H,E"), 1),
            0x64 => (format!("MOV   H,H"), 1),
            0x65 => (format!("MOV   H,L"), 1),
            0x66 => (format!("MOV   H,M"), 1),
            0x67 => (format!("MOV   H,A"), 1),
            0x68 => (format!("MOV   L,B"), 1),
            0x69 => (format!("MOV   L,C"), 1),
            0x6a => (format!("MOV   L,D"), 1),
            0x6b => (format!("MOV   L,E"), 1),
            0x6c => (format!("MOV   L,H"), 1),
            0x6d => (format!("MOV   L,L"), 1),
            0x6e => (format!("MOV   L,M"), 1),
            0x6f => (format!("MOV   L,A"), 1),
            0x70 => (format!("MOV   M,B"), 1),
            0x71 => (format!("MOV   M,C"), 1),
            0x72 => (format!("MOV   M,D"), 1),
            0x73 => (format!("MOV   M,E"), 1),
            0x74 => (format!("MOV   M,H"), 1),
            0x75 => (format!("MOV   M,L"), 1),
            0x76 => (format!("HLT"), 1),
            0x77 => (format!("MOV   M,A"), 1),
            0x78 => (format!("MOV   A,B"), 1),
            0x79 => (format!("MOV   A,C"), 1),
            0x7a => (format!("MOV   A,D"), 1),
            0x7b => (format!("MOV   A,E"), 1),
            0x7c => (format!("MOV   A,H"), 1),
            0x7d => (format!("MOV   A,L"), 1),
            0x7e => (format!("MOV   A,M"), 1),
            0x7f => (format!("MOV   A,A"), 1),
            0x80 => self.alu_add(self.register.b),                                      //ADD   B   ADD B TO ACCUMULATOR
            0x81 => self.alu_add(self.register.c),                                      //ADD   C   ADD C TO ACCUMULATOR
            0x82 => self.alu_add(self.register.d),                                      //ADD   D   ADD D TO ACCUMULATOR
            0x83 => self.alu_add(self.register.e),                                      //ADD   E   ADD E TO ACCUMULATOR
            0x84 => self.alu_add(self.register.h),                                      //ADD   H   ADD H TO ACCUMULATOR
            0x85 => self.alu_add(self.register.l),                                      //ADD   L   ADD L TO ACCUMULATOR
            0x86 => self.alu_add(self.get_m()),                                         //ADD   M   ADD memory TO ACCUMULATOR
            0x87 => self.alu_add(self.register.a),                                      //ADD   A   ADD A TO ACCUMULATOR
            0x88 => self.alu_adc(self.register.b),                                      //ADC   B   ADD B TO ACCUMULATOR WITH CARRY
            0x89 => self.alu_adc(self.register.c),                                      //ADC   C   ADD C TO ACCUMULATOR WITH CARRY
            0x8a => self.alu_adc(self.register.d),                                      //ADC   D   ADD D TO ACCUMULATOR WITH CARRY
            0x8b => self.alu_adc(self.register.e),                                      //ADC   E   ADD E TO ACCUMULATOR WITH CARRY
            0x8c => self.alu_adc(self.register.h),                                      //ADC   H   ADD H TO ACCUMULATOR WITH CARRY
            0x8d => self.alu_adc(self.register.l),                                      //ADC   L   ADD L TO ACCUMULATOR WITH CARRY
            0x8e => self.alu_adc(self.get_m()),                                         //ADC   M   ADD memory TO ACCUMULATOR WITH CARRY
            0x8f => self.alu_adc(self.register.a),                                      //ADC   A   ADD A TO ACCUMULATOR WITH CARRY
            0x90 => self.alu_sub(self.register.b),                                      //SUB   B   SUB B TO ACCUMULATOR
            0x91 => self.alu_sub(self.register.c),                                      //SUB   C   SUB C TO ACCUMULATOR
            0x92 => self.alu_sub(self.register.d),                                      //SUB   D   SUB D TO ACCUMULATOR
            0x93 => self.alu_sub(self.register.e),                                      //SUB   E   SUB E TO ACCUMULATOR
            0x94 => self.alu_sub(self.register.h),                                      //SUB   H   SUB H TO ACCUMULATOR
            0x95 => self.alu_sub(self.register.l),                                      //SUB   L   SUB L TO ACCUMULATOR
            0x96 => self.alu_sub(self.get_m()),                                         //SUB   M   SUB memory TO ACCUMULATOR
            0x97 => self.alu_sub(self.register.a),                                      //SUB   A   SUB A TO ACCUMULATOR
            0x98 => self.alu_sbb(self.register.b),                                      //SBB   B   SUB B TO ACCUMULATOR WITH BORROW
            0x99 => self.alu_sbb(self.register.c),                                      //SBB   C   SUB C TO ACCUMULATOR WITH BORROW
            0x9a => self.alu_sbb(self.register.d),                                      //SBB   D   SUB D TO ACCUMULATOR WITH BORROW
            0x9b => self.alu_sbb(self.register.e),                                      //SBB   E   SUB E TO ACCUMULATOR WITH BORROW
            0x9c => self.alu_sbb(self.register.h),                                      //SBB   H   SUB H TO ACCUMULATOR WITH BORROW
            0x9d => self.alu_sbb(self.register.l),                                      //SBB   L   SUB L TO ACCUMULATOR WITH BORROW
            0x9e => self.alu_sbb(self.get_m()),                                         //SBB   M   SUB memory TO ACCUMULATOR WITH BORROW
            0x9f => self.alu_sbb(self.register.a),                                      //SBB   A   SUB A TO ACCUMULATOR WITH BORROW
            0xa0 => self.alu_ana(self.register.b),                                      //ANA   B   AND B TO ACCUMULATOR
            0xa1 => self.alu_ana(self.register.c),                                      //ANA   C   AND C TO ACCUMULATOR
            0xa2 => self.alu_ana(self.register.d),                                      //ANA   D   AND D TO ACCUMULATOR
            0xa3 => self.alu_ana(self.register.e),                                      //ANA   E   AND E TO ACCUMULATOR
            0xa4 => self.alu_ana(self.register.h),                                      //ANA   H   AND H TO ACCUMULATOR
            0xa5 => self.alu_ana(self.register.l),                                      //ANA   L   AND L TO ACCUMULATOR
            0xa6 => self.alu_ana(self.get_m()),                                         //ANA   M   AND memory TO ACCUMULATOR
            0xa7 => self.alu_ana(self.register.a),                                      //ANA   A   AND A TO ACCUMULATOR
            0xa8 => self.alu_xra(self.register.b),                                      //XRA   B   XOR B TO ACCUMULATOR
            0xa9 => self.alu_xra(self.register.c),                                      //XRA   C   XOR C TO ACCUMULATOR
            0xaa => self.alu_xra(self.register.d),                                      //XRA   D   XOR D TO ACCUMULATOR
            0xab => self.alu_xra(self.register.e),                                      //XRA   E   XOR E TO ACCUMULATOR
            0xac => self.alu_xra(self.register.h),                                      //XRA   H   XOR H TO ACCUMULATOR
            0xad => self.alu_xra(self.register.l),                                      //XRA   L   XOR L TO ACCUMULATOR
            0xae => self.alu_xra(self.get_m()),                                         //XRA   M   XOR memory TO ACCUMULATOR
            0xaf => self.alu_xra(self.register.a),                                      //XRA   A   XOR A TO ACCUMULATOR
            0xb0 => self.alu_ora(self.register.b),                                      //ORA   B   OR B TO ACCUMULATOR
            0xb1 => self.alu_ora(self.register.c),                                      //ORA   C   OR C TO ACCUMULATOR
            0xb2 => self.alu_ora(self.register.d),                                      //ORA   D   OR D TO ACCUMULATOR
            0xb3 => self.alu_ora(self.register.e),                                      //ORA   E   OR E TO ACCUMULATOR
            0xb4 => self.alu_ora(self.register.h),                                      //ORA   H   OR H TO ACCUMULATOR
            0xb5 => self.alu_ora(self.register.l),                                      //ORA   L   OR L TO ACCUMULATOR
            0xb6 => self.alu_ora(self.get_m()),                                         //ORA   M   OR memory TO ACCUMULATOR
            0xb7 => self.alu_ora(self.register.a),                                      //ORA   A   OR A TO ACCUMULATOR
            0xb8 => self.alu_cmp(self.register.b),                                      //CMP   B   COMPARE B TO ACCUMULATOR
            0xb9 => self.alu_cmp(self.register.c),                                      //CMP   C   COMPARE C TO ACCUMULATOR
            0xba => self.alu_cmp(self.register.d),                                      //CMP   D   COMPARE D TO ACCUMULATOR
            0xbb => self.alu_cmp(self.register.e),                                      //CMP   E   COMPARE E TO ACCUMULATOR
            0xbc => self.alu_cmp(self.register.h),                                      //CMP   H   COMPARE H TO ACCUMULATOR
            0xbd => self.alu_cmp(self.register.l),                                      //CMP   L   COMPARE L TO ACCUMULATOR
            0xbe => self.alu_cmp(self.get_m()),                                         //CMP   M   COMPARE memory TO ACCUMULATOR
            0xbf => self.alu_cmp(self.register.b),                                      //CMP   A   COMPARE A TO ACCUMULATOR
            0xc0 => (format!("RNZ"), 1),
            0xc1 => (format!("POP   B"), 1),
            0xc2 => (format!("JNZ   ${:x}{:x}", input.2, input.1), 3),
            0xc3 => (format!("JMP   ${:x}{:x}", input.2, input.1), 3),
            0xc4 => (format!("CNZ   ${:x}{:x}", input.2, input.1), 3),
            0xc5 => (format!("PUSH  B"), 1),
            0xc6 => self.alu_add(self.get_next_byte()),                                 //ADI   #$  ADD data TO ACCUMULATOR
            0xc7 => (format!("RST   0"), 1),
            0xc8 => (format!("RZ"), 1),
            0xc9 => (format!("RET"), 1),
            0xca => (format!("JZ    ${:x}{:x}", input.2, input.1), 3),
            0xcc => (format!("CZ    ${:x}{:x}", input.2, input.1), 3),
            0xcd => (format!("CALL  ${:x}{:x}", input.2, input.1), 3),
            0xce => self.alu_adc(self.get_next_byte()),                                 //ACI   #$  ADD data TO ACCUMULATOR WITH CARRY
            0xcf => (format!("RST   1"), 1),
            0xd0 => (format!("RNC"), 1),
            0xd1 => (format!("POP   D"), 1),
            0xd2 => (format!("JNC   ${:x}{:x}", input.2, input.1), 3),
            0xd3 => (format!("OUT   #${:x}", input.1), 2),
            0xd4 => (format!("CNC   ${:x}{:x}", input.2, input.1), 3),
            0xd5 => (format!("PUSH  D"), 1),
            0xd6 => self.alu_sbb(self.get_next_byte()),                                 //SBB   #$  SUB data TO ACCUMULATOR
            0xd7 => (format!("RST   2"), 1),
            0xd8 => (format!("RC"), 1),
            0xda => (format!("JC    ${:x}{:x}", input.2, input.1), 3),
            0xdb => (format!("IN    #${:x}", input.1), 2),
            0xdc => (format!("CC    ${:x}{:x}", input.2, input.1), 3),
            0xde => self.alu_sbb(self.get_next_byte()),                                 //SBI   #$  SUB data TO ACCUMULATOR WITH BORROW
            0xdf => (format!("RST   3"), 1),
            0xe0 => (format!("RPO"), 1),
            0xe1 => (format!("POP   H"), 1),
            0xe2 => (format!("JPO   ${:x}{:x}", input.2, input.1), 3),
            0xe3 => (format!("XTHL"), 1),
            0xe4 => (format!("CPO   ${:x}{:x}", input.2, input.1), 3),
            0xe5 => (format!("PUSH  H"), 1),
            0xe6 => self.alu_ana(self.get_next_byte()),                                 //ANI   #$  AND data TO ACCUMULATOR
            0xe7 => (format!("RST   4"), 1),
            0xe8 => (format!("RPE"), 1),
            0xe9 => (format!("PCHL"), 1),
            0xea => (format!("JPE   ${:x}{:x}", input.2, input.1), 3),
            0xeb => (format!("XCHG"), 1),
            0xec => (format!("CPE   ${:x}{:x}", input.2, input.1), 3),
            0xee => self.alu_xra(self.get_next_byte()),                                 //XRI   #$  XOR data TO ACCUMULATOR
            0xef => (format!("RST   5"), 1),
            0xf0 => (format!("RP"), 1),
            0xf1 => (format!("POP   PSW"), 1),
            0xf2 => (format!("JP    ${:x}{:x}", input.2, input.1), 3),
            0xf3 => (format!("DI"), 1),
            0xf4 => (format!("CP    ${:x}{:x}", input.2, input.1), 3),
            0xf5 => (format!("PUSH  PSW"), 1),
            0xf6 => self.alu_ora(self.get_next_byte()),                                 //ORI   #$  OR data TO ACCUMULATOR
            0xf7 => (format!("RST   6"), 1),
            0xf8 => (format!("RM"), 1),
            0xf9 => (format!("SPHL"), 1),
            0xfa => (format!("JM    ${:x}{:x}", input.2, input.1), 3),
            0xfb => (format!("EI"), 1),
            0xfc => (format!("CM    ${:x}{:x}", input.2, input.1), 3),
            0xfe => self.alu_cmp(self.get_next_byte()),                                 //CPI   #$  COMPARE data TO ACCUMULATOR
            0xff => (format!("RST   7"), 1),
            _ => (format!("NOP"), 1),
        };
    }
}
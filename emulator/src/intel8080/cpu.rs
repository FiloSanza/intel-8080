use std::rc::Rc;
use std::cell::RefCell;
use std::mem;

use super::bit;
use super::register::Register;
use super::register::Flags;
use super::memory::Memory;

 struct Cpu {
    register: Register,
    memory: Rc<RefCell<Memory>>,
    stop: bool,
    interrupt: bool
}

//This impl block implements Arithmetic Group operations
impl Cpu {
    //Add to the accumulator: A = A + value
    //Instructions:
    // ADD register
    // ADD memory
    // ADI data
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

    //Add to the accumulator with carry: A = A + value + Carry
    //Instructions:
    // ADC register
    // ADC memory
    // ADC data
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
    //Instructions:
    // SUB register
    // SUB memory
    // SUI data
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
    // SBB register
    // SBB memory
    // SBI value
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
    // INR register
    // INR memory
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
    // DCR register
    // DCR memory
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

//This impl block implements Logical Group operations
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

//This impl block implements Data Transfer Group operations
impl Cpu {
    //Load Accumulator Direct: A = Memory[(byte3)(byte2)]
    //Instructions:
    // LDA
    //NO ARE FLAGS AFFECTED
    fn alu_lda(&mut self) {
        let word = self.get_next_word() as usize;
        let value = self.memory.borrow().get(word);
        self.register.a = value;
    }

    //Store Accumulator Direct: Memory[(byte3)(byte2)] = A
    //Instructions:
    // STA
    //NO ARE FLAGS AFFECTED
    fn alu_sta(&mut self) {
        let value = self.register.a;
        let idx = self.get_next_word() as usize;
        self.memory.borrow_mut().set(idx, value);
    }

    //Load H and L direct: (HL) = Memory[(byte3)(byte2)]
    //Instructions:
    // LHLD
    //NO FLAGS ARE AFFECTED
    fn alu_lhld(&mut self) {
        let index = self.get_next_word() as usize;
        let value = self.memory.borrow().get_word(index);
        self.register.set_hl(value);
    }

    //Store H and L direct: Memory[(byte3)(byte2)] = (HL)
    //Instructions:
    // SHLD
    //NO FLAGS ARE AFFECTED
    fn alu_shld(&mut self) {
        let index = self.get_next_word() as usize;
        let value = self.register.get_hl();
        self.memory.borrow_mut().set_word(index, value);
    }

    //Load accumulator indirec: A = Memory[rp] rp can be either HL or DE
    //Instructions:
    // LDAX
    //NO FLAGS ARE AFFECTED
    fn alu_ldax(&mut self, index: u16) {
        let index = index as usize;
        self.register.a = self.memory.borrow().get(index);
    }

    //Store accumulator indirect: Memory[rp] = A rp can be either HL or DE
    //Instructions:
    // STAX
    //NO FLAGS ARE AFFECTED
    fn alu_stax(&mut self, index: u16) {
        self.memory.borrow_mut().set(index as usize, self.register.a);
    }

    //Exchange H and L with D and E: H = D, L = E, D = H, L = E
    //Instructions:
    // XCHG
    //NO FLAGS ARE AFFECTED
    fn alu_xchg(&mut self) {
        mem::swap(&mut self.register.h, &mut self.register.d);
        mem::swap(&mut self.register.l, &mut self.register.e);
    }
}

//This impl block implements Branch Group operations
impl Cpu {
    //Jump and Jump condition: set the pc if the condition is true
    //Instructions:
    // JMP
    // JNZ
    // JZ
    // JNC
    // JC
    // JPO
    // JPE
    // JP
    // JM
    //NO FLAGS ARE AFFECTED
    fn alu_jmp(&mut self, condition: bool) {
        let pos = self.get_next_word();
        if condition {
            self.register.pc = pos;
        }
    }

    //Call and Call condition: set the pc if the condition is true
    //Instructions:
    // CALL
    // CNZ
    // CZ
    // CNC
    // CC
    // CPO
    // CPE
    // CP
    // CM
    //NO FLAGS ARE AFFECTED
    fn alu_call(&mut self, condition: bool) {
        let pos = self.get_next_word();
        if condition {
            self.stack_push(self.register.pc);
            self.register.pc = pos;
        }
    }

    //Return and Return condition: set the pc and sp if the condition is true
    //Instructions:
    // RET
    // RNZ
    // RZ
    // RNC
    // RC
    // RPO
    // RPE
    // RP
    // RM
    //NO FLAGS ARE AFFECTED
    fn alu_ret(&mut self, condition: bool) {
        if condition {
            self.register.pc = self.stack_pop();
        }
    }

    //Restart
    //Instructions:
    // RST 0
    // RST 1
    // RST 2
    // RST 3
    // RST 4
    // RST 5
    // RST 6
    // RST 7
    //NO FLAGS ARE AFFECTED
    fn alu_rst(&mut self, value: u16) {
        self.stack_push(self.register.pc);
        self.register.pc = value * 8;
    }
}

//This impl block implements Stack, I/O and Machine control Group operations
impl Cpu {
    //Exchange stack to with H and L: L = Memory(SP), H = Memory(SP+1)
    //Instructions:
    // XTHL
    //NO FLAGS ARE AFFECTED
    fn alu_xthl(&mut self) {
        let mut mem = self.memory.borrow_mut();
        let sp = mem.get_word(self.register.sp as usize);
        let hl = self.register.get_hl();

        self.register.set_hl(sp);
        mem.set_word(self.register.sp as usize, hl);
    }
}

//This impl block implements some utilities that allow to do some operations with stack and memory
impl Cpu {
    fn stack_push(&mut self, value: u16) {
        self.register.sp = self.register.sp.wrapping_sub(2);
        self.memory.borrow_mut().set_word(self.register.sp as usize, value);
    }

    fn stack_pop(&mut self) -> u16 {
        let result = self.memory.borrow().get_word(self.register.sp as usize);
        self.register.sp = self.register.sp.wrapping_add(2);
        result
    }

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
}

// This impl block implements how the Cpu will be used and will handle the opcodes
impl Cpu {
    pub fn next(&mut self) {
        let opcode = 1;

        match opcode {
            0x00 => { },                                                                //NOP
            0x01 => {                                                                   //LXI   B   SET REGISTER PAIR BC TO data
                let value = self.get_next_word();                                                   
                self.register.set_bc(value);
            },                         
            0x02 => self.alu_stax(self.register.get_bc()),                              //STAX  B   STORE ACCUMULATOR INDIRECT
            0x03 => self.register.set_bc(self.register.get_bc().wrapping_add(1)),       //INX   B   INCREMENT REGISTER PAIR BC
            0x04 => self.register.b = self.alu_inr(self.register.b),                    //INR   B   INCREMENT REGISTER B
            0x05 => self.register.b = self.alu_dcr(self.register.b),                    //DCR   B   DECREMENT REGISTER B
            0x06 => self.register.b = self.get_next_byte(),                             //MVI   B,$ MOVE data INTO REGISTER B
            0x07 => self.alu_rlc(),                                                     //RLC       ROTATE ACCUMULATOR LEFT
            0x09 => self.alu_dad(self.register.get_bc()),                               //DAD   B   ADD REGISTER PAIR BC TO HL
            0x0a => self.alu_ldax(self.register.get_bc()),                              //LDAX  B   LOAD ACCUMULATOR INDIRECT
            0x0b => self.register.set_bc(self.register.get_bc().wrapping_sub(1)),       //DCX   B   DECREMENT REGISTER PAIR BC
            0x0c => self.register.c = self.alu_inr(self.register.c),                    //INR   C   INCREMENT REGISTER C
            0x0d => self.register.c = self.alu_dcr(self.register.c),                    //DCR   C   DECREMENT REGISTER C
            0x0e => self.register.c = self.get_next_byte(),                             //MVI   C,$ MOVE data INTO REGISTER C
            0x0f => self.alu_rrc(),                                                     //RRC       ROTATE ACCUMULATOR RIGHT 
            0x11 => {                                                                   //LXI   D   SET REGISTER PAIR DE TO data
                let value = self.get_next_word();
                self.register.set_de(value);
            },                         
            0x12 => self.alu_stax(self.register.get_de()),                              //STAX  D   STORE ACCUMULATOR INDIRECT
            0x13 => self.register.set_de(self.register.get_de().wrapping_add(1)),       //INX   D   INCREMENT REGISTER PAIR DE
            0x14 => self.register.d = self.alu_inr(self.register.d),                    //INR   D   INCREMENT REGISTER D
            0x15 => self.register.d = self.alu_dcr(self.register.d),                    //DCR   D   DECREMENT REGISTER D
            0x16 => self.register.d = self.get_next_byte(),                             //MVI   D,$ MOVE data INTO REGISTER D
            0x17 => self.alu_ral(),                                                     //RAL       ROTATE ACCUMULATOR LEFT THROUGH CARRY
            0x19 => self.alu_dad(self.register.get_de()),                               //DAD   D   ADD REGISTER PAIR DE TO HL
            0x1a => self.alu_ldax(self.register.get_de()),                              //LDAX  D   LOAD ACCUMULATOR INDIRECT
            0x1b => self.register.set_de(self.register.get_de().wrapping_sub(1)),       //DCX   D   DECREMENT REGISTER PAIR DE
            0x1c => self.register.e = self.alu_inr(self.register.e),                    //INR   E   INCREMENT REGISTER E
            0x1d => self.register.e = self.alu_dcr(self.register.e),                    //DCR   E   DECREMENT REGISTER E
            0x1e => self.register.e = self.get_next_byte(),                             //MVI   E,$ MOVE data INTO REGISTER E
            0x1f => self.alu_rar(),                                                     //RAR       ROTATE ACCUMULATOR RIGHT THROUGH CARRY
            0x21 => {                                                                   //LXI   H   SET REGISTER PAIR HL TO data
                let value = self.get_next_word();
                self.register.set_hl(value);
            },
            0x22 => self.alu_shld(),                                                    //SHLD  #   STORE REGISTER PAIR HL DIRECT
            0x23 => self.register.set_hl(self.register.get_hl().wrapping_add(1)),       //INX   H   INCREMENT REGISTER PAIR HL
            0x24 => self.register.h = self.alu_inr(self.register.h),                    //INR   H   INCREMENT REGISTER H
            0x25 => self.register.h = self.alu_dcr(self.register.h),                    //DCR   H   DECREMENT REGISTER H
            0x26 => self.register.h = self.get_next_byte(),                             //MVI   H,$ MOVE data INTO REGISTER H
            0x27 => self.alu_daa(),                                                     //DAA       DECIMAL ADJUST ACCUMULATION
            0x29 => self.alu_dad(self.register.get_hl()),                               //DAD   H   ADD REGISTER PAIR HL TO HL
            0x2a => self.alu_lhld(),                                                    //LHLD  #   LOAD REGISTER PAIR HL DIRECT
            0x2b => self.register.set_hl(self.register.get_hl().wrapping_sub(1)),       //DCX   H   DECREMENT REGISTER PAIR HL
            0x2c => self.register.l = self.alu_inr(self.register.l),                    //INR   L   INCREMENT REGISTER L
            0x2d => self.register.l = self.alu_dcr(self.register.l),                    //DCR   L   DECREMENT REGISTER L
            0x2e => self.register.l = self.get_next_byte(),                             //MVI   L,$ MOVE data INTO REGISTER L
            0x2f => self.alu_cma(),                                                     //CMA       COMPLEMENT ACCUMULATOR
            0x31 => self.register.sp = self.get_next_word(),                            //LXI   SP  SET SP TO data
            0x32 => self.alu_sta(),                                                     //STA   #   STORE ACCUMULATOR DIRECT
            0x33 => self.register.sp = self.register.sp.wrapping_add(1),                //INX   SP  INCREMENT REGISTER PAIR SP
            0x34 => {                                                                   //INR   M   INCREMENT memory 
                let m = self.alu_inr(self.get_m());
                self.set_m(m);
            },                             
            0x35 => {                                                                   //DCR   M   DECREMENT memory
                let m = self.alu_dcr(self.get_m());
                self.set_m(m);
            },
            0x36 => {                                                                   //MVI   M,$ MOVE data INTO memory
                let value = self.get_next_byte();
                self.set_m(value);
            },
            0x37 => self.alu_stc(),                                                     //STC       SET CARRY 
            0x39 => self.alu_dad(self.register.sp),                                     //DAD   SP  ADD STACK POINTER TO HL
            0x3a => self.alu_lda(),                                                     //LDA   #   LOAD ACCUMULATOR DIRECT 
            0x3b => self.register.sp = self.register.sp.wrapping_sub(1),                //DCX   SP  DECREMENT REGISTER PAIR SP
            0x3c => self.register.a = self.alu_inr(self.register.a),                    //INR   A   INCREMENT REGISTER A
            0x3d => self.register.a = self.alu_dcr(self.register.a),                    //DCR   A   DECREMENT REGISTER A
            0x3e => self.register.a = self.get_next_byte(),                             //MVI   A,$ MOVE data INTO REGISTER A
            0x3f => self.alu_cmc(),                                                     //CMC       COMPLEMENT CARRY
            0x40 => {},                                                                 //MOV   B,B MOVE REGISTER B INTO B
            0x41 => self.register.b = self.register.c,                                  //MOV   B,C MOVE REGISTER C INTO B                  
            0x42 => self.register.b = self.register.d,                                  //MOV   B,D MOVE REGISTER D INTO B
            0x43 => self.register.b = self.register.e,                                  //MOV   B,E MOVE REGISTER E INTO B
            0x44 => self.register.b = self.register.h,                                  //MOV   B,H MOVE REGISTER H INTO B
            0x45 => self.register.b = self.register.l,                                  //MOV   B,L MOVE REGISTER L INTO B
            0x46 => self.register.b = self.get_m(),                                     //MOV   B,M MOVE memory INTO B
            0x47 => self.register.b = self.register.a,                                  //MOV   B,A MOVE REGISTER A INTO B
            0x48 => self.register.c = self.register.b,                                  //MOV   C,B MOVE REGISTER B INTO C
            0x49 => {},                                                                 //MOV   C,C MOVE REGISTER C INTO C
            0x4a => self.register.c = self.register.d,                                  //MOV   C,D MOVE REGISTER D INTO C
            0x4b => self.register.c = self.register.e,                                  //MOV   C,E MOVE REGISTER E INTO C
            0x4c => self.register.c = self.register.h,                                  //MOV   C,H MOVE REGISTER H INTO C
            0x4d => self.register.c = self.register.l,                                  //MOV   C,L MOVE REGISTER L INTO C
            0x4e => self.register.c = self.get_m(),                                     //MOV   C,M MOVE memory INTO C
            0x4f => self.register.c = self.register.a,                                  //MOV   C,A MOVE REGISTER A INTO C
            0x50 => self.register.d = self.register.b,                                  //MOV   D,B MOVE REGISTER B INTO D
            0x51 => self.register.d = self.register.c,                                  //MOV   D,C MOVE REGISTER C INTO D
            0x52 => {},                                                                 //MOV   D,D MOVE REGISTER D INTO D
            0x53 => self.register.d = self.register.e,                                  //MOV   D,E MOVE REGISTER E INTO D
            0x54 => self.register.d = self.register.h,                                  //MOV   D,H MOVE REGISTER H INTO D
            0x55 => self.register.d = self.register.l,                                  //MOV   D,L MOVE REGISTER L INTO D
            0x56 => self.register.d = self.get_m(),                                     //MOV   D,M MOVE memory INTO B
            0x57 => self.register.d = self.register.a,                                  //MOV   D,A MOVE REGISTER A INTO D
            0x58 => self.register.e = self.register.b,                                  //MOV   E,B MOVE REGISTER B INTO E
            0x59 => self.register.e = self.register.c,                                  //MOV   E,C MOVE REGISTER C INTO E
            0x5a => self.register.e = self.register.d,                                  //MOV   E,D MOVE REGISTER D INTO E
            0x5b => {},                                                                 //MOV   E,E MOVE REGISTER E INTO E
            0x5c => self.register.e = self.register.h,                                  //MOV   E,H MOVE REGISTER H INTO E
            0x5d => self.register.e = self.register.l,                                  //MOV   E,L MOVE REGISTER L INTO E
            0x5e => self.register.e = self.get_m(),                                     //MOV   E,M MOVE memory INTO E
            0x5f => self.register.e = self.register.a,                                  //MOV   E,A MOVE REGISTER A INTO E
            0x60 => self.register.h = self.register.b,                                  //MOV   H,B MOVE REGISTER B INTO H
            0x61 => self.register.h = self.register.c,                                  //MOV   H,C MOVE REGISTER C INTO H
            0x62 => self.register.h = self.register.d,                                  //MOV   H,D MOVE REGISTER D INTO H
            0x63 => self.register.h = self.register.e,                                  //MOV   H,E MOVE REGISTER E INTO H
            0x64 => {},                                                                 //MOV   H,H MOVE REGISTER H INTO H
            0x65 => self.register.h = self.register.l,                                  //MOV   H,L MOVE REGISTER L INTO H
            0x66 => self.register.h = self.get_m(),                                     //MOV   H,M MOVE memory INTO H
            0x67 => self.register.h = self.register.a,                                  //MOV   H,A MOVE REGISTER A INTO H
            0x68 => self.register.l = self.register.b,                                  //MOV   L,B MOVE REGISTER B INTO L
            0x69 => self.register.l = self.register.c,                                  //MOV   L,C MOVE REGISTER C INTO L
            0x6a => self.register.l = self.register.d,                                  //MOV   L,D MOVE REGISTER D INTO L
            0x6b => self.register.l = self.register.e,                                  //MOV   L,E MOVE REGISTER E INTO L
            0x6c => self.register.l = self.register.h,                                  //MOV   L,H MOVE REGISTER J INTO L
            0x6d => {},                                                                 //MOV   L,L MOVE REGISTER L INTO L
            0x6e => self.register.l = self.get_m(),                                     //MOV   L,M MOVE memory INTO L
            0x6f => self.register.l = self.register.a,                                  //MOV   L,A MOVE REGISTER A INTO L
            0x70 => self.set_m(self.register.b),                                        //MOV   M,B MOVE REGISTER B INTO memory
            0x71 => self.set_m(self.register.c),                                        //MOV   M,C MOVE REGISTER C INTO memory
            0x72 => self.set_m(self.register.d),                                        //MOV   M,D MOVE REGISTER D INTO memory
            0x73 => self.set_m(self.register.e),                                        //MOV   M,E MOVE REGISTER E INTO memory
            0x74 => self.set_m(self.register.h),                                        //MOV   M,H MOVE REGISTER H INTO memory
            0x75 => self.set_m(self.register.l),                                        //MOV   M,L MOVE REGISTER L INTO memory
            0x76 => self.stop = true,                                                   //HLT   STOP THE CPU
            0x77 => self.set_m(self.register.a),                                        //MOV   M,A MOVE REGISTER A INTO memory
            0x78 => self.register.a = self.register.b,                                  //MOV   A,B MOVE REGISTER B INTO A
            0x79 => self.register.a = self.register.c,                                  //MOV   A,C MOVE REGISTER C INTO A
            0x7a => self.register.a = self.register.d,                                  //MOV   A,D MOVE REGISTER D INTO A
            0x7b => self.register.a = self.register.e,                                  //MOV   A,E MOVE REGISTER E INTO A
            0x7c => self.register.a = self.register.h,                                  //MOV   A,H MOVE REGISTER H INTO A
            0x7d => self.register.a = self.register.l,                                  //MOV   A,L MOVE REGISTER L INTO A
            0x7e => self.register.a = self.get_m(),                                     //MOV   A,M MOVE memory INTO A
            0x7f => {},                                                                 //MOV   A,A MOVE REGISTER A INTO A
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
            0xc0 => self.alu_ret(!self.register.get_flag(Flags::Zero)),                 //RNZ       RETURN IF NOT ZERO
            0xc1 => {                                                                   //POP   B   POP TOP OF THE STACK INTO REGISTER PAIR BC
                let value = self.stack_pop();
                self.register.set_bc(value);
            },                             
            0xc2 => self.alu_jmp(!self.register.get_flag(Flags::Zero)),                 //JNZ   #   JUMP TO ADDR IF NOT ZERO
            0xc3 => self.alu_jmp(true),                                                 //JUMP  #   JUMP TO ADDR UNCONDITIONALLY
            0xc4 => self.alu_call(!self.register.get_flag(Flags::Zero)),                //CNZ   #   CALL ADDR IF NOT ZERO
            0xc5 => self.stack_push(self.register.get_bc()),                            //PUSH  B   PUSH REGISTER PAIR BC ON TOP OF THE STACK
            0xc6 => {                                                                   //ADI   #$  ADD data TO ACCUMULATOR
                let value = self.get_next_byte();
                self.alu_add(value);
            },
            0xc7 => self.alu_rst(0),                                                    //RST   0   RESET 0
            0xc8 => self.alu_ret(self.register.get_flag(Flags::Zero)),                  //RN        RETURN IF ZERO
            0xc9 => self.alu_ret(true),                                                 //RET       RETURN UNCONDITIONALLY
            0xca => self.alu_jmp(self.register.get_flag(Flags::Zero)),                  //JZ    #   JUMP TO ADDR IF ZERO
            0xcc => self.alu_call(self.register.get_flag(Flags::Zero)),                 //CZ    #   CALL ADDR IF ZERO
            0xcd => self.alu_call(true),                                                //CALL  #   CALL addr UNCONDITIONALLY
            0xce => {                                                                   //ACI   #$  ADD data TO ACCUMULATOR WITH CARRY
                let value = self.get_next_byte();
                self.alu_adc(value);
            },                                 
            0xcf => self.alu_rst(1),                                                    //RST   1   RESET 1
            0xd0 => self.alu_ret(!self.register.get_flag(Flags::Carry)),                //RNC       RETURN IF NOT CARRY
            0xd1 => {                                                                   //POP   D   POP TOP OF THE STACK INTO REGISTER PAIR DE
                let value = self.stack_pop();
                self.register.set_de(value);
            },                             
            0xd2 => self.alu_jmp(!self.register.get_flag(Flags::Carry)),                //JNC   #   JUMP TO ADDR IF NOT CARRY
            0xd3 => { let _ = self.get_next_byte(); }                                   //OUT   port
            0xd4 => self.alu_call(!self.register.get_flag(Flags::Carry)),               //CNC   #   CALL ADDR IF NOT CARRY
            0xd5 => self.stack_push(self.register.get_de()),                            //PUSH  BD  PUSH REGISTER PAIR DE ON TOP OF THE STACK
            0xd6 => {                                                                   //SBB   #$  SUB data TO ACCUMULATOR
                let value = self.get_next_byte();
                self.alu_sbb(value);  
            },                                 
            0xd7 => self.alu_rst(2),                                                    //RST   2   RESET 2
            0xd8 => self.alu_ret(self.register.get_flag(Flags::Carry)),                 //RC        RETURN IF CARRY
            0xda => self.alu_jmp(self.register.get_flag(Flags::Carry)),                 //JC    #   JUMP NOT CARRY
            0xdb => { let _ = self.get_next_byte(); }                                   //IN    port
            0xdc => self.alu_call(self.register.get_flag(Flags::Carry)),                //CC    #    CALL ADDR CARRY
            0xde => {                                                                   //SBI   #$  SUB data TO ACCUMULATOR WITH BORROW
                let value = self.get_next_byte();
                self.alu_sbb(value);
            },                                 
            0xdf => self.alu_rst(3),                                                    //RST   3   RESET 3
            0xe0 => self.alu_ret(!self.register.get_flag(Flags::Parity)),               //RPO       RETURN IF PARITY ODD
            0xe1 => {                                                                   //POP   H   POP TOP OF THE STACK INTO REGISTER PAIR HL
                let value = self.stack_pop();
                self.register.set_hl(value);
            },                             
            0xe2 => self.alu_jmp(!self.register.get_flag(Flags::Parity)),               //JPO   #   JUMP TO ADDR IF PARITY ODD
            0xe3 => self.alu_xthl(),                                                    //XTHL      EXCHANGE REGISTER PARI HL WITH STACK TOP
            0xe4 => self.alu_call(!self.register.get_flag(Flags::Parity)),              //CPO   #   CALL ADDR IF PARITY ODD
            0xe5 => self.stack_push(self.register.get_hl()),                            //PUSH  H   PUSH REGISTER PAIR HL ON TOP OF THE STACK
            0xe6 => {                                                                   //ANI   #$  AND data TO ACCUMULATOR
                let value = self.get_next_byte();
                self.alu_ana(value);
            },                                 
            0xe7 => self.alu_rst(4),                                                    //RST   4   RESET 4
            0xe8 => self.alu_ret(self.register.get_flag(Flags::Parity)),                //RPE       RETURN IF PARITY EVEN
            0xe9 => self.register.pc = self.register.get_hl(),                          //PCHL      SET PC TO REGISTER PAIR HL
            0xea => self.alu_jmp(self.register.get_flag(Flags::Parity)),                //JPE   #   JUMP TO ADDR IF PARITY EVEN
            0xeb => self.alu_xchg(),                                                    //XCHG      EXCHANGE H WITH D AND L WITH E
            0xec => self.alu_call(self.register.get_flag(Flags::Parity)),               //CPE   #   CALL ADDR IF PARITY EVEN
            0xee => {                                                                   //XRI   #$  XOR data TO ACCUMULATOR
                let value = self.get_next_byte();
                self.alu_xra(value);
            },                                 
            0xef => self.alu_rst(5),                                                    //RST   5   RESET 5
            0xf0 => self.alu_ret(!self.register.get_flag(Flags::Sign)),                 //RP        RETURN IF POSITIVE
            0xf1 => {                                                                   //POP   PSW POP TOP OF THE STACK INTO AF
                let value = self.stack_pop();
                self.register.set_af(value);
            },                             
            0xf2 => self.alu_jmp(!self.register.get_flag(Flags::Sign)),                 //JP    #   JUMP TO ADDR IF POSITIVE
            0xf3 => self.interrupt = false,                                             //DI        DISABLE INTERRUPTS
            0xf4 => self.alu_call(!self.register.get_flag(Flags::Sign)),                //CP    #   CALL ADDR IF POSITIVE
            0xf5 => self.stack_push(self.register.get_af()),                            //PUSH  PSW PUSH AF ON TOP OF THE STACK
            0xf6 => {                                                                   //ORI   #$  OR data TO ACCUMULATOR
                let value = self.get_next_byte();
                self.alu_ora(value);
            },                                
            0xf7 => self.alu_rst(6),                                                    //RST   6   RESET 6
            0xf8 => self.alu_ret(self.register.get_flag(Flags::Sign)),                  //RM        RETURN IF NEGATIVE
            0xf9 => self.register.sp = self.register.get_hl(),                          //SPHL      SET STACK TOP TO REGISTER PAIR HL
            0xfa => self.alu_jmp(self.register.get_flag(Flags::Sign)),                  //JM    #   JUMP TO ADDR IF NEGATIVE
            0xfb => self.interrupt = true,                                              //EI        ENABLE INTERRUPTS
            0xfc => self.alu_call(self.register.get_flag(Flags::Sign)),                 //CN    #   CALL ADDR IF NEGATIVE
            0xfe => {                                                                   //CPI   #$  COMPARE data TO ACCUMULATOR
                let value = self.get_next_byte();
                self.alu_cmp(value);
            },                                 
            0xff => self.alu_rst(7),                                                    //RST   7   RESET 7
            _ => {},
        };
    }
}
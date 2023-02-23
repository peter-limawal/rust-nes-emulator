use crate::opcodes;
use std::collections::HashMap;

// Flags
bitflags! {
    /// # Status Register (P) http://wiki.nesdev.com/w/index.php/Status_flags
    ///
    ///  7 6 5 4 3 2 1 0
    ///  N V _ B D I Z C
    ///  | |   | | | | +--- Carry Flag
    ///  | |   | | | +----- Zero Flag
    ///  | |   | | +------- Interrupt Disable
    ///  | |   | +--------- Decimal Mode (not used on NES)
    ///  | |   +----------- Break Command
    ///  | +--------------- Overflow Flag
    ///  +----------------- Negative Flag
    ///

    // Declare CPUFlags
    pub struct CPUFlags: u8 {
        const CARRY             = 0b00000001;
        const ZERO              = 0b00000010;
        const INTERRUPT_DISABLE = 0b00000100;
        const DECIMAL_MODE      = 0b00001000;
        const BREAK             = 0b00010000;
        const BREAK2            = 0b00100000;
        const OVERFLOW          = 0b01000000;
        const NEGATIVE          = 0b10000000;
    }
}

// Stack
const STACK: u16 = 0x0100;
const STACK_RESET: u8 = 0xfd;

// Declare CPU struct
pub struct CPU {
    pub register_a: u8,       // register_a address is unsigned 8-bit
    pub register_x: u8,       // register_x address is unsigned 8-bit
    pub register_y: u8,       // register_y address is unsigned 8-bit
    pub status: CPUFlags,     // status is unsigned 8-bit
    pub program_counter: u16, // pc is unsigned 16-bit
    pub stack_pointer: u8,    // stack pointer is unsigned 8-bit
    memory: [u8; 0xFFFF],
}

// Addressing modes
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPage_X,
    ZeroPage_Y,
    Absolute,
    Absolute_X,
    Absolute_Y,
    Indirect_X,
    Indirect_Y,
    NoneAddressing,
}

// Declare Mem (memory) trait
pub trait Mem {
    fn mem_read(&self, addr: u16) -> u8;
    fn mem_write(&mut self, addr: u16, data: u8);

    fn mem_read_u16(&self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }
}

// Implement functionality of Mem for CPU
impl Mem for CPU {
    fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }
}

// Declare Stack trait
trait Stack {
    fn stack_pop(&mut self) -> u8;
    fn stack_push(&mut self, data: u8);

    fn stack_pop_u16(&mut self) -> u16 {
        let lo = self.stack_pop() as u16;
        let hi = self.stack_pop() as u16;
        hi << 8 | lo
    }

    fn stack_push_u16(&mut self, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.stack_push(hi);
        self.stack_push(lo);
    }
}

// Implement functionality of stack for CPU
impl Stack for CPU {
    fn stack_pop(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        self.mem_read((STACK as u16) + self.stack_pointer as u16)
    }

    fn stack_push(&mut self, data: u8) {
        self.mem_write((STACK as u16) + self.stack_pointer as u16, data);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }
}

// Implement functionality of CPU
impl CPU {
    // Create new CPU object
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: CPUFlags::from_bits_truncate(0b100100),
            program_counter: 0,
            stack_pointer: STACK_RESET,
            memory: [0; 0xFFFF],
        }
    }

    // CPU INSTRUCTION HELPER FUNCTIONS

    // Addressing mode interpretation for CPU instructions
    fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.program_counter,

            AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16,

            AddressingMode::ZeroPage_X => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_x) as u16;
                addr
            }

            AddressingMode::ZeroPage_Y => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_y) as u16;
                addr
            }

            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),

            AddressingMode::Absolute_X => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_x as u16);
                addr
            }

            AddressingMode::Absolute_Y => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_y as u16);
                addr
            }

            AddressingMode::Indirect_X => {
                let base = self.mem_read(self.program_counter);
                let ptr: u8 = (base as u8).wrapping_add(self.register_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)
            }

            AddressingMode::Indirect_Y => {
                let base = self.mem_read(self.program_counter);
                let lo = self.mem_read(base as u16);
                let hi = self.mem_read((base as u8).wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let deref = deref_base.wrapping_add(self.register_y as u16);
                deref
            }

            AddressingMode::NoneAddressing => {
                panic!("mode {:?} is not supported", mode);
            }
        }
    }

    // Update zero and negative flags using binary arithmetic
    fn update_zero_and_negative_flags(&mut self, result: u8) {
        if result == 0 {
            self.status.insert(CPUFlags::ZERO);
        } else {
            self.status.remove(CPUFlags::ZERO);
        }

        if result & 0b1000_0000 != 0 {
            self.status.insert(CPUFlags::NEGATIVE);
        } else {
            self.status.remove(CPUFlags::NEGATIVE);
        }
    }

    // Set A = data and update zero and negative flags
    fn set_register_a(&mut self, data: u8) {
        self.register_a = data;
        self.update_zero_and_negative_flags(self.register_a);
    }

    // Set X = data and update zero and negative flags
    fn set_register_x(&mut self, data: u8) {
        self.register_x = data;
        self.update_zero_and_negative_flags(self.register_x);
    }

    // Set Y = data and update zero and negative flags
    fn set_register_y(&mut self, data: u8) {
        self.register_y = data;
        self.update_zero_and_negative_flags(self.register_y);
    }

    // Adding to register A
    // note: ignoring decimal mode http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
    fn add_to_register_a(&mut self, data: u8) {
        let sum = self.register_a as u16
            + data as u16
            + (if self.status.contains(CPUFlags::CARRY) {
                1
            } else {
                0
            }) as u16;

        // Check for carry out
        let carry = sum > 0xff;
        if carry {
            self.sec();
        } else {
            self.clc();
        }

        // Check for overflow
        let result = sum as u8;
        if (data ^ result) & (result ^ self.register_a) & 0x80 != 0 {
            self.status.insert(CPUFlags::OVERFLOW);
        } else {
            self.clv();
        }

        self.set_register_a(result);
    }

    // CPU INSTRUCTION IMPLEMENTATION
    // Refer to https://www.nesdev.org/obelisk-6502-guide/reference.html for more details

    // ADC - Add with Carry: Adds the contents of a memory location to the accumulator together with the carry bit
    // A,Z,C,N = A+M+C
    fn adc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.add_to_register_a(data);
    }

    // AND - Logical AND: Performed bit by bit on the accumulator contents using the contents of a byte of memory
    // A,Z,N = A&M
    fn and(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.set_register_a(data & self.register_a);
    }

    // ASL - Arithmetic Shift Left: Shifts all the bits of the accumulator or memory contents one bit left
    // A,Z,C,N = M*2 or M,Z,C,N = M*2
    fn asl_accumulator(&mut self) {
        let mut data = self.register_a;
        if data >> 7 == 1 {
            self.sec();
        } else {
            self.clc();
        }
        data = data << 1;
        self.set_register_a(data)
    }
    fn asl(&mut self, mode: &AddressingMode) -> u8 {
        let addr = self.get_operand_address(mode);
        let mut data = self.mem_read(addr);
        if data >> 7 == 1 {
            self.sec();
        } else {
            self.clc();
        }
        data = data << 1;
        self.mem_write(addr, data);
        self.update_zero_and_negative_flags(data);
        data
    }

    // BCC - Branch if Carry Clear: If the carry flag is clear then add the relative displacement to the program counter to cause a branch to a new location
    // BCS - Branch if Carry Set: If the carry flag is set then add the relative displacement to the program counter to cause a branch to a new location
    // BEQ - Branch if Equal: If the zero flag is set then add the relative displacement to the program counter to cause a branch to a new location
    // BMI - Branch if Minus: If the negative flag is set then add the relative displacement to the program counter to cause a branch to a new location
    // BNE - Branch if Not Equal: If the zero flag is clear then add the relative displacement to the program counter to cause a branch to a new location
    // BPL - Branch if Positive: If the negative flag is clear then add the relative displacement to the program counter to cause a branch to a new location
    // BVC - Branch if Overflow Clear: If the overflow flag is clear then add the relative displacement to the program counter to cause a branch to a new location
    // BVS - Branch if Overflow Set: If the overflow flag is set then add the relative displacement to the program counter to cause a branch to a new location
    fn branch(&mut self, condition: bool) {
        if condition {
            let jump: i8 = self.mem_read(self.program_counter) as i8;
            let jump_addr = self
                .program_counter
                .wrapping_add(1)
                .wrapping_add(jump as u16);
            self.program_counter = jump_addr;
        }
    }

    // BRK - Force Interrupt: Forces the generation of an interrupt request
    // Break out of run, no implementation

    // BIT - Bit Test
    // A & M, N = M7, V = M6
    fn bit(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        if (self.register_a & data) == 0 {
            self.status.insert(CPUFlags::ZERO);
        } else {
            self.status.remove(CPUFlags::ZERO);
        }

        self.status.set(CPUFlags::NEGATIVE, data & 0b10000000 > 0);
        self.status.set(CPUFlags::OVERFLOW, data & 0b01000000 > 0);
    }

    // CLC - Clear Carry Flag: Set the carry flag to zero
    // C = 0
    fn clc(&mut self) {
        self.status.remove(CPUFlags::CARRY);
    }

    // CLD - Clear Decimal Mode: Sets the decimal mode flag to zero
    // D = 0
    fn cld(&mut self) {
        self.status.remove(CPUFlags::DECIMAL_MODE);
    }

    // CLI - Clear Interrupt Disable: Clears the interrupt disable flag allowing normal interrupt requests to be serviced
    // I = 0
    fn cli(&mut self) {
        self.status.remove(CPUFlags::INTERRUPT_DISABLE);
    }

    // CLV - Clear Overflow Flag: Clears the overflow flag
    // V = 0
    fn clv(&mut self) {
        self.status.remove(CPUFlags::OVERFLOW);
    }

    // CMP - Compare: Compares the contents of the accumulator with another memory held value and sets the zero and carry flags as appropriate
    // Z,C,N = A-M
    // CPX - Compare X Register: Compares the contents of the X register with another memory held value and sets the zero and carry flags as appropriate
    // Z,C,N = X-M
    // CPY - Compare Y Register: Compares the contents of the Y register with another memory held value and sets the zero and carry flags as appropriate
    // Z,C,N = Y-M
    fn compare(&mut self, mode: &AddressingMode, compare_with: u8) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        if data <= compare_with {
            self.sec()
        } else {
            self.clc();
        }
        self.update_zero_and_negative_flags(compare_with.wrapping_sub(data));
    }

    // DEC - Decrement Memory: Subtracts one from the value held at a specified memory location setting the zero and negative flags as appropriate
    // M,Z,N = M-1
    fn dec(&mut self, mode: &AddressingMode) -> u8 {
        let addr = self.get_operand_address(mode);
        let mut data = self.mem_read(addr);
        data = data.wrapping_sub(1);
        self.mem_write(addr, data);
        self.update_zero_and_negative_flags(data);
        data
    }

    // DEX - Decrement X Register: Subtracts one from the X register setting the zero and negative flags as appropriate
    // X,Z,N = X-1
    fn dex(&mut self) {
        self.register_x = self.register_x.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    // DEY - Decrement Y Register: Subtracts one from the Y register setting the zero and negative flags as appropriate
    // Y,Z,N = Y-1
    fn dey(&mut self) {
        self.register_y = self.register_y.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_y);
    }

    // EOR - Exclusive OR: Performed bit by bit on the accumulator contents using the contents of a byte of memory
    // A,Z,N = A^M
    fn eor(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.set_register_a(data ^ self.register_a);
    }

    // INC - Increment Memory: Adds one to the value held at a specified memory location setting the zero and negative flags as appropriate
    // M,Z,N = M+1
    fn inc(&mut self, mode: &AddressingMode) -> u8 {
        let addr = self.get_operand_address(mode);
        let mut data = self.mem_read(addr);
        data = data.wrapping_add(1);
        self.mem_write(addr, data);
        self.update_zero_and_negative_flags(data);
        data
    }

    // INX - Increment X Register: Adds one to the X register setting the zero and negative flags as appropriate
    // X,Z,N = X+1
    fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_x)
    }

    // INY - Increment X Register: Adds one to the Y register setting the zero and negative flags as appropriate
    // X,Z,N = Y+1
    fn iny(&mut self) {
        self.register_y = self.register_y.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_y)
    }

    // JMP - Jump: Sets the program counter to the address specified by the operand
    // Implementation depends on Absolute or Indirect (see fn run())

    // JSR - Jump to Subroutine: Pushes the address (minus one) of the return point on to the stack and then sets the program counter to the target memory address
    fn jsr(&mut self) {
        self.stack_push_u16(self.program_counter + 2 - 1);
        let target_address = self.mem_read_u16(self.program_counter);
        self.program_counter = target_address
    }

    // LDA - Load Accumulator: Loads a byte of memory into the accumulator setting the zero and negative flags as appropriate
    // A,Z,N = M
    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.set_register_a(data);
    }

    // LDX - Load X Register: Loads a byte of memory into the X register setting the zero and negative flags as appropriate
    // X,Z,N = M
    fn ldx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.set_register_x(data);
    }

    // LDY - Load Y Register: Loads a byte of memory into the Y register setting the zero and negative flags as appropriate
    // Y,Z,N = M
    fn ldy(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.set_register_y(data);
    }

    // LSR - Logical Shift Right: Each of the bits in A or M is shift one place to the right
    // A,C,Z,N = A/2 or M,C,Z,N = M/2
    fn lsr_accumulator(&mut self) {
        let mut data = self.register_a;
        if data & 1 == 1 {
            self.sec();
        } else {
            self.clc();
        }
        data = data >> 1;
        self.set_register_a(data);
    }
    fn lsr(&mut self, mode: &AddressingMode) -> u8 {
        let addr = self.get_operand_address(mode);
        let mut data = self.mem_read(addr);
        if data & 1 == 1 {
            self.sec();
        } else {
            self.clc();
        }
        data = data >> 1;
        self.mem_write(addr, data);
        self.update_zero_and_negative_flags(data);
        data
    }

    // NOP - No Operation: Causes no changes to the processor other than the normal incrementing of the program counter to the next instruction
    // Do nothing, no implementation

    // ORA - Logical Inclusive OR: Performed bit by bit on the accumulator contents using the contents of a byte of memory
    // A,Z,N = A|M
    fn ora(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.set_register_a(data | self.register_a);
    }

    // PHA - Push Accumulator: Pushes a copy of the accumulator on to the stack
    fn pha(&mut self) {
        self.stack_push(self.register_a);
    }

    // PHP - Push Processor Status: Pushes a copy of the status flags on to the stack
    fn php(&mut self) {
        // http://wiki.nesdev.com/w/index.php/CPU_status_flag_behavior
        let mut flags = self.status.clone();
        flags.insert(CPUFlags::BREAK);
        flags.insert(CPUFlags::BREAK2);
        self.stack_push(flags.bits());
    }

    // PLA - Pull Accumulator: Pulls an 8 bit value from the stack and into the accumulator
    fn pla(&mut self) {
        let data = self.stack_pop();
        self.set_register_a(data);
    }

    // PLP - Pull Processor Status: Pulls an 8 bit value from the stack and into the processor flags
    fn plp(&mut self) {
        self.status.bits = self.stack_pop();
        self.status.remove(CPUFlags::BREAK);
        self.status.insert(CPUFlags::BREAK2);
    }

    // ROL - Rotate Left: Move each of the bits in either A or M one place to the left
    fn rol(&mut self, mode: &AddressingMode) -> u8 {
        let addr = self.get_operand_address(mode);
        let mut data = self.mem_read(addr);
        let old_carry = self.status.contains(CPUFlags::CARRY);
        if data >> 7 == 1 {
            self.sec();
        } else {
            self.clc();
        }
        data = data << 1;
        if old_carry {
            data = data | 1;
        }
        self.mem_write(addr, data);
        self.update_zero_and_negative_flags(data);
        data
    }
    fn rol_accumulator(&mut self) {
        let mut data = self.register_a;
        let old_carry = self.status.contains(CPUFlags::CARRY);
        if data >> 7 == 1 {
            self.sec();
        } else {
            self.clc();
        }
        data = data << 1;
        if old_carry {
            data = data | 1;
        }
        self.set_register_a(data);
    }

    // ROR - Rotate Right: Move each of the bits in either A or M one place to the right
    fn ror(&mut self, mode: &AddressingMode) -> u8 {
        let addr = self.get_operand_address(mode);
        let mut data = self.mem_read(addr);
        let old_carry = self.status.contains(CPUFlags::CARRY);
        if data & 1 == 1 {
            self.sec();
        } else {
            self.clc();
        }
        data = data >> 1;
        if old_carry {
            data = data | 0b10000000;
        }
        self.mem_write(addr, data);
        self.update_zero_and_negative_flags(data);
        data
    }
    fn ror_accumulator(&mut self) {
        let mut data = self.register_a;
        let old_carry = self.status.contains(CPUFlags::CARRY);
        if data & 1 == 1 {
            self.sec();
        } else {
            self.clc();
        }
        data = data >> 1;
        if old_carry {
            data = data | 0b10000000;
        }
        self.set_register_a(data);
    }

    // RTI - Return from Interrupt: Pulls the processor flags from the stack followed by the program counter (used at the end of an interrupt processing routine)
    fn rti(&mut self) {
        self.status.bits = self.stack_pop();
        self.status.remove(CPUFlags::BREAK);
        self.status.insert(CPUFlags::BREAK2);
        self.program_counter = self.stack_pop_u16();
    }

    // RTS - Return from Subroutine: Pulls the program counter (minus one) from the stack (used at the end of a subroutine to return to the calling routine)
    fn rts(&mut self) {
        self.program_counter = self.stack_pop_u16() + 1;
    }

    // SBC - Subtract with Carry: Subtracts the contents of a memory location to the accumulator together with the not of the carry bit
    // A,Z,C,N = A-M-(1-C)
    fn sbc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.add_to_register_a(((data as i8).wrapping_neg().wrapping_sub(1)) as u8);
    }

    // SEC - Set Carry Flag: Set the carry flag to one
    // C = 1
    fn sec(&mut self) {
        self.status.insert(CPUFlags::CARRY);
    }

    // SED - Set Decimal Flag: Set the decimal mode flag to one
    // D = 1
    fn sed(&mut self) {
        self.status.insert(CPUFlags::DECIMAL_MODE);
    }

    // SEI - Set Interrupt Disable: Set the interrupt disable flag to one
    // I = 1
    fn sei(&mut self) {
        self.status.insert(CPUFlags::INTERRUPT_DISABLE);
    }

    // STA - Store Accumulator: Stores the contents of the accumulator into memory
    // M = A
    fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);
    }

    // STX - Store X Register: Stores the contents of the X register into memory
    // M = X
    fn stx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_x);
    }

    // STY - Store Y Register: Stores the contents of the Y register into memory
    // M = Y
    fn sty(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_y);
    }

    // TAX - Transfer Accumulator to X: Copies the current contents of the accumulator into the X register and sets the zero and negative flags as appropriate
    // X = A
    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    // TAY - Transfer Accumulator to Y: Copies the current contents of the accumulator into the Y register and sets the zero and negative flags as appropriate
    // Y = A
    fn tay(&mut self) {
        self.register_y = self.register_a;
        self.update_zero_and_negative_flags(self.register_y);
    }

    // TSX - Transfer Stack Pointer to X: Copies the current contents of the stack register into the X register and sets the zero and negative flags as appropriate
    // X = S
    fn tsx(&mut self) {
        self.register_x = self.stack_pointer;
        self.update_zero_and_negative_flags(self.register_x);
    }

    // TXA - Transfer X to Accumulator: Copies the current contents of the X register into the accumulator and sets the zero and negative flags as appropriate
    // A = X
    fn txa(&mut self) {
        self.register_a = self.register_x;
        self.update_zero_and_negative_flags(self.register_a);
    }

    // TXS - Transfer X to Stack Pointer: Copies the current contents of the X register into the stack register
    // S = X
    fn txs(&mut self) {
        self.stack_pointer = self.register_x;
    }

    // TYA - Transfer Y to Accumulator: Copies the current contents of the Y register into the accumulator and sets the zero and negative flags as appropriate
    // A = Y
    fn tya(&mut self) {
        self.register_a = self.register_y;
        self.update_zero_and_negative_flags(self.register_a);
    }

    // "Run" CPU with instructions from program
    pub fn run(&mut self) {
        self.run_with_callback(|_| {});
    }
    pub fn run_with_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut CPU),
    {
        let ref opcodes: HashMap<u8, &'static opcodes::OpCode> = *opcodes::OPCODES_MAP; // HashMap of opcodes

        // CPU fetch-execute cycle
        loop {
            callback(self);
            let code = self.mem_read(self.program_counter);
            self.program_counter += 1;
            let program_counter_state = self.program_counter;

            // Error-check opcode
            let opcode = opcodes
                .get(&code)
                .expect(&format!("OpCode {:x} is not recognised", code));

            match code {
                // ADC - Add with Carry
                0x69 | 0x65 | 0x75 | 0x6D | 0x7D | 0x79 | 0x61 | 0x71 => {
                    self.adc(&opcode.mode);
                }
                // AND - Logical AND
                0x29 | 0x25 | 0x35 | 0x2D | 0x3D | 0x39 | 0x21 | 0x31 => {
                    self.and(&opcode.mode);
                }
                // ASL - Arithmetic Shift Left
                0x0A => {
                    self.asl_accumulator();
                }
                0x06 | 0x16 | 0x0E | 0x1E => {
                    self.asl(&opcode.mode);
                }
                // BCC - Branch if Carry Clear
                0x90 => {
                    self.branch(!self.status.contains(CPUFlags::CARRY));
                }
                // BCS - Branch if Carry Set
                0xB0 => {
                    self.branch(self.status.contains(CPUFlags::CARRY));
                }
                // BEQ - Branch if Equal
                0xF0 => {
                    self.branch(self.status.contains(CPUFlags::ZERO));
                }
                // BIT - Bit Test
                0x24 | 0x2C => {
                    self.bit(&opcode.mode);
                }
                // BMI - Branch if Minus
                0x30 => {
                    self.branch(self.status.contains(CPUFlags::NEGATIVE));
                }
                // BNE - Branch if Not Equal
                0xD0 => {
                    self.branch(!self.status.contains(CPUFlags::ZERO));
                }
                // BPL - Branch if Positive
                0x10 => {
                    self.branch(!self.status.contains(CPUFlags::NEGATIVE));
                }
                // BRK - Force Interrupt
                0x00 => {
                    return;
                }
                // BVC - Branch if Overflow Clear
                0x50 => {
                    self.branch(!self.status.contains(CPUFlags::OVERFLOW));
                }
                // BVS - Branch if Overflow Set
                0x70 => {
                    self.branch(self.status.contains(CPUFlags::OVERFLOW));
                }
                // CLC - Clear Carry Flag
                0x18 => {
                    self.clc();
                }
                // CLD - Clear Decimal Mode
                0xD8 => {
                    self.cld();
                }
                // CLI - Clear Interrupt Disable
                0x58 => {
                    self.cli();
                }
                // CLV - Clear Overflow Flag
                0xB8 => {
                    self.clv();
                }
                // CMP - Compare
                0xC9 | 0xC5 | 0xD5 | 0xCD | 0xDD | 0xD9 | 0xC1 | 0xD1 => {
                    self.compare(&opcode.mode, self.register_a);
                }
                // CPX - Compare X Register
                0xE0 | 0xE4 | 0xEC => {
                    self.compare(&opcode.mode, self.register_x);
                }
                // CPY - Compare Y Register
                0xC0 | 0xC4 | 0xCC => {
                    self.compare(&opcode.mode, self.register_y);
                }
                // DEC - Decrement Memory
                0xC6 | 0xD6 | 0xCE | 0xDE => {
                    self.dec(&opcode.mode);
                }
                // DEX - Decrement X Register
                0xCA => {
                    self.dex();
                }
                // DEY - Decrement Y Register
                0x88 => {
                    self.dey();
                }
                // EOR - Exclusive OR
                0x49 | 0x45 | 0x55 | 0x4D | 0x5D | 0x59 | 0x41 | 0x51 => {
                    self.eor(&opcode.mode);
                }
                // INC - Increment Memory
                0xE6 | 0xF6 | 0xEE | 0xFE => {
                    self.inc(&opcode.mode);
                }
                // INX - Increment X Register
                0xE8 => {
                    self.inx();
                }
                // INY - Increment Y Register
                0xC8 => {
                    self.iny();
                }
                // JMP - Jump
                // Absolute
                0x4C => {
                    let mem_address = self.mem_read_u16(self.program_counter);
                    self.program_counter = mem_address;
                }
                // Indirect
                0x6C => {
                    let mem_address = self.mem_read_u16(self.program_counter);
                    let indirect_ref = if (mem_address & 0x00FF) == 0x00FF {
                        let lo = self.mem_read(mem_address);
                        let hi = self.mem_read(mem_address & 0x00FF);
                        (hi as u16) << 8 | (lo as u16)
                    } else {
                        self.mem_read_u16(mem_address)
                    };
                    self.program_counter = indirect_ref;
                }
                // JSR - Jump to Subroutine
                0x20 => {
                    self.jsr();
                }
                // LDA - Load Accumulator
                0xA9 | 0xA5 | 0xB5 | 0xAD | 0xBD | 0xB9 | 0xA1 | 0xB1 => {
                    self.lda(&opcode.mode);
                }
                // LDX - Load X Register
                0xA2 | 0xA6 | 0xB6 | 0xAE | 0xBE => {
                    self.ldx(&opcode.mode);
                }
                // LDY - Load Y Register
                0xA0 | 0xA4 | 0xB4 | 0xAC | 0xBC => {
                    self.ldy(&opcode.mode);
                }
                // LSR - Logical Shift Right
                0x4A => {
                    self.lsr_accumulator();
                }
                0x46 | 0x56 | 0x4E | 0x5E => {
                    self.lsr(&opcode.mode);
                }
                // NOP - No Operation
                0xEA => {
                    // Do nothing
                }
                // ORA - Logical Inclusive OR
                0x09 | 0x05 | 0x15 | 0x0D | 0x1D | 0x19 | 0x01 | 0x11 => {
                    self.ora(&opcode.mode);
                }
                // PHA - Push Accumulator
                0x48 => {
                    self.pha();
                }
                // PHP - Push Processor Status
                0x08 => {
                    self.php();
                }
                // PLA - Pull Accumulator
                0x68 => {
                    self.pla();
                }
                // PLP - Pull Processor Status
                0x28 => {
                    self.plp();
                }
                // ROL - Rotate Left
                0x2A => {
                    self.rol_accumulator();
                }
                0x26 | 0x36 | 0x2E | 0x3E => {
                    self.rol(&opcode.mode);
                }
                // ROR - Rotate Right
                0x6A => {
                    self.ror_accumulator();
                }
                0x66 | 0x76 | 0x6E | 0x7E => {
                    self.ror(&opcode.mode);
                }
                // RTI - Return from Interrupt
                0x40 => {
                    self.rti();
                }
                // RTS - Return from Subroutine
                0x60 => {
                    self.rts();
                }
                // SBC - Subtract with Carry
                0xE9 | 0xE5 | 0xF5 | 0xED | 0xFD | 0xF9 | 0xE1 | 0xF1 => {
                    self.sbc(&opcode.mode);
                }
                // SEC - Set Carry Flag
                0x38 => {
                    self.sec();
                }
                // SED - Set Decimal Flag
                0xF8 => {
                    self.sed();
                }
                // SEI - Set Interrupt Disable
                0x78 => {
                    self.sei();
                }
                // STA - Store Accumulator
                0x85 | 0x95 | 0x8D | 0x9D | 0x99 | 0x81 | 0x91 => {
                    self.sta(&opcode.mode);
                }
                // STX - Store X Register
                0x86 | 0x96 | 0x8E => {
                    self.stx(&opcode.mode);
                }
                // STY - Store Y Register
                0x84 | 0x94 | 0x8C => {
                    self.sty(&opcode.mode);
                }
                // TAX - Transfer Accumulator to X
                0xAA => {
                    self.tax();
                }
                // TAY - Transfer Accumulator to Y
                0xA8 => {
                    self.tay();
                }
                // TSX - Transfer Stack Pointer to X
                0xBA => {
                    self.tsx();
                }
                // TXA - Transfer X to Accumulator
                0x8A => {
                    self.txa();
                }
                // TXS - Transfer X to Stack Pointer
                0x9A => {
                    self.txs();
                }
                // TYA - Transfer Y to Accumulator
                0x98 => {
                    self.tya();
                }
                _ => todo!(),
            }

            if program_counter_state == self.program_counter {
                self.program_counter += (opcode.len - 1) as u16;
            }
        }
    }

    // Restore all register states and initialise program_counter
    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.register_y = 0;
        self.status = CPUFlags::from_bits_truncate(0b100100);
        self.program_counter = self.mem_read_u16(0xFFFC);
        self.stack_pointer = STACK_RESET;
    }

    // Load program into PRG ROM space and save reference to 0xFFFC
    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x0600..(0x0600 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, 0x0600);
    }

    // Load program and run
    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.register_a, 5);
        assert!(cpu.status.bits() & 0b0000_0010 == 0b00);
        assert!(cpu.status.bits() & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.status.bits() & 0b0000_0010 == 0b10);
    }

    #[test]
    fn test_0xa9_lda_negative_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xff, 0x00]);
        assert!(cpu.status.bits() & 0b1000_0000 == 0b1000_0000);
    }

    #[test]
    fn test_lda_from_memory() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0x55);
        cpu.load_and_run(vec![0xa5, 0x10, 0x00]);
        assert_eq!(cpu.register_a, 0x55);
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xaa, 0x00]);
        cpu.reset();
        cpu.register_a = 10;
        cpu.run();
        assert_eq!(cpu.register_x, 10)
    }

    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);
        assert_eq!(cpu.register_x, 0xc1)
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xe8, 0xe8, 0x00]);
        cpu.reset();
        cpu.register_x = 0xff;
        cpu.run();
        assert_eq!(cpu.register_x, 1)
    }
}

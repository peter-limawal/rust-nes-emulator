use crate::cpu::AddressingMode;
use std::collections::HashMap;

// Declare OpCode struct
pub struct OpCode {
    pub code: u8,
    pub mnemonic: &'static str,
    pub len: u8,
    pub cycles: u8,
    pub mode: AddressingMode,
}

// Implement functionality of OpCode
impl OpCode {
    // Create new OpCode object
    fn new(code: u8, mnemonic: &'static str, len: u8, cycles: u8, mode: AddressingMode) -> Self {
        OpCode {
            code: code,
            mnemonic: mnemonic,
            len: len,
            cycles: cycles,
            mode: mode,
        }
    }
}

// Macro for OpCode addressing modes
lazy_static! {
    pub static ref CPU_OPCODES: Vec<OpCode> = vec![
        // Refer to http://www.6502.org/tutorials/6502opcodes.html for more details

        // ADC (ADd with Carry)
        // Affects Flags: N V Z C
        // MODE           SYNTAX       HEX LEN TIM
        // Immediate     ADC #$44      $69  2   2
        OpCode::new(0x69, "ADC", 2, 2, AddressingMode::Immediate),
        // Zero Page     ADC $44       $65  2   3
        OpCode::new(0x65, "ADC", 2, 3, AddressingMode::ZeroPage),
        // Zero Page,X   ADC $44,X     $75  2   4
        OpCode::new(0x75, "ADC", 2, 4, AddressingMode::ZeroPage_X),
        // Absolute      ADC $4400     $6D  3   4
        OpCode::new(0x6d, "ADC", 3, 4, AddressingMode::Absolute),
        // Absolute,X    ADC $4400,X   $7D  3   4+
        OpCode::new(0x7d, "ADC", 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_X),
        // Absolute,Y    ADC $4400,Y   $79  3   4+
        OpCode::new(0x79, "ADC", 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_Y),
        // Indirect,X    ADC ($44,X)   $61  2   6
        OpCode::new(0x61, "ADC", 2, 6, AddressingMode::Indirect_X),
        // Indirect,Y    ADC ($44),Y   $71  2   5+
        OpCode::new(0x71, "ADC", 2, 5/*+1 if page crossed*/, AddressingMode::Indirect_Y),

        // AND (bitwise AND with accumulator)
        // Affects Flags: N Z
        // MODE           SYNTAX       HEX LEN TIM
        // Immediate     AND #$44      $29  2   2
        OpCode::new(0x29, "AND", 2, 2, AddressingMode::Immediate),
        // Zero Page     AND $44       $25  2   3
        OpCode::new(0x25, "AND", 2, 3, AddressingMode::ZeroPage),
        // Zero Page,X   AND $44,X     $35  2   4
        OpCode::new(0x35, "AND", 2, 4, AddressingMode::ZeroPage_X),
        // Absolute      AND $4400     $2D  3   4
        OpCode::new(0x2d, "AND", 3, 4, AddressingMode::Absolute),
        // Absolute,X    AND $4400,X   $3D  3   4+
        OpCode::new(0x3d, "AND", 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_X),
        // Absolute,Y    AND $4400,Y   $39  3   4+
        OpCode::new(0x39, "AND", 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_Y),
        // Indirect,X    AND ($44,X)   $21  2   6
        OpCode::new(0x21, "AND", 2, 6, AddressingMode::Indirect_X),
        // Indirect,Y    AND ($44),Y   $31  2   5+
        OpCode::new(0x31, "AND", 2, 5/*+1 if page crossed*/, AddressingMode::Indirect_Y),

        // ASL (Arithmetic Shift Left)
        // Affects Flags: N Z C
        // MODE           SYNTAX       HEX LEN TIM
        // Accumulator   ASL A         $0A  1   2
        OpCode::new(0x0a, "ASL", 1, 2, AddressingMode::NoneAddressing),
        // Zero Page     ASL $44       $06  2   5
        OpCode::new(0x06, "ASL", 2, 5, AddressingMode::ZeroPage),
        // Zero Page,X   ASL $44,X     $16  2   6
        OpCode::new(0x16, "ASL", 2, 6, AddressingMode::ZeroPage_X),
        // Absolute      ASL $4400     $0E  3   6
        OpCode::new(0x0e, "ASL", 3, 6, AddressingMode::Absolute),
        // Absolute,X    ASL $4400,X   $1E  3   7
        OpCode::new(0x1e, "ASL", 3, 7, AddressingMode::Absolute_X),

        // BIT (test BITs)
        // Affects Flags: N V Z
        // MODE           SYNTAX       HEX LEN TIM
        // Zero Page     BIT $44       $24  2   3
        OpCode::new(0x24, "BIT", 2, 3, AddressingMode::ZeroPage),
        // Absolute      BIT $4400     $2C  3   4
        OpCode::new(0x2c, "BIT", 3, 4, AddressingMode::Absolute),

        // Branch Instructions
        // Affects Flags: none
        // MNEMONIC                       HEX
        // BPL (Branch on PLus)           $10
        OpCode::new(0x10, "BPL", 2, 2 /*(+1 if branch succeeds +2 if to a new page)*/, AddressingMode::NoneAddressing),
        // BMI (Branch on MInus)          $30
        OpCode::new(0x30, "BMI", 2, 2 /*(+1 if branch succeeds +2 if to a new page)*/, AddressingMode::NoneAddressing),
        // BVC (Branch on oVerflow Clear) $50
        OpCode::new(0x50, "BVC", 2, 2 /*(+1 if branch succeeds +2 if to a new page)*/, AddressingMode::NoneAddressing),
        // BVS (Branch on oVerflow Set)   $70
        OpCode::new(0x70, "BVS", 2, 2 /*(+1 if branch succeeds +2 if to a new page)*/, AddressingMode::NoneAddressing),
        // BCC (Branch on Carry Clear)    $90
        OpCode::new(0x90, "BCC", 2, 2 /*(+1 if branch succeeds +2 if to a new page)*/, AddressingMode::NoneAddressing),
        // BCS (Branch on Carry Set)      $B0
        OpCode::new(0xb0, "BCS", 2, 2 /*(+1 if branch succeeds +2 if to a new page)*/, AddressingMode::NoneAddressing),
        // BNE (Branch on Not Equal)      $D0
        OpCode::new(0xd0, "BNE", 2, 2 /*(+1 if branch succeeds +2 if to a new page)*/, AddressingMode::NoneAddressing),
        // BEQ (Branch on EQual)          $F0
        OpCode::new(0xf0, "BEQ", 2, 2 /*(+1 if branch succeeds +2 if to a new page)*/, AddressingMode::NoneAddressing),

        // BRK (BReaK)
        // Affects Flags: B
        // MODE           SYNTAX       HEX LEN TIM
        // Implied       BRK           $00  1   7
        OpCode::new(0x00, "BRK", 1, 7, AddressingMode::NoneAddressing),

        // CMP (CoMPare accumulator)
        // Affects Flags: N Z C
        // MODE           SYNTAX       HEX LEN TIM
        // Immediate     CMP #$44      $C9  2   2
        OpCode::new(0xc9, "CMP", 2, 2, AddressingMode::Immediate),
        // Zero Page     CMP $44       $C5  2   3
        OpCode::new(0xc5, "CMP", 2, 3, AddressingMode::ZeroPage),
        // Zero Page,X   CMP $44,X     $D5  2   4
        OpCode::new(0xd5, "CMP", 2, 4, AddressingMode::ZeroPage_X),
        // Absolute      CMP $4400     $CD  3   4
        OpCode::new(0xcd, "CMP", 3, 4, AddressingMode::Absolute),
        // Absolute,X    CMP $4400,X   $DD  3   4+
        OpCode::new(0xdd, "CMP", 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_X),
        // Absolute,Y    CMP $4400,Y   $D9  3   4+
        OpCode::new(0xd9, "CMP", 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_Y),
        // Indirect,X    CMP ($44,X)   $C1  2   6
        OpCode::new(0xc1, "CMP", 2, 6, AddressingMode::Indirect_X),
        // Indirect,Y    CMP ($44),Y   $D1  2   5+
        OpCode::new(0xd1, "CMP", 2, 5/*+1 if page crossed*/, AddressingMode::Indirect_Y),

        // CPX (ComPare X register)
        // Affects Flags: N Z C
        // MODE           SYNTAX       HEX LEN TIM
        // Immediate     CPX #$44      $E0  2   2
        OpCode::new(0xe0, "CPX", 2, 2, AddressingMode::Immediate),
        // Zero Page     CPX $44       $E4  2   3
        OpCode::new(0xe4, "CPX", 2, 3, AddressingMode::ZeroPage),
        // Absolute      CPX $4400     $EC  3   4
        OpCode::new(0xec, "CPX", 3, 4, AddressingMode::Absolute),

        // CPY (ComPare Y register)
        // Affects Flags: N Z C
        // MODE           SYNTAX       HEX LEN TIM
        // Immediate     CPY #$44      $C0  2   2
        OpCode::new(0xc0, "CPY", 2, 2, AddressingMode::Immediate),
        // Zero Page     CPY $44       $C4  2   3
        OpCode::new(0xc4, "CPY", 2, 3, AddressingMode::ZeroPage),
        // Absolute      CPY $4400     $CC  3   4
        OpCode::new(0xcc, "CPY", 3, 4, AddressingMode::Absolute),

        // DEC (DECrement memory)
        // Affects Flags: N Z
        // MODE           SYNTAX       HEX LEN TIM
        // Zero Page     DEC $44       $C6  2   5
        OpCode::new(0xc6, "DEC", 2, 5, AddressingMode::ZeroPage),
        // Zero Page,X   DEC $44,X     $D6  2   6
        OpCode::new(0xd6, "DEC", 2, 6, AddressingMode::ZeroPage_X),
        // Absolute      DEC $4400     $CE  3   6
        OpCode::new(0xce, "DEC", 3, 6, AddressingMode::Absolute),
        // Absolute,X    DEC $4400,X   $DE  3   7
        OpCode::new(0xde, "DEC", 3, 7, AddressingMode::Absolute_X),

        // EOR (bitwise Exclusive OR)
        // Affects Flags: N Z
        // MODE           SYNTAX       HEX LEN TIM
        // Immediate     EOR #$44      $49  2   2
        OpCode::new(0x49, "EOR", 2, 2, AddressingMode::Immediate),
        // Zero Page     EOR $44       $45  2   3
        OpCode::new(0x45, "EOR", 2, 3, AddressingMode::ZeroPage),
        // Zero Page,X   EOR $44,X     $55  2   4
        OpCode::new(0x55, "EOR", 2, 4, AddressingMode::ZeroPage_X),
        // Absolute      EOR $4400     $4D  3   4
        OpCode::new(0x4d, "EOR", 3, 4, AddressingMode::Absolute),
        // Absolute,X    EOR $4400,X   $5D  3   4+
        OpCode::new(0x5d, "EOR", 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_X),
        // Absolute,Y    EOR $4400,Y   $59  3   4+
        OpCode::new(0x59, "EOR", 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_Y),
        // Indirect,X    EOR ($44,X)   $41  2   6
        OpCode::new(0x41, "EOR", 2, 6, AddressingMode::Indirect_X),
        // Indirect,Y    EOR ($44),Y   $51  2   5+
        OpCode::new(0x51, "EOR", 2, 5/*+1 if page crossed*/, AddressingMode::Indirect_Y),

        // Flag (Processor Status) Instructions
        // Affect Flags: as noted
        // MNEMONIC                       HEX
        // CLC (CLear Carry)              $18
        OpCode::new(0x18, "CLC", 1, 2, AddressingMode::NoneAddressing),
        // SEC (SEt Carry)                $38
        OpCode::new(0x38, "SEC", 1, 2, AddressingMode::NoneAddressing),
        // CLI (CLear Interrupt)          $58
        OpCode::new(0x58, "CLI", 1, 2, AddressingMode::NoneAddressing),
        // SEI (SEt Interrupt)            $78
        OpCode::new(0x78, "SEI", 1, 2, AddressingMode::NoneAddressing),
        // CLV (CLear oVerflow)           $B8
        OpCode::new(0xb8, "CLV", 1, 2, AddressingMode::NoneAddressing),
        // CLD (CLear Decimal)            $D8
        OpCode::new(0xD8, "CLD", 1, 2, AddressingMode::NoneAddressing),
        // SED (SEt Decimal)              $F8
        OpCode::new(0xf8, "SED", 1, 2, AddressingMode::NoneAddressing),

        // INC (INCrement memory)
        // Affects Flags: N Z
        // MODE           SYNTAX       HEX LEN TIM
        // Zero Page     INC $44       $E6  2   5
        OpCode::new(0xe6, "INC", 2, 5, AddressingMode::ZeroPage),
        // Zero Page,X   INC $44,X     $F6  2   6
        OpCode::new(0xf6, "INC", 2, 6, AddressingMode::ZeroPage_X),
        // Absolute      INC $4400     $EE  3   6
        OpCode::new(0xee, "INC", 3, 6, AddressingMode::Absolute),
        // Absolute,X    INC $4400,X   $FE  3   7
        OpCode::new(0xfe, "INC", 3, 7, AddressingMode::Absolute_X),

        // JMP (JuMP)
        // Affects Flags: none
        // MODE           SYNTAX       HEX LEN TIM
        // Absolute      JMP $5597     $4C  3   3
        OpCode::new(0x4c, "JMP", 3, 3, AddressingMode::NoneAddressing), //AddressingMode that acts as Immediate
        // Indirect      JMP ($5597)   $6C  3   5
        OpCode::new(0x6c, "JMP", 3, 5, AddressingMode::NoneAddressing), //AddressingMode:Indirect with 6502 bug

        // JSR (Jump to SubRoutine)
        // Affects Flags: none
        // MODE           SYNTAX       HEX LEN TIM
        // Absolute      JSR $5597     $20  3   6
        OpCode::new(0x20, "JSR", 3, 6, AddressingMode::NoneAddressing),

        // LDA (LoaD Accumulator)
        // Affects Flags: N Z
        // MODE           SYNTAX       HEX LEN TIM
        // Immediate     LDA #$44      $A9  2   2
        OpCode::new(0xa9, "LDA", 2, 2, AddressingMode::Immediate),
        // Zero Page     LDA $44       $A5  2   3
        OpCode::new(0xa5, "LDA", 2, 3, AddressingMode::ZeroPage),
        // Zero Page,X   LDA $44,X     $B5  2   4
        OpCode::new(0xb5, "LDA", 2, 4, AddressingMode::ZeroPage_X),
        // Absolute      LDA $4400     $AD  3   4
        OpCode::new(0xad, "LDA", 3, 4, AddressingMode::Absolute),
        // Absolute,X    LDA $4400,X   $BD  3   4+
        OpCode::new(0xbd, "LDA", 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_X),
        // Absolute,Y    LDA $4400,Y   $B9  3   4+
        OpCode::new(0xb9, "LDA", 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_Y),
        // Indirect,X    LDA ($44,X)   $A1  2   6
        OpCode::new(0xa1, "LDA", 2, 6, AddressingMode::Indirect_X),
        // Indirect,Y    LDA ($44),Y   $B1  2   5+
        OpCode::new(0xb1, "LDA", 2, 5/*+1 if page crossed*/, AddressingMode::Indirect_Y),

        // LDX (LoaD X register)
        // Affects Flags: N Z
        // MODE           SYNTAX       HEX LEN TIM
        // Immediate     LDX #$44      $A2  2   2
        OpCode::new(0xa2, "LDX", 2, 2, AddressingMode::Immediate),
        // Zero Page     LDX $44       $A6  2   3
        OpCode::new(0xa6, "LDX", 2, 3, AddressingMode::ZeroPage),
        // Zero Page,Y   LDX $44,Y     $B6  2   4
        OpCode::new(0xb6, "LDX", 2, 4, AddressingMode::ZeroPage_Y),
        // Absolute      LDX $4400     $AE  3   4
        OpCode::new(0xae, "LDX", 3, 4, AddressingMode::Absolute),
        // Absolute,Y    LDX $4400,Y   $BE  3   4+
        OpCode::new(0xbe, "LDX", 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_Y),

        // LDY (LoaD Y register)
        // Affects Flags: N Z
        // MODE           SYNTAX       HEX LEN TIM
        // Immediate     LDY #$44      $A0  2   2
        OpCode::new(0xa0, "LDY", 2, 2, AddressingMode::Immediate),
        // Zero Page     LDY $44       $A4  2   3
        OpCode::new(0xa4, "LDY", 2, 3, AddressingMode::ZeroPage),
        // Zero Page,X   LDY $44,X     $B4  2   4
        OpCode::new(0xb4, "LDY", 2, 4, AddressingMode::ZeroPage_X),
        // Absolute      LDY $4400     $AC  3   4
        OpCode::new(0xac, "LDY", 3, 4, AddressingMode::Absolute),
        // Absolute,X    LDY $4400,X   $BC  3   4+
        OpCode::new(0xbc, "LDY", 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_X),

        // LSR (Logical Shift Right)
        // Affects Flags: N Z C
        // MODE           SYNTAX       HEX LEN TIM
        // Accumulator   LSR A         $4A  1   2
        OpCode::new(0x4a, "LSR", 1, 2, AddressingMode::NoneAddressing),
        // Zero Page     LSR $44       $46  2   5
        OpCode::new(0x46, "LSR", 2, 5, AddressingMode::ZeroPage),
        // Zero Page,X   LSR $44,X     $56  2   6
        OpCode::new(0x56, "LSR", 2, 6, AddressingMode::ZeroPage_X),
        // Absolute      LSR $4400     $4E  3   6
        OpCode::new(0x4e, "LSR", 3, 6, AddressingMode::Absolute),
        // Absolute,X    LSR $4400,X   $5E  3   7
        OpCode::new(0x5e, "LSR", 3, 7, AddressingMode::Absolute_X),

        // NOP (No OPeration)
        // Affects Flags: none
        // MODE           SYNTAX       HEX LEN TIM
        // Implied       NOP           $EA  1   2
        OpCode::new(0xea, "NOP", 1, 2, AddressingMode::NoneAddressing),

        // ORA (bitwise OR with Accumulator)
        // Affects Flags: N Z
        // MODE           SYNTAX       HEX LEN TIM
        // Immediate     ORA #$44      $09  2   2
        OpCode::new(0x09, "ORA", 2, 2, AddressingMode::Immediate),
        // Zero Page     ORA $44       $05  2   3
        OpCode::new(0x05, "ORA", 2, 3, AddressingMode::ZeroPage),
        // Zero Page,X   ORA $44,X     $15  2   4
        OpCode::new(0x15, "ORA", 2, 4, AddressingMode::ZeroPage_X),
        // Absolute      ORA $4400     $0D  3   4
        OpCode::new(0x0d, "ORA", 3, 4, AddressingMode::Absolute),
        // Absolute,X    ORA $4400,X   $1D  3   4+
        OpCode::new(0x1d, "ORA", 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_X),
        // Absolute,Y    ORA $4400,Y   $19  3   4+
        OpCode::new(0x19, "ORA", 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_Y),
        // Indirect,X    ORA ($44,X)   $01  2   6
        OpCode::new(0x01, "ORA", 2, 6, AddressingMode::Indirect_X),
        // Indirect,Y    ORA ($44),Y   $11  2   5+
        OpCode::new(0x11, "ORA", 2, 5/*+1 if page crossed*/, AddressingMode::Indirect_Y),

        // Register Instructions
        // Affect Flags: N Z
        // MNEMONIC                 HEX
        // TAX (Transfer A to X)    $AA
        OpCode::new(0xaa, "TAX", 1, 2, AddressingMode::NoneAddressing),
        // TXA (Transfer X to A)    $8A
        OpCode::new(0x8a, "TXA", 1, 2, AddressingMode::NoneAddressing),
        // DEX (DEcrement X)        $CA
        OpCode::new(0xca, "DEX", 1, 2, AddressingMode::NoneAddressing),
        // INX (INcrement X)        $E8
        OpCode::new(0xe8, "INX", 1, 2, AddressingMode::NoneAddressing),
        // TAY (Transfer A to Y)    $A8
        OpCode::new(0xa8, "TAY", 1, 2, AddressingMode::NoneAddressing),
        // TYA (Transfer Y to A)    $98
        OpCode::new(0x98, "TYA", 1, 2, AddressingMode::NoneAddressing),
        // DEY (DEcrement Y)        $88
        OpCode::new(0x88, "DEY", 1, 2, AddressingMode::NoneAddressing),
        // INY (INcrement Y)        $C8
        OpCode::new(0xc8, "INY", 1, 2, AddressingMode::NoneAddressing),

        // ROL (ROtate Left)
        // Affects Flags: N Z C
        // MODE           SYNTAX       HEX LEN TIM
        // Accumulator   ROL A         $2A  1   2
        OpCode::new(0x2a, "ROL", 1, 2, AddressingMode::NoneAddressing),
        // Zero Page     ROL $44       $26  2   5
        OpCode::new(0x26, "ROL", 2, 5, AddressingMode::ZeroPage),
        // Zero Page,X   ROL $44,X     $36  2   6
        OpCode::new(0x36, "ROL", 2, 6, AddressingMode::ZeroPage_X),
        // Absolute      ROL $4400     $2E  3   6
        OpCode::new(0x2e, "ROL", 3, 6, AddressingMode::Absolute),
        // Absolute,X    ROL $4400,X   $3E  3   7
        OpCode::new(0x3e, "ROL", 3, 7, AddressingMode::Absolute_X),

        // ROR (ROtate Right)
        // Affects Flags: N Z C
        // MODE           SYNTAX       HEX LEN TIM
        // Accumulator   ROR A         $6A  1   2
        OpCode::new(0x6a, "ROR", 1, 2, AddressingMode::NoneAddressing),
        // Zero Page     ROR $44       $66  2   5
        OpCode::new(0x66, "ROR", 2, 5, AddressingMode::ZeroPage),
        // Zero Page,X   ROR $44,X     $76  2   6
        OpCode::new(0x76, "ROR", 2, 6, AddressingMode::ZeroPage_X),
        // Absolute      ROR $4400     $6E  3   6
        OpCode::new(0x6e, "ROR", 3, 6, AddressingMode::Absolute),
        // Absolute,X    ROR $4400,X   $7E  3   7
        OpCode::new(0x7e, "ROR", 3, 7, AddressingMode::Absolute_X),

        // RTI (ReTurn from Interrupt)
        // Affects Flags: all
        // MODE           SYNTAX       HEX LEN TIM
        // Implied       RTI           $40  1   6
        OpCode::new(0x40, "RTI", 1, 6, AddressingMode::NoneAddressing),

        // RTS (ReTurn from Subroutine)
        // Affects Flags: none
        // MODE           SYNTAX       HEX LEN TIM
        // Implied       RTS           $60  1   6
        OpCode::new(0x60, "RTS", 1, 6, AddressingMode::NoneAddressing),

        // SBC (SuBtract with Carry)
        // Affects Flags: N V Z C
        // MODE           SYNTAX       HEX LEN TIM
        // Immediate     SBC #$44      $E9  2   2
        OpCode::new(0xe9, "SBC", 2, 2, AddressingMode::Immediate),
        // Zero Page     SBC $44       $E5  2   3
        OpCode::new(0xe5, "SBC", 2, 3, AddressingMode::ZeroPage),
        // Zero Page,X   SBC $44,X     $F5  2   4
        OpCode::new(0xf5, "SBC", 2, 4, AddressingMode::ZeroPage_X),
        // Absolute      SBC $4400     $ED  3   4
        OpCode::new(0xed, "SBC", 3, 4, AddressingMode::Absolute),
        // Absolute,X    SBC $4400,X   $FD  3   4+
        OpCode::new(0xfd, "SBC", 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_X),
        // Absolute,Y    SBC $4400,Y   $F9  3   4+
        OpCode::new(0xf9, "SBC", 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_Y),
        // Indirect,X    SBC ($44,X)   $E1  2   6
        OpCode::new(0xe1, "SBC", 2, 6, AddressingMode::Indirect_X),
        // Indirect,Y    SBC ($44),Y   $F1  2   5+
        OpCode::new(0xf1, "SBC", 2, 5/*+1 if page crossed*/, AddressingMode::Indirect_Y),

        // STA (STore Accumulator)
        // Affects Flags: none
        // MODE           SYNTAX       HEX LEN TIM
        // Zero Page     STA $44       $85  2   3
        OpCode::new(0x85, "STA", 2, 3, AddressingMode::ZeroPage),
        // Zero Page,X   STA $44,X     $95  2   4
        OpCode::new(0x95, "STA", 2, 4, AddressingMode::ZeroPage_X),
        // Absolute      STA $4400     $8D  3   4
        OpCode::new(0x8d, "STA", 3, 4, AddressingMode::Absolute),
        // Absolute,X    STA $4400,X   $9D  3   5
        OpCode::new(0x9d, "STA", 3, 5, AddressingMode::Absolute_X),
        // Absolute,Y    STA $4400,Y   $99  3   5
        OpCode::new(0x99, "STA", 3, 5, AddressingMode::Absolute_Y),
        // Indirect,X    STA ($44,X)   $81  2   6
        OpCode::new(0x81, "STA", 2, 6, AddressingMode::Indirect_X),
        // Indirect,Y    STA ($44),Y   $91  2   6
        OpCode::new(0x91, "STA", 2, 6, AddressingMode::Indirect_Y),

        // Stack Instructions
        // MNEMONIC                        HEX TIM
        // TXS (Transfer X to Stack ptr)   $9A  2
        OpCode::new(0x9a, "TXS", 1, 2, AddressingMode::NoneAddressing),
        // TSX (Transfer Stack ptr to X)   $BA  2
        OpCode::new(0xba, "TSX", 1, 2, AddressingMode::NoneAddressing),
        // PHA (PusH Accumulator)          $48  3
        OpCode::new(0x48, "PHA", 1, 3, AddressingMode::NoneAddressing),
        // PLA (PuLl Accumulator)          $68  4
        OpCode::new(0x68, "PLA", 1, 4, AddressingMode::NoneAddressing),
        // PHP (PusH Processor status)     $08  3
        OpCode::new(0x08, "PHP", 1, 3, AddressingMode::NoneAddressing),
        // PLP (PuLl Processor status)     $28  4
        OpCode::new(0x28, "PLP", 1, 4, AddressingMode::NoneAddressing),

        // STX (STore X register)
        // Affects Flags: none
        // MODE           SYNTAX       HEX LEN TIM
        // Zero Page     STX $44       $86  2   3
        OpCode::new(0x86, "STX", 2, 3, AddressingMode::ZeroPage),
        // Zero Page,Y   STX $44,Y     $96  2   4
        OpCode::new(0x96, "STX", 2, 4, AddressingMode::ZeroPage_Y),
        // Absolute      STX $4400     $8E  3   4
        OpCode::new(0x8e, "STX", 3, 4, AddressingMode::Absolute),

        // STY (STore Y register)
        // Affects Flags: none
        // MODE           SYNTAX       HEX LEN TIM
        // Zero Page     STY $44       $84  2   3
        OpCode::new(0x84, "STY", 2, 3, AddressingMode::ZeroPage),
        // Zero Page,X   STY $44,X     $94  2   4
        OpCode::new(0x94, "STY", 2, 4, AddressingMode::ZeroPage_X),
        // Absolute      STY $4400     $8C  3   4
        OpCode::new(0x8c, "STY", 3, 4, AddressingMode::Absolute),
    ];

    // Insert OpCodes into HashMap
    pub static ref OPCODES_MAP: HashMap<u8, &'static OpCode> = {
        let mut map = HashMap::new();
        for cpuop in &*CPU_OPCODES {
            map.insert(cpuop.code, cpuop);
        }
        map
    };
}

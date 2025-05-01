        // Arithmetic instructions
        ADD X0, X1, X2        // X0 = X1 + X2
        ADDI X3, X4, #42      // X3 = X4 + 42
        SUB X5, X6, X7        // X5 = X6 - X7
        SUBI X8, X9, #10      // X8 = X9 - 10
        SUBIS X10, X11, #5    // X10 = X11 - 5 (set flags)
        SUBS X12, X13, X14    // X12 = X13 - X14 (set flags)
        MUL X15, X16, X17     // X15 = X16 * X17

        // Logical instructions
        AND X18, X19, X20     // X18 = X19 & X20
        ANDI X21, X22, #0xFF  // X21 = X22 & 0xFF
        ORR X23, X24, X25     // X23 = X24 | X25
        ORRI X26, X27, #0x1   // X26 = X27 | 0x1
        EOR X28, X29, X30     // X28 = X29 ^ X30
        EORI X0, X1, #0xAA    // X0 = X1 ^ 0xAA

        // Shift instructions
        LSL X2, X3, #4        // X2 = X3 << 4
        LSR X4, X5, #2        // X4 = X5 >> 2

        // Branch instructions
        B label1              // Unconditional branch to label1
        BL label2             // Branch with link to label2
        BR X6                 // Branch to address in X6

        // Conditional branches (all conditions)
        B.EQ label1           // Branch if equal
        B.NE label2           // Branch if not equal
        B.HS label1           // Branch if higher or same (unsigned)
        B.LO label2           // Branch if lower (unsigned)
        B.MI label1           // Branch if negative
        B.PL label2           // Branch if positive or zero
        B.VS label1           // Branch if overflow
        B.VC label2           // Branch if no overflow
        B.HI label1           // Branch if higher (unsigned)
        B.LS label2           // Branch if lower or same (unsigned)
        B.GE label1           // Branch if greater or equal (signed)
        B.LT label2           // Branch if less than (signed)
        B.GT label1           // Branch if greater than (signed)
        B.LE label2           // Branch if less or equal (signed)
        B label1           // Branch always (unconditional)

        // Compare and branch
        CBZ X7, label1        // Branch if X7 is zero
        CBNZ X8, label2       // Branch if X8 is not zero

        // Memory operations
        LDUR X9, [X10, #0]    // Load from memory (X10 + 0) into X9
        STUR X11, [X12, #8]   // Store X11 to memory (X12 + 8)

        // Special emulator instructions
        PRNT X13              // Print X13 contents
        PRNL                 // Print newline
        DUMP                 // Dump all registers and memory
        HALT                 // Halt execution and dump state

// Labels for branching
label1:
        B label2              // Branch to label2

label2:
        B label1              // Branch back to label1
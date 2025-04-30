        ADD X0, X1, X2        // X0 = X1 + X2
        ADDI X3, X4, #42      // X3 = X4 + 42
        B label               // Unconditional branch to "label"

label:
        SUB X5, X6, X7        // X5 = X6 - X7
        B.EQ label            // If the zero flag is set (result of last arithmetic op was 0), branch to "label" again

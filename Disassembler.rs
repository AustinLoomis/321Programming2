/*
Authors:
Austin Loomis   aploomis@iastate.edu
Noah Smith      smithnw@iastate.edu
Camden Klicker  camklic@iastate.edu
*/

use std::{ collections::HashMap, env, fs, process, convert::TryInto };

fn main() {
    let args: Vec<String> = env::args().collect(); // command line arguments
    if args.len() != 2 {
        eprintln!("Usage: {} <binary_file>", args[0]);
        process::exit(1);
    }

    let bytes = fs::read(&args[1]).unwrap_or_else(|err| {
        eprintln!("Error reading file: {}", err);
        process::exit(1);
    });

    let instructions = parse_instructions(&bytes);
    let labels = scan_labels(&instructions);
    disassemble(&instructions, &labels);
}

fn parse_instructions(bytes: &[u8]) -> Vec<u32> {
    bytes
        .chunks_exact(4)
        .map(|chunk| u32::from_be_bytes(chunk.try_into().unwrap()))
        .collect()
}

fn scan_labels(instructions: &[u32]) -> HashMap<u32, String> {
    let mut labels = HashMap::new();
    let mut pc = 0;
    let mut label_count = 0; // Track label count separately

    for &inst in instructions {
        let opcode = (inst >> 21) & 0x7ff;
        match opcode {
            0b000101 => {
                // B
                let raw = inst & 0x03ff_ffff;
                let offset = sign_extend_imm26(raw);
                let target = (pc as i32).wrapping_add(offset) as u32;
                if !labels.contains_key(&target) {
                    labels.insert(target, format!("label_{}", label_count));
                    label_count += 1;
                }
            }
            0b100101 => {
                // BL
                let raw = inst & 0x03ff_ffff;
                let offset = sign_extend_imm26(raw);
                let target = (pc as i32).wrapping_add(offset) as u32;
                if !labels.contains_key(&target) {
                    labels.insert(target, format!("label_{}", label_count));
                    label_count += 1;
                }
            }
            0b01010100..=0b01010111 => {
                // B.cond
                let offset = ((inst >> 5) & 0x7ffff) as i32;
                let target = ((pc as i32) + (offset << 2)) as u32;
                if !labels.contains_key(&target) {
                    labels.insert(target, format!("label_{}", label_count));
                    label_count += 1;
                }
            }
            _ => {}
        }
        pc += 4;
    }
    labels
}

// Sign-extend a raw n-bit immediate, then shift left by 2.
fn sign_extend_imm26(raw: u32) -> i32 {
    // raw is bits[25:0] of the instruction
    let mut val = (raw & 0x03ff_ffff) as i32;
    // if bit25 is set, fill the top 6 bits with 1s
    if (val & (1 << 25)) != 0 {
        val |= !0 << 26;
    }
    // now shift to get a byte offset
    val << 2
}

fn disassemble(instructions: &[u32], labels: &HashMap<u32, String>) {
    let mut pc = 0u32;

    for &inst in instructions {
        // Print any label at this address
        if let Some(label) = labels.get(&pc) {
            println!("{}:", label);
        }

        // opcode fields
        let op26 = inst >> 26; // bits [31:26]
        let op25_31 = (inst >> 25) & 0x7f; // bits [31:25]
        let op24_31 = (inst >> 24) & 0xff; // bits [31:24]

        // Helper for sign-extending imm19 and computing byte offset
        let sign_extend_imm19 = |raw: u32| {
            let mut val = (raw & 0x7ffff) as i32;
            if (val & (1 << 18)) != 0 {
                val |= !0 << 19;
            }
            (val << 2) as u32
        };

        match op26 {
            // Unconditional branch B: opcode 0b000101
            0b000101 => {
                let raw = inst & 0x03ff_ffff;
                let offset = sign_extend_imm26(raw) as u32;
                let target = pc.wrapping_add(offset);
                println!(
                    "    B {}",
                    labels.get(&target).unwrap_or(&format!("label_{}", target / 4))
                );
            }
            0b100101 => {
                let raw = inst & 0x03ff_ffff;
                let offset = sign_extend_imm26(raw) as u32;
                let target = pc.wrapping_add(offset);
                println!(
                    "    BL {}",
                    labels.get(&target).unwrap_or(&format!("label_{}", target / 4))
                );
            }

            // Handle CB-type instructions (8-bit opcode starting at bit 24)
            0b101101 => {
                match op24_31 {
                    0b10110100 => {
                        // CBZ
                        let rt = (inst >> 16) & 0x1f;
                        let offset = (((inst >> 5) & 0x7ffff) as i32) << 2;
                        let target = pc.wrapping_add(offset as u32);
                        println!("    CBZ X{}, label_{}", rt, target / 4);
                    }
                    0b10110101 => {
                        // CBNZ
                        let rt = (inst >> 16) & 0x1f;
                        let offset = (((inst >> 5) & 0x7ffff) as i32) << 2;
                        let target = pc.wrapping_add(offset as u32);
                        println!("    CBNZ X{}, label_{}", rt, target / 4);
                    }
                    _ => println!("    [UNKNOWN CB-TYPE: {:032b}]", inst),
                }
            }

            _ =>
                match op25_31 {
                    // CBZ:  0b10110100
                    // CBNZ: 0b10110101
                    0b0110100 => {
                        let bit24 = (inst >> 24) & 1;
                        let imm19 = (inst >> 5) & 0x7ffff;
                        let offset = sign_extend_imm19(imm19);
                        let target = pc.wrapping_add(offset);
                        let rt = inst & 0x1f;

                        if bit24 == 0 {
                            println!(
                                "    CBZ X{}, {}",
                                rt,
                                labels.get(&target).unwrap_or(&format!("label_{}", target / 4))
                            );
                        } else {
                            println!(
                                "    CBNZ X{}, {}",
                                rt,
                                labels.get(&target).unwrap_or(&format!("label_{}", target / 4))
                            );
                        }
                    }

                    // Conditional branch B.cond: opcode 0b0101010
                    0b0101010 => {
                        let imm19 = (inst >> 5) & 0x7ffff;
                        let offset = sign_extend_imm19(imm19);
                        let target = pc.wrapping_add(offset);
                        let cond = inst & 0xf;
                        let cond_str = match cond {
                            0b0000 => "EQ",
                            0b0001 => "NE",
                            0b0010 => "HS",
                            0b0011 => "LO",
                            0b0100 => "MI",
                            0b0101 => "PL",
                            0b0110 => "VS",
                            0b0111 => "VC",
                            0b1000 => "HI",
                            0b1001 => "LS",
                            0b1010 => "GE",
                            0b1011 => "LT",
                            0b1100 => "GT",
                            0b1101 => "LE",
                            0b1110 => "AL",
                            _ => "??",
                        };

                        println!(
                            "    B.{} {}",
                            cond_str,
                            labels.get(&target).unwrap_or(&format!("label_{}", target / 4))
                        );
                    }

                    // Handle remaining instruction types
                    _ => {
                        // First check for I-type instructions (10-bit opcode starting at bit 22)
                        match (inst >> 22) & 0x3ff {
                            0b1001000100 => {
                                // ADDI
                                let imm = (inst >> 10) & 0xfff;
                                let rn = (inst >> 5) & 0x1f;
                                let rd = inst & 0x1f;
                                println!("    ADDI X{}, X{}, #{}", rd, rn, imm);
                            }
                            0b1001001000 => {
                                // ANDI
                                let imm = (inst >> 10) & 0xfff;
                                let rn = (inst >> 5) & 0x1f;
                                let rd = inst & 0x1f;
                                println!("    ANDI X{}, X{}, #{}", rd, rn, imm);
                            }
                            0b1101001000 => {
                                // EORI
                                let imm = (inst >> 10) & 0xfff;
                                let rn = (inst >> 5) & 0x1f;
                                let rd = inst & 0x1f;
                                println!("    EORI X{}, X{}, #{}", rd, rn, imm);
                            }
                            0b1011001000 => {
                                // ORRI
                                let imm = (inst >> 10) & 0xfff;
                                let rn = (inst >> 5) & 0x1f;
                                let rd = inst & 0x1f;
                                println!("    ORRI X{}, X{}, #{}", rd, rn, imm);
                            }
                            0b1101000100 => {
                                // SUBI
                                let imm = (inst >> 10) & 0xfff;
                                let rn = (inst >> 5) & 0x1f;
                                let rd = inst & 0x1f;
                                println!("    SUBI X{}, X{}, #{}", rd, rn, imm);
                            }
                            0b1111000100 => {
                                // SUBIS
                                let imm = (inst >> 10) & 0xfff;
                                let rn = (inst >> 5) & 0x1f;
                                let rd = inst & 0x1f;
                                println!("    SUBIS X{}, X{}, #{}", rd, rn, imm);
                            }
                            // If not I-type, check D-type and R-type (11-bit opcode starting at bit 21)
                            _ =>
                                match (inst >> 21) & 0x7ff {
                                    0b11111000010 => {
                                        // LDUR
                                        let dt_addr = (inst >> 12) & 0x1f;
                                        let _op = (inst >> 10) & 0x1f;
                                        let rn = (inst >> 5) & 0x1f;
                                        let rt = inst & 0x1f;
                                        println!("    LDUR X{}, [X{}, #{}]", rt, rn, dt_addr);
                                    }
                                    0b11111000000 => {
                                        // STUR
                                        let dt_addr = (inst >> 12) & 0x1f;
                                        let _op = (inst >> 10) & 0x1f;
                                        let rn = (inst >> 5) & 0x1f;
                                        let rt = inst & 0x1f;
                                        println!("    STUR X{}, [X{}, #{}]", rt, rn, dt_addr);
                                    }
                                    0b10001011000 => {
                                        // ADD
                                        let rm = (inst >> 16) & 0x1f;
                                        let rn = (inst >> 5) & 0x1f;
                                        let rd = inst & 0x1f;
                                        println!("    ADD X{}, X{}, X{}", rd, rn, rm);
                                    }
                                    0b10001010000 => {
                                        // AND
                                        let rm = (inst >> 16) & 0x1f;
                                        let rn = (inst >> 5) & 0x1f;
                                        let rd = inst & 0x1f;
                                        println!("    AND X{}, X{}, X{}", rd, rn, rm);
                                    }
                                    0b11001010000 => {
                                        // EOR
                                        let rm = (inst >> 16) & 0x1f;
                                        let rn = (inst >> 5) & 0x1f;
                                        let rd = inst & 0x1f;
                                        println!("    EOR X{}, X{}, X{}", rd, rn, rm);
                                    }
                                    0b11010011011 => {
                                        // LSL
                                        let shamt = (inst >> 10) & 0x3f;
                                        let rn = (inst >> 5) & 0x1f;
                                        let rd = inst & 0x1f;
                                        println!("    LSL X{}, X{}, #{}", rd, rn, shamt);
                                    }
                                    0b11010011010 => {
                                        // LSR
                                        let shamt = (inst >> 10) & 0x3f;
                                        let rn = (inst >> 5) & 0x1f;
                                        let rd = inst & 0x1f;
                                        println!("    LSR X{}, X{}, #{}", rd, rn, shamt);
                                    }
                                    0b10101010000 => {
                                        // ORR
                                        let rm = (inst >> 16) & 0x1f;
                                        let rn = (inst >> 5) & 0x1f;
                                        let rd = inst & 0x1f;
                                        println!("    ORR X{}, X{}, X{}", rd, rn, rm);
                                    }
                                    0b11001011000 => {
                                        // SUB
                                        let rm = (inst >> 16) & 0x1f;
                                        let rn = (inst >> 5) & 0x1f;
                                        let rd = inst & 0x1f;
                                        println!("    SUB X{}, X{}, X{}", rd, rn, rm);
                                    }
                                    0b11101011000 => {
                                        // SUBS
                                        let rm = (inst >> 16) & 0x1f;
                                        let rn = (inst >> 5) & 0x1f;
                                        let rd = inst & 0x1f;
                                        println!("    SUBS X{}, X{}, X{}", rd, rn, rm);
                                    }
                                    0b10011011000 => {
                                        // MUL
                                        let rm = (inst >> 16) & 0x1f;
                                        let rn = (inst >> 5) & 0x1f;
                                        let rd = inst & 0x1f;
                                        println!("    MUL X{}, X{}, X{}", rd, rn, rm);
                                    }
                                    0b11010110000 => {
                                        // BR
                                        let rn = (inst >> 5) & 0x1f;
                                        println!("    BR X{}", rn);
                                    }
                                    0b11111111101 => {
                                        // PRNT (custom instruction)
                                        let rd = inst & 0x1f;
                                        println!("    PRNT X{}", rd);
                                    }
                                    0b11111111100 => {
                                        // PRNL (custom instruction)
                                        println!("    PRNL");
                                    }
                                    0b11111111110 => {
                                        // DUMP (custom instruction)
                                        println!("    DUMP");
                                    }
                                    0b11111111111 => {
                                        // HALT (custom instruction)
                                        println!("    HALT");
                                    }
                                    _ => println!("    [UNKNOWN R-TYPE: {:032b}]", inst),
                                }
                        }
                    }
                }
        }
        pc = pc.wrapping_add(4);
    }
}

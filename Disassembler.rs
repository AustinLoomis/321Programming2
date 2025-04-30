use std::{
    collections::HashMap,
    env,
    fs,
    process,
    convert::TryInto,
};

fn main() {
    let args: Vec<String> = env::args().collect(); // Collect command line arguments
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
    bytes.chunks_exact(4)
        .map(|chunk| u32::from_be_bytes(chunk.try_into().unwrap()))
        .collect()
}

fn scan_labels(instructions: &[u32]) -> HashMap<u32, String> {
    let mut labels = HashMap::new();
    let mut pc = 0;
    let mut label_count = 0;  // Track label count separately

    for &inst in instructions {
        let opcode = (inst >> 21) & 0x7FF;
        match opcode {
            0b000101 => { // B
                let offset = ((inst & 0x3FFFFFF) as i32) << 2;
                let target = (pc as i32 + offset) as u32;
                if !labels.contains_key(&target) {
                    labels.insert(target, format!("label_{}", label_count));
                    label_count += 1;
                }
            },
            0b100101 => { // BL
                let offset = ((inst & 0x3FFFFFF) as i32) << 2;
                let target = (pc as i32 + offset) as u32;
                if !labels.contains_key(&target) {
                    labels.insert(target, format!("label_{}", label_count));
                    label_count += 1;
                }
            },
            0b01010100..=0b01010111 => { // B.cond
                let offset = ((inst >> 5) & 0x7FFFF) as i32;
                let target = (pc as i32 + (offset << 2)) as u32;
                if !labels.contains_key(&target) {
                    labels.insert(target, format!("label_{}", label_count));
                    label_count += 1;
                }
            },
            _ => {}
        }
        pc += 4;
    }
    labels
}

fn disassemble(instructions: &[u32], labels: &HashMap<u32, String>) {
    let mut pc = 0u32;

    for &inst in instructions {
        // Print any label at this address
        if let Some(label) = labels.get(&pc) {
            println!("{}:", label);
        }

        // Extract primary opcode fields
        let op26 = inst >> 26;                   // bits [31:26]
        let op25_31 = (inst >> 25) & 0x7F;       // bits [31:25]
        //let op24_31 = (inst >> 24) & 0xFF;       // bits [31:24] not needed

        // Helper for sign-extending imm19 and computing byte offset
        let sign_extend_imm19 = |raw: u32| {
            let mut val = (raw & 0x7FFFF) as i32;
            if val & (1 << 18) != 0 {
                val |= !0 << 19;
            }
            (val << 2) as u32
        };

        match op26 {
            // Unconditional branch (B): opcode 0b000101
            0b000101 => {
                let imm26 = inst & 0x03FFFFFF;
                let offset = (imm26 << 2) as u32;
                let target = pc.wrapping_add(offset);
                println!("    B {}", labels.get(&target).unwrap_or(&format!("label_{}", target / 4)));
            }

            _ => match op25_31 {
                // CBZ / CBNZ: opcode7 0b0110100
                0b0110100 => {
                    let bit24 = (inst >> 24) & 1;
                    let imm19 = (inst >> 5) & 0x7FFFF;
                    let offset = sign_extend_imm19(imm19);
                    let target = pc.wrapping_add(offset);
                    let rt = inst & 0x1F;

                    if bit24 == 0 {
                        println!("    CBZ X{}, {}", rt,
                            labels.get(&target).unwrap_or(&format!("label_{}", target / 4)));
                    } else {
                        println!("    CBNZ X{}, {}", rt,
                            labels.get(&target).unwrap_or(&format!("label_{}", target / 4)));
                    }
                }

                // Conditional branch (B.cond): opcode7 0b0101010
                0b0101010 => {
                    let imm19 = (inst >> 5) & 0x7FFFF;
                    let offset = sign_extend_imm19(imm19);
                    let target = pc.wrapping_add(offset);
                    let cond = inst & 0xF;
                    let cond_str = match cond {
                        0b0000 => "EQ", 0b0001 => "NE",
                        0b0010 => "HS", 0b0011 => "LO",
                        0b0100 => "MI", 0b0101 => "PL",
                        0b0110 => "VS", 0b0111 => "VC",
                        0b1000 => "HI", 0b1001 => "LS",
                        0b1010 => "GE", 0b1011 => "LT",
                        0b1100 => "GT", 0b1101 => "LE",
                        0b1110 => "AL", _ => "??",
                    };

                    println!("    B.{} {}", cond_str,
                        labels.get(&target).unwrap_or(&format!("label_{}", target / 4)));
                }

                // Handle I-type and R-type instructions
                _ => match (inst >> 24) & 0xFF {
                    // I-type instructions (8-bit opcode)
                    0b10010000..=0b10111111 => match (inst >> 22) & 0x3FF {
                        0b1001000100 => {  // ADDI
                            let imm = (inst >> 10) & 0xFFF;
                            let rn = (inst >> 5) & 0x1F;
                            let rd = inst & 0x1F;
                            println!("    ADDI X{}, X{}, #{}", rd, rn, imm);
                        },
                        0b1001001000 => {  // ANDI
                            let imm = (inst >> 10) & 0xFFF;
                            let rn = (inst >> 5) & 0x1F;
                            let rd = inst & 0x1F;
                            println!("    ANDI X{}, X{}, #{}", rd, rn, imm);
                        },
                        0b1101001000 => {  // EORI
                            let imm = (inst >> 10) & 0xFFF;
                            let rn = (inst >> 5) & 0x1F;
                            let rd = inst & 0x1F;
                            println!("    EORI X{}, X{}, #{}", rd, rn, imm);
                        },
                        0b1011001000 => {  // ORRI
                            let imm = (inst >> 10) & 0xFFF;
                            let rn = (inst >> 5) & 0x1F;
                            let rd = inst & 0x1F;
                            println!("    ORRI X{}, X{}, #{}", rd, rn, imm);
                        },
                        0b1101000100 => {  // SUBI
                            let imm = (inst >> 10) & 0xFFF;
                            let rn = (inst >> 5) & 0x1F;
                            let rd = inst & 0x1F;
                            println!("    SUBI X{}, X{}, #{}", rd, rn, imm);
                        },
                        0b1111000100 => {  // SUBIS
                            let imm = (inst >> 10) & 0xFFF;
                            let rn = (inst >> 5) & 0x1F;
                            let rd = inst & 0x1F;
                            println!("    SUBIS X{}, X{}, #{}", rd, rn, imm);
                        },
                        _ => println!("    [UNKNOWN I-TYPE: {:032b}]", inst),
                    },

                    // R-type instructions
                    _ => match (inst >> 21) & 0x7FF {
                        0b10001011000 => { // ADD
                            let rm = (inst >> 16) & 0x1F;
                            let rn = (inst >> 5) & 0x1F;
                            let rd = inst & 0x1F;
                            println!("    ADD X{}, X{}, X{}", rd, rn, rm);
                        },
                        0b10001010000 => { // AND
                            let rm = (inst >> 16) & 0x1F;
                            let rn = (inst >> 5) & 0x1F;
                            let rd = inst & 0x1F;
                            println!("    AND X{}, X{}, X{}", rd, rn, rm);
                        },
                        0b11001010000 => { // EOR
                            let rm = (inst >> 16) & 0x1F;
                            let rn = (inst >> 5) & 0x1F;
                            let rd = inst & 0x1F;
                            println!("    EOR X{}, X{}, X{}", rd, rn, rm);
                        },
                        0b11010011011 => { // LSL
                            let rm = (inst >> 16) & 0x1F;
                            let rn = (inst >> 5) & 0x1F;
                            let rd = inst & 0x1F;
                            println!("    LSL X{}, X{}, X{}", rd, rn, rm);
                        },
                        0b11010011010 => { // LSR
                            let rm = (inst >> 16) & 0x1F;
                            let rn = (inst >> 5) & 0x1F;
                            let rd = inst & 0x1F;
                            println!("    LSR X{}, X{}, X{}", rd, rn, rm);
                        },
                        0b10101010000 => { // ORR
                            let rm = (inst >> 16) & 0x1F;
                            let rn = (inst >> 5) & 0x1F;
                            let rd = inst & 0x1F;
                            println!("    ORR X{}, X{}, X{}", rd, rn, rm);
                        },
                        0b11001011000 => { // SUB
                            let rm = (inst >> 16) & 0x1F;
                            let rn = (inst >> 5) & 0x1F;
                            let rd = inst & 0x1F;
                            println!("    SUB X{}, X{}, X{}", rd, rn, rm);
                        },
                        _ => println!("    [UNKNOWN R-TYPE: {:032b}]", inst),
                    }
                }
            }
        }
        pc = pc.wrapping_add(4);
    }
}
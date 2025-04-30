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
    let mut pc = 0;

    for &inst in instructions {
        if let Some(label) = labels.get(&pc) {
            println!("{}:", label);
        }

        // First check shortest opcodes (most specific)
        match () {
            // B-type (6-bit opcode: bits 26-31)
            _ if (inst >> 26) == 0b000101 => {
                let offset = ((inst & 0x03FFFFFF) as i32) << 2;
                let target = pc.wrapping_add(offset as u32);
                println!("    B label_{}", target/4);
            },
            
            // CB-type (8-bit opcode: bits 31-24)
            _ if (inst >> 24) == 0b01010100 => {  // B.cond
                let cond_br_addr = (inst >> 5) & 0x7FFFF;  // bits 23-5 (19 bits)
                let rt = inst & 0x1F;                     // bits 4-0 (5 bits)
                let offset = (cond_br_addr as i32) << 2;   // Shift left 2 for word alignment
                let target = pc.wrapping_add(offset as u32);
    
                let cond_str = match (cond_br_addr >> 15) & 0xF {  // Condition is in bits 23-20
                    0x0 => "EQ", 0x1 => "NE", 0x2 => "HS",
                    0x3 => "LO", 0x4 => "MI", 0x5 => "PL",
                    0x6 => "VS", 0x7 => "VC", 0x8 => "HI",
                    0x9 => "LS", 0xA => "GE", 0xB => "LT",
                    0xC => "GT", 0xD => "LE", _ => "??",
                };
    
                if rt == 0 {
                    println!("    B.{} label_{}", cond_str, target/4);
                } else {
                    println!("    CBZ X{}, label_{}", rt, target/4); 
                }
            },
            
            // I-type (10-bit opcode: bits 22-31)
            _ => match (inst >> 22) & 0x3FF {  // Check 10-bit opcode
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
                
                // R-type (11-bit opcode: bits 21-31)
                _ => match (inst >> 21) & 0x7FF {
                    0b10001011000 => { // ADD
                        let rm = (inst >> 16) & 0x1F;
                        let rn = (inst >> 5) & 0x1F;
                        let rd = inst & 0x1F;
                        println!("    ADD X{}, X{}, X{}", rd, rn, rm);
                    },
                    0b11001011000 => { // SUB
                        let rm = (inst >> 16) & 0x1F;
                        let rn = (inst >> 5) & 0x1F;
                        let rd = inst & 0x1F;
                        println!("    SUB X{}, X{}, X{}", rd, rn, rm);
                    },
                    _ => println!("    [UNKNOWN: {:032b}]", inst),
                }
            }
        }
        pc += 4;
    }
}
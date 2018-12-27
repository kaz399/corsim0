use crate::bitdecode::*;
use crate::cpu::M0System;
use crate::cpuflag::add_with_carry;
use crate::cpuflag::ArmV6m;
use crate::debug_info::{b16_fmt, b32_fmt};
use crate::device::SystemMapAccess;

fn bit_count(bytecode: u32) -> u32 {
    let mut count = 0;
    for i in 0..32 {
        if (bytecode & (1 << i)) == 1 {
            count += 1;
        }
    }
    count
}

// instructions (special)

pub fn unpredicable(system: &mut M0System) -> u32 {
    println!("\t UNPREDICABLE ERROR");
    0
}

pub fn not_impremented(system: &mut M0System) -> u32 {
    println!("\t (not impremented)");
    0
}

// instructions: A
// instructions: B

pub fn b_16(bytecode: u16, system: &mut M0System) -> u32 {
    // Ref:DDI0403D_arm_architecture_v7m_reference_manual.pdf p.239
    system.cpu.pc += 2;
    if bytecode & (0b1 << 12) == 0b1 {
        println!("conditional branch");
    }
    else {
        let field = parse_bit_u(&bytecode, "11100 iiiiiiiiiii").unwrap();
        let imm11: u32 = field["i"] as u32;
        let sign_flag: u32 = imm11 >> 10;
        println!("s:{}", sign_flag);
        let mut imm32: u32 = 0;
        if sign_flag == 0b1 {
            imm32 = 0xffffffff & !0b111111111111;
            println!("minus imm32:{:08x}", imm32);
            
        }
        imm32 |=  imm11 << 1;
        println!("imm32:{:08x}", imm32);
        let next_pc: u32 = system.cpu.pc.wrapping_add(imm32);
        system.cpu.pc = next_pc;

        println!("\t\t b\t#{:+}\t\t; {:08x}", imm32 as i32, next_pc);
    }
    1
}

pub fn b_32(bytecode32: u32, system: &mut M0System) -> u32 {
    // Ref:DDI0403D_arm_architecture_v7m_reference_manual.pdf p.239
    // Ref: Thumb-2SupplementReferencemanual.pdf p.122
    system.cpu.pc += 4;
    if bytecode32 & (0b1 << 12) == 0b1 {
        let field = parse_bit_u(&bytecode32, "111110 S iiiiiiiiii 10 j 1 k aaaaaaaaaaa").unwrap();
        let sign_flag: u32 = field["S"];
        let i1: u32 = !(field["j"] ^ field["S"]) & 0b1;
        let i2: u32 = !(field["k"] ^ field["S"]) & 0b1;
        let imm10: u32 = field["i"];
        let imm11: u32 = field["a"];
        let mut imm32: u32 = 0;
        if sign_flag == 0b1 {
            imm32 = 0xffffffff & !0b111111111111111111111111;
        }
        imm32 |= i1 << 24 | i2 << 23 | imm10 << 12 | imm11 << 1;
        let next_pc: u32 = system.cpu.pc.wrapping_add(imm32);
        system.cpu.pc = next_pc;

        println!("\t\t b\t#{:+}\t\t; {:08x}", imm32 as i32, next_pc);
    }
    else {
        println!("conditional branch");
    }
    1
}

pub fn bl_32(bytecode32: u32, system: &mut M0System) -> u32 {
    // Ref:DDI0403D_arm_architecture_v7m_reference_manual.pdf p.248
    system.cpu.pc += 4;
    let field = parse_bit_u(&bytecode32, "11110 S iiiiiiiiii 11 j 1 k aaaaaaaaaaa").unwrap();
    let sign_flag: u32 = field["S"];
    let i1: u32 = !(field["j"] ^ field["S"]) & 0b1;
    let i2: u32 = !(field["k"] ^ field["S"]) & 0b1;
    let imm10: u32 = field["i"];
    let imm11: u32 = field["a"];
    let mut imm32: u32 = 0;
    if sign_flag == 0b1 {
        imm32 = 0xffffffff & 0b000000000000000000000000;
    }
    imm32 |= i1 << 24 | i2 << 23 | imm10 << 12 | imm11 << 1;
    let next_pc: u32 = system.cpu.pc.wrapping_add(imm32);

    system.cpu.lr = (system.cpu.pc & 0xfffffffe) | 0b1;
    system.cpu.pc = next_pc;

    println!("\t\t bl\t#{:+}\t\t; {:08x}", imm32 as i32, next_pc);
    1
}

pub fn bkpt(bytecode: u16, system: &mut M0System) -> u32 {
    // Ref: Thumb-2SupplementReferencemanual.pdf p.132
    println!("\t bkpt");
    not_impremented(system)
}

pub fn bx(bytecode: u16, system: &mut M0System) -> u32 {
    system.cpu.pc += 2;
    let field = parse_bit_u(&bytecode, "010001 11 0 mmmm").unwrap();
    let rm: usize = field["m"] as usize;

    match rm {
        15 => (),
        14 => {
            println!("\t\t bx\tlr");
            system.cpu.pc = system.cpu.lr & 0xfffffffe;
        },
        _ => {
            println!("\t\t bx\tr{}", rm);
            system.cpu.pc = system.cpu.r[rm] & 0xfffffffe;
        },
    }
    1
}
// instructions: C

pub fn cbz(bytecode: u16, system: &mut M0System) -> u32 {
    println!("\t\t cbz");
    not_impremented(system)
}

pub fn cbnz(bytecode: u16, system: &mut M0System) -> u32 {
    println!("\t\t cbnz");
    not_impremented(system)
}

// instructions: C
// instructions: D

pub fn dbg(bytecode: u16, system: &mut M0System) -> u32 {
    // Ref: Thumb-2SupplementReferencemanual.pdf p.164
    println!("\t dpg");
    not_impremented(system)
}

// instructions: E
// instructions: F
// instructions: G
// instructions: H

pub fn hint_32(bytecode32: u32, system: &mut M0System) -> u32 {
    // Ref:DDI0403D_arm_architecture_v7m_reference_manual.pdf p.170
    not_impremented(system)
}

// instructions: I

pub fn it(bytecode: u16, system: &mut M0System) -> u32 {
    // Ref: Thumb-2SupplementReferencemanual.pdf p.176
    println!("\t it");
    not_impremented(system)
}

// instructions: J
// instructions: L
// instructions: M

pub fn mrs_32(bytecode32: u32, system: &mut M0System) -> u32 {
    // Ref:DDI0403D_arm_architecture_v7m_reference_manual.pdf p.357
    not_impremented(system)
}

pub fn msr_32(bytecode32: u32, system: &mut M0System) -> u32 {
    // Ref:DDI0403D_arm_architecture_v7m_reference_manual.pdf p.358
    not_impremented(system)
}

// instructions: N

pub fn nop(bytecode: u16, system: &mut M0System) -> u32 {
    // Ref: Thumb-2SupplementReferencemanual.pdf p.273
    println!("\t nop");
    system.cpu.pc += 2;
    1
}

pub fn nop_32(bytecode32: u32, system: &mut M0System) -> u32 {
    // Ref:DDI0403D_arm_architecture_v7m_reference_manual.pdf p.366
    println!("\t nop (32bit)");
    system.cpu.pc += 4;
    1
}

// instructions: O
// instructions: P

pub fn pop(bytecode: u16, system: &mut M0System) -> u32 {
    // Ref: Thumb-2SupplementReferencemanual.pdf p.293
    println!("\t pop");
    not_impremented(system)
}

pub fn push(bytecode: u16, system: &mut M0System) -> u32 {
    // Ref: Thumb-2SupplementReferencemanual.pdf p.295
    system.cpu.pc += 2;
    let reglist: u16 = (bytecode & 0xff) as u16;
    println!("\t push reglist:{}", b16_fmt(reglist));
    if reglist == 0 {
        unpredicable(system)
    } else {
        let original_sp: u32 = system.cpu.sp[system.cpu.ctrl_spsel];
        let mut current_sp: u32 = system.cpu.sp[system.cpu.ctrl_spsel] - bit_count(reglist as u32);
        system.cpu.sp[system.cpu.ctrl_spsel] = current_sp;
        for i in 0..13 {
            if (reglist & (1 << i)) != 0 {
                println!("\t\t push r{} to {:08x}", i, current_sp);
                system.system_map.write32(current_sp, system.cpu.r[i]);
                current_sp += 4;
            }
        }
        assert_eq!(system.cpu.sp[system.cpu.ctrl_spsel], original_sp);

        1
    }
}

// instructions: Q
// instructions: R
// instructions: S

pub fn sev(bytecode: u16, system: &mut M0System) -> u32 {
    // Ref: Thumb-2SupplementReferencemanual.pdf p.596
    println!("\t sev");
    not_impremented(system)
}

// instructions: T
// instructions: U
// instructions: V
// instructions: W

pub fn wfe(bytecode: u16, system: &mut M0System) -> u32 {
    // Ref: Thumb-2SupplementReferencemanual.pdf p.610
    println!("\t wfe");
    not_impremented(system)
}

pub fn wfi(bytecode: u16, system: &mut M0System) -> u32 {
    // Ref: Thumb-2SupplementReferencemanual.pdf p.612
    println!("\t wfi");
    not_impremented(system)
}

// instructions: X
// instructions: Y

pub fn cpu_yield(bytecode: u16, system: &mut M0System) -> u32 {
    // Ref: Thumb-2SupplementReferencemanual.pdf p.614
    println!("\t yield");
    not_impremented(system)
}

// instructions: Z

use crate::bitdecode::*;
use crate::cpuflag::add_with_carry;
use crate::cpuflag::ArmV6m;
use crate::cpuflag::CalcFlags;
use crate::debug_info::{b16_fmt, b32_fmt};
use crate::device::SystemMap;
use crate::device::SystemMapAccess;
use crate::instruction::*;

pub struct CortexM0 {
    pub r: [u32; 13],
    pub sp: [u32; 2],
    pub lr: u32,
    pub pc: u32,

    pub ctrl_spsel: usize,
    pub ctrl_npriv: usize,
    pub primask_pm: usize,

    pub apsr: u32,
    pub ipsr: u32,
    pub epsr: u32,

    pub actlr: u32,
    pub cpuid: u32,
    pub icsr: u32,
    pub vtor: u32,
    pub aircr: u32,
    pub scr: u32,
    pub ccr: u32,
    pub shpr2: u32,
    pub shpr3: u32,
    pub shcsr: u32,
    pub dfsr: u32,
}

impl Default for CortexM0 {
    fn default() -> Self {
        Self {
            r: [0; 13],
            sp: [0; 2],
            lr: 0,
            pc: 0,

            ctrl_spsel: 0,
            ctrl_npriv: 0,
            primask_pm: 0,

            apsr: 0,
            ipsr: 0,
            epsr: 0,
            actlr: 0,
            cpuid: 0x410cc200,
            icsr: 0,
            vtor: 0,
            aircr: 0,
            scr: 0,
            ccr: 0xffffffff,
            shpr2: 0,
            shpr3: 0,
            shcsr: 0,
            dfsr: 0,
        }
    }
}

pub struct M0System {
    pub cpu: CortexM0,
    pub system_map: SystemMap,
}

impl M0System {
    pub fn new(system_map: SystemMap) -> M0System {
        println!("*CREATE NEW SYSTEM");
        M0System {
            cpu: CortexM0 {
                ..CortexM0::default()
            },
            system_map: system_map,
        }
    }
}

pub trait SystemCtrl {
    fn reset(&mut self);
    fn dump(&self);
    fn execute(&mut self) -> u32;
}

impl SystemCtrl for M0System {
    fn reset(&mut self) {
        println!("*RESET CPU");
        self.cpu = CortexM0 {
            ..CortexM0::default()
        };
        self.cpu.sp[self.cpu.ctrl_spsel] =
            self.system_map.read32(self.cpu.vtor).unwrap() & 0xfffffffc;
        self.cpu.pc = self.system_map.read32(self.cpu.vtor + 4).unwrap() & 0xfffffffe;
    }

    fn dump(&self) {
        let cpu: &CortexM0 = &self.cpu;
        println!("*REGISTER DUMP --------------------------------------------------");
        for n in 0..cpu.r.len() {
            println!("  r{:02}:\t\t{:08x}  {}", n, cpu.r[n], b32_fmt(cpu.r[n]));
        }
        if cpu.ctrl_spsel == 0 {
            println!(" >r13 (msp):\t{:08x}  {}", cpu.sp[0], b32_fmt(cpu.sp[0]));
            println!("  r13 (psp):\t{:08x}  {}", cpu.sp[1], b32_fmt(cpu.sp[1]));
        } else {
            println!("  r13 (msp):\t{:08x}  {}", cpu.sp[0], b32_fmt(cpu.sp[0]));
            println!(" >r13 (psp):\t{:08x}  {}", cpu.sp[1], b32_fmt(cpu.sp[1]));
        }
        println!("  r14 (lr):\t{:08x}  {}", cpu.lr, b32_fmt(cpu.lr));
        println!("  r15 (pc):\t{:08x}  {}", cpu.pc, b32_fmt(cpu.pc));
        println!("  apsr:\t\t{:08x}  {}", cpu.apsr, b32_fmt(cpu.apsr));
        println!("  ipsr:\t\t{:08x}  {}", cpu.ipsr, b32_fmt(cpu.ipsr));
        println!("  epsr:\t\t{:08x}  {}", cpu.epsr, b32_fmt(cpu.epsr));
    }

    fn execute(&mut self) -> u32 {
        get_thumb_instruction(self)
    }
}

fn get_thumb_instruction(system: &mut M0System) -> u32 {
    let bytecode: u16 = system.system_map.read16(system.cpu.pc).unwrap();

    println!(
        "adrs:{:08x}\t{:04x}({})",
        system.cpu.pc,
        bytecode,
        b16_fmt(bytecode)
    );

    // Ref: Thumb-2SupplementReferenceManual.pdf p.42
    bitcode_u_ex!(
        bytecode,
        "111**",
        "11100",
        instruction_32bit(bytecode, system)
    );
    // Ref: Thumb-2SupplementReferenceManual.pdf p.43

    // 8bit
    bitcode_u!(
        bytecode,
        "11011110",
        undefined_instruction(bytecode, system)
    );
    bitcode_u!(bytecode, "11011111", service_call(bytecode, system));
    bitcode_u!(
        bytecode,
        "01000111",
        branch_exchange_instruction_set(bytecode, system)
    );
    // 6bit
    bitcode_u!(bytecode, "000110", add_substract_register(bytecode, system));
    bitcode_u!(
        bytecode,
        "000111",
        add_substract_immediate(bytecode, system)
    );
    bitcode_u!(
        bytecode,
        "010000",
        data_processing_register(bytecode, system)
    );
    bitcode_u!(
        bytecode,
        "010001",
        special_data_processing(bytecode, system)
    );
    // 5bit
    bitcode_u!(bytecode, "01001", load_from_literal_pool(bytecode, system));
    bitcode_u!(
        bytecode,
        "01100",
        store_word_immediate_offset(bytecode, system)
    );
    bitcode_u!(
        bytecode,
        "01101",
        loade_word_immediate_offset(bytecode, system)
    );
    bitcode_u!(
        bytecode,
        "01110",
        store_byte_immediate_offset(bytecode, system)
    );
    bitcode_u!(
        bytecode,
        "01111",
        loade_byte_immediate_offset(bytecode, system)
    );
    bitcode_u!(
        bytecode,
        "10000",
        store_halfward_immediate_offset(bytecode, system)
    );
    bitcode_u!(
        bytecode,
        "10001",
        load_halfward_immediate_offset(bytecode, system)
    );
    bitcode_u!(bytecode, "10010", store_to_stack(bytecode, system));
    bitcode_u!(bytecode, "10011", load_from_stack(bytecode, system));
    bitcode_u!(bytecode, "10100", add_to_pc(bytecode, system));
    bitcode_u!(bytecode, "10101", add_to_sp(bytecode, system));
    bitcode_u!(bytecode, "11000", store_multiple(bytecode, system));
    bitcode_u!(bytecode, "11001", load_multiple(bytecode, system));
    bitcode_u!(bytecode, "11100", unconditional_branch(bytecode, system));
    // 4bit
    bitcode_u!(
        bytecode,
        "0101",
        load_store_register_offset(bytecode, system)
    );
    bitcode_u!(bytecode, "1011", miscellaneous(bytecode, system));
    bitcode_u!(bytecode, "1101", conditional_branch(bytecode, system));
    // 3bit
    bitcode_u!(
        bytecode,
        "000",
        shift_by_immediate_move_register(bytecode, system)
    );
    bitcode_u!(
        bytecode,
        "001",
        add_substract_compare_move_immediate(bytecode, system)
    );

    decode_error(bytecode, system)
}

// 000 opecode[2] imm[4] Rm[3] Rd[3]
fn shift_by_immediate_move_register(bytecode: u16, system: &mut M0System) -> u32 {
    println!("\t shift by immediate, move register");
    not_impremented(system)
}

// 000110 opc[1] Rm[3] Rn[3] Rd[3]
fn add_substract_register(bytecode: u16, system: &mut M0System) -> u32 {
    println!("\t Add/substract register");
    not_impremented(system)
}

// 000111 opc[1] imm[3] Rn[3] Rd[3]
fn add_substract_immediate(bytecode: u16, system: &mut M0System) -> u32 {
    println!("\t Add/substract immediate");
    not_impremented(system)
}

// 001 opecode[2] Rdn[3] imm[8]
fn add_substract_compare_move_immediate(bytecode: u16, system: &mut M0System) -> u32 {
    println!("\t Add/Sub/Compare/Move immediate");
    not_impremented(system)
}

// 010000 opecode[4] Rm[3] Rdn[3]
fn data_processing_register(bytecode: u16, system: &mut M0System) -> u32 {
    println!("\t Data-processing register");
    not_impremented(system)
}

// 010001 opecode[2] DN[1] Rm[3] Rdn[3]
fn special_data_processing(bytecode: u16, system: &mut M0System) -> u32 {
    println!("\t Special data processing");
    not_impremented(system)
}

// 01000111 L[1]  Rm[3] 000
fn branch_exchange_instruction_set(bytecode: u16, system: &mut M0System) -> u32 {
    println!("\t Branch/exchange instruction set");
    not_impremented(system)
}

// 01001 Rd[3] PC-relative-imm[8]
fn load_from_literal_pool(bytecode: u16, system: &mut M0System) -> u32 {
    // Ref: Thumb-2SupplementReferencemanual.pdf p.186
    println!("\t Load from Literal Pool (ldr literal)");
    let regnum: usize = ((bytecode >> 8) & 0b111) as usize;
    let imm: u16 = bytecode & 0b11111111;
    let imm32: u32 = (imm << 2) as u32;
    let pc_aligned: u32 = system.cpu.pc & 0xfffffffc;
    let load_address: u32 = pc_aligned + imm32;
    println!(
        "\t\t ldr  r{}, [pc, #{}]  ;b load from {:08x}",
        regnum, imm32, load_address
    );
    let load_data: u32 = system.system_map.read32(load_address).unwrap();
    println!("\t Read data:0x{:08x}", load_data);
    match regnum {
        15 => {
            println!("jump");
            if load_data & 0x3 != 0 {
                return unpredicable(system);
            }
            system.cpu.pc = load_data;
        }
        14 => {
            println!("change stack pointer");
            system.cpu.sp[system.cpu.ctrl_spsel] = load_data;
            system.cpu.pc += 2;
        }
        _ => {
            system.cpu.r[regnum] = load_data;
            system.cpu.pc += 2;
        }
    }
    1
}

// 0101 opecode[3] Rm[3] Rn[3] Rd[3]
fn load_store_register_offset(bytecode: u16, system: &mut M0System) -> u32 {
    println!("\t Load/Store register offset");
    not_impremented(system)
}

// 01100 imm[5] Rn[3] Rd[3]
fn store_word_immediate_offset(bytecode: u16, system: &mut M0System) -> u32 {
    // Ref: Thumb-2SupplementReferenceManual.pdf p.421
    println!("\t Store word immediate offset");
    let imm: u16 = (bytecode >> 6) & 0b11111;
    let regnum_base: usize = ((bytecode >> 3) & 0b111) as usize;
    let regnum_target: usize = (bytecode & 0b111) as usize;

    println!("\t\t str r{}, [r{}, #{}]", regnum_target, regnum_base, imm);
    not_impremented(system)
}

// 01101 imm[5] Rn[3] Rd[3]
fn loade_word_immediate_offset(bytecode: u16, system: &mut M0System) -> u32 {
    println!("\t Load word immediate offset");
    not_impremented(system)
}

// 01110 imm[5] Rn[3] Rd[3]
fn store_byte_immediate_offset(bytecode: u16, system: &mut M0System) -> u32 {
    println!("\t Store byte immediate offset");
    not_impremented(system)
}

// 01111 imm[5] Rn[3] Rd[3]
fn loade_byte_immediate_offset(bytecode: u16, system: &mut M0System) -> u32 {
    println!("\t Load byte immediate offset");
    not_impremented(system)
}

//10000 imm[5]  Rn[3] Rd[3]
fn store_halfward_immediate_offset(bytecode: u16, system: &mut M0System) -> u32 {
    println!("\t Store halfword immediate offset");
    not_impremented(system)
}

//10001 imm[5]  Rn[3] Rd[3]
fn load_halfward_immediate_offset(bytecode: u16, system: &mut M0System) -> u32 {
    println!("\t Load halfword immediate offset");
    not_impremented(system)
}

// 10010 Rd[3] SP-relative-imm[8]
fn store_to_stack(bytecode: u16, system: &mut M0System) -> u32 {
    println!("\t Store to stack");
    not_impremented(system)
}

// 10011 Rd[3] SP-relative-imm[8]
fn load_from_stack(bytecode: u16, system: &mut M0System) -> u32 {
    println!("\t Load from stack");
    not_impremented(system)
}

// 10100 Rd[3] imm[8]
fn add_to_pc(bytecode: u16, system: &mut M0System) -> u32 {
    println!("\t Add to PC");
    not_impremented(system)
}

// 10101 Rd[3] imm[8]
fn add_to_sp(bytecode: u16, system: &mut M0System) -> u32 {
    let regnum: usize = ((bytecode >> 8) & 0b111) as usize;
    let imm: u16 = bytecode & 0b11111111;
    let imm32: u32 = (imm << 2) as u32;
    println!("\t\t add  r{}, sp, #{}", regnum, imm32);
    let r: ArmV6m = add_with_carry(system.cpu.sp[system.cpu.ctrl_spsel], imm32, 0);
    system.cpu.r[regnum] = r.result;
    system.cpu.apsr = r.flags_to_apsr();
    system.cpu.pc += 2;
    1
}

// 1011 x[12]
// Ref: Thumb-2SpplementReferenceManual.pdf p.49
fn miscellaneous(bytecode: u16, system: &mut M0System) -> u32 {
    println!("\t Miscellaneous 16-bit instractions {}", b16_fmt(bytecode));
    let bit_11_08 = (bytecode >> 8) & 0xf;
    let bit_07_04 = (bytecode >> 4) & 0xf;
    let bit_03_00 = bytecode & 0xf;
    match bit_11_08 {
        0b0000 => adjust_stack_pointer(bytecode, system),
        0b0010 => sign_zero_extend(bytecode, system),
        0b0001 | 0b0011 => cbz(bytecode, system),
        0b1001 | 0b1011 => cbnz(bytecode, system),
        0b0100 | 0b0101 => push(bytecode, system),
        0b1100 | 0b1101 => pop(bytecode, system),
        0b1110 => bkpt(bytecode, system),
        0b1111 => match bit_03_00 {
            0b0000 => nop_compatible_hints(bytecode, system),
            _ => it(bytecode, system),
        },
        0b0110 => match bit_03_00 {
            _ => not_impremented(system),
        },
        _ => unpredicable(system),
    }
}

fn nop_compatible_hints(bytecode: u16, system: &mut M0System) -> u32 {
    // Ref: Thumb-2SupplementReferencemanual.pdf p.22
    let hint_number: u8 = (bytecode >> 4 & 0x0f) as u8;
    match hint_number {
        0x0 => nop(bytecode, system),
        0x1 => cpu_yield(bytecode, system),
        0x2 => wfe(bytecode, system),
        0x3 => wfi(bytecode, system),
        0x4 => sev(bytecode, system),
        0xf => dbg(bytecode, system),
        _ => unpredicable(system),
    }
}

// 1011 0000 opc[1] imm[7]
fn adjust_stack_pointer(bytecode: u16, system: &mut M0System) -> u32 {
    println!("\t Adjust stack pointer");
    let opc: u16 = bytecode & 0b10000000;
    let imm: u16 = bytecode & 0b01111111;
    let imm32: u32 = (imm << 2) as u32;
    if opc == 0 {
        // Ref: Thumb-2SupplementReferencemanual.pdf p.108
        println!("\t\t add  sp, sp, #{}", imm32);
        let r: ArmV6m = add_with_carry(system.cpu.sp[system.cpu.ctrl_spsel], imm32, 0);
        system.cpu.sp[system.cpu.ctrl_spsel] = r.result;
        system.cpu.apsr = r.flags_to_apsr();
    } else {
        // Ref: Thumb-2SupplementReferencemanual.pdf p.453
        println!("\t\t sub  sp, sp, #-{}", imm32);
        let r: ArmV6m = add_with_carry(system.cpu.sp[system.cpu.ctrl_spsel], !imm32, 1);
        system.cpu.sp[system.cpu.ctrl_spsel] = r.result;
        system.cpu.apsr = r.flags_to_apsr();
    }
    system.cpu.pc += 2;
    1
}

// 1011 0010 opc[1] imm[7]
fn sign_zero_extend(bytecode: u16, system: &mut M0System) -> u32 {
    println!("\t Sign/Zero extend");
    not_impremented(system)
}

//11000 Rn[3] imm[8]
fn store_multiple(bytecode: u16, system: &mut M0System) -> u32 {
    println!("\t Store multiple");
    not_impremented(system)
}

//11001 Rn[3] imm[8]
fn load_multiple(bytecode: u16, system: &mut M0System) -> u32 {
    println!("\t Load multiple");
    not_impremented(system)
}

// 1101 cond[2] imm[8]
fn conditional_branch(bytecode: u16, system: &mut M0System) -> u32 {
    println!("\t Conditional branch");
    not_impremented(system)
}

// 11011110 x[8]
fn undefined_instruction(bytecode: u16, system: &mut M0System) -> u32 {
    println!("\t (Undefined Instruction (32bit) {:016b})", bytecode);
    not_impremented(system)
}

// 1111100 x[2] 111 x[20]
fn undefined_instruction_32(bytecode32: u32, system: &mut M0System) -> u32 {
    println!("\t (Undefined Instruction {:032b})", bytecode32);
    not_impremented(system)
}

// 11011111 imm[8]
fn service_call(bytecode: u16, system: &mut M0System) -> u32 {
    println!("\t Service call");
    not_impremented(system)
}

// 11100 imm[11]
fn unconditional_branch(bytecode: u16, system: &mut M0System) -> u32 {
    println!("\t Unconditioal Branch");
    not_impremented(system)
}

// (11101 | 11110 | 11111) x[14]
fn instruction_32bit(bytecode: u16, system: &mut M0System) -> u32 {
    // Ref:DDI0403D_arm_architecture_v7m_reference_manual.pdf p.164
    println!("\t 32-bit instruction (1111)");
    let bytecode_lower: u16 = system.system_map.read16(system.cpu.pc + 2).unwrap();
    let bytecode32 = (bytecode as u32) << 16 | bytecode_lower as u32;
    println!("\t bytecode 32bit {:08x}", bytecode32);
    // op1 == 0b01
    bitcode_u!(
        bytecode32,
        "111 01 00**0**",
        load_and_store_multiple(bytecode32, system)
    );
    bitcode_u!(
        bytecode32,
        "111 01 00**1**",
        load_and_store_double_exclusive_table_branch(bytecode32, system)
    );
    bitcode_u!(
        bytecode32,
        "111 01 01*****",
        data_processing_shifted_register(bytecode32, system)
    );
    bitcode_u!(
        bytecode32,
        "111 01 1xxxxxx",
        coprocessor_instructions(bytecode32, system)
    );

    // op1 == 0b10
    bitcode_u!(
        bytecode32,
        "111 10 *0***** **** 0",
        data_processing_modified_immediate(bytecode32, system)
    );
    bitcode_u!(
        bytecode32,
        "111 10 *1***** **** 0",
        data_processing_plain_binary_immediate(bytecode32, system)
    );
    bitcode_u!(
        bytecode32,
        "111 10 ******* **** 1",
        branch_miscellaneous(bytecode32, system)
    );

    // op1 == 0b11
    bitcode_u!(
        bytecode32,
        "111 11 000***0",
        store_single_data_item(bytecode32, system)
    );
    bitcode_u!(
        bytecode32,
        "111 11 00**001",
        load_byte_memory_hints(bytecode32, system)
    );
    bitcode_u!(
        bytecode32,
        "111 11 00**011",
        load_harfword_memory_hints(bytecode32, system)
    );
    bitcode_u!(bytecode32, "111 11 00**101", load_word(bytecode32, system));
    bitcode_u!(
        bytecode32,
        "111 11 00**121",
        undefined_instruction_32(bytecode32, system)
    );
    bitcode_u!(
        bytecode32,
        "111 11 010****",
        data_processing_register_32(bytecode32, system)
    );
    bitcode_u!(
        bytecode32,
        "111 11 0110***",
        multiply_accumlate_absolutre_difference(bytecode32, system)
    );
    bitcode_u!(
        bytecode32,
        "111 11 0111***",
        long_multiply_accumlate_divide(bytecode32, system)
    );
    bitcode_u!(
        bytecode32,
        "111 1",
        coprocessor_instructions(bytecode32, system)
    );
    found_bug(bytecode, system)
}

fn load_and_store_multiple(bytecode32: u32, system: &mut M0System) -> u32 {
    // Ref:DDI0403D_arm_architecture_v7m_reference_manual.pdf p.171
    println!("\t Load multiple ans store multiple");
    // let field = parse_bit_u(&bytecode32, "111 0100 aa0babbbb").unwrap();
    // match field["a"] {
    //     0b010 => stm_stmia_stmea_32(bytecode32, system),
    //     0b011 => {
    //         if field["b"] == 0b11101 {
    //             return pop_32(bytecode32, system);
    //         }
    //         else {
    //             return ldm__ldmia_ldmfd_32(bytecode32, system);
    //         }
    //     },
    //     0b100 => {
    //         if field["b"] == 0b11101 {
    //             return push_32(bytecode32, system);
    //         }
    //         else {
    //             return stmdb_stmfd_32(bytecode32, system);
    //         }
    //     },
    //     0b101 => ldmdb_ldmea_32(bytecode32, system),
    //     _ => not_impremented(system),
    // }

    not_impremented(system)
}

fn load_and_store_double_exclusive_table_branch(bytecode32: u32, system: &mut M0System) -> u32 {
    // Ref:DDI0403D_arm_architecture_v7m_reference_manual.pdf p.172
    println!("\t Load/store dual or exclusive, table branch");
    // let field = parse_bit_u(&bytecode32, "111 0100 aa1aa**** ******** bbbb").unwrap();
    // match field["a"] {
    //     0b0000 => strex_32(bytecode32, system),
    //     0b0001 => ldrex_32(bytecode32, system),
    //     0b0010 | 0b0110 | 0b1000 | 0b1010 | 0b1100 | 0b1110 => strd_32(bytecode32, system),
    //     0b0011 | 0b0111 | 0b1001 | 0b1011 | 0b1101 | 0b1111 => ldrd_32(bytecode32, system),
    //     0b0100 => match field["b"] {
    //         0b0100 => strexb_32(bytecode32, system),
    //         0b0101 => strexh_32(bytecode32, system),
    //         _ => undefined_instruction_32(bytecode32, system),
    //     },
    //     0b0101 => match field["b"] {
    //         0b0000 => tbb_32(bytecode32, system),
    //         0b0001 => tbh_32(bytecode32, system),
    //         0b0100 => ldrexb_32(bytecode32, system),
    //         0b0101 => ldrexh_32(bytecode32, system),
    //         _ => undefined_instruction_32(bytecode32, system),
    //     },
    //     _ => undefined_instruction_32(bytecode32, system),
    // }

    undefined_instruction_32(bytecode32, system)
}

fn data_processing_shifted_register(bytecode32: u32, system: &mut M0System) -> u32 {
    // Ref:DDI0403D_arm_architecture_v7m_reference_manual.pdf p.179
    println!("\t Data processing (shifted register)");
    undefined_instruction_32(bytecode32, system)
}

fn data_processing_modified_immediate(bytecode32: u32, system: &mut M0System) -> u32 {
    // Ref:DDI0403D_arm_architecture_v7m_reference_manual.pdf p.165
    println!("\t Data processing (modified immediate)");
    undefined_instruction_32(bytecode32, system)
}

fn data_processing_plain_binary_immediate(bytecode32: u32, system: &mut M0System) -> u32 {
    // Ref:DDI0403D_arm_architecture_v7m_reference_manual.pdf p.168
    println!("\t Data processing (plain binary immediate)");
    undefined_instruction_32(bytecode32, system)
}

fn branch_miscellaneous(bytecode32: u32, system: &mut M0System) -> u32 {
    // Ref:DDI0403D_arm_architecture_v7m_reference_manual.pdf p.169
    println!("\t Branch and micscellaneous control");
    let field = parse_bit_u(&bytecode32, "111 10 aaaaaaa **** 1 bbb").unwrap();
    let op1 = field["b"];
    let op = field["a"];
    let sub_bitcode: u32 = field["captured"];

    println!(
        "\t sub_bitcode:{:010b} op1: {:03b} op1:{:07b}",
        sub_bitcode, op1, op
    );

    // bitcode_l_ex!(sub_bitcode, "011100* 0*0", "*111*** 0*0", b_32(bytecode32, system));
    // bitcode_l!(sub_bitcode, "011100* 0*0", msr_32(bytecode32, system));
    // bitcode_l!(sub_bitcode, "0111010 0*0", hint_32(bytecode32, system));
    // bitcode_l!(sub_bitcode, "0111011 0*0", miscellaneous_control_32(bytecode32, system));
    // bitcode_l!(sub_bitcode, "011111* 0*0", mrs_32(bytecode32, system));
    // bitcode_l!(sub_bitcode, "1111111 010", undefined_instruction_32(bytecode32, system));
    // bitcode_l!(sub_bitcode, "******* 0*1", b_32(bytecode32, system));
    bitcode_l!(sub_bitcode, "******* 1*1", bl_32(bytecode32, system));

    undefined_instruction_32(bytecode32, system)
}

fn miscellaneous_control_32(bytecode32: u32, system: &mut M0System) -> u32 {
    // Ref:DDI0403D_arm_architecture_v7m_reference_manual.pdf p.170
    println!("\t Hint instructions (32bit)");
    not_impremented(system)
}

fn store_single_data_item(bytecode32: u32, system: &mut M0System) -> u32 {
    // Ref:DDI0403D_arm_architecture_v7m_reference_manual.pdf p.178
    println!("\t Store single data item");
    undefined_instruction_32(bytecode32, system)
}

fn load_byte_memory_hints(bytecode32: u32, system: &mut M0System) -> u32 {
    // Ref:DDI0403D_arm_architecture_v7m_reference_manual.pdf p.176
    println!("\t Load byte, memory hints");
    undefined_instruction_32(bytecode32, system)
}

fn load_harfword_memory_hints(bytecode32: u32, system: &mut M0System) -> u32 {
    // Ref:DDI0403D_arm_architecture_v7m_reference_manual.pdf p.174
    println!("\t Load harfword, memory hints");
    let field = parse_bit_u(&bytecode32, "111 1100 aa 011 nnnn tttt bbbbbb").unwrap();
    let op1: u32 = field["a"];
    let op2: u32 = field["b"];
    let rn: u32 = field["b"];
    let rt: u32 = field["t"];
    let sub_bitcode: u32 = field["captured"];
    println!("\t op1:{:02b} op2:{:06b} Rn:{} Rt:{}", op1, op2, rn, rt);

    if rt == 0b1111 {
        bitcode_l_ex!(
            sub_bitcode,
            "00 000000 ****",
            "00 000000 1111",
            nop_32(bytecode32, system)
        );
    } else {
        bitcode_l!(
            sub_bitcode,
            "0* ****** 1111 ****",
            ldrh_32(bytecode32, system)
        );
    }
    0
}

fn ldrh_32(bytecode32: u32, system: &mut M0System) -> u32 {
    0
}

fn load_word(bytecode32: u32, system: &mut M0System) -> u32 {
    // Ref:DDI0403D_arm_architecture_v7m_reference_manual.pdf p.173
    println!("\t Load word");
    undefined_instruction_32(bytecode32, system)
}

fn data_processing_register_32(bytecode32: u32, system: &mut M0System) -> u32 {
    // Ref:DDI0403D_arm_architecture_v7m_reference_manual.pdf p.181
    println!("\t Data procerssing register (32bit)");
    not_impremented(system)
}

fn multiply_accumlate_absolutre_difference(bytecode32: u32, system: &mut M0System) -> u32 {
    // Ref:DDI0403D_arm_architecture_v7m_reference_manual.pdf p.186
    println!("\t Multiply, multiply accumulate, and absolute difference (32bit)");
    not_impremented(system)
}

fn long_multiply_accumlate_divide(bytecode32: u32, system: &mut M0System) -> u32 {
    // Ref:DDI0403D_arm_architecture_v7m_reference_manual.pdf p.187
    println!("\t Long multiply, long multiply accumulate, and divide (32bit)");
    not_impremented(system)
}

fn coprocessor_instructions(bytecode32: u32, system: &mut M0System) -> u32 {
    println!("\t Coprocessor (32bit)");
    not_impremented(system)
}

fn decode_error(bytecode: u16, system: &mut M0System) -> u32 {
    println!("\t DECODE_ERROR:{:08b}", bytecode);
    unpredicable(system)
}

fn found_bug(bytecode: u16, system: &mut M0System) -> u32 {
    println!("\t (SYSTEM_ERROR:FOUND BUG):{:08b}", bytecode);
    unpredicable(system)
}

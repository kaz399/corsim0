use crate::cpuflag::add_with_carry;
use crate::cpuflag::ArmV6m;
use crate::cpuflag::CalcFlags;
use crate::device::SystemMap;
use crate::device::SystemMapAccess;

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
    let bitcode: u16 = bytecode >> 8;

    println!(
        "adrs:{:08x}\t{:04x}({})",
        system.cpu.pc,
        bytecode,
        b16_fmt(bytecode)
    );
    // Ref: Thumb-2SupplementReferenceManual.pdf p.43
    match bitcode {
        0b11011110 => undefined_instruction(bytecode, system),
        0b11011111 => service_call(bytecode, system),
        0b01000111 => branch_exchange_instruction_set(bytecode, system),
        some_8bit => match some_8bit >> 2 {
            // truncate to 6bit
            0b000110 => add_substract_register(bytecode, system),
            0b000111 => add_substract_immediate(bytecode, system),
            0b010000 => data_processing_register(bytecode, system),
            0b010001 => special_data_processing(bytecode, system),
            some_6bit => match some_6bit >> 1 {
                // truncate to 5bit
                0b01001 => load_from_literal_pool(bytecode, system),
                0b01100 => store_word_immediate_offset(bytecode, system),
                0b01101 => loade_word_immediate_offset(bytecode, system),
                0b01110 => store_byte_immediate_offset(bytecode, system),
                0b01111 => loade_byte_immediate_offset(bytecode, system),
                0b10000 => store_halfward_immediate_offset(bytecode, system),
                0b10001 => load_halfward_immediate_offset(bytecode, system),
                0b10010 => store_to_stack(bytecode, system),
                0b10011 => load_from_stack(bytecode, system),
                0b10100 => add_to_pc(bytecode, system),
                0b10101 => add_to_sp(bytecode, system),
                0b11000 => store_multiple(bytecode, system),
                0b11001 => load_multiple(bytecode, system),
                0b11100 => unconditional_branch(bytecode, system),
                0b11101 => instruction_32bit_11101(bytecode, system),
                some_5bit => match some_5bit >> 1 {
                    // truncate to 4bit
                    0b0101 => load_store_register_offset(bytecode, system),
                    0b1011 => miscellaneous(bytecode, system),
                    0b1101 => conditional_branch(bytecode, system),
                    0b1111 => instruction_32bit_1111(bytecode, system),
                    some_4bit => match some_4bit >> 1 {
                        // truncate to 3bit
                        0b000 => shift_by_immediate_move_register(bytecode, system),
                        0b001 => add_substract_compare_move_immediate(bytecode, system),
                        _ => decode_error(bytecode, system),
                    },
                },
            },
        },
    }
}

fn b32_fmt(bin: u32) -> String {
    let mut bin_tmp: u32 = bin;
    let mut bin_str: String = format!("{:04b}", bin_tmp & 0b1111);
    bin_tmp = bin_tmp >> 4;

    for _i in 0..7 {
        bin_str = format!("{:04b}_{}", bin_tmp & 0b1111, bin_str);
        bin_tmp = bin_tmp >> 4;
    }
    bin_str
}

fn b16_fmt(bin: u16) -> String {
    let mut bin_tmp: u16 = bin;
    let mut bin_str: String = format!("{:04b}", bin_tmp & 0b1111);
    bin_tmp = bin_tmp >> 4;

    for _i in 0..3 {
        bin_str = format!("{:04b}_{}", bin_tmp & 0b1111, bin_str);
        bin_tmp = bin_tmp >> 4;
    }
    bin_str
}

fn bit_count(bytecode: u32) -> u32 {
    let mut count = 0;
    for i in 0..32 {
        if (bytecode & (1 << i)) == 1 {
            count += 1;
        }
    }
    count
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
    println!("\t (Undefined Instruction {:08b})", bytecode);
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

// 11101 x[12]
fn instruction_32bit_11101(bytecode: u16, system: &mut M0System) -> u32 {
    println!("\t 32-bit instruction (11101)");
    not_impremented(system)
}

// 1111 x[13]
fn instruction_32bit_1111(bytecode: u16, system: &mut M0System) -> u32 {
    // Ref: Thumb-2SupplementReferenceManual.pdf p.52
    println!("\t 32-bit instruction (1111)");
    let bytecode32: u32 = system.system_map.read32(system.cpu.pc).unwrap();
    println!("\t bytecode 32bit {:08x}", bytecode32);
    match (bytecode >> 11) & 0b1 {
        0b0 => data_processing_instructions_32(bytecode32, system),
        0b1 => load_and_store_single_data_item_32(bytecode32, system),
        _ => found_bug(bytecode, system),
    }
}

fn data_processing_instructions_32(bytecode32: u32, system: &mut M0System) -> u32 {
    not_impremented(system)
}

fn load_and_store_single_data_item_32(bytecode32: u32, system: &mut M0System) -> u32 {
    // Ref: Thumb-2SupplementReferenceManual.pdf p.66
    println!("\t Load and store single data item, and memory hints");
    let load: u32 = (bytecode32 >> 20) & 0b1;
    let signed: u32 = (bytecode32 >> 24) & 0b1;
    let upward: u32 = (bytecode32 >> 23) & 0b1;
    let size: u32 = (bytecode32 >> 21) & 0b11;
    let regnum_base: u32 = (bytecode32 >> 16) & 0b1111; // Rn
    let regnum_target: u32 = (bytecode32 >> 12) & 0b1111; // Rt
    let regnum_offset: u32 = bytecode32 & 0b1111; // Rm
    let imm: u32 = bytecode32 & 0b111111111111;

    match regnum_base {
        // 11111 00 S U size[2] 1 1111 Rt[4] imm12[12]
        0b1111 => load_store_32_format_1(), // p.186
        _ => not_impremented(system),
    }
}

fn load_store_32_format_1() -> u32 {
    0
}

fn decode_error(bytecode: u16, system: &mut M0System) -> u32 {
    println!("\t DECODE_ERROR:{:08b}", bytecode);
    unpredicable(system)
}

fn found_bug(bytecode: u16, system: &mut M0System) -> u32 {
    println!("\t (SYSTEM_ERROR:FOUND BUG):{:08b}", bytecode);
    unpredicable(system)
}

// instructions

fn unpredicable(system: &mut M0System) -> u32 {
    println!("\t UNPREDICABLE ERROR");
    0
}

fn not_impremented(system: &mut M0System) -> u32 {
    println!("\t (not impremented)");
    0
}

fn add_sp(system: &mut M0System) -> u32 {
    println!("\t add sp");
    not_impremented(system)
}

fn cbz(bytecode: u16, system: &mut M0System) -> u32 {
    println!("\t\t cbz");
    not_impremented(system)
}

fn cbnz(bytecode: u16, system: &mut M0System) -> u32 {
    println!("\t\t cbnz");
    not_impremented(system)
}

fn push(bytecode: u16, system: &mut M0System) -> u32 {
    // Ref: Thumb-2SupplementReferencemanual.pdf p.295
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
        system.cpu.pc += 2;
        1
    }
}

fn pop(bytecode: u16, system: &mut M0System) -> u32 {
    // Ref: Thumb-2SupplementReferencemanual.pdf p.293
    println!("\t pop");
    not_impremented(system)
}

fn it(bytecode: u16, system: &mut M0System) -> u32 {
    // Ref: Thumb-2SupplementReferencemanual.pdf p.176
    println!("\t it");
    not_impremented(system)
}

fn bkpt(bytecode: u16, system: &mut M0System) -> u32 {
    // Ref: Thumb-2SupplementReferencemanual.pdf p.132
    println!("\t bkpt");
    not_impremented(system)
}

fn dbg(bytecode: u16, system: &mut M0System) -> u32 {
    // Ref: Thumb-2SupplementReferencemanual.pdf p.164
    println!("\t dpg");
    not_impremented(system)
}

fn nop(bytecode: u16, system: &mut M0System) -> u32 {
    // Ref: Thumb-2SupplementReferencemanual.pdf p.273
    println!("\t nop");
    system.cpu.pc += 2;
    1
}

fn cpu_yield(bytecode: u16, system: &mut M0System) -> u32 {
    // Ref: Thumb-2SupplementReferencemanual.pdf p.614
    println!("\t yield");
    not_impremented(system)
}

fn wfe(bytecode: u16, system: &mut M0System) -> u32 {
    // Ref: Thumb-2SupplementReferencemanual.pdf p.610
    println!("\t wfe");
    not_impremented(system)
}

fn wfi(bytecode: u16, system: &mut M0System) -> u32 {
    // Ref: Thumb-2SupplementReferencemanual.pdf p.612
    println!("\t wfi");
    not_impremented(system)
}

fn sev(bytecode: u16, system: &mut M0System) -> u32 {
    // Ref: Thumb-2SupplementReferencemanual.pdf p.596
    println!("\t sev");
    not_impremented(system)
}

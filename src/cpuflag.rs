#[derive(Debug)]
pub struct ArmV6m {
    pub result: u32,
    pub n: u32,
    pub z: u32,
    pub c: u32,
    pub v: u32,
    pub q: u32,
    pub apsr: u32,
}

impl Default for ArmV6m {
    fn default() -> Self {
        Self {
            result: 0,
            n: 0,
            z: 0,
            c: 0,
            v: 0,
            q: 0,
            apsr: 0,
        }
    }
}

pub trait CalcFlags {
    fn new(apsr: u32) -> ArmV6m;
    fn flags_to_apsr(self) -> u32;
    fn cond(self, cond: u32) -> (bool, &'static str);
}

impl CalcFlags for ArmV6m {
    fn new(apsr: u32) -> ArmV6m {
        ArmV6m {
            result: 0,
            n: (apsr >> 31) & 0b1,
            z: (apsr >> 30) & 0b1,
            c: (apsr >> 29) & 0b1,
            v: (apsr >> 28) & 0b1,
            q: (apsr >> 27) & 0b1,
            apsr: apsr,
        }
    }

    fn flags_to_apsr(self) -> u32 {
        let mut apsr: u32 =
            &(self.n << 31) | &(self.z << 30) | &(self.c << 29) | &(self.v << 28) | &(self.q << 27);
        apsr |= &self.apsr & 0b0000011111111111111111111111111;
        apsr
    }

    fn cond(self, cond: u32) -> (bool, &'static str) {
        match cond & 0b1111 {
            0b0000 => (self.z == 1, "eq"),
            0b0001 => (self.z == 0, "ne"),
            0b0010 => (self.c == 1, "cs"),
            0b0011 => (self.c == 0, "cc"),
            0b0100 => (self.n == 1, "mi"),
            0b0101 => (self.n == 0, "pl"),
            0b0110 => (self.v == 1, "vs"),
            0b0111 => (self.v == 0, "vc"),
            0b1000 => (self.c == 1 && self.z == 0, "hi"),
            0b1001 => (self.c == 0 && self.z == 1, "ls"),
            0b1010 => (self.n == self.v, "ge"),
            0b1011 => (self.n != self.v, "lt"),
            0b1100 => ((self.z == 0 && (self.n != self.v)), "gt"),
            0b1101 => ((self.z == 1 && (self.n == self.v)), "le"),
            0b1110 => (true, "al"),
            _ => (false, "*UNDEFINED*"),
        }
    }
}

#[derive(Debug)]
pub struct IfThenFlags {
    pub cond: u32,
    pub encode: u32,
    pub epsr: u32,
    pub flags: ArmV6m,
}

impl Default for IfThenFlags {
    fn default() -> Self {
        Self {
            cond: 0,
            encode: 0,
            epsr: 0,
            flags: { ArmV6m::default() },
        }
    }
}

pub trait IfThenCtrl {
    fn new(apsr: u32, epsr: u32) -> IfThenFlags;
    fn in_it_block(self) -> bool;
    fn last_in_it_block(self) -> bool;
    fn update_epsr(&mut self) -> u32;
    fn cond(self) -> (bool, &'static str);
}

impl IfThenCtrl for IfThenFlags {
    fn new(apsr: u32, epsr: u32) -> IfThenFlags {
        let upper_bit: u32 = (epsr >> 25) & 0b11;
        let lower_bit: u32 = (epsr >> 10) & 0b111111;
        let itstate: u32 = (upper_bit << 6) | lower_bit;
        let mut it: IfThenFlags = IfThenFlags {
            cond: (itstate >> 5) & 0b111,
            encode: 0,
            epsr: epsr,
            flags: ArmV6m {
                ..ArmV6m::new(apsr)
            },
        };
        if it.cond == 0b000 {
            it.encode = 0b00000;
        } else {
            it.encode = (itstate & 0b11110) | 0b00001;
        }
        it
    }

    fn in_it_block(self) -> bool {
        self.encode & 0b01111 != 0
    }

    fn last_in_it_block(self) -> bool {
        self.encode & 0b01111 == 0b01000
    }

    fn update_epsr(&mut self) -> u32 {
        if self.encode & 0b1111 == 0b01000 {
            self.cond = 0b000;
            self.encode = 0b00000;
        } else {
            self.encode = (self.encode << 1) & 0b11111;
        }
        let itstate: u32 = (self.cond << 5) | self.encode;
        self.epsr = (itstate & 0b1100000) << 20;
        self.epsr |= (itstate & 0b0011111) << 10;
        self.epsr
    }

    fn cond(self) -> (bool, &'static str) {
        let encode_bit = self.encode & 0b1;
        let cond = self.cond << 1 | encode_bit;
        self.flags.cond(cond)
    }
}

pub fn add_with_carry(a: u32, b: u32, carry: u32) -> ArmV6m {
    let a64: i64 = a as i64;
    let b64: i64 = b as i64;
    let c64: i64 = carry as i64;
    let r64: i64 = a64 + b64 + c64;
    let mut r: ArmV6m = ArmV6m::default();

    r.result = (r64 & 0xffffffff) as u32;
    match r64 {
        val if val < 0 => r.n = 1,
        val if val == 0 => r.z = 1,
        val if (val & 0x00010000) != 0 => r.c = 1,
        val if (a & 0x80000000) != (val & 0x80000000) as u32 => r.v = 1,
        _ => (),
    }
    r.apsr = (r.n << 31) | (r.z << 30) | (r.c << 29) | (r.v << 28) | (r.q << 27);
    println!("\t Result:{:08x}\n\t {:?}", r.result, r);
    r
}

#[test]
fn test_arm_v6_flags_1() {
    let flag: ArmV6m = { ArmV6m::default() };
    println!("{:?}", flag);
    assert_eq!(flag.result, 0);
    assert_eq!(flag.n, 0);
    assert_eq!(flag.z, 0);
    assert_eq!(flag.c, 0);
    assert_eq!(flag.v, 0);
    assert_eq!(flag.q, 0);
    assert_eq!(flag.apsr, 0);
    let flag1: u32 = flag.apsr;
    let flag2: u32 = flag.flags_to_apsr();
    assert_eq!(flag1, flag2);
}

#[test]
fn test_arm_v6_flags_2() {
    let flag: ArmV6m = { ArmV6m::new(0) };
    println!("{:?}", flag);
    assert_eq!(flag.result, 0);
    assert_eq!(flag.n, 0);
    assert_eq!(flag.z, 0);
    assert_eq!(flag.c, 0);
    assert_eq!(flag.v, 0);
    assert_eq!(flag.q, 0);
    assert_eq!(flag.apsr, 0);
    let flag1: u32 = flag.apsr;
    let flag2: u32 = flag.flags_to_apsr();
    assert_eq!(flag1, flag2);
}

#[test]
fn test_arm_v6_flags_3() {
    let test_pattern: u32 = 0b0000011111111111111111111111111;
    let flag: ArmV6m = { ArmV6m::new(test_pattern) };
    println!("{:?}", flag);
    assert_eq!(flag.result, 0);
    assert_eq!(flag.n, 0);
    assert_eq!(flag.z, 0);
    assert_eq!(flag.c, 0);
    assert_eq!(flag.v, 0);
    assert_eq!(flag.q, 0);
    assert_eq!(flag.apsr, test_pattern);
    let flag1: u32 = flag.apsr;
    let flag2: u32 = flag.flags_to_apsr();
    assert_eq!(flag1, flag2);
}

#[test]
fn test_arm_v6_flags_4() {
    let test_pattern: u32 = 0b1 << 31;
    let flag: ArmV6m = { ArmV6m::new(test_pattern) };
    println!("{:?}", flag);
    assert_eq!(flag.result, 0);
    assert_eq!(flag.n, 1);
    assert_eq!(flag.z, 0);
    assert_eq!(flag.c, 0);
    assert_eq!(flag.v, 0);
    assert_eq!(flag.q, 0);
    assert_eq!(flag.apsr, test_pattern);
    let flag1: u32 = flag.apsr;
    let flag2: u32 = flag.flags_to_apsr();
    assert_eq!(flag1, flag2);
}

#[test]
fn test_arm_v6_flags_5() {
    let test_pattern: u32 = 0b1 << 30;
    let flag: ArmV6m = { ArmV6m::new(test_pattern) };
    println!("{:?}", flag);
    assert_eq!(flag.result, 0);
    assert_eq!(flag.n, 0);
    assert_eq!(flag.z, 1);
    assert_eq!(flag.c, 0);
    assert_eq!(flag.v, 0);
    assert_eq!(flag.q, 0);
    assert_eq!(flag.apsr, test_pattern);
    let flag1: u32 = flag.apsr;
    let flag2: u32 = flag.flags_to_apsr();
    assert_eq!(flag1, flag2);
}

#[test]
fn test_arm_v6_flags_6() {
    let test_pattern: u32 = 0b1 << 29;
    let flag: ArmV6m = { ArmV6m::new(test_pattern) };
    println!("{:?}", flag);
    assert_eq!(flag.result, 0);
    assert_eq!(flag.n, 0);
    assert_eq!(flag.z, 0);
    assert_eq!(flag.c, 1);
    assert_eq!(flag.v, 0);
    assert_eq!(flag.q, 0);
    assert_eq!(flag.apsr, test_pattern);
    let flag1: u32 = flag.apsr;
    let flag2: u32 = flag.flags_to_apsr();
    assert_eq!(flag1, flag2);
}

#[test]
fn test_arm_v6_flags_7() {
    let test_pattern: u32 = 0b1 << 28;
    let flag: ArmV6m = { ArmV6m::new(test_pattern) };
    println!("{:?}", flag);
    assert_eq!(flag.result, 0);
    assert_eq!(flag.n, 0);
    assert_eq!(flag.z, 0);
    assert_eq!(flag.c, 0);
    assert_eq!(flag.v, 1);
    assert_eq!(flag.q, 0);
    assert_eq!(flag.apsr, test_pattern);
    let flag1: u32 = flag.apsr;
    let flag2: u32 = flag.flags_to_apsr();
    assert_eq!(flag1, flag2);
}

#[test]
fn test_arm_v6_flags_8() {
    let test_pattern: u32 = 0b1 << 27;
    let flag: ArmV6m = { ArmV6m::new(test_pattern) };
    println!("{:?}", flag);
    assert_eq!(flag.result, 0);
    assert_eq!(flag.n, 0);
    assert_eq!(flag.z, 0);
    assert_eq!(flag.c, 0);
    assert_eq!(flag.v, 0);
    assert_eq!(flag.q, 1);
    assert_eq!(flag.apsr, test_pattern);
    let flag1: u32 = flag.apsr;
    let flag2: u32 = flag.flags_to_apsr();
    assert_eq!(flag1, flag2);
}

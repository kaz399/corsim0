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
    fn flags_to_apsr(&self) -> u32;
    fn cond(&self, cond: u32) -> (bool, String);
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

    fn flags_to_apsr(&self) -> u32 {
        let mut apsr: u32 =
            self.n << 31 | self.z << 30 | self.c << 29 | self.v << 28 | self.q << 27;
        apsr |= self.apsr & 0b0000011111111111111111111111111;
        apsr
    }

    fn cond(&self, cond: u32) -> (bool, String) {
        match cond & 0b1111 {
            0b0000 => (self.z == 1, "eq".to_string()),
            0b0001 => (self.z == 0, "ne".to_string()),
            0b0010 => (self.c == 1, "cs".to_string()),
            0b0011 => (self.c == 0, "cc".to_string()),
            0b0100 => (self.n == 1, "mi".to_string()),
            0b0101 => (self.n == 0, "pl".to_string()),
            0b0110 => (self.v == 1, "vs".to_string()),
            0b0111 => (self.v == 0, "vc".to_string()),
            0b1000 => (self.c == 1 && self.z == 0, "hi".to_string()),
            0b1001 => (self.c == 0 && self.z == 1, "ls".to_string()),
            0b1010 => (self.n == self.v, "ge".to_string()),
            0b1011 => (self.n != self.v, "lt".to_string()),
            0b1100 => (self.z == 0 && self.n != self.v, "gt".to_string()),
            0b1101 => (self.z == 1 && self.n == self.v, "le".to_string()),
            0b1110 => (true, "al".to_string()),
            _ => (false, "*UNDEFINED*".to_string()),
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
    fn in_it_block(&self) -> bool;
    fn last_in_it_block(&self) -> bool;
    fn update_epsr(&mut self) -> u32;
    fn cond(&self) -> (bool, String);
}

impl IfThenCtrl for IfThenFlags {
    fn new(apsr: u32, epsr: u32) -> IfThenFlags {
        let upper_bit: u32 = (epsr >> 25) & 0b11;
        let lower_bit: u32 = (epsr >> 10) & 0b111111;
        let itstate: u32 = (upper_bit << 6) | lower_bit;
        let mut encode: u32 = 0b00000;
        if (itstate >> 5) & 0b111 != 0b000 {
            encode = (itstate & 0b11110) | 0b00001;
        }
        IfThenFlags {
            cond: (itstate >> 5) & 0b111,
            encode: encode,
            epsr: epsr,
            flags: ArmV6m {
                ..ArmV6m::new(apsr)
            },
        }
    }

    fn in_it_block(&self) -> bool {
        self.encode & 0b01111 != 0
    }

    fn last_in_it_block(&self) -> bool {
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

    fn cond(&self) -> (bool, String) {
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

#[cfg(test)]
mod tests {
    use super::*;

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
        let flag1: u32 = flag.flags_to_apsr();
        let flag2: u32 = flag.apsr;
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
        let flag1: u32 = flag.flags_to_apsr();
        let flag2: u32 = flag.apsr;
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
        let flag1: u32 = flag.flags_to_apsr();
        let flag2: u32 = flag.apsr;
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
        let flag1: u32 = flag.flags_to_apsr();
        let flag2: u32 = flag.apsr;
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
        let flag1: u32 = flag.flags_to_apsr();
        let flag2: u32 = flag.apsr;
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
        let flag1: u32 = flag.flags_to_apsr();
        let flag2: u32 = flag.apsr;
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
        let flag1: u32 = flag.flags_to_apsr();
        let flag2: u32 = flag.apsr;
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
        let flag1: u32 = flag.flags_to_apsr();
        let flag2: u32 = flag.apsr;
        assert_eq!(flag1, flag2);
    }

    #[test]
    fn test_arm_v6_cond_1() {
        let flag: ArmV6m = { ArmV6m::new(0) };
        let (_, cond_str) = flag.cond(0b0000);
        assert_eq!(cond_str, "eq");
        let (_, cond_str) = flag.cond(0b0001);
        assert_eq!(cond_str, "ne");
        let (_, cond_str) = flag.cond(0b0010);
        assert_eq!(cond_str, "cs");
        let (_, cond_str) = flag.cond(0b0011);
        assert_eq!(cond_str, "cc");
        let (_, cond_str) = flag.cond(0b0100);
        assert_eq!(cond_str, "mi");
        let (_, cond_str) = flag.cond(0b0101);
        assert_eq!(cond_str, "pl");
        let (_, cond_str) = flag.cond(0b0110);
        assert_eq!(cond_str, "vs");
        let (_, cond_str) = flag.cond(0b0111);
        assert_eq!(cond_str, "vc");
        let (_, cond_str) = flag.cond(0b1000);
        assert_eq!(cond_str, "hi");
        let (_, cond_str) = flag.cond(0b1001);
        assert_eq!(cond_str, "ls");
        let (_, cond_str) = flag.cond(0b1010);
        assert_eq!(cond_str, "ge");
        let (_, cond_str) = flag.cond(0b1011);
        assert_eq!(cond_str, "lt");
        let (_, cond_str) = flag.cond(0b1100);
        assert_eq!(cond_str, "gt");
        let (_, cond_str) = flag.cond(0b1101);
        assert_eq!(cond_str, "le");
        let (_, cond_str) = flag.cond(0b1110);
        assert_eq!(cond_str, "al");
        let (_, cond_str) = flag.cond(0b1111);
        assert_eq!(cond_str, "*UNDEFINED*");
    }

    #[test]
    fn test_arm_v6_cond_2() {
        let test_cond: u32 = 0b0010;
        let test_not_cond: u32 = &test_cond | 1;
        let mut flag: ArmV6m = { ArmV6m::new(0) };
        flag.c = 0;
        let (tf, _) = flag.cond(test_cond);
        assert_eq!(tf, false);
        let (tf, _) = flag.cond(test_not_cond);
        assert_eq!(tf, true);
        flag.c = 1;
        let (tf, _) = flag.cond(test_cond);
        assert_eq!(tf, true);
        let (tf, _) = flag.cond(test_not_cond);
        assert_eq!(tf, false);
    }

    #[test]
    fn test_arm_v6_cond_4() {
        let test_cond: u32 = 0b0100;
        let test_not_cond: u32 = &test_cond | 1;
        let mut flag: ArmV6m = { ArmV6m::new(0) };
        flag.n = 0;
        let (tf, _) = flag.cond(test_cond);
        assert_eq!(tf, false);
        let (tf, _) = flag.cond(test_not_cond);
        assert_eq!(tf, true);
        flag.n = 1;
        let (tf, _) = flag.cond(test_cond);
        assert_eq!(tf, true);
        let (tf, _) = flag.cond(test_not_cond);
        assert_eq!(tf, false);
    }

    #[test]
    fn test_arm_v6_cond_5() {
        let test_cond: u32 = 0b0110;
        let test_not_cond: u32 = &test_cond | 1;
        let mut flag: ArmV6m = { ArmV6m::new(0) };
        flag.v = 0;
        let (tf, _) = flag.cond(test_cond);
        assert_eq!(tf, false);
        let (tf, _) = flag.cond(test_not_cond);
        assert_eq!(tf, true);
        flag.v = 1;
        let (tf, _) = flag.cond(test_cond);
        assert_eq!(tf, true);
        let (tf, _) = flag.cond(test_not_cond);
        assert_eq!(tf, false);
    }

    #[test]
    fn test_arm_v6_cond_3() {
        let test_cond: u32 = 0;
        let test_not_cond: u32 = &test_cond | 1;
        let mut flag: ArmV6m = { ArmV6m::new(0) };
        flag.z = 0;
        let (tf, _) = flag.cond(test_cond);
        assert_eq!(tf, false);
        let (tf, _) = flag.cond(test_not_cond);
        assert_eq!(tf, true);
        flag.z = 1;
        let (tf, _) = flag.cond(test_cond);
        assert_eq!(tf, true);
        let (tf, _) = flag.cond(test_not_cond);
        assert_eq!(tf, false);
    }

    #[test]
    fn test_if_then_1() {
        let if_then: IfThenFlags = { IfThenFlags::default() };
        assert_eq!(if_then.cond, 0);
        assert_eq!(if_then.encode, 0);
        assert_eq!(if_then.epsr, 0);

        assert_eq!(if_then.flags.result, 0);
        assert_eq!(if_then.flags.n, 0);
        assert_eq!(if_then.flags.z, 0);
        assert_eq!(if_then.flags.c, 0);
        assert_eq!(if_then.flags.v, 0);
        assert_eq!(if_then.flags.q, 0);
        assert_eq!(if_then.flags.apsr, 0);
        let flag1: u32 = if_then.flags.flags_to_apsr();
        let flag2: u32 = if_then.flags.apsr;
        assert_eq!(flag1, flag2);
    }

    #[test]
    fn test_if_then_2() {
        let if_then: IfThenFlags = { IfThenFlags::new(0, 0) };
        assert_eq!(if_then.cond, 0);
        assert_eq!(if_then.encode, 0);
        assert_eq!(if_then.epsr, 0);
    }
}

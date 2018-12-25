use std::collections::HashMap;

pub const NO_COND: &str = "";

pub trait BitCode {
    fn new(val: u64) -> Self;
    fn to_64(&self) -> u64;
    fn bit_width(&self) -> usize;
}

impl BitCode for u8 {
    fn new(val: u64) -> Self {
        val as u8
    }

    fn to_64(&self) -> u64 {
        self.clone() as u64
    }

    fn bit_width(&self) -> usize {
        8
    }
}

impl BitCode for u16 {
    fn new(val: u64) -> Self {
        val as u16
    }

    fn to_64(&self) -> u64 {
        self.clone() as u64
    }

    fn bit_width(&self) -> usize {
        16
    }
}

impl BitCode for u32 {
    fn new(val: u64) -> Self {
        val as u32
    }

    fn to_64(&self) -> u64 {
        self.clone() as u64
    }

    fn bit_width(&self) -> usize {
        32
    }
}

impl BitCode for u64 {
    fn new(val: u64) -> Self {
        val as u64
    }

    fn to_64(&self) -> u64 {
        self.clone()
    }

    fn bit_width(&self) -> usize {
        64
    }
}

pub fn bitmatch_upper<T: BitCode>(bitcode: &T, pattern: &str) -> bool {
    if pattern.len() == 0 {
        return false;
    }
    let normalized_pattern: String = pattern.replace(" ", "");
    let width: usize = bitcode.bit_width();
    let bitcode64: u64 = bitcode.to_64() as u64;
    let mut code: u64 = 0;
    let mut mask: u64 = 0;
    let mut pattern_length: usize = 0;
    let mut enable_shift: bool = false;
    for c in normalized_pattern.chars() {
        if enable_shift {
            code = code << 1;
            mask = mask << 1;
        }
        match c {
            '0' => {
                code = code | 0b0;
                mask = mask | 0b1;
                enable_shift = true;
                pattern_length += 1;
            }
            '1' => {
                code = code | 0b1;
                mask = mask | 0b1;
                enable_shift = true;
                pattern_length += 1;
            }
            ' ' | '|' => enable_shift = false,
            _ => {
                code = code | 0b0;
                mask = mask | 0b0;
                enable_shift = true;
                pattern_length += 1;
            }
        }
    }
    code = code << (width - pattern_length);
    mask = mask << (width - pattern_length);
    (bitcode64 & mask) == code
}

pub fn bitmatch_lower<T: BitCode>(bitcode: &T, pattern: &str) -> bool {
    if pattern.len() == 0 {
        return false;
    }
    let normalized_pattern: String = pattern.replace(" ", "");
    let bitcode64: u64 = bitcode.to_64() as u64;
    let mut code: u64 = 0;
    let mut mask: u64 = 0;
    let mut enable_shift: bool = false;
    for c in normalized_pattern.chars() {
        if enable_shift {
            code = code << 1;
            mask = mask << 1;
        }
        match c {
            '0' => {
                code = code | 0b0;
                mask = mask | 0b1;
                enable_shift = true;
            }
            '1' => {
                code = code | 0b1;
                mask = mask | 0b1;
                enable_shift = true;
            }
            ' ' | '|' => enable_shift = false,
            _ => {
                code = code | 0b0;
                mask = mask | 0b0;
                enable_shift = true;
            }
        }
    }
    (bitcode64 & mask) == code
}

pub fn check_bitcode_upper<T: BitCode>(bitcode: &T, cond: &str, exclude: &str) -> bool {
    let cond_list = cond.split("|");
    let exclude_list = exclude.split("|");

    for ex in exclude_list {
        // println!("ex:{}", ex);
        if bitmatch_upper(bitcode, ex) == true {
            return false;
        }
    }
    for cd in cond_list {
        // println!("cd:{}", cd);
        if bitmatch_upper(bitcode, cd) == true {
            return true;
        }
    }
    false
}

pub fn parse_bit<T: BitCode>(bitcode: &T, format: &str) -> Result<HashMap<String, T>, bool> {
    let mut result: HashMap<String, T> = HashMap::new();
    let normalized_format: String = format.replace(" ", "");
    let bitcode64: u64 = bitcode.to_64();
    let mut current_bit: usize = bitcode.bit_width() - 1;

    if normalized_format.len() != bitcode.bit_width() {
        return Err(false);
    }

    // let mut bit_count: usize = 0;
    for c in normalized_format.chars() {
        // print!(
        //     "bit{}: {} --- ",
        //     bit_count,
        //     (bitcode64 >> current_bit) & 0b1
        // );
        // bit_count += 1;
        match c {
            '0' | '1' | '_' | ' ' | '|' => (),
            key => match result.get_mut(&key.to_string()) {
                Some(value) => {
                    let mut value64 = value.to_64();
                    // print!("update '{}':{} -> ", key, value64);
                    value64 = (value64 << 1) | ((bitcode64 >> current_bit) & 0b1);
                    // println!("{}", value64);
                    *value = T::new(value64)
                }
                None => {
                    // println!("new '{}':{} -> ", key, (bitcode64 >> current_bit) & 0b1);
                    result.insert(key.to_string(), T::new((bitcode64 >> current_bit) & 0b1));
                }
            },
        }
        if current_bit > 0 {
            current_bit -= 1;
        }
    }

    Ok(result)
}

#[macro_export]
macro_rules! bitcode {
    ($bitcode:expr, $bitmatch:expr, $func:expr) => {
        if check_bitcode_upper(&$bitcode, $bitmatch, NO_COND) {
            return $func;
        }
    };
}

#[macro_export]
macro_rules! bitcode_ex {
    ($bitcode:expr, $bitmatch:expr, $bitexclude:expr, $func:expr) => {
        if check_bitcode_upper(&$bitcode, $bitmatch, $bitexclude) {
            return $func;
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitmach_16_1() {
        let bitcode: u16 = 0b1;
        assert_eq!(bitmatch_upper(&bitcode, "_______________1"), true);
        assert_eq!(bitmatch_upper(&bitcode, "____ ____ ____ ___1"), true);
        assert_eq!(
            bitmatch_upper(&bitcode, "_ _ _ _ _ _ _ _ _ _ _ _ _ _ _1"),
            true
        );
        assert_eq!(
            bitmatch_upper(
                &bitcode,
                "  _   _   _   _   _   _   _   _   _   _   _   _   _   _ _  1"
            ),
            true
        );
        assert_eq!(
            bitmatch_upper(
                &bitcode,
                "  _   _   _   _   _   _   _   _   _   _   _   _   _   _ _  1    "
            ),
            true
        );
        assert_eq!(bitmatch_upper(&bitcode, "_______________0"), false);
        assert_eq!(bitmatch_upper(&bitcode, "________________"), true);

        for i in 1..16 {
            assert_eq!(bitmatch_upper(&(bitcode << i), "_______________1"), false);
            assert_eq!(bitmatch_upper(&(bitcode << i), "_______________0"), true);
            assert_eq!(bitmatch_upper(&(bitcode << i), "________________"), true);
        }
    }

    #[test]
    fn test_bitmach_16_2() {
        let bitcode: u16 = 0b1111111111111111;
        assert_eq!(bitmatch_upper(&bitcode, "1111111111111111"), true);
        assert_eq!(bitmatch_upper(&bitcode, "0000000000000000"), false);

        assert_eq!(bitmatch_upper(&bitcode, "1_1_1_1_1_1_1_1_"), true);
        assert_eq!(bitmatch_upper(&bitcode, "0_0_0_0_0_0_0_0_"), false);

        let bitcode: u16 = 0b0101010101010101;
        assert_eq!(bitmatch_upper(&bitcode, "1_1_1_1_1_1_1_1_"), false);
        assert_eq!(bitmatch_upper(&bitcode, "0_0_0_0_0_0_0_0_"), true);

        let bitcode_s: u16 = bitcode << 1;
        assert_eq!(bitmatch_upper(&bitcode_s, "1_1_1_1_1_1_1_1_"), true);
        assert_eq!(bitmatch_upper(&bitcode_s, "0_0_0_0_0_0_0_0_"), false);

        let bitcode_s: u16 = bitcode << 8;
        assert_eq!(bitmatch_upper(&bitcode_s, "1_1_1_1_________"), false);
        assert_eq!(bitmatch_upper(&bitcode_s, "0_0_0_0_________"), true);

        let bitcode_s: u16 = bitcode << 9;
        assert_eq!(bitmatch_upper(&bitcode_s, "1_1_1_1_________"), true);
        assert_eq!(bitmatch_upper(&bitcode_s, "0_0_0_0_________"), false);
    }

    #[test]
    fn test_bitmach_16_3() {
        let bitcode: u16 = 0b1111111100000000;
        assert_eq!(bitmatch_upper(&bitcode, "11111111"), true);
        assert_eq!(bitmatch_lower(&bitcode, "11111111"), false);
        assert_eq!(bitmatch_upper(&bitcode, "00000000"), false);
        assert_eq!(bitmatch_lower(&bitcode, "00000000"), true);
    }

    #[test]
    fn test_bitmach_32_1() {
        let bitcode: u32 = 0b1;
        assert_eq!(
            bitmatch_upper(&bitcode, "_______________________________1"),
            true
        );
        assert_eq!(
            bitmatch_upper(&bitcode, "_______________________________0"),
            false
        );
        assert_eq!(
            bitmatch_upper(&bitcode, "________________________________"),
            true
        );

        for i in 1..32 {
            assert_eq!(
                bitmatch_upper(&(bitcode << i), "_______________________________1"),
                false
            );
            assert_eq!(
                bitmatch_upper(&(bitcode << i), "_______________________________0"),
                true
            );
            assert_eq!(
                bitmatch_upper(&(bitcode << i), "________________________________"),
                true
            );
        }
    }

    #[test]
    fn test_bitmach_32_2() {
        let bitcode: u32 = 0b11111111111111110000000000000000;
        assert_eq!(bitmatch_upper(&bitcode, "1111111111111111"), true);
        assert_eq!(bitmatch_upper(&bitcode, "0000000000000000"), false);

        assert_eq!(bitmatch_upper(&bitcode, "1_1_1_1_1_1_1_1_"), true);
        assert_eq!(bitmatch_upper(&bitcode, "0_0_0_0_0_0_0_0_"), false);

        let bitcode: u32 = 0b01010101010101010000000000000000;
        assert_eq!(bitmatch_upper(&bitcode, "1_1_1_1_1_1_1_1_"), false);
        assert_eq!(bitmatch_upper(&bitcode, "0_0_0_0_0_0_0_0_"), true);
        assert_eq!(bitmatch_upper(&(bitcode << 1), "1_1_1_1_1_1_1_1_"), true);
        assert_eq!(bitmatch_upper(&(bitcode << 1), "0_0_0_0_0_0_0_0_"), false);
        assert_eq!(bitmatch_upper(&(bitcode << 8), "1_1_1_1_________"), false);
        assert_eq!(bitmatch_upper(&(bitcode << 8), "0_0_0_0_________"), true);
        assert_eq!(bitmatch_upper(&(bitcode << 9), "1_1_1_1_________"), true);
        assert_eq!(bitmatch_upper(&(bitcode << 9), "0_0_0_0_________"), false);
    }

    #[test]
    fn test_bitmach_32_3() {
        let bitcode: u32 = 0b11111111101010100101010100000000;
        assert_eq!(bitmatch_upper(&bitcode, "11111111"), true);
        assert_eq!(bitmatch_lower(&bitcode, "11111111"), false);
        assert_eq!(bitmatch_upper(&bitcode, "00000000"), false);
        assert_eq!(bitmatch_lower(&bitcode, "00000000"), true);
    }

    #[test]
    fn test_check_bitcode_upper_1() {
        let bitcode: u32 = 0b11111111101010100101010100000000;
        assert_eq!(check_bitcode_upper(&bitcode, "11111111", ""), true);
        assert_eq!(check_bitcode_upper(&bitcode, "11111111", NO_COND), true);
        assert_eq!(
            check_bitcode_upper(&bitcode, "11111111", "1111111111"),
            true
        );
        assert_eq!(
            check_bitcode_upper(&bitcode, "11111111", "0000000000"),
            true
        );
        assert_eq!(
            check_bitcode_upper(&bitcode, "11111111", "1111111111|0000000000"),
            true
        );
        assert_eq!(
            check_bitcode_upper(&bitcode, "11111111", "11111111 11|00000000 00"),
            true
        );
        assert_eq!(
            check_bitcode_upper(&bitcode, "11111111", "1111111110"),
            false
        );
        assert_eq!(
            check_bitcode_upper(&bitcode, "11111111", "11111111 11|00000000 00|11111111 10"),
            false
        );
        assert_eq!(
            check_bitcode_upper(
                &bitcode,
                "11111111",
                "  11111111 11|00000000 00|11111111 10  "
            ),
            false
        );
    }

    #[test]
    fn test_parse_bit_1() {
        let bitcode: u32 = 0b11111111101010100101010100000000;
        match parse_bit(&bitcode, "aaaaaaaabbbbbbbbccccccccdddddddd") {
            Ok(capture) => {
                println!("{:?}", capture);
                assert_eq!(capture["a"], 0b11111111);
                assert_eq!(capture["b"], 0b10101010);
                assert_eq!(capture["c"], 0b01010101);
                assert_eq!(capture["d"], 0b00000000);
                assert_eq!(capture.get("e"), None);
            }
            Err(e) => {
                assert_eq!(e, true);
            }
        }
    }

    #[test]
    fn test_parse_bit_2() {
        let bitcode: u16 = 0b1111111100000000;
        match parse_bit(&bitcode, "aaaaaaaadddddddd") {
            Ok(capture) => {
                println!("{:?}", capture);
                assert_eq!(capture["a"], 0b11111111);
                assert_eq!(capture.get("b"), None);
                assert_eq!(capture.get("c"), None);
                assert_eq!(capture["d"], 0b00000000);
                assert_eq!(capture.get("e"), None);
            }
            Err(e) => {
                assert_eq!(e, true);
            }
        }
    }

    #[test]
    fn test_parse_bit_3() {
        let bitcode: u8 = 0b00001111;
        match parse_bit(&bitcode, "____aaaa") {
            Ok(capture) => {
                println!("{:?}", capture);
                assert_eq!(capture["a"], 0b1111);
                assert_eq!(capture.get("b"), None);
                assert_eq!(capture.get("_"), None);
            }
            Err(e) => {
                assert_eq!(e, true);
            }
        }
    }

    #[test]
    fn test_parse_bit_4() {
        let bitcode: u8 = 0b00001111;
        match parse_bit(&bitcode, "abababab") {
            Ok(capture) => {
                println!("{:?}", capture);
                assert_eq!(capture["a"], 0b0011);
                assert_eq!(capture.get("b"), Some(&0b0011));
                assert_eq!(capture.get("_"), None);
            }
            Err(e) => {
                assert_eq!(e, true);
            }
        }
    }

    #[test]
    fn test_parse_bit_5() {
        let bitcode: u8 = 0b11001110;
        match parse_bit(&bitcode, "aa bb ccc b") {
            Ok(capture) => {
                println!("{:?}", capture);
                assert_eq!(capture["a"], 0b11);
                assert_eq!(capture.get("b"), Some(&0b000));
                assert_eq!(capture.get("c"), Some(&0b111));
            }
            Err(e) => {
                assert_eq!(e, true);
            }
        }

        match parse_bit(&bitcode, "aaaabbbb") {
            Ok(capture) => {
                println!("{:?}", capture);
                assert_eq!(capture["a"], 0b1100);
                assert_eq!(capture.get("b"), Some(&0b1110));
            }
            Err(e) => {
                assert_eq!(e, true);
            }
        }

        let capture = parse_bit(&bitcode, "aaaabbbb").unwrap();
        assert_eq!(capture["a"], 0b1100);
        assert_eq!(capture.get("b"), Some(&0b1110));
    }
}

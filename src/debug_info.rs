pub fn b32_fmt(bin: u32) -> String {
    let mut bin_tmp: u32 = bin;
    let mut bin_str: String = format!("{:04b}", bin_tmp & 0b1111);
    bin_tmp = bin_tmp >> 4;

    for _i in 0..7 {
        bin_str = format!("{:04b}_{}", bin_tmp & 0b1111, bin_str);
        bin_tmp = bin_tmp >> 4;
    }
    bin_str
}

pub fn b16_fmt(bin: u16) -> String {
    let mut bin_tmp: u16 = bin;
    let mut bin_str: String = format!("{:04b}", bin_tmp & 0b1111);
    bin_tmp = bin_tmp >> 4;

    for _i in 0..3 {
        bin_str = format!("{:04b}_{}", bin_tmp & 0b1111, bin_str);
        bin_tmp = bin_tmp >> 4;
    }
    bin_str
}

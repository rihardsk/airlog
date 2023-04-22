use micromath::F32Ext;

fn u32_len(num: u32) -> u8 {
    let mut count = 0;
    let mut num = num;
    while num > 0 {
        num /= 10_u32;
        count += 1;
    }
    count
}

// TODO: fix output when value is 0
// TODO: Works only on positive values, precision must be <=4
pub fn format_u32_measurement(value: u32, pad_main: u8, unit: &str) -> heapless::String<16> {
    let mut output: heapless::String<16> = heapless::String::new();

    let int_len = u32_len(value);
    for _ in 0..(pad_main - int_len.min(pad_main)) {
        output.push_str(" ").unwrap();
    }
    ufmt::uwrite!(output, "{} {}", value, unit).unwrap();

    output
}

// TODO: Works only on positive values, precision must be <=4
pub fn format_float_measurement(
    value: f32,
    pad_main: u8,
    precision: u8,
    unit: &str,
) -> heapless::String<16> {
    let mut output: heapless::String<16> = heapless::String::new();
    let int_part = value.floor() as u32;

    let int_len = u32_len(int_part);
    for _ in 0..(pad_main - int_len.min(pad_main)) {
        output.push_str(" ").unwrap();
    }
    let mut frac_text: heapless::String<5> = heapless::String::new();
    if precision > 0 {
        frac_text.push_str(".").unwrap();
        let times = 10_u32.pow(precision as u32);
        let frac_part = (value.fract() * times as f32) as u32;
        let frac_len = u32_len(frac_part);
        for _ in 0..(precision - frac_len) {
            frac_text.push_str("0").unwrap();
        }
        if frac_part > 0 {
            ufmt::uwrite!(frac_text, "{}", frac_part).unwrap();
        }
    }
    ufmt::uwrite!(output, "{}{} {}", int_part, frac_text.as_str(), unit).unwrap();

    output
}

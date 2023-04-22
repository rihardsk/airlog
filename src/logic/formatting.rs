use micromath::F32Ext;

fn u32_len(num: u32) -> u8 {
    if num == 0 {
        return 1;
    }
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
        let frac_part = (value.fract() * times as f32).round() as u32;
        let frac_len = u32_len(frac_part);
        // defmt::info!("fract: {=f32}, frac_part {=u32}, frac_len {=u8}", value.fract(), frac_part, frac_len);
        for _ in 0..(precision - frac_len) {
            frac_text.push_str("0").unwrap();
        }
        ufmt::uwrite!(frac_text, "{}", frac_part).unwrap();
    }
    ufmt::uwrite!(output, "{}{} {}", int_part, frac_text.as_str(), unit).unwrap();

    output
}

#[cfg(test)]
pub mod tests {
    use super::format_float_measurement;
    use super::format_u32_measurement;

    pub fn format_zero() {
        let res = format_u32_measurement(0, 4, "ppm");
        assert_eq!(res.as_str(), "   0 ppm");
    }

    pub fn format_ten() {
        let res = format_u32_measurement(10, 4, "ppm");
        assert_eq!(res.as_str(), "  10 ppm");
    }

    pub fn format_single_digit() {
        let res = format_u32_measurement(2, 4, "ppm");
        assert_eq!(res.as_str(), "   2 ppm");
    }

    pub fn format_all_digits() {
        let res = format_u32_measurement(1234, 4, "ppm");
        assert_eq!(res.as_str(), "1234 ppm");
    }

    pub fn format_more_digits() {
        let res = format_u32_measurement(12345, 4, "ppm");
        assert_eq!(res.as_str(), "12345 ppm");
    }

    pub fn format_dont_pad() {
        let res = format_u32_measurement(22, 0, "ppm");
        assert_eq!(res.as_str(), "22 ppm");
    }

    pub fn format_float_zero() {
        let res = format_float_measurement(0., 2, 2, "°C");
        assert_eq!(res.as_str(), " 0.00 °C");
    }

    pub fn format_float_small_fract() {
        let res = format_float_measurement(1.01, 2, 2, "°C");
        assert_eq!(res.as_str(), " 1.01 °C");
    }

    pub fn format_float_smaller_fract() {
        let res = format_float_measurement(1.001, 2, 2, "°C");
        assert_eq!(res.as_str(), " 1.00 °C");
    }

    pub fn format_float_single_digit() {
        let res = format_float_measurement(2., 2, 2, "°C");
        assert_eq!(res.as_str(), " 2.00 °C");
    }

    pub fn format_float_more_digits() {
        let res = format_float_measurement(123.125, 2, 2, "°C");
        assert_eq!(res.as_str(), "123.13 °C");
    }
}

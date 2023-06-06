use micromath::F32Ext;

// TODO: consider refactoring if more colormaps are added

// _RdYlGn_data from https://github.com/matplotlib/matplotlib/blob/b3bd929cf07ea35479fded8f739126ccc39edd6d/lib/matplotlib/_cm.py
// (0.6470588235294118 , 0.0                 , 0.14901960784313725),
// (0.84313725490196079, 0.18823529411764706 , 0.15294117647058825),
// (0.95686274509803926, 0.42745098039215684 , 0.2627450980392157 ),
// (0.99215686274509807, 0.68235294117647061 , 0.38039215686274508),
// (0.99607843137254903, 0.8784313725490196  , 0.54509803921568623),
// (1.0                , 1.0                 , 0.74901960784313726),
// (0.85098039215686272, 0.93725490196078431 , 0.54509803921568623),
// (0.65098039215686276, 0.85098039215686272 , 0.41568627450980394),
// (0.4                , 0.74117647058823533 , 0.38823529411764707),
// (0.10196078431372549, 0.59607843137254901 , 0.31372549019607843),
// (0.0                , 0.40784313725490196 , 0.21568627450980393)

#[rustfmt::skip]
const RDYLGN_DATA: [(f32, f32, f32); 11] = [
    (0.0, 0.407_843_14, 0.215_686_28),
    (0.101_960_786, 0.596_078_46, 0.313_725_5,),
    (0.4, 0.741_176_5, 0.388_235_3),
    (0.650_980_4, 0.850_980_4, 0.415_686_28,),
    (0.850_980_4, 0.937_254_9, 0.545_098_07,),
    (1.0, 1.0, 0.749_019_6),
    (0.996_078_43, 0.878_431_4, 0.545_098_07),
    (0.992_156_86, 0.682_352_96, 0.380_392_16,),
    (0.956_862_75, 0.427_450_98, 0.262_745_1),
    (0.843_137_26, 0.188_235_3, 0.152_941_18,),
    (0.647_058_84, 0.0, 0.149_019_61),
];

#[rustfmt::skip]
const SIMPLE_DATA: [(f32, f32, f32); 4] = [
    (0., 1., 0.),
    (1., 1., 0.),
    (1., 0., 0.),
    (0., 0., 1.),
];

// TODO: make an abstraction for color position specification so that they're
// not hardcoded here

type ColorSegment = (f32, f32, f32, f32);
#[rustfmt::skip]
const CO2_COLORS: [ColorSegment; 4] = [
    // Set some points relative to some specific co2 values. We take 424 ppm as
    // base and 3000 ppm as the maximum
    (0., 1., 0., 0.),
    (1., 1., 0., ((1000. - 424.) / (3000. - 424.))),
    (1., 0., 0., ((1600. - 424.) / (3000. - 424.))),
    (0., 0., 1., 1.),
];

const VOC_COLORS: [ColorSegment; 4] = [
    // Set some points relative to some specific Sensiron VOC index values which
    // range from 0 to 500 with 100 representing average air quality
    (0., 0., 1., 0.),
    (0., 1., 0., (100. / 500.)),
    (1., 1., 0., (200. / 500.)),
    (1., 0., 0., 1.),
];

const TEMP_COLORS: [ColorSegment; 4] = [
    // TODO: make this work for negative temperatures
    //
    // Set some points relative to some specific temperatures. 45 degrees will
    // be the "maximum" temperature
    (0., 0., 1., 0.),
    (0., 1., 0., (15. / 45.)),
    (1., 1., 0., (25. / 45.)),
    (1., 0., 0., 1.),
];

const PM10_COLORS: [ColorSegment; 4] = [
    // Set some points relative to some specific particle mass concentration values. 45 degrees will
    // be the "maximum" temperature
    (0., 0., 1., 0.),
    (0., 1., 0., (10. / 50.)),
    (1., 1., 0., (20. / 50.)),
    (1., 0., 0., 1.),
];

#[rustfmt::skip]
const PRESSURE_COLORS: [ColorSegment; 5] = [
    // Globally recorded lowest (990 hPa), highest (1085 hPa) and mean (1013.25)
    // atmospheric pressures at sea level are good reference points that we
    // could use, but we'll opt for a greater resolution by selecting a smaller
    // range – we'll use the 3 year minimum/maximum temperatures in Latvia
    // (approx. 990 and 1040 hPa).
    (0., 0., 1., 0.),
    (0., 1., 1., ((1000. - 990.) / (1040. - 990.))),
    (0., 1., 0., ((1013.25 - 990.) / (1040. - 990.))),
    (1., 0., 0., ((1026. - 990.) / (1040. - 990.))),
    (1., 0., 1., 1.),
];

pub fn linear_interpolating_map(colors: &[(f32, f32, f32)], fraction: f32) -> (f32, f32, f32) {
    let max_idx = colors.len() - 1;
    let float_idx: f32 = (max_idx as f32 * fraction).min(max_idx as f32);
    let below = float_idx.floor() as usize;
    let above = float_idx.ceil() as usize;
    let remainder = float_idx - below as f32;

    let (r_below, g_below, b_below) = colors[below];
    let (r_above, g_above, b_above) = colors[above];
    let r_adjust = (r_above - r_below) * remainder;
    let g_adjust = (g_above - g_below) * remainder;
    let b_adjust = (b_above - b_below) * remainder;

    (r_below + r_adjust, g_below + g_adjust, b_below + b_adjust)
}

pub fn linear_position_interpolating_map(
    colors: &[ColorSegment],
    fraction: f32,
) -> (f32, f32, f32) {
    let mut below: Option<ColorSegment> = None;
    let mut above: Option<ColorSegment> = None;
    // Find the first distance (4th component) that's greater than the fraction
    for color in colors {
        if fraction <= color.3 {
            above = Some(*color);
            break;
        }
        below = Some(*color);
    }
    let (below, above) = match (below, above) {
        // Inputs were faulty, come up with something generic
        (None, None) => ((0., 0., 0., 0.), (1., 1., 1., 1.)),
        (None, Some(x)) => (x, x),
        (Some(x), None) => (x, x),
        (Some(x), Some(y)) => (x, y),
    };

    let (r_below, g_below, b_below, loc_below) = below;
    let (r_above, g_above, b_above, loc_above) = above;

    let interpolation = (fraction - loc_below) / (loc_above - loc_below);
    let interpolation = interpolation.min(1.).max(0.);

    let r_adjust = (r_above - r_below) * interpolation;
    let g_adjust = (g_above - g_below) * interpolation;
    let b_adjust = (b_above - b_below) * interpolation;

    (r_below + r_adjust, g_below + g_adjust, b_below + b_adjust)
}

fn fractions_to_rgb(colors: (f32, f32, f32)) -> (u8, u8, u8) {
    let (r, g, b) = colors;
    ((255. * r) as u8, (255. * g) as u8, (255. * b) as u8)
}

pub fn rdylgn_map(fraction: f32) -> (f32, f32, f32) {
    linear_interpolating_map(&RDYLGN_DATA, fraction)
}

pub fn rdylgn_map_rgb(fraction: f32) -> (u8, u8, u8) {
    fractions_to_rgb(rdylgn_map(fraction))
}

pub fn simple_map(fraction: f32) -> (f32, f32, f32) {
    linear_interpolating_map(&SIMPLE_DATA, fraction)
}

pub fn simple_map_rgb(fraction: f32) -> (u8, u8, u8) {
    fractions_to_rgb(simple_map(fraction))
}

pub fn co2_map(fraction: f32) -> (f32, f32, f32) {
    linear_position_interpolating_map(&CO2_COLORS, fraction)
}

pub fn co2_map_rgb(fraction: f32) -> (u8, u8, u8) {
    fractions_to_rgb(co2_map(fraction))
}

pub fn voc_map(fraction: f32) -> (f32, f32, f32) {
    linear_position_interpolating_map(&VOC_COLORS, fraction)
}

pub fn voc_map_rgb(fraction: f32) -> (u8, u8, u8) {
    fractions_to_rgb(voc_map(fraction))
}

pub fn temp_map(fraction: f32) -> (f32, f32, f32) {
    linear_position_interpolating_map(&TEMP_COLORS, fraction)
}

pub fn temp_map_rgb(fraction: f32) -> (u8, u8, u8) {
    fractions_to_rgb(temp_map(fraction))
}

pub fn pm10_map(fraction: f32) -> (f32, f32, f32) {
    linear_position_interpolating_map(&PM10_COLORS, fraction)
}

pub fn pm10_map_rgb(fraction: f32) -> (u8, u8, u8) {
    fractions_to_rgb(pm10_map(fraction))
}

pub fn pressure_map(fraction: f32) -> (f32, f32, f32) {
    linear_position_interpolating_map(&PRESSURE_COLORS, fraction)
}

pub fn pressure_map_rgb(fraction: f32) -> (u8, u8, u8) {
    fractions_to_rgb(pressure_map(fraction))
}

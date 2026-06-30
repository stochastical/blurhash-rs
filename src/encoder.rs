use std::f32::consts::PI;

use crate::colour_space::{linear_to_s_rgb, s_rgb_to_linear, sign_pow};
use crate::string_coding::encode83;

const NUM_CHANNELS: usize = 3;
const MIN_COMPONENTS: usize = 1;
const MAX_COMPONENTS: usize = 9;

pub fn encode_dc(r: f32, g: f32, b: f32) -> u32 {
    let (rounded_r, rounded_g, rounded_b) =
        (linear_to_s_rgb(r), linear_to_s_rgb(g), linear_to_s_rgb(b));
    (rounded_r << 16) + (rounded_g << 8) + rounded_b
}

pub fn encode_ac(r: f32, g: f32, b: f32, maximum_value: f32) -> u32 {
    let (quant_r, quant_g, quant_b) = (
        f32::max(
            0.0,
            f32::min(
                18.0,
                f32::floor(sign_pow(r / maximum_value, 0.5) * 9.0 + 9.5),
            ),
        ) as u32,
        f32::max(
            0.0,
            f32::min(
                18.0,
                f32::floor(sign_pow(g / maximum_value, 0.5) * 9.0 + 9.5),
            ),
        ) as u32,
        f32::max(
            0.0,
            f32::min(
                18.0,
                f32::floor(sign_pow(b / maximum_value, 0.5) * 9.0 + 9.5),
            ),
        ) as u32,
    );
    quant_r * 19 * 19 + quant_g * 19 + quant_b
}

pub fn multiply_basis_function(
    components: (usize, usize),
    width: usize,
    height: usize,
    pixels: &[u8],
    bytes_per_row: usize,
) -> [f32; NUM_CHANNELS] {
    let (mut r, mut g, mut b) = (0.0, 0.0, 0.0);
    let normalisation: f32 = if components.0 == 0 && components.1 == 0 {
        1.0
    } else {
        2.0
    };

    // let basis_function = |x, y| -> f32::cos(PI * (components.0 as f32) * (x as f32) / (width as f32)) * f32::cos(PI * components.1 as f32 * y as f32 / height as f32);

    for y in 0..height {
        for x in 0..width {
            let basis = f32::cos(PI * (components.0 as f32) * (x as f32) / (width as f32))
                * f32::cos(PI * components.1 as f32 * y as f32 / height as f32);

            r += basis * s_rgb_to_linear(pixels[3 * x + 0 + y * bytes_per_row].into());
            g += basis * s_rgb_to_linear(pixels[3 * x + 1 + y * bytes_per_row].into());
            b += basis * s_rgb_to_linear(pixels[3 * x + 2 + y * bytes_per_row].into());
        }
    }

    let scale = normalisation / (width as f32 * height as f32);

    [r * scale, g * scale, b * scale]
}

pub fn encode(
    components: (usize, usize),
    width: usize,
    height: usize,
    pixels: &[u8],
    bytes_per_row: usize,
) -> Option<String> {
    if components.0 < MIN_COMPONENTS
        || components.0 > MAX_COMPONENTS
        || components.1 < MIN_COMPONENTS
        || components.1 > MAX_COMPONENTS
    {
        return None;
    }

    let mut factors: Vec<Vec<[f32; NUM_CHANNELS]>> =
        vec![vec![[0.0; NUM_CHANNELS]; components.0]; components.1];

    for y in 0..(components.1 as usize) {
        for x in 0..(components.0 as usize) {
            let factor = multiply_basis_function((x, y), width, height, pixels, bytes_per_row);
            factors[y][x][0] = factor[0];
            factors[y][x][1] = factor[1];
            factors[y][x][2] = factor[2];
        }
    }

    let dc = factors[0][0];
    let ac: Vec<f32> = factors
        .iter()
        .flatten()
        .skip(1)
        .flatten()
        .copied()
        .collect();
    dbg!(&ac);
    let ac_count = components.0 * components.1 - 1;
    dbg!(ac_count);

    let mut hash = String::with_capacity(2 + 4 + (9 * 9 - 1) * 2 + 1);

    let size_flag = (components.0 - 1) + (components.1 - 1) * 9;
    encode83(size_flag as u32, 1, &mut hash);

    let maximum_value;
    if ac_count > 0 {
        let mut actual_maximum_value = 0.0;
        for i in 0..(ac_count * 3) {
            actual_maximum_value = ac[i].abs().max(actual_maximum_value)
        }

        let quantised_maximum_value = f32::max(
            0.0,
            f32::min(82.0, f32::floor(actual_maximum_value as f32 * 166.0 - 0.5)),
        ) as i32;
        maximum_value = (quantised_maximum_value as f32 + 1.0) / 166.0;
        encode83(quantised_maximum_value as u32, 1, &mut hash);
    } else {
        maximum_value = 1.0;
        encode83(0, 1, &mut hash);
    }

    encode83(encode_dc(dc[0], dc[1], dc[2]), 4, &mut hash);

    for i in 0..ac_count {
        encode83(
            encode_ac(ac[i * 3 + 0], ac[i * 3 + 1], ac[i * 3 + 2], maximum_value),
            2,
            &mut hash,
        );
    }

    Some(hash)
}

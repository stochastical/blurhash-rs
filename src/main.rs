use std::f32::consts::PI;

use image::GenericImageView;

mod colour_space;
use colour_space::{linear_to_s_rgb, s_rgb_to_linear, sign_pow};

mod encoder;
use encoder::{encode_ac, encode_dc};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let img = image::open("test/pic1.png")?;
    let (width, height) = img.dimensions();
    let bytes_per_row = width * 3;

    // img.as_bytes();
    let rgb_pixels = img.to_rgb8().into_raw();

    dbg!(blur_hash_for_pixels(
        (4, 3),
        width as usize,
        height as usize,
        &rgb_pixels,
        bytes_per_row as usize,
    ));

    Ok(())
}

const NUM_CHANNELS: usize = 3;
const CHARACTERS: &[char] = &[
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I',
    'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b',
    'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u',
    'v', 'w', 'x', 'y', 'z', '#', '$', '%', '*', '+', '-', '.', ':', ';', '=', '?', '@', '[', ']',
    '^', '_', '{', '|', '}', '~',
];

fn encode_int(value: u32, length: usize, destination: &mut String) {
    let mut divisor = 1;

    for _ in 0..(length - 1) {
        divisor *= 83;
    }

    for _ in 0..length {
        let digit = (value / divisor) % 83;
        divisor /= 83;
        destination.push(CHARACTERS[digit as usize]);
    }
}

fn blur_hash_for_pixels(
    components: (usize, usize), // TODO: usize
    width: usize,
    height: usize,
    rgb: &[u8],
    bytes_per_row: usize,
) -> Option<String> {
    let buffer = &[0; 2 + 4 + (9 * 9 - 1) * 2 + 1];

    if components.0 < 1 || components.0 > 9 || components.1 < 1 || components.1 > 9 {
        return None;
    }

    let mut factors: Vec<Vec<[f32; NUM_CHANNELS]>> =
        vec![vec![[0.0; NUM_CHANNELS]; components.0]; components.1];

    for y in 0..(components.1 as usize) {
        for x in 0..(components.0 as usize) {
            let factor = multiply_basis_function(x, y, width, height, rgb, bytes_per_row);
            factors[y][x][0] = factor[0];
            factors[y][x][1] = factor[1];
            factors[y][x][2] = factor[2];
        }
    }

    let dc = factors[0][0];
    let ac = factors[0][1];
    let ac_count = components.0 * components.1 - 1;

    let mut ptr = String::with_capacity(2 + 4 + (9 * 9 - 1) * 2 + 1); // TODO: should be pointing to buffer

    let size_flag = (components.0 - 1) + (components.1 - 1) * 9;
    encode_int(size_flag as u32, 1, &mut ptr);

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
        encode_int(quantised_maximum_value as u32, 1, &mut ptr);
    } else {
        maximum_value = 1.0;
        encode_int(0, 1, &mut ptr);
    }

    encode_int(encode_dc(dc[0], dc[1], dc[2]), 4, &mut ptr);

    for i in 0..ac_count {
        encode_int(
            encode_ac(ac[i * 3 + 0], ac[i * 3 + 1], ac[i * 3 + 2], maximum_value),
            2,
            &mut ptr,
        );
    }

    Some(ptr)
}



fn multiply_basis_function(
    components: (usize, usize),
    width: usize,
    height: usize,
    pixels: &[u8],
    bytes_per_row: usize,
) -> [f32; NUM_CHANNELS] {
    let (mut r, mut g, mut b) = (0.0, 0.0, 0.0);
    let normalisation: f32 = if components.0 == 0 && components.1 == 0 {
        0.0
    } else {
        2.0
    };

    let basis_function = |x, y| -> f32::cos(PI * (components.0 as f32) * (x as f32) / (width as f32)) * f32::cos(PI * components.1 as f32 * y as f32 / height as f32);

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

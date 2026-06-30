use std::f32::consts::PI;

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

fn linear_to_s_rgb(value: f32) -> u32 {
    let v = f32::max(0.0, f32::min(1.0, value)); // TODO: clamp
    if v <= 0.0031308 {
        (v * 12.92 * 255.0 + 0.5) as u32
    } else {
        ((1.055 * v.powf(1.0 / 2.4) - 0.055) * 255.0 + 0.5) as u32
    }
}

fn s_rgb_to_linear(value: u8) -> f32 {
    let v: f32 = value as f32 / 255.0;
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

fn sign_pow(value: f32, exp: f32) -> f32 {
    value.abs().powf(exp).copysign(value)
}

fn main() {
    println!("Hello, world!");
}

fn blur_hash_for_pixels(
    x_components: usize, // TODO: usize
    y_components: usize,
    width: usize,
    height: usize,
    rgb: &[u8],
    bytes_per_row: usize,
) -> Option<String> {
    let buffer = &[0; 2 + 4 + (9 * 9 - 1) * 2 + 1];

    if x_components < 1 || x_components > 9 || y_components < 1 || y_components > 9 {
        return None;
    }

    let mut factors: Vec<Vec<[f32; NUM_CHANNELS]>> =
        vec![vec![[0.0; NUM_CHANNELS]; x_components]; y_components];

    for y in 0..(y_components as usize) {
        for x in 0..(x_components as usize) {
            let factor = multiply_basis_function(x, y, width, height, rgb, bytes_per_row);
            factors[y][x][0] = factor[0];
            factors[y][x][1] = factor[1];
            factors[y][x][2] = factor[2];
        }
    }

    let dc = factors[0][0];
    let ac = factors[0][1];
    let ac_count = x_components * y_components - 1;

    let mut ptr = String::with_capacity(2 + 4 + (9 * 9 - 1) * 2 + 1); // TODO: should be pointing to buffer

    let size_flag = (x_components - 1) + (y_components - 1) * 9;
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

fn encode_dc(r: f32, g: f32, b: f32) -> u32 {
    let (rounded_r, rounded_g, rounded_b) =
        (linear_to_s_rgb(r), linear_to_s_rgb(g), linear_to_s_rgb(b));
    (rounded_r << 16) + (rounded_g << 8) + rounded_b
}

fn encode_ac(r: f32, g: f32, b: f32, maximum_value: f32) -> u32 {
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

fn multiply_basis_function(
    x_component: usize,
    y_component: usize,
    width: usize,
    height: usize,
    rgb: &[u8],
    bytes_per_row: usize,
) -> [f32; NUM_CHANNELS] {
    let (mut r, mut g, mut b) = (0.0, 0.0, 0.0);
    let normalisation: f32 = if x_component == 0 && y_component == 0 {
        0.0
    } else {
        2.0
    };

    for y in 0..height {
        for x in 0..width {
            let basis = f32::cos(PI * (x_component as f32) * (x as f32) / (width as f32))
                * f32::cos(PI * y_component as f32 * y as f32 / height as f32);

            r += basis * s_rgb_to_linear(rgb[3 * x + 0 + y * bytes_per_row].into());
            g += basis * s_rgb_to_linear(rgb[3 * x + 1 + y * bytes_per_row].into());
            b += basis * s_rgb_to_linear(rgb[3 * x + 2 + y * bytes_per_row].into());
        }
    }

    let scale = normalisation / (width as f32 * height as f32);

    [r * scale, g * scale, b * scale]
}

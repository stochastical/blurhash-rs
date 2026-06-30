use crate::colour_space::{linear_to_s_rgb, sign_pow};

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


pub fn linear_to_s_rgb(value: f32) -> u32 {
    let v = f32::max(0.0, f32::min(1.0, value)); // TODO: clamp
    if v <= 0.0031308 {
        (v * 12.92 * 255.0 + 0.5) as u32
    } else {
        ((1.055 * v.powf(1.0 / 2.4) - 0.055) * 255.0 + 0.5) as u32
    }
}

// Swift: sRGBToLinear<Type: BinaryInteger>(_ value: Type) -> Float
// rename ttansfoems
pub fn s_rgb_to_linear(value: u8) -> f32 {
    let v: f32 = value as f32 / 255.0;
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

pub fn sign_pow(value: f32, exp: f32) -> f32 {
    value.abs().powf(exp).copysign(value)
}

use crate::colour_space::s_rgb_to_linear;

const NUM_CHANNELS: usize = 3;

pub fn decode_dc(value: u32) -> [f32; NUM_CHANNELS] {
    [
        s_rgb_to_linear((value >> 16) as u8),
        (s_rgb_to_linear((value >> 8) as u8) as u8 & 255) as f32,
        s_rgb_to_linear((value & 255) as u8),
    ]
}

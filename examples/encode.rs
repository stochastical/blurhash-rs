use image::GenericImageView;

use blurhash_rs::encoder::encode;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let img = image::open("test/pic1.png")?;
    let (width, height) = img.dimensions();
    let bytes_per_row = width * 3;

    // img.as_bytes();
    let rgb_pixels = img.to_rgb8().into_raw();

    dbg!(encode(
        (4, 3),
        width as usize,
        height as usize,
        &rgb_pixels,
        bytes_per_row as usize,
    ));

    Ok(())
}


#[cfg(test)]
mod tests {
    use iconforge_lib::output::ico_writer::encode_ico;
    use iconforge_lib::renderer::ICO_SIZES;
    use image::RgbaImage;

    #[test]
    fn test_ico_roundtrip() {
        // Create simple test images for each size
        let images: Vec<(u32, RgbaImage)> = ICO_SIZES
            .iter()
            .map(|&size| {
                let mut img = RgbaImage::new(size, size);
                for y in 0..size {
                    for x in 0..size {
                        let r = ((x as f32 / size as f32) * 255.0) as u8;
                        let g = ((y as f32 / size as f32) * 255.0) as u8;
                        let b = 128u8;
                        let a = 255u8;
                        img.put_pixel(x, y, image::Rgba([r, g, b, a]));
                    }
                }
                (size, img)
            })
            .collect();

        let ico_bytes = encode_ico(&images).expect("encode_ico should succeed");

        // Roundtrip
        let mut cursor = std::io::Cursor::new(&ico_bytes[..]);
        let dir = ico::IconDir::read(&mut cursor).expect("should read ICO");

        assert_eq!(
            dir.entries().len(),
            ICO_SIZES.len(),
            "ICO should contain {} entries",
            ICO_SIZES.len()
        );
    }
}

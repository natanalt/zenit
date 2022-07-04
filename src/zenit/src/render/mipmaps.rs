use glam::*;

// TODO: consider removing this and perhaps replacing it with something better

/// Halves the RGBA8 image from specified buffer by averaging between
/// neighboring pixels.
pub fn downscale_rgba8(source_size: UVec2, source: &[u8]) -> Vec<u8> {
    assert_eq!(source_size.x * source_size.y * 4, source.len() as u32);

    let mut result = Vec::with_capacity(source.len() / 4);

    let sample = move |x: u32, y: u32| -> UVec4 {
        let base = (y * source_size.x + x) * 4;
        UVec4::new(
            source[(base + 0) as usize] as u32,
            source[(base + 1) as usize] as u32,
            source[(base + 2) as usize] as u32,
            source[(base + 3) as usize] as u32,
        )
    };

    for y in 0..source_size.y / 2 {
        for x in 0..source_size.x / 2 {
            let source_x = x * 2;
            let source_y = y * 2;

            let a = sample(source_x + 0, source_y + 0);
            let b = sample(source_x + 1, source_y + 0);
            let c = sample(source_x + 0, source_y + 1);
            let d = sample(source_x + 1, source_y + 1);

            let avg = (a + b + c + d) / 4;
            result.push(avg.x as u8);
            result.push(avg.y as u8);
            result.push(avg.z as u8);
            result.push(avg.w as u8);
        }
    }

    result
}

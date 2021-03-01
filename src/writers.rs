// TODO trait?
use crate::math::Vec3;

pub fn write_header(width: u32, height: u32) {
    println!("P3 {} {}\n255", width, height);
}

pub fn write_color(color: Vec3, num_samples: u32) {
    let average = color / (num_samples as f64);

    let srgb = Vec3::new(
        average.x.powf(1.0 / 2.2),
        average.y.powf(1.0 / 2.2),
        average.z.powf(1.0 / 2.2),
    );

    let (pixel_r, pixel_g, pixel_b) = (
        (srgb.x * 255.999) as u8,
        (srgb.y * 255.999) as u8,
        (srgb.z * 255.999) as u8,
    );

    println!("{} {} {}", pixel_r, pixel_g, pixel_b);
}

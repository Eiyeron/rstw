// TODO trait?
use nalgebra::Vector3;

pub fn write_header(width: u32, height: u32) -> () {
    println!("P3 {} {}\n255", width, height);
}

pub fn write_color(color: Vector3<f64>, num_samples: u32) {
    let average = color / (num_samples as f64);

    let srgb = Vector3::new(
        average.x.powf(1.0 / 2.2),
        average.y.powf(1.0 / 2.2),
        average.z.powf(1.0 / 2.2),
    );
    let bytes = Vector3::new(
        (srgb.x * 255.999) as u8,
        (srgb.y * 255.999) as u8,
        (srgb.z * 255.999) as u8,
    );

    println!("{} {} {}", bytes.x, bytes.y, bytes.z);
}

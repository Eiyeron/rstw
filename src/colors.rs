use crate::math::{vpowf, Vec3};

pub fn linear_to_bt709(linear: &Vec3) -> Vec3 {
    vpowf(linear, 1.0 / 2.2)
}

pub fn downscale_to_8bit(color: &Vec3) -> (u8, u8, u8) {
    (
        (color.x * 255.999) as u8,
        (color.y * 255.999) as u8,
        (color.z * 255.999) as u8,
    )
}

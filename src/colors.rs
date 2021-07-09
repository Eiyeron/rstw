use crate::math::{vpowf, Vec3};

// sRGB is *not* BT.709
// https://en.wikipedia.org/wiki/Rec._709#Relationship_to_sRGB
pub fn linear_to_srgb(linear: &Vec3) -> Vec3 {
    linear.map(|u| {
        if u < 0.0031308 {
            u * 12.92 // * 323/25
        } else {
            (211. * u.powf(5. / 12.) - 11.) / 200.
        }
    })
}

fn offset_limit_to_255(v: f64) -> u8 {
    (v * 255. + 0.5).floor().clamp(0., 255.) as u8
}

pub fn downscale_to_8bit(color: &Vec3) -> (u8, u8, u8) {
    // This works and converts into a [0;1] range and slightly works better than
    // v * 255 as it gives relatively more range for extreme values like 0 or 255.
    // but it has an absolute error value quite important that oscillates in [0.5;1]

    // (
    //     (color.x * 255.999) as u8,
    //     (color.y * 255.999) as u8,
    //     (color.z * 255.999) as u8,
    // )

    // floor(255*v +0.5)/255. yields a better constant absolute error range that goes
    // in [0;0.5].

    (
        offset_limit_to_255(color.x),
        offset_limit_to_255(color.y),
        offset_limit_to_255(color.z),
    )
}

use crate::Vec3;

use std::rc::Rc;

pub trait Texture {
    fn value(&self, u: f64, v: f64, p: &Vec3) -> Vec3;
}

pub struct SolidColor {
    pub albedo: Vec3,
}

// TODO I'm getting tired of having Rc everywhere.
pub struct Checkerboard {
    pub albedo_odd: Rc<dyn Texture>,
    pub albedo_even: Rc<dyn Texture>,
}

impl SolidColor {
    pub fn new(r: f64, g: f64, b: f64) -> SolidColor {
        SolidColor {
            albedo: Vec3::new(r, g, b),
        }
    }
}

impl Texture for SolidColor {
    fn value(&self, u: f64, v: f64, p: &Vec3) -> Vec3 {
        self.albedo
    }
}

impl Texture for Checkerboard {
    fn value(&self, u: f64, v: f64, p: &Vec3) -> Vec3 {
        // Stolen from the tutorial.
        // Wouldn't fract/fmod be faster?
        let sines = f64::sin(10.0 * p.x) * f64::sin(10.0 * p.y) * f64::sin(10.0 * p.z);
        if sines < 0.0 {
            self.albedo_odd.value(u, v, p)
        } else {
            self.albedo_even.value(u, v, p)
        }
    }
}

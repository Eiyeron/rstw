use crate::noise::Perlin;
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

pub struct Noise {
    pub perlin: Perlin,
    pub scale: f64
}

pub struct TurbulentNoise {
    pub perlin: Perlin,
    pub scale: f64,
    pub depth: u32,
}


pub struct MarbleNoise {
    pub perlin: Perlin,
    pub scale: f64,
    pub depth: u32,
}

impl SolidColor {
    pub fn new(r: f64, g: f64, b: f64) -> SolidColor {
        SolidColor {
            albedo: Vec3::new(r, g, b),
        }
    }
}

impl Texture for SolidColor {
    fn value(&self, _u: f64, _v: f64, _p: &Vec3) -> Vec3 {
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

impl Texture for Noise {
    fn value(&self, _u: f64, _v: f64, p: &Vec3) -> Vec3 {
        let scaled = p * self.scale;
        Vec3::from_element((self.perlin.noise(&scaled)+ 1.0) / 2.0)
    }
}

fn pertubation(perlin:&Perlin, depth:u32, p:&Vec3) -> f64 {
    let mut acc = 0.0;
    let mut scaled = *p;
    let mut weight = 1.0;
    for _i in 0..depth {
        acc += perlin.noise(&scaled) * weight;
        weight /= 2.0;
        scaled *= 2.0;
    }
    acc.abs()
}

impl Texture for TurbulentNoise {
    fn value(&self, _u: f64, _v: f64, p: &Vec3) -> Vec3 {
        Vec3::from_element(pertubation(&self.perlin, self.depth, &p))
    }
}

impl Texture for MarbleNoise {
    fn value(&self, _u: f64, _v: f64, p: &Vec3) -> Vec3 {
        let noise_value = pertubation(&self.perlin, self.depth, &p);
        let v = (1.0 + f64::sin(10.0 * noise_value + self.scale * p.z)) / 2.0;
        Vec3::from_element(v)
    }
}

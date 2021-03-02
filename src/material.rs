use crate::Vec3;
use crate::{HitRecord, Ray};
use nalgebra::Vector3;
use rand_distr::{Distribution, Uniform, UnitSphere};

pub trait Material {
    fn scatter(&self, ray: &Ray, rec: &HitRecord) -> Option<(Ray, Vec3)>;
}

pub struct Lambertian {
    pub albedo: Vec3,
}

pub struct Metal {
    pub albedo: Vec3,
    pub roughness: f64,
}

pub struct Dielectric {
    pub ior: f64,
}

impl Material for Lambertian {
    fn scatter(&self, ray: &Ray, rec: &HitRecord) -> Option<(Ray, Vec3)> {
        let v = UnitSphere.sample(&mut rand::thread_rng());
        let mut scatter_direction = rec.normal + Vector3::from_row_slice(&v);

        let ee_x = epsilon_equal(scatter_direction.x, 0.0, 1.0e-8);
        let ee_y = epsilon_equal(scatter_direction.y, 0.0, 1.0e-8);
        let ee_z = epsilon_equal(scatter_direction.z, 0.0, 1.0e-8);
        if ee_x && ee_y && ee_z {
            scatter_direction = rec.normal;
        }

        Some((
            Ray {
                origin: rec.p,
                direction: scatter_direction,
                time: ray.time,
            },
            self.albedo,
        ))
    }
}

impl Material for Metal {
    fn scatter(&self, ray: &Ray, rec: &HitRecord) -> Option<(Ray, Vec3)> {
        let v: [f64; 3] = UnitSphere.sample(&mut rand::thread_rng());

        let unit_direction = ray.direction.normalize();

        let refraction_ratio = {
            if rec.front_facing {
                1. / /*self.ior*/ 2.5
            } else {
                /*self.ior*/
                2.5
            }
        };
        let cos_theta = (-unit_direction).dot(&rec.normal).min(1.0);
        let attenuation = self.albedo.lerp(
            &Vec3::new(1.0, 1.0, 1.0),
            schlick_reflectance(cos_theta, refraction_ratio),
        );

        let reflected = reflect(&ray.direction.normalize(), &rec.normal);
        let scattered = Ray {
            origin: rec.p,
            direction: (reflected + self.roughness * Vec3::new(v[0], v[1], v[2])).normalize(),
            time: ray.time,
        };
        if scattered.direction.dot(&rec.normal) > 0.0 {
            return Some((scattered, attenuation));
        }
        None
    }
}

impl Material for Dielectric {
    fn scatter(&self, ray: &Ray, rec: &HitRecord) -> Option<(Ray, Vec3)> {
        let attenuation = Vec3::from_element(1.0);
        let unit_direction = ray.direction.normalize();

        let refraction_ratio = {
            if rec.front_facing {
                1. / self.ior
            } else {
                self.ior
            }
        };

        let cos_theta = (-unit_direction).dot(&rec.normal).min(1.0);
        let sin_theta = (1. - cos_theta.powi(2)).sqrt();

        let outward = {
            let probability = Uniform::from(0.0..1.0).sample(&mut rand::thread_rng());
            if refraction_ratio * sin_theta > 1.
                || schlick_reflectance(cos_theta, refraction_ratio) > probability
            {
                reflect(&unit_direction, &rec.normal)
            } else {
                refract(&unit_direction, &rec.normal, refraction_ratio)
            }
        };
        let scattered = Ray {
            origin: rec.p,
            direction: outward,
            time: ray.time,
        };
        Some((scattered, attenuation))
    }
}

fn epsilon_equal(a: f64, b: f64, epsilon: f64) -> bool {
    (a - b).abs() < epsilon
}

fn reflect(i: &Vec3, n: &Vec3) -> Vec3 {
    i - n * (n.dot(i) * 2.0)
}

fn refract(i: &Vec3, n: &Vec3, eta: f64) -> Vec3 {
    let ni = Vector3::dot(n, i);
    let k: f64 = 1.0 - eta.powi(2) * (1.0 - ni.powi(2));

    if k < 0.0 {
        Vector3::zeros()
    } else {
        i * eta - n * (eta * ni + k.sqrt())
    }
}

fn schlick_reflectance(cosine: f64, ref_idx: f64) -> f64 {
    let r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
    let r0_2 = r0.powi(2);
    r0_2 + (1.0 - r0_2) * (1.0 - cosine).powf(5.0)
}

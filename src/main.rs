mod hittable;
mod material;
mod math;
mod writers;

use hittable::*;
use material::*;
use math::Vec3;
use rand_distr::{Distribution, Uniform, UnitDisc};
use std::rc::Rc;
use writers::*;

pub struct Ray {
    origin: Vec3,
    direction: Vec3,
}

impl Ray {
    pub fn at(&self, t: f64) -> Vec3 {
        self.origin + t * self.direction
    }
}

pub struct HitRecord<'a> {
    t: f64,
    p: Vec3,
    normal: Vec3,
    front_facing: bool,
    material: &'a dyn Material,
}

impl<'a> HitRecord<'a> {
    pub fn set_face_normal(incoming: Vec3, outward_normal: Vec3) -> (bool, Vec3) {
        let front_facing = incoming.dot(&outward_normal) < 0.0;
        let normal = {
            if front_facing {
                outward_normal
            } else {
                -outward_normal
            }
        };
        (front_facing, normal)
    }
    pub fn from(
        t: f64,
        p: Vec3,
        incoming: Vec3,
        normal: Vec3,
        material: &dyn Material,
    ) -> HitRecord {
        let (front_facing, normal) = HitRecord::set_face_normal(incoming, normal);
        HitRecord {
            t,
            p,
            normal,
            front_facing,
            material,
        }
    }
}

struct Camera {
    origin: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
    lower_left_corner: Vec3,

    u: Vec3,
    v: Vec3,
    w: Vec3,

    lens_radius: f64,
    // time0: f64,
    // time1: f64,
}

impl Camera {
    pub fn new(
        eye: Vec3,
        target: Vec3,
        up: Vec3,
        vertical_fov: f64,
        aspect_ratio: f64,
        aperture: f64,
        focus_distance: f64,
    ) -> Camera {
        let theta = vertical_fov.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = viewport_height * aspect_ratio;

        let w = (eye - target).normalize();
        let u = (up.cross(&w)).normalize();
        let v = w.cross(&u);

        let horizontal = focus_distance * viewport_width * u;
        let vertical = focus_distance * viewport_height * v;
        Camera {
            origin: eye,
            horizontal,
            vertical,

            u,
            v,
            w,

            lower_left_corner: eye - horizontal / 2. - vertical / 2. - focus_distance * w,
            lens_radius: aperture / 2.,
        }
    }

    pub fn get_ray(&self, s: f64, t: f64) -> Ray {
        // TODO offset with random
        let rd: [f64; 2] = UnitDisc.sample(&mut rand::thread_rng());
        let offset = (self.u * rd[0] + self.v * rd[1]) * self.lens_radius;
        Ray {
            origin: self.origin + offset,
            direction: self.lower_left_corner + s * self.horizontal + t * self.vertical
                - self.origin
                - offset,
        }
    }
}

fn ray_color(ray: Ray, hittable: &dyn Hittable, depth: u16) -> Vec3 {
    if depth == 0 {
        return Vec3::zeros();
    }

    if let Some(hit) = hittable.hit(&ray, 0.01, f64::INFINITY) {
        if let Some((outgoing_ray, attenuation)) = hit.material.scatter(&ray, &hit) {
            let intermediate_result = ray_color(outgoing_ray, hittable, depth - 1);
            return Vec3::new(
                intermediate_result.x * attenuation.x,
                intermediate_result.y * attenuation.y,
                intermediate_result.z * attenuation.z,
            );
        } else {
            return Vec3::zeros();
        }
    }

    let unit_dir = ray.direction.normalize();
    let t = 0.5 * (unit_dir.y + 1.0);
    // Vec3::zeros().lerp(&Vec3::new(1.0, 1.0, 1.0), t)
    Vec3::new(1.0, 1.0, 1.0).lerp(&Vec3::new(0.5, 0.7, 1.0), t)
}

fn main() {
    let max_depth = 50;
    let num_iterations = 100;
    let aspect_ratio = 1.0;
    let render_width = 128;
    let render_height = (render_width as f64 / aspect_ratio) as u32;
    let eye = Vec3::new(0.0, 2.0, -5.0);
    let target = Vec3::zeros();
    let cam = Camera::new(
        eye,
        target,
        Vec3::new(0.0, 1.0, 0.0),
        60.,
        aspect_ratio,
        0.0, // Aperture
        (eye - target).norm(),
    );

    let lambertian: Rc<dyn Material> = Rc::new(Lambertian {
        albedo: Vec3::new(0.2, 0.4, 0.6),
    });
    let lambertian_2: Rc<dyn Material> = Rc::new(Lambertian {
        albedo: Vec3::new(0.6, 0.6, 0.6),
    });
    let metal: Rc<dyn Material> = Rc::new(Metal {
        albedo: Vec3::new(0.7, 0.6, 0.5),
        roughness: 0.0,
    });
    let glass: Rc<dyn Material> = Rc::new(Dielectric { ior: 1.5 });

    // No need for a hittable_list. A simple vector is largely enough for the process.
    // A scene load/save could be interesting to add.
    let mut objects: Vec<Rc<dyn Hittable>> = Vec::new();
    objects.push(Rc::new(Sphere {
        center: Vec3::new(0.0, -1005.0, 0.0),
        radius: 1000.0,
        material: lambertian_2,
    }));
    for y in (-5..5).step_by(3) {
        for x in -10..10 {
            let r = f64::hypot(x as f64, y as f64);
            objects.push(Rc::new(Sphere {
                center: Vec3::new((x * 3) as f64, 0.0, (y * 3) as f64),
                radius: r / 5. + 0.2,
                material: lambertian.clone(),
            }));
            objects.push(Rc::new(Sphere {
                center: Vec3::new((x * 3) as f64, 0.0, ((y + 1) * 3) as f64),
                radius: r / 5. + 0.2,
                material: metal.clone(),
            }));
            objects.push(Rc::new(Sphere {
                center: Vec3::new((x * 3) as f64, 0.0, ((y + 2) * 3) as f64),
                radius: r / 5. + 0.2 + 0.6,
                material: glass.clone(),
            }));
        }
    }

    let world = BvhNode::from_slice(&objects[..], 0.0, 10000.0);

    write_header(render_width, render_height);
    let jitter_distribution = Uniform::from(0.0..1.0);
    let rand_generator = &mut rand::thread_rng();
    for y in 0..render_height {
        if y % 100 == 0 {
            eprintln!("{} lines remaining", render_height - y);
        }
        for x in 0..render_width {
            let mut sum = Vec3::zeros();
            for _sample in 0..num_iterations {
                // TODO Add jittering for subpixel sampling
                let jitter_x = jitter_distribution.sample(rand_generator);
                let jitter_y = jitter_distribution.sample(rand_generator);
                let s = (jitter_x + (x as f64)) / (render_width as f64 - 1.0);
                let t = 1.0 - (jitter_y + (y as f64)) / (render_height as f64 - 1.0);

                let ray = cam.get_ray(s, t);
                sum += ray_color(ray, &world, max_depth);
            }
            write_color(sum, num_iterations);
        }
    }
}

mod hittable;
mod material;
mod writers;

use hittable::*;
use material::*;
use nalgebra::Vector3;
use rand_distr::{Distribution, Uniform, UnitDisc};
use std::rc::Rc;
use writers::*;

pub struct Ray {
    origin: Vector3<f64>,
    direction: Vector3<f64>,
}

impl Ray {
    pub fn at(&self, t: f64) -> Vector3<f64> {
        self.origin + t * self.direction
    }
}

pub struct HitRecord<'a> {
    t: f64,
    p: Vector3<f64>,
    normal: Vector3<f64>,
    front_facing: bool,
    material: &'a dyn Material,
}

impl<'a> HitRecord<'a> {
    pub fn set_face_normal(
        incoming: Vector3<f64>,
        outward_normal: Vector3<f64>,
    ) -> (bool, Vector3<f64>) {
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
        p: Vector3<f64>,
        incoming: Vector3<f64>,
        normal: Vector3<f64>,
        material: &dyn Material,
    ) -> HitRecord {
        let (front_facing, normal) = HitRecord::set_face_normal(incoming, normal);
        HitRecord {
            t: t,
            p: p,
            normal: normal,
            front_facing: front_facing,
            material: material,
        }
    }
}

pub struct AABB {
    min: Vector3<f64>,
    max: Vector3<f64>,
}

impl AABB {
    pub fn union(&self, other: &AABB) -> AABB {
        let min = Vector3::new(
            f64::min(self.min.x, other.min.x),
            f64::min(self.min.y, other.min.y),
            f64::min(self.min.z, other.min.z),
        );
        let max = Vector3::new(
            f64::max(self.max.x, other.max.x),
            f64::max(self.max.y, other.max.y),
            f64::max(self.max.z, other.max.z),
        );
        AABB { min, max }
    }

    fn test_component(
        min_component: f64,
        max_component: f64,
        ray_pos_component: f64,
        ray_dir_component: f64,
        t_min: f64,
        t_max: f64,
    ) -> bool {
        let ray_inv = ray_dir_component.recip();

        let mut t0 = (min_component - ray_pos_component) * ray_inv;
        let mut t1 = (max_component - ray_pos_component) * ray_inv;

        if ray_inv < 0.0 {
            std::mem::swap(&mut t0, &mut t1);
        }

        let t_min = f64::max(t0, t_min);
        let t_max = f64::min(t1, t_max);
        if t_max <= t_min {
            return false;
        }
        true
    }

    pub fn intersects(&self, ray: &Ray, t_min: f64, t_max: f64) -> bool {
        AABB::test_component(
            self.min.x,
            self.max.x,
            ray.origin.x,
            ray.direction.x,
            t_min,
            t_max,
        ) && AABB::test_component(
            self.min.y,
            self.max.y,
            ray.origin.y,
            ray.direction.y,
            t_min,
            t_max,
        ) && AABB::test_component(
            self.min.z,
            self.max.z,
            ray.origin.z,
            ray.direction.z,
            t_min,
            t_max,
        )
    }
}
struct Camera {
    origin: Vector3<f64>,
    horizontal: Vector3<f64>,
    vertical: Vector3<f64>,
    lower_left_corner: Vector3<f64>,

    u: Vector3<f64>,
    v: Vector3<f64>,
    w: Vector3<f64>,

    lens_radius: f64,
    // time0: f64,
    // time1: f64,
}

impl Camera {
    pub fn new(
        eye: Vector3<f64>,
        target: Vector3<f64>,
        up: Vector3<f64>,
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
            horizontal: horizontal,
            vertical: vertical,

            u: u,
            v: v,
            w: w,

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

fn ray_color(ray: Ray, hittable: &dyn Hittable, depth: u16) -> Vector3<f64> {
    if depth == 0 {
        return Vector3::zeros();
    }

    if let Some(hit) = hittable.hit(&ray, 0.01, f64::INFINITY) {
        if let Some((outgoing_ray, attenuation)) = hit.material.scatter(&ray, &hit) {
            let intermediate_result = ray_color(outgoing_ray, hittable, depth - 1);
            return Vector3::new(
                intermediate_result.x * attenuation.x,
                intermediate_result.y * attenuation.y,
                intermediate_result.z * attenuation.z,
            );
        } else {
            return Vector3::zeros();
        }
    }

    let unit_dir = ray.direction.normalize();
    let t = 0.5 * (unit_dir.y + 1.0);
    // Vector3::zeros().lerp(&Vector3::new(1.0, 1.0, 1.0), t)
    Vector3::new(1.0, 1.0, 1.0).lerp(&Vector3::new(0.5, 0.7, 1.0), t)
}

fn main() {
    let max_depth = 50;
    let num_iterations = 100;
    let aspect_ratio = 1.0;
    let render_width = 128;
    let render_height = (render_width as f64 / aspect_ratio) as u32;
    let eye = Vector3::new(0.0, 2.0, -5.0);
    let target = Vector3::zeros();
    let cam = Camera::new(
        eye,
        target,
        Vector3::new(0.0, 1.0, 0.0),
        60.,
        aspect_ratio,
        0.0, // Aperture
        (eye - target).norm(),
    );

    let lambertian: Rc<dyn Material> = Rc::new(Lambertian {
        albedo: Vector3::new(0.2, 0.4, 0.6),
    });
    let lambertian_2: Rc<dyn Material> = Rc::new(Lambertian {
        albedo: Vector3::new(0.6, 0.6, 0.6),
    });
    let metal: Rc<dyn Material> = Rc::new(Metal {
        albedo: Vector3::new(0.7, 0.6, 0.5),
        roughness: 0.0,
    });
    let glass: Rc<dyn Material> = Rc::new(Dielectric { ior: 1.5 });

    let mut objects: Vec<Rc<dyn Hittable>> = Vec::new();
    objects.push(Rc::new(Sphere {
        center: Vector3::new(0.0, -1005.0, 0.0),
        radius: 1000.0,
        material: lambertian_2,
    }));
    for y in (-5..5).step_by(3) {
        for x in -10..10 {
            let r = f64::hypot(x as f64, y as f64);
            objects.push(Rc::new(Sphere {
                center: Vector3::new((x * 3) as f64, 0.0, (y * 3) as f64),
                radius: r / 5. + 0.2,
                material: lambertian.clone(),
            }));
            objects.push(Rc::new(Sphere {
                center: Vector3::new((x * 3) as f64, 0.0, ((y + 1) * 3) as f64),
                radius: r / 5. + 0.2,
                material: metal.clone(),
            }));
            objects.push(Rc::new(Sphere {
                center: Vector3::new((x * 3) as f64, 0.0, ((y + 2) * 3) as f64),
                radius: r / 5. + 0.2 + 0.6,
                material: glass.clone(),
            }));
        }
    }

    let world = BvhNode::from_slice(&objects[..], 0.0, 10000.0);

    write_header(render_width, render_height);
    let jitter_distribution = Uniform::from(0.0..1.0);
    for y in 0..render_height {
        if y % 100 == 0 {
            eprintln!("{} lines remaining", render_height - y);
        }
        for x in 0..render_width {
            let mut sum = Vector3::zeros();
            for _sample in 0..num_iterations {
                // TODO Add jittering for subpixel sampling
                let jitter_x = jitter_distribution.sample(&mut rand::thread_rng());
                let jitter_y = jitter_distribution.sample(&mut rand::thread_rng());
                let s = (jitter_x + (x as f64)) / (render_width as f64 - 1.0);
                let t = 1.0 - (jitter_y + (y as f64)) / (render_height as f64 - 1.0);

                let ray = cam.get_ray(s, t);
                sum += ray_color(ray, &world, max_depth);
            }
            write_color(sum, num_iterations);
        }
    }
}

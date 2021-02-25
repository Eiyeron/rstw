use nalgebra::Vector3;
use rand_distr::{UnitDisc, UnitSphere, Distribution, Uniform};
use std::rc::Rc;

trait Hittable {
    fn hit(&self, ray:&Ray, t_min:f64, t_max:f64) -> Option<HitRecord>;
    fn bounding_box(&self) -> Option<AABB>;
}

trait Material {
    fn scatter(&self, ray:&Ray, rec:&HitRecord) -> Option<(Ray, Vector3<f64>)>;
}

struct Ray {
    origin: Vector3<f64>,
    direction: Vector3<f64>,
}

impl Ray {
    pub fn at(&self, t:f64) -> Vector3<f64> {
        self.origin + t * self.direction
    }
}

struct HitRecord<'a> {
    t: f64,
    p: Vector3<f64>,
    normal: Vector3<f64>,
    front_facing: bool,
    material: &'a dyn Material
}

impl<'a> HitRecord<'a> {
    pub fn set_face_normal(incoming:Vector3<f64>, outward_normal:Vector3<f64>) -> (bool, Vector3<f64>) {
        let front_facing = incoming.dot(&outward_normal) < 0.0;
        let normal = {
            if front_facing {
                outward_normal
            }
            else {
                -outward_normal
            }
        };
        (front_facing, normal)
    }
    pub fn from(t:f64, p:Vector3<f64>, incoming:Vector3<f64>, normal:Vector3<f64>, material:&dyn Material) -> HitRecord {
        let(front_facing, normal) = HitRecord::set_face_normal(incoming, normal);
        HitRecord {
            t:t,
            p:p,
            normal: normal,
            front_facing: front_facing,
            material: material
        }
    }
}

struct AABB {
    min: Vector3<f64>,
    max: Vector3<f64>
}

impl AABB {
    pub fn union(&self, other:&AABB) -> AABB {
        let min = Vector3::new(f64::min(self.min.x, other.min.x), f64::min(self.min.y, other.min.y), f64::min(self.min.z, other.min.z));
        let max = Vector3::new(f64::max(self.max.x, other.max.x), f64::max(self.max.y, other.max.y), f64::max(self.max.z, other.max.z));
        AABB{min, max}
    }

    fn test_component(min_component:f64, max_component:f64, ray_pos_component:f64, ray_dir_component:f64, t_min:f64, t_max:f64) -> bool {
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
    pub fn intersects(&self, ray:&Ray, t_min:f64, t_max:f64) -> bool {
        AABB::test_component(self.min.x, self.max.x, ray.origin.x, ray.direction.x, t_min, t_max)
            &&
        AABB::test_component(self.min.y, self.max.y, ray.origin.y, ray.direction.y, t_min, t_max)
            &&
        AABB::test_component(self.min.z, self.max.z, ray.origin.z, ray.direction.z, t_min, t_max)
    }
}
struct Sphere {
    center: Vector3<f64>,
    radius: f64,
    material: Rc<dyn Material>
}

impl Hittable for Sphere {
    fn hit(&self, ray:&Ray, t_min:f64, t_max:f64) -> Option<HitRecord> {
        let oc = ray.origin - self.center;
        let a = ray.direction.norm_squared();
        let half_b = oc.dot(&ray.direction);
        let c = oc.norm_squared() - self.radius.powi(2);

        let discriminant = a.mul_add(-c, half_b.powi(2));
        if discriminant < 0.0 {
            return None;
        }
        let sqrtd = discriminant.sqrt();

        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || root > t_max {
            root = (-half_b + sqrtd) / a;
            if root < t_min || root > t_max {
                return None;
            }
        }

        let p = ray.at(root);
        let outward_normal = (p - self.center) / self.radius;
        Some(HitRecord::from(root, p, ray.direction, outward_normal, self.material.as_ref()))
    }

    fn bounding_box(&self) -> Option<AABB> {
        Some(AABB {
            min: self.center - Vector3::new(self.radius, self.radius, self.radius),
            max: self.center + Vector3::new(self.radius, self.radius, self.radius)
    })
    }
}

struct BvhNode {
    left: Rc<dyn Hittable>,
    right: Rc<dyn Hittable>,
    node_box: AABB
}

impl BvhNode {
    fn from_slice(data: &[Rc<dyn Hittable>], t0:f64, t1:f64) -> BvhNode {
        let span = data.len();
        let mut copy = Vec::new();
        match span {
            1 => {
                BvhNode {
                    left: data[0].clone(),
                    right: data[0].clone(),
                    node_box: data[0].as_ref().bounding_box().unwrap_or(AABB{
                        min:Vector3::new(0.0,0.0,0.0),
                        max:Vector3::new(0.0,0.0,0.0)
                    })
                }
            },
            2 => {
                let left = data[0].as_ref();
                let right = data[1].as_ref();
                let box_left = left.bounding_box().unwrap_or(AABB{
                    min:Vector3::new(0.0,0.0,0.0),
                    max:Vector3::new(0.0,0.0,0.0)
                });
                let box_right = right.bounding_box().unwrap_or(AABB{
                    min:Vector3::new(0.0,0.0,0.0),
                    max:Vector3::new(0.0,0.0,0.0)
                });
                BvhNode {
                    left: data[0].clone(),
                    right: data[1].clone(),
                    node_box: AABB::union(&box_left, &box_right),
                }
            },
            _ => {
                for hittable in data {
                    let hittable = Rc::clone(&hittable);
                    copy.push(hittable);
                }
                copy.sort_by(|left, right| {
                    BvhNode::box_compare(left.as_ref(), right.as_ref(), Uniform::from(0..3).sample(&mut rand::thread_rng()))
                });
                let mid = span / 2;
                let (left, right) = copy.split_at(mid);

                let left = BvhNode::from_slice(left, t0, t1);
                let right = BvhNode::from_slice(right, t0, t1);
                let box_left = left.bounding_box().unwrap_or(AABB{
                    min:Vector3::new(0.0,0.0,0.0),
                    max:Vector3::new(0.0,0.0,0.0)
                });
                let box_right = right.bounding_box().unwrap_or(AABB{
                    min:Vector3::new(0.0,0.0,0.0),
                    max:Vector3::new(0.0,0.0,0.0)
                });
                BvhNode {
                    left: Rc::new(left),
                    right: Rc::new(right),
                    node_box: AABB::union(&box_left, &box_right)
                }
            },
        }
    }

    fn box_compare(a:&dyn Hittable, b:&dyn Hittable, axis:u8) -> std::cmp::Ordering {
        let box_a = a.bounding_box();
        let box_b = b.bounding_box();
        if box_a.is_none() || box_b.is_none() {
            eprintln!("No bbox in BvhNode constructor");
        }

        let box_a = box_a.unwrap_or(AABB{
            min:Vector3::new(0.0,0.0,0.0),
            max:Vector3::new(0.0,0.0,0.0)
        });
        let box_b = box_b.unwrap_or(AABB{
            min:Vector3::new(0.0,0.0,0.0),
            max:Vector3::new(0.0,0.0,0.0)
        });
        match axis {
            0 => box_a.min.x.partial_cmp(&box_b.min.x).unwrap(),
            1 => box_a.min.y.partial_cmp(&box_b.min.y).unwrap(),
            2 => box_a.min.z.partial_cmp(&box_b.min.z).unwrap(),
            _ => panic!("Unreachable")
        }
    }

}

impl Hittable for BvhNode {

    fn hit(&self, ray:&Ray, t_min:f64, t_max:f64) -> Option<HitRecord> {
        if !self.node_box.intersects(&ray, t_min, t_max) {
            return None;
        }
        if let Some(hit_left) = self.left.as_ref().hit(&ray, t_min, t_max) {
            let t_max:f64 = hit_left.t;
            if let Some(hit_right) = self.right.as_ref().hit(&ray, t_min, t_max) {
                return Some(hit_right);
            }
            return Some(hit_left);
        }
        return self.right.as_ref().hit(&ray, t_min, t_max);
    }

    fn bounding_box(&self) -> Option<AABB> {
        Some(AABB{min: self.node_box.min, max:self.node_box.max})
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
    pub fn new(eye:Vector3<f64>, target:Vector3<f64>, up:Vector3<f64>,
               vertical_fov:f64, aspect_ratio: f64, aperture:f64, focus_distance:f64) -> Camera {
        let focal_length = 1.0;
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

            u:u,
            v:v,
            w:w,

            lower_left_corner: eye - horizontal / 2. - vertical / 2. - focus_distance * w,
            lens_radius: aperture / 2.
        }
    }

    pub fn get_ray(&self, s:f64, t:f64) -> Ray {
        // TODO offset with random
        let rd: [f64; 2] = UnitDisc.sample(&mut rand::thread_rng());
        let offset = (self.u * rd[0] + self.v * rd[1]) * self.lens_radius;
        Ray {
            origin: self.origin + offset,
            direction: self.lower_left_corner + s * self.horizontal + t * self.vertical - self.origin - offset
        }
    }
}

struct Lambertian {
    albedo:Vector3<f64>
}

fn epsilon_equal(a:f64, b:f64, epsilon:f64) -> bool {
    (a - b).abs() < epsilon
}

impl Material for Lambertian {
    fn scatter(&self, _ray:&Ray, rec:&HitRecord) -> Option<(Ray, Vector3<f64>)> {
        let v: [f64; 3] = UnitSphere.sample(&mut rand::thread_rng());

        let mut scatter_direction = rec.normal + Vector3::new(v[0], v[1], v[2]);

        let ee_x = epsilon_equal(scatter_direction.x, 0.0, 1.0e-8);
        let ee_y = epsilon_equal(scatter_direction.y, 0.0, 1.0e-8);
        let ee_z = epsilon_equal(scatter_direction.z, 0.0, 1.0e-8);
        if ee_x && ee_y && ee_z {
            scatter_direction = rec.normal;
        }

        Some((Ray{origin:rec.p, direction:scatter_direction}, self.albedo))
    }
}

fn write_color(color:Vector3<f64>, num_samples:u32) {
    let samples = num_samples as f64;
    let average = Vector3::new(color.x / samples, color.y / samples, color.z / samples);


    let srgb = Vector3::new(average.x.powf(1.0/2.2), average.y.powf(1.0/2.2), average.z.powf(1.0/2.2));
    let bytes = Vector3::new((srgb.x * 255.999) as u8, (srgb.y * 255.999) as u8, (srgb.z * 255.999) as u8);

    println!("{} {} {}", bytes.x, bytes.y, bytes.z);
}

fn ray_color(ray:Ray, hittable:&dyn Hittable, depth:u16) -> Vector3<f64> {
    if depth == 0 {
        return Vector3::new(0.0, 0.0, 0.0);
    }

    if let Some(hit) = hittable.hit(&ray, 0.01, 10000.0) {
        if let Some((outgoing_ray, attenuation)) = hit.material.scatter(&ray, &hit) {
            let intermediate_result = ray_color(outgoing_ray, hittable, depth - 1);
            return Vector3::new(intermediate_result.x * attenuation.x, intermediate_result.y * attenuation.y, intermediate_result.z * attenuation.z);
        }
        else {
            return Vector3::new(0.0, 0.0, 0.0);
        }
    }

    let unit_dir = ray.direction.normalize();
    let t = 0.5 * (unit_dir.y + 1.0);
    Vector3::new(0.0, 0.0, 0.0).lerp(&Vector3::new(1.0, 1.0, 1.0), t)

}

fn main() {
    let max_depth = 50;
    let num_iterations = 1000;
    let render_width= 2560;
    let render_height = 1440;
    let eye = Vector3::new(0.0, 5.0, -10.0);
    let target = Vector3::new(0.0, 0.0, 0.0);
    let cam = Camera::new(eye, target, Vector3::new(0.0, 1.0, 0.0),
        60.,
        render_width as f64 / render_height as f64,
        1.0 , // Aperture
        (eye - target).norm());

    let lambertian:Rc<dyn Material> = Rc::new(Lambertian{ albedo: Vector3::new(0.2, 0.4, 0.6) });
    let lambertian_2:Rc<dyn Material> = Rc::new(Lambertian{ albedo: Vector3::new(0.8, 0.8, 0.8) });

    let mut objects:Vec<Rc<dyn Hittable>> = Vec::new();
    for y in -5..5 {
        for x in -10..10 {
            let r = f64::hypot(x as f64, y as f64);
            objects.push(Rc::new(Sphere {
                center: Vector3::new((x * 3) as f64, 0.0, (y * 3) as f64),
                radius:r/5. + 0.2,
                material: Rc::clone(&lambertian)
            }));
        }
    }

    let world = BvhNode::from_slice(&objects[..], 0.0, 10000.0);

    println!("P3 {} {}\n255", render_width, render_height);
    for y in 0..render_height {
        if y%100 == 0 {
            eprintln!("{} lines remaining", render_height - y);
        }
        for x in 0..render_width {
            let mut sum = Vector3::new(0.0, 0.0, 0.0);
            for _sample in 0..num_iterations {
                // TODO Add jittering for subpixel sampling
                let s = x as f64/(render_width as f64 - 1.0);
                let t = 1.0 - (y as f64/(render_height as f64 - 1.0));

                let ray = cam.get_ray(s, t);
                sum += ray_color(ray, &world, max_depth);
            }
            write_color(sum, num_iterations);
        }
    }
}

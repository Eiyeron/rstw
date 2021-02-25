use nalgebra::Vector3;

struct Ray {
    origin: Vector3<f64>,
    direction: Vector3<f64>,
}

impl Ray {
    pub fn at(&self, t:f64) -> Vector3<f64> {
        self.origin + t * self.direction
    }
}

struct HitRecord {
    t: f64,
    p: Vector3<f64>,
    normal: Vector3<f64>,
    front_facing: bool
}

impl HitRecord {
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
    pub fn from(t:f64, p:Vector3<f64>, incoming:Vector3<f64>, normal:Vector3<f64>) -> HitRecord {
        let(front_facing, normal) = HitRecord::set_face_normal(incoming, normal);
        HitRecord {
            t:t,
            p:p,
            normal: normal,
            front_facing: front_facing
        }
    }
}

struct AABB {
    min: Vector3<f64>,
    max: Vector3<f64>
}

trait Hittable {
    fn hit(&self, ray:&Ray, t_min:f64, t_max:f64) -> Option<HitRecord>;
    fn bounding_box(&self) -> AABB;
}

struct Sphere {
    center: Vector3<f64>,
    radius: f64,
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
        Some(HitRecord::from(root, p, ray.direction, outward_normal))
    }

    fn bounding_box(&self) -> AABB {
        AABB {
            min: self.center - Vector3::new(self.radius, self.radius, self.radius),
            max: self.center + Vector3::new(self.radius, self.radius, self.radius)
        }
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
        Ray {
            origin: self.origin,
            direction: self.lower_left_corner + s * self.horizontal + t * self.vertical - self.origin
        }
    }
}

fn main() {
    let eye = Vector3::new(0.0, 5.0, -10.0);
    let target = Vector3::new(0.0, 0.0, 0.0);
    let cam = Camera::new(eye, target, Vector3::new(0.0, 1.0, 0.0),
        60.,
        800./600.,
        0.0, // Aperture
        (eye - target).norm());

    let sphere = Sphere { center: Vector3::new(0.0, 0.0, 0.0), radius:2.0};
    println!("P3 {} {}\n255", 800, 600);
    for y in 0..600 {
        for x in 0..800 {
            let s = x as f64/(800.0 - 1.0);
            let t = y as f64/(600.0 - 1.0);

            let ray = cam.get_ray(s, t);
            if let Some(hit) = sphere.hit(&ray, 0.0, 1000.0) {
                let distance = (eye - hit.p).norm();
                let mapped = ((distance - 5.0) / 5.0 * 255.0) as u16;
                println!("{} {} {}", mapped, mapped, mapped);
            }
            else {
                println!("{} {} {}", 0, 0, 0);
            }
        }
    }
}

use crate::material::Material;
use crate::math::*;
use crate::{HitRecord, Ray};
use nalgebra::Vector3;
use rand::RngCore;
use rand_distr::{Distribution, Uniform};
use std::sync::Arc;

use std::f64::consts::{PI, TAU};

pub trait Hittable: Sync + Send {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
    fn bounding_box(&self, t0: f64, t1: f64) -> Option<AABB>;
}

pub struct Sphere {
    pub center: Vec3,
    pub radius: f64,
    pub material: Arc<dyn Material>,
}

// - Sphere -

impl Sphere {
    pub fn get_uv(p: &Vec3) -> (f64, f64) {
        let theta = (-p.y).acos();
        let phi = f64::atan2(-p.z, p.x) + std::f64::consts::PI;

        (phi / TAU, theta / PI)
    }
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        match ray_sphere_intersection(&self.center, self.radius, &ray, t_min, t_max) {
            Some((root, point, normal)) => {
                let (u, v) = Sphere::get_uv(&normal);
                Some(HitRecord::from_uv(
                    root,
                    point,
                    ray.direction,
                    normal,
                    self.material.as_ref(),
                    u,
                    v,
                ))
            }
            None => None,
        }
    }

    fn bounding_box(&self, _t0: f64, _t1: f64) -> Option<AABB> {
        let radius_vector = Vector3::from_element(self.radius);
        Some(AABB::new(
            self.center - radius_vector,
            self.center + radius_vector,
        ))
    }
}

pub struct MovingSphere {
    pub center_begin: Vec3,
    pub center_end: Vec3,
    pub time_begin: f64,
    pub time_end: f64,
    pub radius: f64,
    pub material: Arc<dyn Material>,
}

impl MovingSphere {
    fn center_at(&self, t: f64) -> Vec3 {
        let ratio = (t - self.time_begin) / (self.time_end - self.time_begin);
        self.center_begin.lerp(&self.center_end, ratio)
    }
}

impl Hittable for MovingSphere {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let center_at_time = self.center_at(ray.time);
        match ray_sphere_intersection(&center_at_time, self.radius, &ray, t_min, t_max) {
            Some((root, point, normal)) => Some(HitRecord::from(
                root,
                point,
                ray.direction,
                normal,
                self.material.as_ref(),
            )),
            None => None,
        }
    }

    fn bounding_box(&self, _t0: f64, _t1: f64) -> Option<AABB> {
        let radius_vector = Vec3::from_element(self.radius);
        let box_at_begin = AABB::new(
            self.center_begin - radius_vector,
            self.center_begin + radius_vector,
        );
        let box_at_end = AABB::new(
            self.center_end - radius_vector,
            self.center_end + radius_vector,
        );
        Some(AABB::union(&box_at_begin, &box_at_end))
    }
}

fn ray_sphere_intersection(
    center: &Vec3,
    radius: f64,
    ray: &Ray,
    t_min: f64,
    t_max: f64,
) -> Option<(f64, Vec3, Vec3)> {
    let oc = ray.origin - center;
    let a = ray.direction.norm_squared();
    let half_b = oc.dot(&ray.direction);
    let c = oc.norm_squared() - radius.powi(2);

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
    let new_point = ray.at(root);
    let outward_normal = (new_point - center) / radius;
    Some((root, new_point, outward_normal))
}

// - Planes --

pub struct XyPlane {
    pub min: Vec2,
    pub max: Vec2,
    pub k: f64,
    pub material: Arc<dyn Material>,
}

impl Hittable for XyPlane {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let t = (self.k - ray.origin.z) / ray.direction.z;

        if t < t_min || t > t_max {
            return None;
        }

        let x = ray.origin.x + t * ray.direction.x;
        let y = ray.origin.y + t * ray.direction.y;

        if x < self.min.x || x > self.max.x || y < self.min.y || y > self.max.y {
            return None;
        }

        let u = (x - self.min.x) / (self.max.x - self.min.x);
        let v = (y - self.min.y) / (self.max.y - self.min.y);

        Some(HitRecord::from_uv(
            t,
            ray.at(t),
            ray.direction,
            Vec3::new(0., 0., 1.),
            self.material.as_ref(),
            u,
            v,
        ))
    }

    fn bounding_box(&self, _t0: f64, _t1: f64) -> Option<AABB> {
        Some(AABB {
            min: Vec3::new(self.min.x, self.min.y, self.k - 1e-4),
            max: Vec3::new(self.max.x, self.max.y, self.k + 1e-4),
        })
    }
}

pub struct XzPlane {
    pub min: Vec2,
    pub max: Vec2,
    pub k: f64,
    pub material: Arc<dyn Material>,
}

impl Hittable for XzPlane {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let t = (self.k - ray.origin.y) / ray.direction.y;

        if t < t_min || t > t_max {
            return None;
        }

        let x = ray.origin.x + t * ray.direction.x;
        let z = ray.origin.z + t * ray.direction.z;

        if x < self.min.x || x > self.max.x || z < self.min.y || z > self.max.y {
            return None;
        }

        let u = (x - self.min.x) / (self.max.x - self.min.x);
        let v = (z - self.min.y) / (self.max.y - self.min.y);

        Some(HitRecord::from_uv(
            t,
            ray.at(t),
            ray.direction,
            Vec3::new(0., 1., 0.),
            self.material.as_ref(),
            u,
            v,
        ))
    }

    fn bounding_box(&self, _t0: f64, _t1: f64) -> Option<AABB> {
        Some(AABB {
            min: Vec3::new(self.min.x, self.k - 1e-4, self.min.y),
            max: Vec3::new(self.max.x, self.k + 1e-4, self.max.y),
        })
    }
}

pub struct YzPlane {
    pub min: Vec2,
    pub max: Vec2,
    pub k: f64,
    pub material: Arc<dyn Material>,
}

impl Hittable for YzPlane {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let t = (self.k - ray.origin.x) / ray.direction.x;

        if t < t_min || t > t_max {
            return None;
        }

        let y = ray.origin.y + t * ray.direction.y;
        let z = ray.origin.z + t * ray.direction.z;

        if y < self.min.x || y > self.max.x || z < self.min.y || z > self.max.y {
            return None;
        }

        let u = (y - self.min.x) / (self.max.x - self.min.x);
        let v = (z - self.min.y) / (self.max.y - self.min.y);

        Some(HitRecord::from_uv(
            t,
            ray.at(t),
            ray.direction,
            Vec3::new(1., 0., 0.),
            self.material.as_ref(),
            u,
            v,
        ))
    }

    fn bounding_box(&self, _t0: f64, _t1: f64) -> Option<AABB> {
        Some(AABB {
            min: Vec3::new(self.k - 1e-4, self.min.x, self.min.y),
            max: Vec3::new(self.k + 1e-4, self.max.x, self.max.y),
        })
    }
}

// - Box -
// Sigint: A cardboard box? Why are you...?
// Renamed to Cube because Box is already a thing in Rust
pub struct Cube {
    sides: HittableList,
    bbox: AABB,
}

impl Cube {
    pub fn new(
        min: Vec3,
        max: Vec3,
        material: Arc<dyn Material>,
        t0: f64,
        t1: f64,
        rng: &mut impl RngCore,
    ) -> Cube {
        let mut side_vec: Vec<Arc<dyn Hittable>> = vec![];

        side_vec.push(Arc::new(XyPlane {
            min: min.xy(),
            max: max.xy(),
            k: min.z,
            material: material.clone(),
        }));
        side_vec.push(Arc::new(XyPlane {
            min: min.xy(),
            max: max.xy(),
            k: max.z,
            material: material.clone(),
        }));

        side_vec.push(Arc::new(XzPlane {
            min: min.xz(),
            max: max.xz(),
            k: min.y,
            material: material.clone(),
        }));
        side_vec.push(Arc::new(XzPlane {
            min: min.xz(),
            max: max.xz(),
            k: max.y,
            material: material.clone(),
        }));

        side_vec.push(Arc::new(YzPlane {
            min: min.yz(),
            max: max.yz(),
            k: min.x,
            material: material.clone(),
        }));
        side_vec.push(Arc::new(YzPlane {
            min: min.yz(),
            max: max.yz(),
            k: max.x,
            material: material.clone(),
        }));

        Cube {
            sides: HittableList::from_slice(&side_vec, t0, t1, rng),
            bbox: AABB::new(min, max),
        }
    }
}

impl Hittable for Cube {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        self.sides.hit(ray, t_min, t_max)
    }

    fn bounding_box(&self, t0: f64, t1: f64) -> Option<AABB> {
        Some(self.bbox.clone())
    }
}

// - Container structures -

pub struct BvhNode {
    left: Arc<dyn Hittable>,
    right: Arc<dyn Hittable>,
    node_box: AABB,
}

impl BvhNode {
    pub fn from_slice(
        data: &[Arc<dyn Hittable>],
        t0: f64,
        t1: f64,
        rng: &mut impl RngCore,
    ) -> BvhNode {
        let span = data.len();
        let mut copy = Vec::new();
        match span {
            1 => BvhNode {
                left: data[0].clone(),
                right: data[0].clone(),
                node_box: data[0]
                    .as_ref()
                    .bounding_box(t0, t1)
                    .unwrap_or(AABB::zeros()),
            },
            2 => {
                let left = data[0].as_ref();
                let right = data[1].as_ref();
                let box_left = left.bounding_box(t0, t1).unwrap_or(AABB::zeros());
                let box_right = right.bounding_box(t0, t1).unwrap_or(AABB::zeros());
                BvhNode {
                    left: data[0].clone(),
                    right: data[1].clone(),
                    node_box: AABB::union(&box_left, &box_right),
                }
            }
            _ => {
                for hittable in data {
                    let hittable = Arc::clone(&hittable);
                    copy.push(hittable);
                }
                copy.sort_by(|left, right| {
                    BvhNode::box_compare(
                        left.as_ref(),
                        right.as_ref(),
                        Uniform::from(0..3).sample(rng),
                    )
                });
                let mid = span / 2;
                let (left, right) = copy.split_at(mid);

                let left = BvhNode::from_slice(left, t0, t1, rng);
                let right = BvhNode::from_slice(right, t0, t1, rng);
                let box_left = left.bounding_box(t0, t1).unwrap_or(AABB::zeros());
                let box_right = right.bounding_box(t0, t1).unwrap_or(AABB::zeros());
                BvhNode {
                    left: Arc::new(left),
                    right: Arc::new(right),
                    node_box: AABB::union(&box_left, &box_right),
                }
            }
        }
    }

    fn box_compare(a: &dyn Hittable, b: &dyn Hittable, axis: u8) -> std::cmp::Ordering {
        let box_a = a.bounding_box(0.0, 0.0);
        let box_b = b.bounding_box(0.0, 0.0);
        if box_a.is_none() || box_b.is_none() {
            eprintln!("No bbox in BvhNode constructor");
        }

        let box_a = box_a.unwrap_or(AABB::zeros());
        let box_b = box_b.unwrap_or(AABB::zeros());
        match axis {
            0 => box_a.min.x.partial_cmp(&box_b.min.x).unwrap(),
            1 => box_a.min.y.partial_cmp(&box_b.min.y).unwrap(),
            2 => box_a.min.z.partial_cmp(&box_b.min.z).unwrap(),
            _ => panic!("Unreachable"),
        }
    }
}

impl Hittable for BvhNode {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        if !self.node_box.intersects(&ray, t_min, t_max) {
            return None;
        }
        if let Some(hit_left) = self.left.as_ref().hit(&ray, t_min, t_max) {
            let t_max: f64 = hit_left.t;
            if let Some(hit_right) = self.right.as_ref().hit(&ray, t_min, t_max) {
                return Some(hit_right);
            }
            return Some(hit_left);
        }
        return self.right.as_ref().hit(&ray, t_min, t_max);
    }

    fn bounding_box(&self, _t0: f64, _t1: f64) -> Option<AABB> {
        Some(self.node_box.clone())
    }
}

pub struct HittableList {
    hittables: Vec<Arc<dyn Hittable>>,
    list_boundaries: AABB,
}

impl HittableList {
    pub fn from_slice(
        data: &[Arc<dyn Hittable>],
        t0: f64,
        t1: f64,
        _rng: &mut impl RngCore,
    ) -> HittableList {
        let mut aabb = AABB::zeros();
        for hittable in data {
            if let Some(other) = hittable.bounding_box(t0, t1) {
                aabb = aabb.union(&other);
            }
        }

        HittableList {
            hittables: data.to_vec(),
            list_boundaries: aabb,
        }
    }
}

impl Hittable for HittableList {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let mut closest_record: Option<HitRecord> = None;
        for hittable in &self.hittables {
            if let Some(record) = hittable.as_ref().hit(ray, t_min, t_max) {
                if closest_record.is_none() || record.t < closest_record.as_ref().unwrap().t {
                    closest_record = Some(record);
                }
            }
        }
        closest_record
    }

    fn bounding_box(&self, _t0: f64, _t1: f64) -> Option<AABB> {
        Some(self.list_boundaries.clone())
    }
}

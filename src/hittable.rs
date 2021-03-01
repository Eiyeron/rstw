use crate::material::Material;
use crate::{HitRecord, Ray, AABB};
use nalgebra::Vector3;
use rand_distr::{Distribution, Uniform};
use std::rc::Rc;

pub trait Hittable {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
    fn bounding_box(&self) -> Option<AABB>;
}

pub struct Sphere {
    pub center: Vector3<f64>,
    pub radius: f64,
    pub material: Rc<dyn Material>,
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
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
        Some(HitRecord::from(
            root,
            p,
            ray.direction,
            outward_normal,
            self.material.as_ref(),
        ))
    }

    fn bounding_box(&self) -> Option<AABB> {
        let radius_vector = Vector3::from_element(self.radius);
        Some(AABB {
            min: self.center - radius_vector,
            max: self.center + radius_vector,
        })
    }
}

pub struct BvhNode {
    left: Rc<dyn Hittable>,
    right: Rc<dyn Hittable>,
    node_box: AABB,
}

impl BvhNode {
    pub fn from_slice(data: &[Rc<dyn Hittable>], t0: f64, t1: f64) -> BvhNode {
        let span = data.len();
        let mut copy = Vec::new();
        match span {
            1 => BvhNode {
                left: data[0].clone(),
                right: data[0].clone(),
                node_box: data[0].as_ref().bounding_box().unwrap_or(AABB {
                    min: Vector3::zeros(),
                    max: Vector3::zeros(),
                }),
            },
            2 => {
                let left = data[0].as_ref();
                let right = data[1].as_ref();
                let box_left = left.bounding_box().unwrap_or(AABB {
                    min: Vector3::zeros(),
                    max: Vector3::zeros(),
                });
                let box_right = right.bounding_box().unwrap_or(AABB {
                    min: Vector3::zeros(),
                    max: Vector3::zeros(),
                });
                BvhNode {
                    left: data[0].clone(),
                    right: data[1].clone(),
                    node_box: AABB::union(&box_left, &box_right),
                }
            }
            _ => {
                for hittable in data {
                    let hittable = Rc::clone(&hittable);
                    copy.push(hittable);
                }
                copy.sort_by(|left, right| {
                    BvhNode::box_compare(
                        left.as_ref(),
                        right.as_ref(),
                        Uniform::from(0..3).sample(&mut rand::thread_rng()),
                    )
                });
                let mid = span / 2;
                let (left, right) = copy.split_at(mid);

                let left = BvhNode::from_slice(left, t0, t1);
                let right = BvhNode::from_slice(right, t0, t1);
                let box_left = left.bounding_box().unwrap_or(AABB {
                    min: Vector3::zeros(),
                    max: Vector3::zeros(),
                });
                let box_right = right.bounding_box().unwrap_or(AABB {
                    min: Vector3::zeros(),
                    max: Vector3::zeros(),
                });
                BvhNode {
                    left: Rc::new(left),
                    right: Rc::new(right),
                    node_box: AABB::union(&box_left, &box_right),
                }
            }
        }
    }

    fn box_compare(a: &dyn Hittable, b: &dyn Hittable, axis: u8) -> std::cmp::Ordering {
        let box_a = a.bounding_box();
        let box_b = b.bounding_box();
        if box_a.is_none() || box_b.is_none() {
            eprintln!("No bbox in BvhNode constructor");
        }

        let box_a = box_a.unwrap_or(AABB {
            min: Vector3::zeros(),
            max: Vector3::zeros(),
        });
        let box_b = box_b.unwrap_or(AABB {
            min: Vector3::zeros(),
            max: Vector3::zeros(),
        });
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

    fn bounding_box(&self) -> Option<AABB> {
        Some(AABB {
            min: self.node_box.min,
            max: self.node_box.max,
        })
    }
}

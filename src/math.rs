use crate::Ray;
use rand::distributions::Distribution;
use rand::RngCore;

pub type Vec3 = nalgebra::Vector3<f64>;

pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn union(&self, other: &AABB) -> AABB {
        let min = Vec3::new(
            f64::min(self.min.x, other.min.x),
            f64::min(self.min.y, other.min.y),
            f64::min(self.min.z, other.min.z),
        );
        let max = Vec3::new(
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

pub fn generate_vector(dist: &impl Distribution<f64>, rng: &mut impl RngCore) -> Vec3 {
    Vec3::new(dist.sample(rng), dist.sample(rng), dist.sample(rng))
}

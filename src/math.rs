use crate::Ray;
use rand::distributions::Distribution;
use rand::RngCore;

pub type Vec2 = nalgebra::Vector2<f64>;
pub type Vec3 = nalgebra::Vector3<f64>;

pub fn vpowf(v: &Vec3, factor: f64) -> Vec3 {
    v.map(|f| f.powf(factor))
}

pub fn vmin(a: &Vec3, b: &Vec3) -> Vec3 {
    a.zip_map(b, f64::min)
}

pub fn vmax(a: &Vec3, b: &Vec3) -> Vec3 {
    a.zip_map(b, f64::max)
}

#[derive(Clone)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> AABB {
        AABB { min, max }
    }

    pub fn zeros() -> AABB {
        AABB {
            min: Vec3::zeros(),
            max: Vec3::zeros(),
        }
    }

    pub fn union(&self, other: &AABB) -> AABB {
        let min = vmin(&self.min, &other.min);
        let max = vmax(&self.max, &other.max);
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

// Kept for reference
pub fn _trilinear_interpolation(c: [[[f64; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
    let mut acc = 0.0;
    for i in 0..2 {
        for j in 0..2 {
            for k in 0..2 {
                let fi = i as f64;
                let fj = j as f64;
                let fk = k as f64;
                let il = fi * u + (1.0 - fi) * (1.0 - u);
                let jl = fj * v + (1.0 - fj) * (1.0 - v);
                let kl = fk * w + (1.0 - fk) * (1.0 - w);
                acc += il * jl * kl * c[i][j][k];
            }
        }
    }
    acc
}

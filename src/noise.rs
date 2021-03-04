use crate::math::trilinear_interpolation;
use crate::Vec3;
use rand::seq::SliceRandom;
use rand::RngCore;
use rand_distr::{Distribution, Uniform};

const point_count: usize = 256;

pub struct Perlin {
    random_floats: [f64; point_count],
    permutations_x: [usize; point_count],
    permutations_y: [usize; point_count],
    permutations_z: [usize; point_count],
}

impl Perlin {
    fn generate_permutations(rng: &mut impl RngCore) -> [usize; point_count] {
        let mut base = [0; point_count];
        for i in 0..base.len() {
            base[i] = i;
        }
        base.shuffle(rng);
        base
    }
    pub fn new(rng: &mut impl RngCore) -> Perlin {
        let unit_dist = Uniform::from(0.0..1.0);
        let mut random_floats: [f64; point_count] = [0.0; point_count];
        for v in random_floats.iter_mut() {
            *v = unit_dist.sample(rng);
        }

        Perlin {
            random_floats,
            permutations_x: Perlin::generate_permutations(rng),
            permutations_y: Perlin::generate_permutations(rng),
            permutations_z: Perlin::generate_permutations(rng),
        }
    }

    fn uninterpolated_noise(&self, p: &Vec3) -> f64 {
        let p = p * 4.0;
        let ix = ((p.x as i64) as usize) % point_count;
        let iy = ((p.y as i64) as usize) % point_count;
        let iz = ((p.z as i64) as usize) % point_count;

        self.random_floats
            [self.permutations_x[ix] ^ self.permutations_y[iy] ^ self.permutations_z[iz]]
    }

    pub fn noise(&self, p: &Vec3) -> f64 {
        let u = p.x - p.x.floor();
        let v = p.y - p.y.floor();
        let w = p.z - p.z.floor();
        let u = u.powi(2) * (3.0 - 2.0 * u);
        let v = v.powi(2) * (3.0 - 2.0 * v);
        let w = w.powi(2) * (3.0 - 2.0 * w);

        // Fun fact : casting from f64 to usize a negative value returns 0 (clamped)
        let i = p.x.floor() as i64 as usize;
        let j = p.y.floor() as i64 as usize;
        let k = p.z.floor() as i64 as usize;
        // Chief kiss
        let mut vals: [[[f64; 2]; 2]; 2] = [[[0.0; 2]; 2]; 2];

        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    let (x, y, z) = (
                        (i + di) as usize % point_count,
                        (j + dj) as usize % point_count,
                        (k + dk) as usize % point_count,
                    );
                    // TODO range variables into usize
                    vals[di as usize][dj as usize][dk as usize] = self.random_floats
                        [self.permutations_x[x] ^ self.permutations_y[y] ^ self.permutations_z[z]];
                }
            }
        }

        trilinear_interpolation(vals, u, v, w)
    }
}

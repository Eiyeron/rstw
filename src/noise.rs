use crate::Vec3;
use rand::seq::SliceRandom;
use rand::RngCore;
use rand_distr::{Distribution, UnitSphere};

const POINT_COUNT: usize = 256;

pub struct Perlin {
    random_vectors: [Vec3; POINT_COUNT],
    permutations_x: [usize; POINT_COUNT],
    permutations_y: [usize; POINT_COUNT],
    permutations_z: [usize; POINT_COUNT],
}

impl Perlin {
    fn generate_permutations(rng: &mut impl RngCore) -> [usize; POINT_COUNT] {
        let mut base = [0; POINT_COUNT];
        for i in 0..base.len() {
            base[i] = i;
        }
        base.shuffle(rng);
        base
    }
    pub fn new(rng: &mut impl RngCore) -> Perlin {
        let mut random_vectors: [Vec3; POINT_COUNT] = [Vec3::zeros(); POINT_COUNT];
        for v in random_vectors.iter_mut() {
            *v = Vec3::from_row_slice(&UnitSphere.sample(rng));
        }

        Perlin {
            random_vectors,
            permutations_x: Perlin::generate_permutations(rng),
            permutations_y: Perlin::generate_permutations(rng),
            permutations_z: Perlin::generate_permutations(rng),
        }
    }

    fn _uninterpolated_noise(&self, p: &Vec3) -> f64 {
        let p = p * 4.0;
        let ix = ((p.x as i64) as usize) % POINT_COUNT;
        let iy = ((p.y as i64) as usize) % POINT_COUNT;
        let iz = ((p.z as i64) as usize) % POINT_COUNT;

        self.random_vectors
            [self.permutations_x[ix] ^ self.permutations_y[iy] ^ self.permutations_z[iz]].x
    }

    fn perlin_interpolation(c: [[[Vec3; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
        let uu = u.powi(2) * (3.0 - 2.0 * u);
        let vv = v.powi(2) * (3.0 - 2.0 * v);
        let ww = w.powi(2) * (3.0 - 2.0 * w);

        let mut acc = 0.0;
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let fi = i as f64;
                    let fj = j as f64;
                    let fk = k as f64;
                    let weight_v = Vec3::new(u - fi, v - fj, w - fk);
                    acc += (fi * uu + (1.0 - fi)*(1.0 - uu))
                        * (fj * vv + (1.0 - fj)*(1.0 - vv))
                        * (fk * ww + (1.0 - fk)*(1.0 - ww))
                        * c[i][j][k].dot(&weight_v)
                }
            }
        }
        acc
    }

    pub fn noise(&self, p: &Vec3) -> f64 {
        let u = p.x - p.x.floor();
        let v = p.y - p.y.floor();
        let w = p.z - p.z.floor();

        // Fun fact : casting from f64 to usize a negative value returns 0 (clamped)
        let i = p.x.floor() as i64 as usize;
        let j = p.y.floor() as i64 as usize;
        let k = p.z.floor() as i64 as usize;
        // Chief kiss
        let mut vals: [[[Vec3; 2]; 2]; 2] = [[[Vec3::zeros(); 2]; 2]; 2];

        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    let (x, y, z) = (
                        (i + di) as usize % POINT_COUNT,
                        (j + dj) as usize % POINT_COUNT,
                        (k + dk) as usize % POINT_COUNT,
                    );
                    // TODO range variables into usize
                    vals[di as usize][dj as usize][dk as usize] = self.random_vectors
                        [self.permutations_x[x] ^ self.permutations_y[y] ^ self.permutations_z[z]];
                }
            }
        }

        // trilinear_interpolation(vals, u, v, w)
        Perlin::perlin_interpolation(vals, u, v, w)
    }
}

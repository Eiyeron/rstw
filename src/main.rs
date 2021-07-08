mod hittable;
mod material;
mod math;
mod noise;
mod render;
mod scheduler;
mod texture;
mod writers;

use crate::noise::Perlin;
use hittable::*;
use material::*;
use math::*;
use rand::rngs::SmallRng;
use rand::RngCore;
use rand::SeedableRng;
use rand_distr::{Distribution, Uniform};
use render::*;
use scheduler::Scheduler;
use std::sync::Arc;
use std::time::Instant;
use texture::*;
use writers::*;

fn _sky_gradient(dir: &Vec3) -> Vec3 {
    let unit_dir = dir.normalize();
    let t = 0.5 * (unit_dir.y + 1.0);
    Vec3::new(1.0, 1.0, 1.0).lerp(&Vec3::new(0.5, 0.7, 1.0), t)
}

fn ray_color(
    ray: Ray,
    background: &Vec3,
    hittable: &dyn Hittable,
    depth: u16,
    rng: &mut impl RngCore,
) -> Vec3 {
    if depth == 0 {
        return Vec3::zeros();
    }

    if let Some(hit) = hittable.hit(&ray, 0.01, f64::INFINITY) {
        let emitted = hit.material.emitted(hit.u, hit.v, &hit.p);
        return match hit.material.scatter(&ray, &hit, rng) {
            Some((outgoing_ray, attenuation)) => {
                let color = ray_color(outgoing_ray, background, hittable, depth - 1, rng);
                emitted
                    + Vec3::new(
                        color.x * attenuation.x,
                        color.y * attenuation.y,
                        color.z * attenuation.z,
                    )
            }
            None => emitted,
        };
    }
    *background
}

// TODO Adapt to add the background and emitted.
fn _ray_color_loop(ray: Ray, hittable: &dyn Hittable, depth: u16, rng: &mut impl RngCore) -> Vec3 {
    let mut current_ray = ray;
    let mut accumulated_color = Vec3::new(1.0, 1.0, 1.0);

    for _n in 0..depth {
        if let Some(hit) = hittable.hit(&current_ray, 0.01, f64::INFINITY) {
            match hit.material.scatter(&current_ray, &hit, rng) {
                Some((outgoing_ray, attenuation)) => {
                    accumulated_color.x *= attenuation.x;
                    accumulated_color.y *= attenuation.y;
                    accumulated_color.z *= attenuation.z;
                    current_ray = outgoing_ray;
                }
                // Ray was absorbed, stop.
                None => return Vec3::zeros(),
            }
        }
        // No touch, end on sky
        else {
            let unit_dir = current_ray.direction.normalize();
            let t = 0.5 * (unit_dir.y + 1.0);
            let sky_ray = Vec3::new(1.0, 1.0, 1.0).lerp(&Vec3::new(0.5, 0.7, 1.0), t);
            return Vec3::new(
                accumulated_color.x * sky_ray.x,
                accumulated_color.y * sky_ray.y,
                accumulated_color.z * sky_ray.z,
            );
        }
    }

    accumulated_color
}

fn earth(center: Vec3, radius: f64) -> Arc<Sphere> {
    let earth_texture = Arc::new(ImageTexture::from_path("data/longlat.png"));
    let earth_mat = Arc::new(Lambertian {
        albedo: earth_texture,
    });
    Arc::new(Sphere {
        center,
        radius,
        material: earth_mat,
    })
}

fn _wave_scene() -> BvhNode {
    let lambertian: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Arc::new(SolidColor::new(0.2, 0.4, 0.6)),
    });

    let checker = Arc::new(Checkerboard {
        albedo_odd: Arc::new(SolidColor::new(0.2, 0.4, 0.6)),
        albedo_even: Arc::new(SolidColor::new(0.6, 0.6, 0.2)),
    });

    let lambertian_2: Arc<dyn Material> = Arc::new(Lambertian { albedo: checker });
    let metal: Arc<dyn Material> = Arc::new(Metal {
        albedo: Arc::new(SolidColor::new(0.7, 0.6, 0.5)),
        roughness: 0.0,
    });
    let glass: Arc<dyn Material> = Arc::new(Dielectric { ior: 1.5 });

    // No need for a hittable_list. A simple vector is largely enough for the process.
    // A scene load/save could be interesting to add.
    let mut objects: Vec<Arc<dyn Hittable>> = Vec::new();
    objects.push(Arc::new(Sphere {
        center: Vec3::new(0.0, -1005.0, 0.0),
        radius: 1000.0,
        material: lambertian_2,
    }));
    for y in (-5..5).step_by(3) {
        for x in -10..10 {
            let r = f64::hypot(x as f64, y as f64);
            let begin = Vec3::new((x * 3) as f64, 0.0, (y * 3) as f64);
            objects.push(Arc::new(MovingSphere {
                center_begin: begin,
                center_end: begin + Vec3::new(0.0, 1.0, 0.0),
                radius: r / 5. + 0.2,
                material: lambertian.clone(),
                time_begin: 0.0,
                time_end: 1.0,
            }));
            let begin = Vec3::new((x * 3) as f64, 0.0, (y * 3 + 3) as f64);
            objects.push(Arc::new(MovingSphere {
                center_begin: begin,
                center_end: begin + Vec3::new(0.0, 1.0, 0.0),
                radius: r / 5. + 0.2,
                material: metal.clone(),
                time_begin: 0.0,
                time_end: 1.0,
            }));
            let begin = Vec3::new((x * 3) as f64, 0.0, (y * 3 + 6) as f64);
            objects.push(Arc::new(MovingSphere {
                center_begin: begin,
                center_end: begin + Vec3::new(0.0, 1.0, 0.0),
                radius: r / 5. + 0.2,
                material: glass.clone(),
                time_begin: 0.0,
                time_end: 1.0,
            }));
        }
    }

    let mut rng = SmallRng::seed_from_u64(0xDEADBEEF);
    BvhNode::from_slice(&objects[..], 0.0, f64::INFINITY, &mut rng)
}

fn book_cover_scene() -> BvhNode {
    let mut world_elements: Vec<Arc<dyn Hittable>> = vec![];
    let mut rng = SmallRng::seed_from_u64(0xDEADBEEF);

    let _checker = Arc::new(Checkerboard {
        albedo_odd: Arc::new(SolidColor::new(0.2, 0.4, 0.6)),
        albedo_even: Arc::new(SolidColor::new(0.6, 0.6, 0.2)),
    });

    let noise = Arc::new(MarbleNoise {
        perlin: Perlin::new(&mut rng),
        scale: 4.,
        depth: 5,
    });

    let ground_mat = Arc::new(Lambertian { albedo: noise });
    world_elements.push(Arc::new(Sphere {
        center: Vec3::new(0.0, -1000.0, 0.0),
        radius: 1000.0,
        material: ground_mat,
    }));

    let material_distribution = Uniform::from(0..4);

    let uniform_dist = Uniform::from(0.0..1.0);
    let metal_dist = Uniform::from(0.5..1.0);
    let emissive_dist = Uniform::from(0.5..4.0);
    let metal_roughness_dist = Uniform::from(0.0..0.5);
    let position_dist = Uniform::from(-0.9..0.9);
    let glass_mat: Arc<dyn Material> = Arc::new(Dielectric { ior: 1.5 });

    let big_sphere_pos_1 = Vec3::new(0.0, 1.0, 0.0);
    let big_sphere_pos_2 = Vec3::new(-4.0, 1.0, 0.0);
    let big_sphere_pos_3 = Vec3::new(4.0, 1.0, 0.0);
    for x in -11..11 {
        for y in -11..11 {
            let center = Vec3::new(
                x as f64 + position_dist.sample(&mut rng),
                0.2,
                y as f64 + position_dist.sample(&mut rng),
            );
            if (center - big_sphere_pos_1).norm() < 1.5 {
                continue;
            }
            if (center - big_sphere_pos_2).norm() < 1.5 {
                continue;
            }
            if (center - big_sphere_pos_3).norm() < 1.5 {
                continue;
            }
            let selector = material_distribution.sample(&mut rng);
            let material: Arc<dyn Material> = match selector {
                0 => {
                    let a1 = generate_vector(&uniform_dist, &mut rng);
                    let a2 = generate_vector(&uniform_dist, &mut rng);
                    let albedo = Arc::new(SolidColor::new(a1.x * a2.x, a1.y * a2.y, a1.z * a2.z));
                    Arc::new(Lambertian { albedo })
                }
                1 => {
                    let albedo = Arc::new(SolidColor {
                        albedo: generate_vector(&metal_dist, &mut rng),
                    });
                    Arc::new(Metal {
                        albedo,
                        roughness: metal_roughness_dist.sample(&mut rng),
                    })
                }
                2 => Arc::clone(&glass_mat),
                3 => {
                    let emissive = Arc::new(SolidColor {
                        albedo: generate_vector(&emissive_dist, &mut rng),
                    });
                    Arc::new(DiffuseLight { emissive })
                }
                _ => panic!("Unreachable"),
            };

            world_elements.push(match selector {
                0 => {
                    let center_2 =
                        center + Vec3::new(0.0, metal_roughness_dist.sample(&mut rng), 0.0);
                    Arc::new(MovingSphere {
                        center_begin: center,
                        center_end: center_2,
                        time_begin: 0.0,
                        time_end: 1.0,
                        radius: 0.2,
                        material,
                    })
                }
                1 | 2 | 3 => Arc::new(Sphere {
                    center,
                    radius: 0.2,
                    material,
                }),
                _ => panic!("Unreachable"),
            });
        }
    }

    let mat1: Arc<dyn Material> = Arc::new(Dielectric { ior: 1.5 });
    world_elements.push(Arc::new(Sphere {
        center: big_sphere_pos_1,
        radius: 1.0,
        material: Arc::clone(&mat1),
    }));
    world_elements.push(Arc::new(Sphere {
        center: big_sphere_pos_1,
        radius: -0.8,
        material: mat1,
    }));
    world_elements.push(earth(big_sphere_pos_2, 1.0));
    let mat2: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Arc::new(SolidColor::new(0.4, 0.2, 0.1)),
    });
    world_elements.push(Arc::new(Sphere {
        center: big_sphere_pos_2,
        radius: 1.0,
        material: mat2,
    }));
    let mat3: Arc<dyn Material> = Arc::new(Metal {
        albedo: Arc::new(SolidColor::new(0.7, 0.6, 0.5)),
        roughness: 0.0,
    });
    world_elements.push(Arc::new(Sphere {
        center: big_sphere_pos_3,
        radius: 1.0,
        material: mat3,
    }));

    world_elements.push(Arc::new(Sphere {
        center: Vec3::new(0.0, 10.0, 0.0),
        radius: 2.0,
        material: Arc::new(DiffuseLight {
            emissive: Arc::new(SolidColor::new(5.0, 5.0, 5.0)),
        }),
    }));

    let mut rng = SmallRng::seed_from_u64(0xDEADBEEF);
    BvhNode::from_slice(&world_elements[..], 0.0, f64::INFINITY, &mut rng)
}

fn main() {
    let max_depth = 500;
    let num_threads = 8;
    let num_iterations = 800;
    let aspect_ratio = 16.0 / 9.0;
    let render_width = 400;
    let render_height: usize = (render_width as f64 / aspect_ratio) as usize;
    let eye = Vec3::new(0.0, 2.0, -5.0);
    let target = Vec3::zeros();
    let world = Arc::new(book_cover_scene());
    let before = Instant::now();
    // Camera derives Copy+Clone, the structure will be copied to the threads.
    let cam = Camera::new(
        eye,
        target,
        Vec3::new(0.0, 1.0, 0.0),
        60.,
        aspect_ratio,
        0.0, // Aperture
        (eye - target).norm(),
        0.0,
        1.0,
    );

    let final_buffer = Scheduler::run_threaded(
        &world,
        &cam,
        num_iterations,
        num_threads,
        render_width,
        render_height as usize,
        max_depth,
    );
    eprintln!("Render took {} seconds", before.elapsed().as_secs());
    write_header(render_width as u32, render_height as u32);
    for v in final_buffer {
        write_color(v, num_iterations as u32);
    }
}

mod hittable;
mod material;
mod math;
mod noise;
mod render;
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
use std::rc::Rc;
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

fn earth(center: Vec3, radius: f64) -> Rc<Sphere> {
    let earth_texture = Rc::new(ImageTexture::from_path("data/longlat.png"));
    let earth_mat = Rc::new(Lambertian {
        albedo: earth_texture,
    });
    Rc::new(Sphere {
        center,
        radius,
        material: earth_mat,
    })
}

fn _wave_scene() -> BvhNode {
    let lambertian: Rc<dyn Material> = Rc::new(Lambertian {
        albedo: Rc::new(SolidColor::new(0.2, 0.4, 0.6)),
    });

    let checker = Rc::new(Checkerboard {
        albedo_odd: Rc::new(SolidColor::new(0.2, 0.4, 0.6)),
        albedo_even: Rc::new(SolidColor::new(0.6, 0.6, 0.2)),
    });

    let lambertian_2: Rc<dyn Material> = Rc::new(Lambertian { albedo: checker });
    let metal: Rc<dyn Material> = Rc::new(Metal {
        albedo: Rc::new(SolidColor::new(0.7, 0.6, 0.5)),
        roughness: 0.0,
    });
    let glass: Rc<dyn Material> = Rc::new(Dielectric { ior: 1.5 });

    // No need for a hittable_list. A simple vector is largely enough for the process.
    // A scene load/save could be interesting to add.
    let mut objects: Vec<Rc<dyn Hittable>> = Vec::new();
    objects.push(Rc::new(Sphere {
        center: Vec3::new(0.0, -1005.0, 0.0),
        radius: 1000.0,
        material: lambertian_2,
    }));
    for y in (-5..5).step_by(3) {
        for x in -10..10 {
            let r = f64::hypot(x as f64, y as f64);
            let begin = Vec3::new((x * 3) as f64, 0.0, (y * 3) as f64);
            objects.push(Rc::new(MovingSphere {
                center_begin: begin,
                center_end: begin + Vec3::new(0.0, 1.0, 0.0),
                radius: r / 5. + 0.2,
                material: lambertian.clone(),
                time_begin: 0.0,
                time_end: 1.0,
            }));
            let begin = Vec3::new((x * 3) as f64, 0.0, (y * 3 + 3) as f64);
            objects.push(Rc::new(MovingSphere {
                center_begin: begin,
                center_end: begin + Vec3::new(0.0, 1.0, 0.0),
                radius: r / 5. + 0.2,
                material: metal.clone(),
                time_begin: 0.0,
                time_end: 1.0,
            }));
            let begin = Vec3::new((x * 3) as f64, 0.0, (y * 3 + 6) as f64);
            objects.push(Rc::new(MovingSphere {
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
    let mut world_elements: Vec<Rc<dyn Hittable>> = vec![];
    let mut rng = SmallRng::seed_from_u64(0xDEADBEEF);

    let _checker = Rc::new(Checkerboard {
        albedo_odd: Rc::new(SolidColor::new(0.2, 0.4, 0.6)),
        albedo_even: Rc::new(SolidColor::new(0.6, 0.6, 0.2)),
    });

    let noise = Rc::new(MarbleNoise {
        perlin: Perlin::new(&mut rng),
        scale: 4.,
        depth: 5,
    });

    let ground_mat = Rc::new(Lambertian { albedo: noise });
    world_elements.push(Rc::new(Sphere {
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
    let glass_mat: Rc<dyn Material> = Rc::new(Dielectric { ior: 1.5 });

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
            let material: Rc<dyn Material> = match selector {
                0 => {
                    let a1 = generate_vector(&uniform_dist, &mut rng);
                    let a2 = generate_vector(&uniform_dist, &mut rng);
                    let albedo = Rc::new(SolidColor::new(a1.x * a2.x, a1.y * a2.y, a1.z * a2.z));
                    Rc::new(Lambertian { albedo })
                }
                1 => {
                    let albedo = Rc::new(SolidColor {
                        albedo: generate_vector(&metal_dist, &mut rng),
                    });
                    Rc::new(Metal {
                        albedo,
                        roughness: metal_roughness_dist.sample(&mut rng),
                    })
                }
                2 => Rc::clone(&glass_mat),
                3 => {
                    let emissive = Rc::new(SolidColor {
                        albedo: generate_vector(&emissive_dist, &mut rng),
                    });
                    Rc::new(DiffuseLight { emissive })
                }
                _ => panic!("Unreachable"),
            };

            world_elements.push(match selector {
                0 => {
                    let center_2 =
                        center + Vec3::new(0.0, metal_roughness_dist.sample(&mut rng), 0.0);
                    Rc::new(MovingSphere {
                        center_begin: center,
                        center_end: center_2,
                        time_begin: 0.0,
                        time_end: 1.0,
                        radius: 0.2,
                        material,
                    })
                }
                1 | 2 | 3 => Rc::new(Sphere {
                    center,
                    radius: 0.2,
                    material,
                }),
                _ => panic!("Unreachable"),
            });
        }
    }

    let mat1: Rc<dyn Material> = Rc::new(Dielectric { ior: 1.5 });
    world_elements.push(Rc::new(Sphere {
        center: big_sphere_pos_1,
        radius: 1.0,
        material: Rc::clone(&mat1),
    }));
    world_elements.push(Rc::new(Sphere {
        center: big_sphere_pos_1,
        radius: -0.8,
        material: mat1,
    }));
    world_elements.push(earth(big_sphere_pos_2, 1.0));
    let mat3: Rc<dyn Material> = Rc::new(Metal {
        albedo: Rc::new(SolidColor::new(0.7, 0.6, 0.5)),
        roughness: 0.0,
    });
    world_elements.push(Rc::new(Sphere {
        center: big_sphere_pos_3,
        radius: 1.0,
        material: mat3,
    }));

    world_elements.push(Rc::new(Sphere {
        center: Vec3::new(0.0, 10.0, 0.0),
        radius: 2.0,
        material: Rc::new(DiffuseLight {
            emissive: Rc::new(SolidColor::new(5.0, 5.0, 5.0)),
        }),
    }));

    let mut rng = SmallRng::seed_from_u64(0xDEADBEEF);
    BvhNode::from_slice(&world_elements[..], 0.0, f64::INFINITY, &mut rng)
}

fn main() {
    let max_depth = 50;
    let num_iterations = 100;
    let aspect_ratio = 16.0 / 9.0;
    let render_width = 1920;
    let render_height = (render_width as f64 / aspect_ratio) as u32;
    let eye = Vec3::new(0.0, 2.0, -5.0);
    let target = Vec3::zeros();
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
    let world = book_cover_scene();
    write_header(render_width, render_height);
    let jitter_distribution = Uniform::from(0.0..1.0);
    let mut rng = SmallRng::seed_from_u64(0xDEADBEEF);

    let before_render = Instant::now();
    for y in 0..render_height {
        if y % 100 == 0 {
            eprintln!("{} lines remaining", render_height - y);
        }
        for x in 0..render_width {
            let mut sum = Vec3::zeros();
            for _sample in 0..num_iterations {
                let jitter_x = jitter_distribution.sample(&mut rng);
                let jitter_y = jitter_distribution.sample(&mut rng);
                let s = (jitter_x + (x as f64)) / (render_width as f64 - 1.0);
                let t = 1.0 - (jitter_y + (y as f64)) / (render_height as f64 - 1.0);

                let ray = cam.get_ray(s, t, &mut rng);
                sum += ray_color(ray, &Vec3::zeros(), &world, max_depth, &mut rng);
            }
            write_color(sum, num_iterations);
        }
    }
    let time_spent = before_render.elapsed().as_secs();
    eprintln!("Render took {:.2} seconds", time_spent);
}

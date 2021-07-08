use crate::hittable::BvhNode;
use crate::math::Vec3;
use crate::ray_color;
use crate::render::{Camera, RenderTile, Subregion};
use rand::rngs::SmallRng;
use rand::SeedableRng;
use rand_distr::{Distribution, Uniform};
use std::sync::Arc;
use std::thread::JoinHandle;

// Making a struct is a forward thought.
//
// I'm thinking of a more fledged out scheduler with a tile pool for a
// determined number of threads.
pub struct Scheduler {}

impl Scheduler {
    fn spawn_thread(
        world: &Arc<BvhNode>,
        cam: &Camera,
        tid: usize,
        num_threads: usize,
        render_width: usize,
        render_height: usize,
        num_iterations: usize,
        max_depth: u16,
    ) -> JoinHandle<RenderTile> {
        let local_world = world.clone();
        let local_camera = cam.clone();
        let subregion = Subregion::slice_vertically(tid, num_threads, render_width, render_height);
        std::thread::spawn(move || {
            let mut worker = RenderTile::new(subregion, local_world, local_camera);
            let jitter_distribution = Uniform::from(0.0..1.0);
            let mut rng = SmallRng::seed_from_u64(tid as u64);

            let width_minus_one = render_width as f64 - 1.0;
            let height_minus_one = render_height as f64 - 1.0;

            for y in 0..worker.region.height {
                let tile_y_offset = y * worker.region.width;
                let final_y_offset = (y + worker.region.y) as f64;

                for x in 0..worker.region.width {
                    let mut sum = Vec3::zeros();
                    let final_x_offset = (x + worker.region.x) as f64;

                    for _sample in 0..num_iterations {
                        let jitter_x = jitter_distribution.sample(&mut rng);
                        let jitter_y = jitter_distribution.sample(&mut rng);

                        let s = (jitter_x + final_x_offset) / width_minus_one;
                        let t = 1.0 - (jitter_y + final_y_offset) / height_minus_one;

                        let ray = worker.camera.get_ray(s, t, &mut rng);
                        sum += ray_color(
                            ray,
                            &Vec3::zeros(),
                            worker.scene.as_ref(),
                            max_depth,
                            &mut rng,
                        );
                    }
                    worker.buffer[tile_y_offset + x] = sum;
                }
            }
            worker
        })
    }

    pub fn run_threaded(
        world: &Arc<BvhNode>,
        cam: &Camera,
        num_iterations: usize,
        num_threads: usize,
        render_width: usize,
        render_height: usize,
        max_depth: u16,
    ) -> Vec<Vec3> {
        let mut thread_handles = vec![];

        for tid in 0..num_threads {
            thread_handles.push(Scheduler::spawn_thread(
                world,
                cam,
                tid,
                num_threads,
                render_width,
                render_height,
                num_iterations,
                max_depth,
            ));
        }

        // Untile data and blit to the final buffer.
        let mut final_buffer = vec![Vec3::zeros(); (render_height * render_width) as usize];
        for tid in thread_handles {
            match tid.join() {
                Ok(worker) => {
                    for y in 0..worker.region.height {
                        let y_offset = worker.region.y + y;
                        let x_offset = y * worker.region.width;
                        let out_buffer_y_offset = y_offset * render_width + worker.region.x;
                        for x in 0..worker.region.width {
                            let in_index = x_offset + x;
                            let out_index = out_buffer_y_offset + x;
                            final_buffer[out_index] = worker.buffer[in_index];
                        }
                    }
                }
                Err(err) => std::panic::panic_any(err),
            };
        }
        final_buffer
    }
}

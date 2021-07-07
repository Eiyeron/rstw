use crate::math::Vec3;
use crate::Hittable;
use crate::Material;
use rand::RngCore;
use rand_distr::{Distribution, Uniform, UnitDisc};
use std::sync::Arc;

pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
    pub time: f64,
}

impl Ray {
    pub fn at(&self, t: f64) -> Vec3 {
        self.origin + t * self.direction
    }
}

pub struct HitRecord<'a> {
    pub t: f64,
    pub p: Vec3,
    pub normal: Vec3,
    pub front_facing: bool,
    pub material: &'a dyn Material,
    pub u: f64,
    pub v: f64,
}

impl<'a> HitRecord<'a> {
    pub fn set_face_normal(incoming: Vec3, outward_normal: Vec3) -> (bool, Vec3) {
        let front_facing = incoming.dot(&outward_normal) < 0.0;
        let normal = {
            if front_facing {
                outward_normal
            } else {
                -outward_normal
            }
        };
        (front_facing, normal)
    }
    pub fn from(
        t: f64,
        p: Vec3,
        incoming: Vec3,
        normal: Vec3,
        material: &dyn Material,
    ) -> HitRecord {
        let (front_facing, normal) = HitRecord::set_face_normal(incoming, normal);
        HitRecord {
            t,
            p,
            normal,
            front_facing,
            material,
            u: 0.0,
            v: 0.0,
        }
    }
    pub fn from_uv(
        t: f64,
        p: Vec3,
        incoming: Vec3,
        normal: Vec3,
        material: &dyn Material,
        u: f64,
        v: f64,
    ) -> HitRecord {
        let (front_facing, normal) = HitRecord::set_face_normal(incoming, normal);
        HitRecord {
            t,
            p,
            normal,
            front_facing,
            material,
            u,
            v,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Camera {
    pub origin: Vec3,
    pub horizontal: Vec3,
    pub vertical: Vec3,
    pub lower_left_corner: Vec3,

    pub u: Vec3,
    pub v: Vec3,
    pub w: Vec3,

    pub lens_radius: f64,
    pub time_begin: f64,
    pub time_end: f64,
}

impl Camera {
    pub fn new(
        eye: Vec3,
        target: Vec3,
        up: Vec3,
        vertical_fov: f64,
        aspect_ratio: f64,
        aperture: f64,
        focus_distance: f64,
        time_0: f64,
        time_1: f64,
    ) -> Camera {
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
            horizontal,
            vertical,

            u,
            v,
            w,

            lower_left_corner: eye - horizontal / 2. - vertical / 2. - focus_distance * w,
            lens_radius: aperture / 2.,
            time_begin: time_0,
            time_end: time_1,
        }
    }

    pub fn get_ray(&self, s: f64, t: f64, rng: &mut impl RngCore) -> Ray {
        let rd: [f64; 2] = UnitDisc.sample(rng);
        let offset = (self.u * rd[0] + self.v * rd[1]) * self.lens_radius;
        // TODO seedable shutter time
        let shutter_time = Uniform::from(self.time_begin..self.time_end).sample(rng);
        Ray {
            origin: self.origin + offset,
            direction: self.lower_left_corner + s * self.horizontal + t * self.vertical
                - self.origin
                - offset,
            time: shutter_time,
        }
    }
}

pub struct Subregion {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

impl Subregion {
    pub fn grid_cell(
        x: usize,
        y: usize,
        cells_x: usize,
        cells_y: usize,
        render_width: usize,
        render_height: usize,
    ) -> Subregion {
        let base_cell_width = render_width / cells_x;
        let current_cell_width = {
            if x == cells_x - 1 {
                render_width - (base_cell_width * (cells_x - 1))
            } else {
                base_cell_width
            }
        };
        let base_cell_height = render_height / cells_y;
        let current_cell_height = {
            if y == cells_y - 1 {
                render_height - (base_cell_height * (cells_y - 1))
            } else {
                base_cell_height
            }
        };
        Subregion {
            x: base_cell_width * x,
            y: base_cell_height * y,
            width: current_cell_width,
            height: current_cell_height,
        }
    }

    pub fn slice_vertically(
        y: usize,
        cells_y: usize,
        render_width: usize,
        render_height: usize,
    ) -> Subregion {
        let base_cell_height = render_height / cells_y;
        let current_cell_height = {
            if y == cells_y - 1 {
                render_height - (base_cell_height * (cells_y - 1))
            } else {
                base_cell_height
            }
        };
        Subregion {
            x: 0,
            y: base_cell_height * y,
            width: render_width,
            height: current_cell_height,
        }
    }

    pub fn area(&self) -> usize {
        self.width * self.height
    }
}

pub struct RenderTile {
    pub region: Subregion,
    pub buffer: Vec<Vec3>,
    pub scene: Arc<dyn Hittable>,
    pub camera: Camera,
}

impl RenderTile {
    pub fn new(region: Subregion, scene: Arc<dyn Hittable>, camera: Camera) -> RenderTile {
        let buffer_size = region.area();
        RenderTile {
            region,
            buffer: vec![Vec3::zeros(); buffer_size],
            scene,
            camera,
        }
    }
}

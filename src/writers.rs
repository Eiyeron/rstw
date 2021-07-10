// TODO trait?
use crate::colors;
use crate::math::Vec3;
use image::png::PngEncoder;
use image::ColorType;
use std::convert::TryInto;
use std::io::Write;

pub trait ImageWriter {
    // TODO Extract linear → sRGB conversion out of the interface
    fn write_to(
        &self,
        out: &mut dyn Write,
        data: &[Vec3],
        width: usize,
        height: usize,
        num_samples: u32,
    );
}

pub struct PPMWriter;

impl PPMWriter {
    pub fn write_header(out: &mut dyn Write, width: usize, height: usize) {
        writeln!(out, "P3 {} {}\n255", width, height).unwrap();
    }

    pub fn write_color(out: &mut dyn Write, color: &Vec3, num_samples: u32) {
        let average = color / (num_samples as f64);

        let srgb = colors::linear_to_srgb(&average);
        let (r, g, b) = colors::downscale_to_8bit(&srgb);

        writeln!(out, "{} {} {}", r, g, b).unwrap();
    }
}

impl ImageWriter for PPMWriter {
    fn write_to(
        &self,
        out: &mut dyn Write,
        data: &[Vec3],
        width: usize,
        height: usize,
        num_samples: u32,
    ) {
        assert_eq!(data.len(), width * height);
        PPMWriter::write_header(out, width, height);
        data.iter()
            .for_each(|v| PPMWriter::write_color(out, v, num_samples));
    }
}

pub struct PNGWriter;

impl ImageWriter for PNGWriter {
    fn write_to(
        &self,
        out: &mut dyn Write,
        data: &[Vec3],
        width: usize,
        height: usize,
        num_samples: u32,
    ) {
        let encoder = PngEncoder::new(out);
        let mut encodable_data = vec![];
        for c in data {
            let average = c / num_samples as f64;
            let srgb = colors::linear_to_srgb(&average);
            let (r, g, b) = colors::downscale_to_8bit(&srgb);
            encodable_data.push(r);
            encodable_data.push(g);
            encodable_data.push(b);
        }
        encoder
            .encode(
                &encodable_data,
                width.try_into().unwrap(),
                height.try_into().unwrap(),
                ColorType::Rgb8,
            )
            .unwrap();
    }
}

pub fn guess_output_format(extension: &str) -> Option<Box<dyn ImageWriter>> {
    let cleaned_extension = extension.to_lowercase();
    match &cleaned_extension as &str {
        "ppm" => Some(Box::new(PPMWriter {})),
        "png" => Some(Box::new(PNGWriter {})),
        _ => None,
    }
}

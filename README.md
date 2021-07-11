# RSTW
## A toy raytracer

![Sample picture](/.meta/rotation_boxes.png)

This is the result of me following [Ray Tracing in One Weekend][rtiow],
more or less closely in an attempt at grokking some aspects of raytracing. The
project will follow at least up to The Next Week before a probable hiatus until
I understand Importance Sampling.

## Changes from the original project
- The implementation language : Rust
- Command line support
- Math based on [nalgebra] and [rand] (and rand_distr).
- Using the [image] for PNG read/write.
- Multi-threading based on slicing or tiling the final buffer to attribute one
  to each thread. Raw important speed boost for CPU with higher number of
  cores.
- A global `Transform` hittable wrapping a `Mat4` instead of specific ones.

## Command line
- `-h | --help` (provided by [arg])
- `--width [width]` Render width, default 400
- `--height [height]` Render height, default 300
- `-d [num] | --depth [num]` Ray depth (a.k.a. number of bounces), default 10
- `-s [num] | --samples [num]` Samples per pixel, default 100
- `-t [num] | --thread [num]` Number of worker threads, default 4
- `-o [path] | --output [path]` Path for file output (`-o -` to force output
  to stdout).
  - Reads the filename's extension to guess the encoding type.
    - `.png`
    - `.ppm` (and fallback format)

Here's the command line to generate the provided picture:

```sh
                    # -- Delimits cargo's arguments from the executable's
$ cargo run --release -- --width 600 --height 600 -d 10 -s 1000 -t 8 -o .meta/rotation_boxes.png
```

(The actual result *will* change according to what scene is currently written
in the source code.)

## Vague ideas for the future

- Mesh support.
- Scene description
- Animation support (mostly rendering N frames)
- Explore ideas around game engine oriented probes (ambient and reflection)

[rtiow]:https://raytracing.github.io/
[image]:https://crates.io/crates/image
[nalgebra]:https://nalgebra.org/
[rand]:https://rust-random.github.io/book/
[arg]: https://github.com/DoumanAsh/arg.rs

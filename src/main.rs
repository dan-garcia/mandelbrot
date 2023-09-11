use std::path::PathBuf;
use image::ImageFormat::PNG;
use num::Complex;
use clap::{Parser, ValueEnum};
use image::ColorType;
use image::png::PNGEncoder;
use std::fs::File;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'F', long)]
    file: PathBuf,

    #[arg(short = 'W', long)]
    width: usize,

    #[arg(short = 'H', long)]
    height: usize,

    #[arg(short = 'L', long)]
    upper_left: Complex<f64>,

    #[arg(short = 'R', long)]
    lower_right: Complex<f64>,

    #[arg(short = 'l', long)]
    limit: Limit,
}
#[derive(Debug, Clone, Copy, ValueEnum)]
enum Limit {
    Low,
    Medium,
    High
}

impl Limit{
    fn value(&self) -> usize {
        match *self {
            Limit::Low => 256,
            Limit::Medium => 512,
            Limit::High => 1024
        }
    }
}

fn main() {
    let args = Args::parse();
}

fn render(pixels: &mut [u8], bounds: (usize, usize), upper_left: Complex<f64>, lower_right: Complex<f64>, limit: Limit){
    assert!(pixels.len() == bounds.0 * bounds.1);
    for row in 0..bounds.1 {
        for col in 0..bounds.0 {
            let point = pixel_to_point(bounds, (col, row), upper_left, lower_right);
            pixels[row * bounds.0 + col] = 
                match escape_time(point, limit){
                    None => 0,
                    Some(count) => (limit.value() - count)  as u8
                };
        }
    }
}

fn write_image(filename: &str, pixels: &[u8], bounds: (usize, usize)) -> Result<(), std::io::Error>{
    let output = File::create(filename)?;
    let encoder = PNGEncoder::new(output);
    encoder.encode(pixels, bounds.0 as u32, bounds.1 as u32, ColorType::Gray(8))?;

    Ok(())
}

/// Try to prove if `c` is in the Mandelbrot set, using `limit` iterations.
/// 
/// If `c` is proven to be outside of the set, return `Some(i)`,
/// where `i` is the number of iterations it took to prove it
/// was not a member of the set. Otherwise, if not proven to be
/// outisde the set, return `None`.
fn escape_time(c: Complex<f64>, limit: Limit) -> Option<usize>{
    let mut z = Complex {re: 0.0, im: 0.0};
    for i in 0..limit.value(){
        if z.norm_sqr() > 4.0 {
            return Some(i);
        }
        z = z * z + c;
    }
    None
}

fn pixel_to_point(bounds: (usize, usize), pixel: (usize, usize), upper_left: Complex<f64>, lower_right: Complex<f64>) -> Complex<f64>{
    let (width, height) = (lower_right.re - upper_left.re,
                                upper_left.im - lower_right.im);
    Complex {
        re: upper_left.re + pixel.0 as f64 * width / bounds.0 as f64,
        im: upper_left.im - pixel.1 as f64 * height / bounds.1 as f64
    }
}

#[test]
fn test_pixel_to_point(){
    assert_eq!(pixel_to_point((100, 200), 
                                (25, 175), 
                                Complex{re: -1.0, im: 1.0}, 
                                Complex{re: 1.0, im: -1.0}),
                Complex{re: -0.5, im: -0.75});
}

use std::path::PathBuf;
use num::Complex;
use clap::{Parser, ValueEnum};
use image::{ColorType, ImageEncoder};
use image::codecs::png::PngEncoder;
use std::fs::File;

#[derive(Parser)]
#[command(author, version, about, long_about = None, allow_hyphen_values(true))]
struct Args {
    #[arg(short = 'F', long)]
    file: PathBuf,

    #[arg(short = 'W', long)]
    width: usize,

    #[arg(short = 'H', long)]
    height: usize,

    //FIXME: having to use re and im parts as 
    //      workaround getting clap to parse complex nums
    #[arg(short = 'L', long)]
    upper_left_re: f64,

    #[arg(short = 'l', long)]
    upper_left_im: f64,

    #[arg(short = 'R', long)]
    lower_right_re: f64,

    #[arg(short = 'r', long)]
    lower_right_im: f64,

    #[arg(short = 'x', long)]
    limit: Limit,

    #[arg(short = 't', long)]
    threads : u8
}
#[derive(Debug, Clone, Copy, ValueEnum)]
enum Limit {
    VeryLow,
    Low,
    Medium,
    High
}

impl Limit{
    fn value(&self) -> usize {
        match *self {
            Limit::VeryLow => 128,
            Limit::Low => 256,
            Limit::Medium => 512,
            Limit::High => 1024
        }
    }
}

fn main() {
    let args = Args::parse();
    let bounds = (args.width, args.height);
    let upper_left = Complex {re: args.upper_left_re, im: args.upper_left_im};
    let lower_right = Complex {re: args.lower_right_re, im: args.lower_right_im};


    let mut pixels = vec![0; args.width * args.height];
    if args.threads == 1 {
        render(&mut pixels, bounds, upper_left, lower_right, args.limit);
    } else if args.threads > 1{
        let rows_per_band = args.height / (args.threads as usize) + 1;
        let bands : Vec<&mut [u8]> = pixels.chunks_mut(rows_per_band * args.width).collect();
        crossbeam::scope(|spawner| {
            for (i, band) in bands.into_iter().enumerate() {
                let top = rows_per_band * i;
                let height = band.len() / args.width;
                let band_bounds = (args.width, height);
                let band_upper_left = pixel_to_point(bounds, (0, top), upper_left, lower_right);
                let band_lower_right = pixel_to_point(bounds, (args.width, top + height), upper_left, lower_right);
                spawner.spawn(move |_| {
                    render(band, band_bounds, band_upper_left, band_lower_right, args.limit);
                });
            }
        }).unwrap();
    } else {
        panic!("Please enter a valid value for # of threads")
    }
    
    let _ = write_image(args.file.to_str().unwrap_or("default"), &pixels, (args.width, args.height));

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
    let encoder = PngEncoder::new(output);
    match encoder.write_image(pixels, bounds.0 as u32, bounds.1 as u32, ColorType::L8){
        Ok(()) => (),
        Err(e) => panic!("oops, {}", e)
    }

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

use std::env;
use std::fs::File;
use std::str::FromStr;

use image::png::PNGEncoder;
use image::ColorType;
use num::Complex;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 5 {
        eprintln!("Usage: {} FILE PIXELS UPPERLEFT LOWERRIGHT", args[0]);
        eprintln!(
            "Example: {} mandel.png 1000x750 -1.20,0.35 -1,0.20",
            args[0]
        );
        std::process::exit(1);
    }

    let bounds = parse_pair(&args[2], 'x').expect("Error parsing image dimensions.");
    let upper_left = parse_complex(&args[3]).expect("Error parsing upper left corner point.");
    let lower_right = parse_complex(&args[4]).expect("Error parsing lower right corner point.");

    let total_pixels = bounds.0 * bounds.1;
    let mut pixels = vec![0; total_pixels];

    // Now we let Rayon take care of the parallelism
    // let threads = num_cpus::get();
    // println!("Running on {threads}");
    {
        // let bands: Vec<&mut [u8]> = pixels.chunks_mut(rows_per_band * bounds.0).collect();
        let bands: Vec<(usize, &mut [u8])> = pixels.chunks_mut(bounds.0).enumerate().collect();

        bands.into_par_iter().for_each(|(i, band)| {
            let top = i;
            let width = bounds.0;
            let height = 1;
            let band_bounds = (width, height); // Just one row
            let band_upper_left = pixel_to_point(bounds, (0, top), upper_left, lower_right);
            let band_lower_right =
                pixel_to_point(bounds, (width, top + height), upper_left, lower_right);
            render(band, band_bounds, band_upper_left, band_lower_right);
        });
    }

    write_image(&args[1], &pixels, bounds).expect("Error writing PNG file.");
}

/// Try to determine if `c` is in the Mandelbrot set, using at most `limit`
/// iterations to decide.
///
/// If `c` is not a member, return `Some(i)`, where `i` is the number of
/// iterations it took for `c` to leave the circle of radius 2 centered on the
/// origin. If `c` seems to be a member (more precisely, if we reached the
/// iteration limit without being able to prove that `c` is not a member),
/// return `None`.
fn escape_time(c: Complex<f64>, limit: usize) -> Option<usize> {
    let mut z = Complex { re: 0.0, im: 0.0 };
    for i in 0..limit {
        if z.norm_sqr() > 4.0 {
            return Some(i);
        }
        z = z * z + c;
    }

    None
}

/// Parse the string `s` as a coordinate pair, like `"400x600"` or `"1.0,0.5`.
///
/// Specifically, `s` should have the form <left><separator><right>, where <separator>
/// is the character given by the `separator` argument, and <left> and <right> are both
/// strings that can be parsed to `T` through `T::from_str`. `separator` must be an
/// ASCII character.
///
/// If `s` parses correctly, return `Some<(x, y)>`. Otherwise, return `None`.
fn parse_pair<T: FromStr>(s: &str, separator: char) -> Option<(T, T)> {
    match s.find(separator) {
        None => None,
        Some(index) => {
            let left = &s[..index];
            // Try to convert to type T
            let left = T::from_str(left);

            // Note that we are ignoring the separator char, at index
            let right = &s[index + 1..];
            let right = T::from_str(right);

            // We need both conversions to have succeeded
            if let (Ok(left), Ok(right)) = (left, right) {
                Some((left, right))
            } else {
                None
            }
        }
    }
}

#[test]
fn test_parse_pair() {
    // Can't parse empty string
    assert_eq!(parse_pair::<i32>("", ','), None);
    // Missing second value
    assert_eq!(parse_pair::<i32>("10,", ','), None);
    // Missing first value
    assert_eq!(parse_pair::<i32>(",10", ','), None);
    // Missing separator
    assert_eq!(parse_pair::<i32>("1020", ','), None);
    // Can parse both numbers
    assert_eq!(parse_pair::<i32>("10,20", ','), Some((10, 20)));
    // Can't convert 20xy to i32, so the parsing fails
    assert_eq!(parse_pair::<i32>("10,20xy", ','), None);
    // Also works when separator is alphabetic
    assert_eq!(parse_pair::<i32>("20x10", 'x'), Some((20, 10)));
    // Missing second value
    assert_eq!(parse_pair::<f64>("0.5x", 'x'), None);
    // Also works with floats
    assert_eq!(parse_pair::<f64>("0.5x1.5", 'x'), Some((0.5, 1.5)));
}

fn parse_complex(s: &str) -> Option<Complex<f64>> {
    let pair = parse_pair(s, ',')?;

    Some(Complex {
        re: pair.0,
        im: pair.1,
    })
}

/// Given the row and column of a pixel in the output image, return the
/// corresponding point on the complex plane.
///
/// `bounds` is a pair giving the width and height of the image in pixels.
/// `pixel` is a (column, row) pair indicating a particular pixel in that image.
/// The `upper_left` and `lower_right` parameters are points on the complex plane
/// designating the area our image covers.
fn pixel_to_point(
    bounds: (usize, usize),
    pixel: (usize, usize),
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) -> Complex<f64> {
    // We treat re as x and im as y
    let (width, height) = (
        lower_right.re - upper_left.re,
        upper_left.im - lower_right.im,
    );

    let pixel_x = pixel.0 as f64;
    let pixel_y = pixel.1 as f64;
    let bounds_x = bounds.0 as f64;
    let bounds_y = bounds.1 as f64;

    Complex {
        re: upper_left.re + pixel_x * width / bounds_x,
        im: upper_left.im - pixel_y * height / bounds_y,
        // We subtract because pixel y increases as we go down,
        // but the imaginary component increases as we go up
    }
}

#[test]
fn test_pixel_to_point() {
    assert_eq!(
        pixel_to_point(
            (100, 200),
            (25, 175),
            Complex { re: -1.0, im: 1.0 },
            Complex { re: 1.0, im: -1.0 },
        ),
        Complex {
            re: -0.5,
            im: -0.75,
        }
    );
}

fn render(
    pixels: &mut [u8],
    bounds: (usize, usize),
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) {
    assert_eq!(pixels.len(), bounds.0 * bounds.1);

    for row in 0..bounds.1 {
        for column in 0..bounds.0 {
            let point = pixel_to_point(bounds, (column, row), upper_left, lower_right);
            let pixel_index = row * bounds.0 + column;
            pixels[pixel_index] = match escape_time(point, 255) {
                None => 0,                        // Black color
                Some(count) => 255 - count as u8, // The bigger count is, the darker the color
            };
        }
    }
}

/// Write the buffer `pixels`, whose dimensions are given by `bounds`, to the
/// file named `filename`
fn write_image(
    filename: &str,
    pixels: &[u8],
    bounds: (usize, usize),
) -> Result<(), std::io::Error> {
    let output = File::create(filename)?;

    let encoder = PNGEncoder::new(output);
    encoder.encode(pixels, bounds.0 as u32, bounds.1 as u32, ColorType::Gray(8))?;

    Ok(())
}

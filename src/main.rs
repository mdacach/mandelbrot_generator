use std::str::FromStr;

use num::Complex;

fn main() {
    // square_loop(-2.1);
    escape_time(
        Complex {
            re: 0.25,
            im: -0.51,
        },
        1000,
    );
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

            // We need both converions to have succeeded
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

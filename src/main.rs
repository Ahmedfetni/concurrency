
extern crate num;
use num::{Complex, traits::bounds};

#[allow(dead_code)]
fn complex_square_add_loop(c:Complex<f64>){
    let mut z = Complex{re:0.0, im:0.0};
    loop {
        z = z * z + c;
    }
}


// to determine wether a value is in the mandelbrot set
fn escape_time(c:Complex<f64>, limit: u32) -> Option<u32> {
    let mut z = Complex{re: 0.0, im: 0.0};
    for i in 0..limit {
        z =  z*z +c ;
        if z.norm().sqrt() > 4.0 {
            return Some(i);
        }
    }
    None 
}

//parsing command line argument as a complex number 
use std::str::FromStr;

fn parse_pair<T: FromStr>(s: &str, separator: char) -> Option<(T, T)> {
    match s.find(separator) {
        None => None,
        Some(index) => {
            match (T::from_str(&s[..index]), T::from_str(&s[index + 1..])) {
                                (Ok(l), Ok(r)) => Some((l, r)), // means if both matched elements are ok return a tuple 
                            _=> None
            }
        }
    }
}

//convert parsed pair into a complex element
fn parse_complex(s: &str) ->Option<Complex<f64>>{
    match parse_pair(s,',') {
        None => None,
        Some((re,im)) => Some(Complex{re, im})
    }
}

//Mapping from Pixels to Complex Numbers

fn pixel_to_point(bounds: (usize, usize),
                    pixel: (usize, usize),
                    upper_left: Complex<f64>,
                    lower_right: Complex<f64>,
) -> Complex<f64>{

        let (width,height) = (lower_right.re- upper_left.re, upper_left.im-lower_right.im);

        Complex {
            re: upper_left.re + pixel.0 as f64 * width / bounds.0 as f64,
            im: upper_left.im - pixel.1 as f64 * height / bounds.1 as f64
        }
           

}

//plotting the set 
fn render(
    pixels: &mut [u8],
    bounds:(usize,usize),
    upper_left: Complex<f64>,
    lower_right: Complex<f64>
){
    assert!(pixels.len() == bounds.0* bounds.1);
    for  row in 0..bounds.1 {
        for col in 0..bounds.0{
            let point =  pixel_to_point(bounds, (col,row), upper_left, lower_right);
            pixels[row + bounds.0+col] = match escape_time(point, 255) {
                None =>0, //Black color 
                Some(count)=> 255- count as u8 //as darker the number that took longer to escape the circle 
            };
        }
    }
}


//Writing Image Files 

extern crate image;
use image::ColorType;
use image::png::PNGEncoder;
use std::fs::File;

fn write_image(filename:&str,pixels:&[u8],bounds:(usize,usize)) -> Result<(),std::io::Error>{
        let output = File::create(filename)?;
        let encoder = PNGEncoder::new(output);

        encoder.encode(&pixels, bounds.0 as u32, bounds.1 as u32,ColorType::Gray(8));

        Ok(())
}
/*
    let output  = match File::create(filename){
        Ok(f)=> {f},
        Err(e) => {return Err(e);}
    }

    is the same as 

    let output = File::create(filename)?;
 */


//in the main function we will put concurrency to work
use std::io::Write;
extern crate crossbeam;

fn main() {
    let args:Vec<String> = std::env::args().collect();

    if args.len() != 5 {
        writeln!(std::io::stderr(),
        "Usage: mandelbrot FILE PIXELS UPPERLEFT LOWERRIGHT")
        .unwrap();
        writeln!(std::io::stderr(),
        "Example: {} mandel.png 1000x750 -1.20,0.35 -1,0.20",
        args[0])
        .unwrap();
        std::process::exit(1);
    }

    let bounds = parse_pair(&args[2], 'x').expect("error parsing image dimensions");

    let upper_left = parse_complex(&args[3]).expect("error parsing upper left corner point");
    
    let lower_right = parse_complex(&args[4]).expect("error parsing lower right corner point");

    let mut pixels = vec![0; bounds.0 * bounds.1];
    
    let threads = 8;
let rows_per_band = bounds.1 / threads + 1;


    let bands: Vec<&mut [u8]> =pixels.chunks_mut(rows_per_band * bounds.0).collect();

    crossbeam::scope(|spawner| {
        for (i, band) in bands.into_iter().enumerate() {
            
            let top = rows_per_band * i;
            
            let height = band.len() / bounds.0;
            
            let band_bounds = (bounds.0, height);
            
            let band_upper_left =
            pixel_to_point(bounds, (0, top), upper_left, lower_right);
            
            let band_lower_right =
            pixel_to_point(bounds, (bounds.0, top + height),
            upper_left, lower_right);
            
            spawner.spawn(move || {render(band, band_bounds, band_upper_left, band_lower_right);
            });
        }
    });


    
    write_image(&args[1], &pixels, bounds).expect("error writing PNG file");
    
    



}

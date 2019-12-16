extern crate image;

use std::io::{self, prelude::*};
use std::fs::File;

use image::{ImageBuffer, LumaA, ImageFormat};

type Digit = u8;
type DigitMatrix = Vec<Vec<Digit>>;
type Layer = DigitMatrix;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum Pixel {
    Black = 0,
    White = 1,
    Transparent = 2
}

// byte pair for LumaA pixel
impl From<Pixel> for [u8; 2] {
    fn from(pixel: Pixel) -> Self {
        let luma: u8 = match pixel {
            Pixel::Black => 0xff,
            Pixel::White | Pixel::Transparent => 0x00
        };
        let alpha: u8 = match pixel {
            Pixel::Transparent => 0x00,
            _ => 0xff
        };

        [luma, alpha]
    }
}

struct Image {
    width: u32,
    height: u32,
    data: Vec<Pixel>
}

impl Into<ImageBuffer<LumaA<u8>, Vec<u8>>> for Image {
    fn into(self) -> ImageBuffer<LumaA<u8>, Vec<u8>> {
        let pixels_luma_alpha = self.data.into_iter()
                                         .map(|p| -> [u8; 2] { p.into() });
        let mut pixel_vec: Vec<u8> = Vec::new();
        for pixel in pixels_luma_alpha {
            pixel_vec.extend_from_slice(&pixel);
        }
        ImageBuffer::from_raw(self.width, self.height, pixel_vec).unwrap()
    }
}

trait ChunksExactVec<T> {
    fn chunks_exact_vec(&self, chunk_size: usize) -> Vec<Vec<T>>;
}

impl<T: Clone> ChunksExactVec<T> for Vec<T> {
    fn chunks_exact_vec(&self, chunk_size: usize) -> Vec<Vec<T>> {
        self.chunks_exact(chunk_size).map(|s| Vec::from(s)).collect()
    }
}

trait CountDigitOccurrences {
    fn count_digit_occurrences(&self, digit: Digit) -> usize;
}

impl CountDigitOccurrences for Layer {
    fn count_digit_occurrences(&self, digit: Digit) -> usize {
        self.into_iter().flatten().filter(|&d| *d == digit).count()
    }
}

fn parse_input(buffer: String, width: u32, height: u32) -> Vec<Layer> {
    let chars_iter = buffer.trim().chars();
    let digits: Vec<Digit> = chars_iter
                             .map(|c| c.to_digit(10).unwrap() as Digit)
                             .collect();
    let rows: DigitMatrix = digits.chunks_exact_vec(width as usize);

    rows.chunks_exact_vec(height as usize)
}

fn solve_part1(layers: &Vec<Layer>) -> usize {
    let zero_digits_in_layers = layers.clone()
                                      .into_iter()
                                      .map(|l| l.count_digit_occurrences(0));
    let (least_zeros_index, _) = zero_digits_in_layers.enumerate()
                                                      .min_by_key(|(_, x)| *x)
                                                      .unwrap();
    let least_zeros_layer: &Layer = &layers[least_zeros_index];
    let one_digits_count = least_zeros_layer.count_digit_occurrences(1);
    let two_digits_count = least_zeros_layer.count_digit_occurrences(2);

    one_digits_count * two_digits_count
}

fn process_image(layers: Vec<Layer>, width: u32, height: u32) -> Vec<Pixel> {
    let image_size = width * height;
    let layers_pixels: Vec<Vec<Digit>> = layers.into_iter()
                                               .map(|l| l.into_iter()
                                                         .flatten()
                                                         .collect())
                                               .collect();
    let mut img: Vec<Pixel> = vec![Pixel::Transparent; image_size as usize];
    // first layer is the top, so start processing the image starting
    // with the last layer
    for layer in layers_pixels.into_iter().rev() {
        for (i, pixel) in layer.into_iter().enumerate() {
            match pixel {
                0 => { img[i] = Pixel::Black; }
                1 => { img[i] = Pixel::White; }
                _ => { continue; }
            }
        }
    }
    img
}

fn main() -> io::Result<()> {
    const WIDTH: u32 = 25;
    const HEIGHT: u32 = 6;

    let mut f = File::open("8.txt")?;
    let mut buffer = String::new();
    f.read_to_string(&mut buffer)?;
    let layers = parse_input(buffer, WIDTH, HEIGHT);
    println!("Part 1 answer: {}", solve_part1(&layers));

    let img = Image {
        width: WIDTH,
        height: HEIGHT,
        data: process_image(layers, WIDTH, HEIGHT)
    };
    let img_buf: ImageBuffer<LumaA<u8>, Vec<u8>> = img.into();
    const SAVE_PATH: &'static str = "8.png";
    img_buf.save_with_format(SAVE_PATH, ImageFormat::PNG)?;
    println!("Image saved as {}", SAVE_PATH);
    Ok(())
}

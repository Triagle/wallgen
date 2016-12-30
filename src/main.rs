extern crate argparse;
extern crate image;
extern crate rand;

use std::fmt::Display;
use std::error::Error;
use std::fs::File;
use std::path::Path;
use std::fmt;
use rand::Rng;
use argparse::{ArgumentParser, Store};

// Shapes to draw
trait Drawable {

    fn draw(&self, px: &mut image::Rgb<u8>, x: u32, y: u32) -> bool;
}

struct Point(u32, u32);

struct Rect {
    origin: Point,
    length: u32,
    height: u32,
    colour: image::Rgb<u8>
}
impl Drawable for Rect {
    fn draw(&self, px: &mut image::Rgb<u8>, x: u32, y: u32) -> bool {
        if x > self.origin.0 && x < self.origin.0 + self.length && y > self.origin.1 && y < self.origin.1 + self.height {
            *px = self.colour;
            true
        } else {
            false
        }
    }
}

struct Circle {
    origin: Point,
    radius: u32,
    colour: image::Rgb<u8>
}
impl Drawable for Circle {
    fn draw(&self, px: &mut image::Rgb<u8>, x: u32, y: u32) -> bool {
        if (x as i32 - self.origin.0 as i32).pow(2) + (y as i32 - self.origin.1 as i32).pow(2) < self.radius.pow(2) as i32 {
            *px = self.colour;
            true
        } else {
            false
        }
    }
}

fn hex_char_to_n(c: char) -> Option<u8> {
    ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F'].iter().position(|&hc| hc == c).map(|i| i as u8)
}
fn parse_hex(string: &str) -> u8 {
    let string = String::from(string);
    string.chars()
     .rev()
     .zip((0..string.len()))
     .fold(0, |acc, (c, p)|
           if let Some(n) = hex_char_to_n(c) {
               acc + n * (16 as u8).pow(p as u32)
           } else {
               panic!("Invalid hex character: {}", c);
           })
}
fn colour_parse(string: &str) -> image::Rgb<u8> {
    // Parse hex colour codes in the "#RRGGBB"
    let data = String::from(string).chars()
        .skip(1) // Drop the first '#' character
        .collect::<Vec<char>>()
        .chunks(2)
        .map(|chars| if chars.len() == 2 {
            let mut s = String::new();
            for c in chars {
                s.push(*c);
            }
            parse_hex(s.as_str())
        } else {
            panic!("Invalid hex code: {}", string);
        })
        .collect::<Vec<u8>>();
    if data.len() == 3 {
        image::Rgb::<u8> {
            data: [data[0], data[1], data[2]]
        }
    } else {
        panic!("Invalid hex code: {}", string);
    }
}
fn main() {
    let mut height = 768;
    let mut width = 1366;
    let mut background = String::from("#000000FF");
    let mut colours = String::from("#FFFFFFFF,#FF0000FF,#00FF00FF,#0000FFFF");
    let mut shape_type = String::from("Circle");
    let mut shape_count = 10;
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Generate a wallpaper with some colours and shapes.");
        ap.refer(&mut height)
            .add_option(&["-h", "--height"], Store, "Set the image height.");
        ap.refer(&mut width)
            .add_option(&["-w", "--width"], Store, "Set the image width.");
        ap.refer(&mut background)
            .add_option(&["-b", "--background"], Store, "Set the image background colour.");
        ap.refer(&mut colours)
            .add_option(&["-c", "--colours"], Store, "Set the image colours (comma separated #RRGGBB values).");
        ap.refer(&mut shape_count)
            .add_option(&["-n", "--num-shapes"], Store, "Set the number of shapes generated.");
        ap.refer(&mut shape_type)
            .add_option(&["-s", "--shape-type"], Store, "Set the type of shapes generated (Circle, Rectangle). Default is Circle.");
        ap.parse_args_or_exit();
    }
    let background_colour = colour_parse(background.as_str());
    let shape_colours = colours.split(',')
        .map(|colour| colour_parse(colour)).collect::<Vec<image::Rgb<u8>>>();

    let mut imgbuf = image::ImageBuffer::from_pixel(width, height, background_colour);

    let mut rng = rand::thread_rng();
    let mut shapes: Vec<Box<Drawable>> = vec![];
    for _ in 0..shape_count {
        match shape_type.as_str() {
            "Circle" => {
                shapes.push(Box::new(Circle {
                    origin: Point(rng.gen::<u32>() % width, rng.gen::<u32>() % height),
                    radius: rng.gen::<u32>() % 250,
                    colour: *rand::thread_rng().choose(&shape_colours).unwrap()
                }));
            }
            "Rectangle" => {
                shapes.push(Box::new(Rect {
                    origin: Point(rng.gen::<u32>() % width, rng.gen::<u32>() % height),
                    length: rng.gen::<u32>() % 250,
                    height: rng.gen::<u32>() % 250,
                    colour: *rand::thread_rng().choose(&shape_colours).unwrap()
                }));
            }
            _ => panic!("Unsupported shape type: {}", shape_type)
        }

    }
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        for shape in &shapes {
            shape.draw(pixel, x, y);
        }
    }
    let ref mut fout = File::create(&Path::new("imageout.png")).unwrap();
    let _ = image::ImageRgb8(imgbuf).save(fout, image::PNG);
}

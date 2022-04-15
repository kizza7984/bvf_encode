use byteorder::{ByteOrder, LittleEndian};
use image::GenericImageView;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;
#[macro_use]
extern crate clap;

#[derive(Clone, Debug)]
struct Vector {
    start: u8,
    end: u8,
}

impl Vector {
    fn bytes(&self) -> [u8; 2] {
        [self.start, self.end]
    }
}

struct Line {
    num_vectors: usize,
    vectors: Vec<Vector>,
}

impl Line {
    fn bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![self.num_vectors as u8];
        for vector in &self.vectors {
            bytes.extend(vector.bytes());
        }
        bytes
    }
}

struct Metadata {
    frame_rate: u8,
    frame_count: u32,
    horizontal_resolution: u8,
    vertical_resolution: u8,
}

impl Metadata {
    fn bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.frame_rate);
        let mut buf: Vec<u8> = vec![0; 4];
        LittleEndian::write_u32(&mut buf, self.frame_count);
        bytes.extend(buf);
        bytes.push(self.horizontal_resolution);
        bytes.push(self.vertical_resolution);
        bytes
    }
}

fn main() -> io::Result<()> {
    // Command line arguments
    let matches = clap_app!(myapp =>
        (name: "BVF Encoder")
        (version: "1.0")
        (about: "Does awesome things")
        (@arg INPUT_DIRECTORY: +required "Sets the input file to use")
        (@arg OUTPUT_FILE: -o --output +takes_value "Output file name")
        (@arg FRAME_RATE: -r --("frame-rate") +required +takes_value "Input video frame rate")
        (@arg FRAME_COUNT: -c --("frame-count") +required +takes_value "Number of frames in the input video")
        (@arg HORIZONTAL_RESOLUTION: -h --("horizontal-resolution") +required +takes_value "Input video horizontal resolution")
        (@arg VERTICAL_RESOLUTION: -v --("vertical-resolution") +required +takes_value "Input video vertical resolution")
    )
    .get_matches();

    // File and Metadata
    let mut file = File::create(matches.value_of("OUTPUT_FILE").unwrap_or("output.bvf")).expect("Couldn't create output file");
    let metadata = Metadata {
        frame_rate: matches.value_of("FRAME_RATE").unwrap().parse().unwrap(),
        frame_count: matches.value_of("FRAME_COUNT").unwrap().parse().unwrap(),
        horizontal_resolution: matches.value_of("HORIZONTAL_RESOLUTION").unwrap().parse().unwrap(),
        vertical_resolution: matches.value_of("VERTICAL_RESOLUTION").unwrap().parse().unwrap(),
    };

    // Write Metadata
    file.write_all(&metadata.bytes()).unwrap();

    // Parse frames
    for n in 1..=metadata.frame_count {
        let path = PathBuf::from(matches.value_of("INPUT_DIRECTORY").unwrap()).join(format!("frame{n}.png"));
        let img = image::open(path).expect("Input frame missing");
        let (img_width, img_height) = img.dimensions();
        assert_eq!(img_width as u8, metadata.horizontal_resolution);
        assert_eq!(img_height as u8, metadata.vertical_resolution);
        let mut frame: Vec<Vec<Vector>> = vec![vec![]; img_height as usize];
        let (mut start, mut end): (Option<u8>, Option<u8>) = (None, None);

        let mut prev_pixel_b = false;

        for pixel in img.pixels() {
            let x: u8 = pixel.0.try_into().expect("Input image too wide");
            let line: usize = pixel.1.try_into().expect("Input image too tall");
            let pixel_b = pixel.2[0] < 255 / 2;

            // Detect change in series 0000111110000
            if pixel_b != prev_pixel_b && x != 0 {
                if prev_pixel_b == false {
                    // Rising edge 0 -> 1
                    start = Some(x);
                } else {
                    // Falling edge 1 -> 0
                    end = Some(x - 1);
                }
            }

            // Detect a start at the beginning of a line 111|1111 because there isn't a change to detect
            if x == 0 && pixel_b {
                start = Some(x);
            }

            // Detect an end at the end of a line 11111|1 because there isn't a change to detect
            if x + 1 == img_width as u8 && pixel_b {
                end = Some(x);
            }

            // If end is set then we have a complete vector to add
            if end.is_some() {
                frame[line].push(Vector {
                    start: start.unwrap(),
                    end: end.unwrap(),
                });
                start = None;
                end = None;
            }

            prev_pixel_b = pixel_b;
        }

        // Write frame
        write_frame(&frame, &file);
    }

    Ok(())
}

fn write_frame(frame: &Vec<Vec<Vector>>, mut file: &File) {
    //println!("{:?}", frame);
    for line in frame {
        file.write_all(
            &Line {
                num_vectors: line.len(),
                vectors: line.to_vec(),
            }
            .bytes(),
        )
        .unwrap();
    }
}


use std::{error::Error, io::Write};
use image::{GrayImage, Luma};
use crate::get_size;



fn encode_frame(frame: &GrayImage) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut lengths: Vec<u8> = Vec::new();
    let mut color = false;
    let mut length: u8 = 0;

    for y in 0..frame.height() {
        for x in 0..frame.width() {
            let pixel = frame.get_pixel(x, y).0[0] >= 127;

            if pixel == color {
                if length == 0xFF {
                    lengths.push(0xFF);
                    lengths.push(0);
                    length = 1;
                } else {
                    length += 1;
                }
            } else {
                lengths.push(length);
                color = pixel;
                length = 1;
            }
        }
    }

    // Force total length to full image.
    if length > 0 {
        lengths.push(length);
    }
    // Force end color to be off.
    // if color {
    //     lengths.push(0);
    // }

    Ok(lengths)
}

fn frames_difference(frame_a: &GrayImage, frame_b: &GrayImage) -> Result<GrayImage, Box<dyn Error>> {
    if frame_a.width() != frame_b.width() || frame_a.height() != frame_b.height() {
        panic!("Frames width & heights do not match.");
    }

    let (width, height) = (frame_a.width(), frame_b.height());

    let mut frame_difference = GrayImage::new(width, height);
    for x in 0..width {
        for y in 0..height {
            let a_pixel = frame_a.get_pixel(x, y).0[0] >= 127;
            let b_pixel = frame_b.get_pixel(x, y).0[0] >= 127;

            if a_pixel != b_pixel {
                frame_difference.put_pixel(x, y, Luma([255; 1]));
            }
        }
    }
    Ok(frame_difference)
}

pub fn encode_frames(frames: &Vec<GrayImage>) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buffer: Vec<u8> = Vec::new();
    
    let (width, height) = get_size(frames)?;

    for (index, frame) in frames.iter().enumerate() {
        let last_frame = if index > 0 {
            frames[index - 1].clone()
        } else {
            GrayImage::new(width, height)
        };

        let frame_difference = frames_difference(&last_frame, frame)?;

        let frame_encoded = encode_frame(&frame_difference)?;
        buffer.write(&frame_encoded)?;
    }

    Ok(buffer)
}

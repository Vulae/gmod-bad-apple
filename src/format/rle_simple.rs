
use std::{error::Error, io::Write};
use image::GrayImage;
use crate::get_size;
use super::common::frames_difference;



fn encode_frame(frame: &GrayImage) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut lengths: Vec<u8> = Vec::new();
    let mut color = false;
    let mut length: u8 = 0;

    for pixel in frame.pixels() {
        let pixel = pixel.0[0] >= 127;

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

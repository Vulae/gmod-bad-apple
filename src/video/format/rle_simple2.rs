
use std::{error::Error, io::Write};
use image::GrayImage;
use super::common::{frames_difference, get_size};



fn encode_frame(frame: &GrayImage) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut lengths: Vec<u64> = Vec::new();
    let mut color = false;
    let mut length: u64 = 0;

    for pixel in frame.pixels() {
        let pixel = pixel.0[0] >= 127;

        if pixel == color {
            length += 1;
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

    // Encode lengths into variable-length numbers.
    let mut buffer: Vec<u8> = Vec::new();

    for length in lengths {
        // Each length is encoded as 1-n bytes.
        // Where each byte is "EVVVVVVV"
        // Where "E" is the bit that indicates to include another byte.
        // And "V" are the bytes to construct number with.

        // TODO: Don't hard code all possibilities.
        if length <= 0b01111111 {
            buffer.push(length as u8);
        } else if length <= 0b0011111111111111 {
            buffer.push((0b10000000 | (length & 0b01111111)) as u8);
            buffer.push((length >> 7) as u8);
        } else {
            panic!("Failed to encode variable-length number.");
        }
    }

    Ok(buffer)
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

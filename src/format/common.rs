
use std::error::Error;
use image::{GrayImage, Luma};



pub fn frames_difference(frame_a: &GrayImage, frame_b: &GrayImage) -> Result<GrayImage, Box<dyn Error>> {
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

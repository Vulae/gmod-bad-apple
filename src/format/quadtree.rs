
// There is actually no decoder for this currently.
// It all proof of concept and W.I.P. at the moment.

use core::fmt;
use std::{cmp, error::Error, io::Write};
use bitstream_io::{BitWrite, BitWriter, LittleEndian};
use image::{GrayImage, Luma};
use crate::get_size;

use super::common::frames_difference;



#[derive(Debug, Clone)]
enum BoundingBoxError {
    CannotSplit
}

impl fmt::Display for BoundingBoxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BoundingBoxError::CannotSplit => write!(f, "Cannot split bounding box, Bounding box is at minimum size."),
        }
    }
}

impl Error for BoundingBoxError { }

#[derive(Clone)]
struct BoundingBox {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl BoundingBox {

    pub fn new(x: u32, y: u32, width: u32, height: u32) -> BoundingBox {
        BoundingBox { x, y, width, height }
    }

    pub fn split(&mut self) -> Result<Vec<BoundingBox>, Box<dyn Error>> {
        let split_width = (self.width as f64) / 2.;
        let split_height = (self.height as f64) / 2.;

        if self.width == 1 {
            Ok(vec![
                BoundingBox::new(self.x, self.y, 1, split_height.ceil() as u32),
                BoundingBox::new(self.x, self.y + (split_height.ceil() as u32), 1, split_height.floor() as u32)
            ])
        } else if self.height == 1 {
            Ok(vec![
                BoundingBox::new(self.x, self.y, split_width.ceil() as u32, 1),
                BoundingBox::new(self.x + (split_width.ceil() as u32), self.y, split_width.floor() as u32, 1)
            ])
        } else if self.width < 2 || self.height < 2 {
            Err(Box::new(BoundingBoxError::CannotSplit))
        } else {
            Ok(vec![
                BoundingBox::new(self.x, self.y, split_width.ceil() as u32, split_height.ceil() as u32),
                BoundingBox::new(self.x + (split_width.ceil() as u32), self.y, split_width.floor() as u32, split_height.ceil() as u32),
                BoundingBox::new(self.x, self.y + (split_height.ceil() as u32), split_width.ceil() as u32, split_height.floor() as u32),
                BoundingBox::new(self.x + (split_width.ceil() as u32), self.y + (split_height.ceil() as u32), split_width.floor() as u32, split_height.floor() as u32)
            ])
        }
    }

    pub fn iter_points(&self) -> impl Iterator<Item = (u32, u32)> + '_ {
        (self.x..self.x + self.width)
            .flat_map(move |x| (self.y..self.y + self.height).map(move |y| (x, y)))
    }

}



struct QuadTree {
    pub bounding_box: BoundingBox,
    pub quadrants: Option<Vec<Box<QuadTree>>>,
    pub color: bool,
}

impl QuadTree {
    
    pub fn new(bounding_box: BoundingBox) -> QuadTree {
        QuadTree { bounding_box, quadrants: None, color: false }
    }

    pub fn split(&mut self, image: &GrayImage) -> bool {
        // If image under bounding box is all same color.
        let first_pixel = image.get_pixel(self.bounding_box.x, self.bounding_box.y).0[0] >= 127;
        if self.bounding_box.iter_points().all(|(x, y)| {
            let pixel = image.get_pixel(x, y).0[0] >= 127;
            pixel == first_pixel
        }) {
            self.color = first_pixel;
            return false;
        }

        if let Ok(bounding_box_split) = self.bounding_box.split() {
            self.quadrants = Some(
                bounding_box_split.iter().map(|bounding_box| {
                    let mut quadtree = QuadTree::new(bounding_box.clone());
                    quadtree.split(image);
                    Box::new(quadtree)
                }).collect::<Vec<_>>()
            );

            return true;
        } else {
            // this should never happen.
            panic!("QuadTree failed to split.");
        }
    }

    pub fn iter_depth_first(&mut self) -> impl Iterator<Item = &QuadTree> {
        let mut stack: Vec<&QuadTree> = vec![self];
        std::iter::from_fn(move || stack.pop().map(|node| {
            let res = node;
            if let Some(quadrants) = &node.quadrants {
                quadrants.iter().rev().for_each(|quadrant| {
                    stack.push(&quadrant);
                });
            }
            res
        }))
    }

}





fn encode_frame(frame: &GrayImage) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buffer: Vec<u8> = Vec::new();
    let mut writer = BitWriter::endian(&mut buffer, LittleEndian);

    let mut quadtree = QuadTree::new(BoundingBox::new(0, 0, frame.width(), frame.height()));
    
    quadtree.split(frame);

    for node in quadtree.iter_depth_first() {
        if node.quadrants.is_none() {
            if node.bounding_box.width != 1 || node.bounding_box.height != 1 {
                writer.write_bit(true)?;
            }
            writer.write_bit(node.color)?;
        } else {
            writer.write_bit(false)?;
        }
    }

    writer.byte_align()?;

    Ok(buffer)
}

// TODO: max_pixels_frame is currently broken.
// For some reason encoding will break if any pixels are held for next frame?
pub fn encode_frames(frames: &Vec<GrayImage>, max_pixels_frame: u64) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buffer: Vec<u8> = Vec::new();
    
    let (width, height) = get_size(frames)?;

    let mut pending_pixels: Vec<(u64, u64)> = Vec::new();

    for (index, frame) in frames.iter().enumerate() {
        let last_frame = if index > 0 {
            frames[index - 1].clone()
        } else {
            GrayImage::new(width, height)
        };

        // Add the difference of pixels to pending pixels queue.
        frames_difference(frame, &last_frame)?
            .pixels()
            .enumerate()
            .for_each(|(i, pixel)| {
                if pixel.0[0] < 127 { return }
                let x = (i as u64) % (width as u64);
                let y = (i as u64) / (width as u64);
                pending_pixels.push((x, y));
            });

        // Pixels to encode this frame.
        let num_pixels = cmp::min(max_pixels_frame, pending_pixels.len() as u64);
        let pixels_to_encode: Vec<(u64, u64)> = pending_pixels.drain(0..(num_pixels as usize)).collect();

        // Create frame from pixels to encode.
        let mut frame_difference_limited = GrayImage::new(width, height);
        pixels_to_encode.iter().for_each(|(x, y)| {
            frame_difference_limited.put_pixel(x.clone() as u32, y.clone() as u32, Luma([ 255 ]));
        });

        // Encode frame.
        let frame_encoded = encode_frame(&frame_difference_limited)?;
        buffer.write(&frame_encoded)?;
    }

    Ok(buffer)
}

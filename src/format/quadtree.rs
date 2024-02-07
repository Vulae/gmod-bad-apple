
// There is actually no decoder for this currently.
// It all proof of concept and W.I.P. at the moment.

use std::{error::Error, io::Write, mem};
use bitstream_io::{BitWrite, BitWriter, LittleEndian};
use image::GrayImage;
use crate::get_size;
use super::common::frames_difference;



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

    pub fn split(&mut self) -> Option<(BoundingBox, BoundingBox, BoundingBox, BoundingBox)> {

        if self.width < 2 || self.height < 2 {
            return None;
        }

        let split_width = (self.width as f64) / 2.;
        let split_height = (self.height as f64) / 2.;

        Some((
            BoundingBox::new(
                self.x,
                self.y,
                split_width.ceil() as u32,
                split_height.ceil() as u32
            ),
            BoundingBox::new(
                self.x + (split_width.ceil() as u32),
                self.y,
                split_width.floor() as u32,
                split_height.ceil() as u32
            ),
            BoundingBox::new(
                self.x,
                self.y + (split_height.ceil() as u32),
                split_width.ceil() as u32,
                split_height.floor() as u32
            ),
            BoundingBox::new(
                self.x + (split_width.ceil() as u32),
                self.y + (split_height.ceil() as u32),
                split_width.floor() as u32,
                split_height.floor() as u32
            )
        ))
    }

    pub fn iter_points(&self) -> impl Iterator<Item = (u32, u32)> + '_ {
        (self.x..self.x + self.width)
            .flat_map(move |x| (self.y..self.y + self.height).map(move |y| (x, y)))
    }

}



struct QuadTree {
    pub bounding_box: BoundingBox,
    pub quadrants: Option<(Box<QuadTree>, Box<QuadTree>, Box<QuadTree>, Box<QuadTree>)>,
    pub color: bool,
}

impl Drop for QuadTree {
    fn drop(&mut self) {
        if let Some(quadrants) = mem::take(&mut self.quadrants) {
            drop(quadrants.0);
            drop(quadrants.1);
            drop(quadrants.2);
            drop(quadrants.3);
        }
    }
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

        if let Some(bounding_box_split) = self.bounding_box.split() {

            let mut a = QuadTree::new(bounding_box_split.0);
            let mut b = QuadTree::new(bounding_box_split.1);
            let mut c = QuadTree::new(bounding_box_split.2);
            let mut d = QuadTree::new(bounding_box_split.3);

            a.split(image);
            b.split(image);
            c.split(image);
            d.split(image);

            self.quadrants = Some((
                Box::new(a),
                Box::new(b),
                Box::new(c),
                Box::new(d)
            ));

            return true;

        } else {
            // TODO: What to do when a box like 0 0 2 1 shows up?
            // panic!("Cannot split quadtree, quadtree is already at minimum size.");
            self.color = first_pixel;
            return false;
        }
    }

    pub fn iter_depth_first(&mut self) -> impl Iterator<Item = &QuadTree> {
        let mut stack: Vec<&QuadTree> = vec![self];
        std::iter::from_fn(move || stack.pop().map(|node| {
            let res = node;
            if let Some((a, b, c, d)) = &node.quadrants {
                stack.push(d);
                stack.push(c);
                stack.push(b);
                stack.push(a);
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
            writer.write_bit(true)?;
            writer.write_bit(node.color)?;
        } else {
            writer.write_bit(false)?;
        }
    }

    writer.byte_align()?;

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

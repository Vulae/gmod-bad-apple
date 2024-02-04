
use std::error::Error;
use image::GrayImage;
mod rle_simple;

#[derive(Debug, Clone, Copy)]
pub enum Format {
    RleSimple = 1,
    QuadTree = 2,
}

impl Format {

    pub fn encode(self, frames: &Vec<GrayImage>) -> Result<Vec<u8>, Box<dyn Error>> {
        match self {
            Format::RleSimple => rle_simple::encode_frames(frames),
            Format::QuadTree => panic!("QuadTree format is not yet supported."),
        }
    }

}

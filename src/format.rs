
use std::error::Error;
use clap::ValueEnum;
use image::GrayImage;
mod rle_simple;
mod quadtree;
mod common;
mod rle_simple2;



#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum Format {
    RleSimple = 1,
    QuadTree = 2,
    RleSimple2 = 3,
}

impl Format {

    pub fn encode(self, frames: &Vec<GrayImage>) -> Result<Vec<u8>, Box<dyn Error>> {
        match self {
            Format::RleSimple => rle_simple::encode_frames(frames),
            Format::QuadTree => quadtree::encode_frames(frames, 0xFFFF),
            Format::RleSimple2 => rle_simple2::encode_frames(frames),
        }
    }

}

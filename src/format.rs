
use std::error::Error;
use clap::ValueEnum;
use image::GrayImage;
mod rle_simple;
mod quadtree;
mod common;



/// Encoding sizes 60x45 @ 7fps:
/// 
/// rle - 228kb  
/// quadtree - 157kb  
/// 
/// Both of them encode basically instantly.
#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum Format {
    RleSimple = 1,
    QuadTree = 2,
}

impl Format {

    pub fn encode(self, frames: &Vec<GrayImage>) -> Result<Vec<u8>, Box<dyn Error>> {
        match self {
            Format::RleSimple => rle_simple::encode_frames(frames),
            Format::QuadTree => quadtree::encode_frames(frames),
        }
    }

}

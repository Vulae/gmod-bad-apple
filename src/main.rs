
//
// cargo run -- "media/frames.zip" 30 "media/audio.wav" "media/output.bin" 60 45 5
//

use std::{error::Error, fs::File, io::{Read, Write}};
use base64::Engine;
use bitstream_io::{ByteWrite, ByteWriter, LittleEndian};
use clap::Parser;
use image::{imageops, EncodableLayout, GrayImage};
use indicatif::{ProgressBar, ProgressStyle};



#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(index = 1)]
    frames: String,
    #[arg(index = 2)]
    frames_fps: u64,
    #[arg(index = 3)]
    audio: String,
    #[arg(index = 4)]
    output: String,
    #[arg(index = 5)]
    out_width: u64,
    #[arg(index = 6)]
    out_height: u64,
    #[arg(index = 7)]
    out_fps: u64,
}



fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    println!("Frames: \"{}\" {}fps", &args.frames, &args.frames_fps);
    println!("Audio: \"{}\"", &args.audio);
    println!("Output: \"{}\" {}x{} {}fps", &args.output, &args.out_width, &args.out_height, &args.out_fps);



    let zip_file = File::open(&args.frames)?;
    let mut zip = zip::ZipArchive::new(zip_file)?;

    let mut frames: Vec<GrayImage> = vec![];

    println!("Loading frames. . .");

    let n_frames = &args.frames_fps / &args.out_fps;
    println!("Using every {} frames", n_frames);

    let progress = ProgressBar::new((zip.len() as u64) / n_frames);
    progress.set_style(ProgressStyle::with_template("Frame {pos}/{len}")?);

    for i in 0..zip.len() {
        if (i as u64) % n_frames != 0 { continue; }

        let mut file = zip.by_index(i)?;

        let mut data: Vec<u8> = vec![];
        let _len = file.read_to_end(&mut data)?;

        let image_format = image::ImageFormat::from_path(file.name())?;
        let image = image::load_from_memory_with_format(data.as_bytes(), image_format)?;
        let image = image.to_luma8();
        let image = imageops::resize(&image, args.out_width as u32, args.out_height as u32, imageops::FilterType::Triangle);

        frames.push(image);

        progress.set_position((i as u64) / n_frames);
    }

    println!("Num frames: {}", frames.len());



    let width = frames[0].width();
    let height = frames[0].height();

    if frames.iter().any(|frame| {
        frame.width() != width || frame.height() != height
    }) {
        panic!("Frame width & heights do not match.");
    }



    println!("Generating output. . .");

    let mut buffer: Vec<u8> = Vec::new();
    let mut writer = ByteWriter::endian(&mut buffer, LittleEndian);

    writer.write(width as u8)?;
    writer.write(height as u8)?;
    writer.write(frames.len() as u16)?;

    for frame in frames {

        // Simplified RLE
        // Flip flops between off/on based on num pixels that were on or off.

        let mut lengths: Vec<u8> = Vec::new();
        let mut color = false;
        let mut length: u8 = 0;

        for y in 0..frame.height() {
            for x in 0..frame.width() {
                let pixel = frame.get_pixel(x, y).0[0] >= 127;

                if pixel == color {
                    if length == 0xFF {
                        lengths.push(0xFF);
                        lengths.push(0x00);
                        length = 0;
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
        if color {
            lengths.push(0);
        }

        writer.write(lengths.len() as u16)?;
        writer.write_bytes(&lengths)?;
    }



    println!("Writing output.");

    let mut out_file = File::create(&args.output)?;
    out_file.write(b"Base64Stream = \"")?;
    out_file.write_all(base64::prelude::BASE64_STANDARD.encode(&buffer).as_bytes())?;
    out_file.write(b"\"")?;
    // out_file.write_all(&buffer)?;
    out_file.flush()?;



    Ok(())
}



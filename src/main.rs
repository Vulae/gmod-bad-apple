
use std::{error::Error, fs::File, io::{Read, Write}};
use base64::Engine;
use bitstream_io::{BitWrite, BitWriter, ByteWrite, ByteWriter, LittleEndian};
use clap::Parser;
use image::{EncodableLayout, GrayImage};



#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    frames: String,
    #[arg(short, long)]
    audio: String,
    #[arg(short, long)]
    output: String,
}



fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    println!("Frames: \"{}\"", &args.frames);
    println!("Audio: \"{}\"", &args.audio);
    println!("Output: \"{}\"", &args.output);



    let zip_file = File::open(&args.frames)?;
    let mut zip = zip::ZipArchive::new(zip_file)?;

    let mut frames: Vec<GrayImage> = vec![];

    println!("Loading frames. . .");

    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;

        let mut data: Vec<u8> = vec![];
        let _len = file.read_to_end(&mut data)?;

        let image_format = image::ImageFormat::from_path(file.name())?;
        let image = image::load_from_memory_with_format(data.as_bytes(), image_format)?.to_luma8();

        frames.push(image);
    }

    println!("Num frames: {}", frames.len());



    let width = frames[0].width();
    let height = frames[0].height();

    if frames.iter().any(|frame| {
        frame.width() != width || frame.height() != height
    }) {
        panic!("Frame width & heights do not match.");
    }

    println!("Size: {}x{}", width, height);



    // 10 seconds
    frames.drain(60..);



    println!("Generating output. . .");

    let mut buffer: Vec<u8> = Vec::new();
    let mut writer = ByteWriter::endian(&mut buffer, LittleEndian);

    writer.write(width as u16)?;
    writer.write(height as u16)?;
    writer.write(frames.len() as u32)?;

    for frame in frames {

        let mut frame_buffer: Vec<u8> = Vec::new();
        let mut frame_writer = BitWriter::endian(&mut frame_buffer, LittleEndian);

        // Simplified RLE
        // Flip flops between off/on based on num pixels that were on or off.

        let mut current_color = false;
        let mut current_length: u8 = 0;

        for y in 0..frame.height() {
            for x in 0..frame.width() {
                let pixel = frame.get_pixel(x, y);
                let pixel_color = pixel.0[0] >= 127;

                if current_color == pixel_color {
                    if current_length == 0xFF {
                        frame_writer.write(8, current_length)?;
                        current_length = 0;
                        current_color = !current_color;
                    } else {
                        current_length += 1;
                    }
                } else {
                    frame_writer.write(8, current_length)?;
                    current_length = 0;
                    current_color = pixel_color;
                }
            }
        }

        if current_length > 0 {
            frame_writer.write(8, current_length)?;
        }

        frame_writer.byte_align()?;

        writer.write_bytes(&frame_buffer)?;
    }



    println!("Writing output.");

    let mut out_file = File::create(&args.output)?;
    out_file.write(b"let Base64Stream = \"")?;
    out_file.write_all(base64::prelude::BASE64_STANDARD.encode(&buffer).as_bytes())?;
    // out_file.write_all(&buffer)?;
    out_file.write(b"\"")?;
    out_file.flush()?;



    Ok(())
}



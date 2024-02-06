
// 
// Garry's Mod e2 has a VERY hard time rendering and uploading very large e2 files.
// So the biggest I can get working is under 512KiB
// 
// There are many settings to use.
// Here's 2 of them that work pretty well.
// 
// cargo run -- "media/frames.zip" 30 "media/audio.wav" "media/output.bin" 40 30 10
// (Works without exceeding tick quota)
// 
// cargo run -- "media/frames.zip" 30 "media/audio.wav" "media/output.bin" 60 45 5
// (Exceeds tick quota)
// 

use std::{error::Error, fs::File, io::{Read, Write}, sync::mpsc, thread};
use base64::Engine;
use bitstream_io::{ByteWrite, ByteWriter, LittleEndian};
use clap::Parser;
use format::Format;
use image::{imageops, EncodableLayout, GrayImage};
use zip::ZipArchive;
mod format;



#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(index = 1)]
    frames: String,
    #[arg(index = 2)]
    frames_fps: u32,
    #[arg(index = 3)]
    audio: String,
    #[arg(index = 4)]
    output: String,
    #[arg(index = 5)]
    out_width: u32,
    #[arg(index = 6)]
    out_height: u32,
    #[arg(index = 7)]
    out_fps: u32,
    #[arg(short, long, value_enum, default_value_t = Format::RleSimple)]
    format: Format,
}





fn load_frames(zip: &mut ZipArchive<File>, in_fps: u32, width: u32, height: u32, fps: u32) -> Result<Vec<GrayImage>, Box<dyn Error>> {
    let mut current_time: f64 = 0.0;
    let in_frametime: f64 = 1.0 / (in_fps as f64);
    let frametime: f64 = 1.0 / (fps as f64);

    let (tx, rx) = mpsc::channel();

    for index in 0..zip.len() {
        current_time += in_frametime;
        if current_time > frametime {
            current_time -= frametime;
        } else {
            continue;
        }

        // if index != 120 { continue; }

        let mut file = zip.by_index(index)?;

        let image_format = image::ImageFormat::from_path(file.name()).unwrap();

        let mut data: Vec<u8> = vec![];
        let _len = file.read_to_end(&mut data)?;

        let thread_tx = tx.clone();

        thread::spawn(move || {
            let image = image::load_from_memory_with_format(data.as_bytes(), image_format).unwrap();
            let image = image.to_luma8();
            // Resizing takes a VERY VERY long time, So we use everything we can.
            let image = imageops::resize(&image, width, height, imageops::FilterType::Lanczos3);

            thread_tx.send((index, image)).unwrap();
        });
    }

    drop(tx);

    let frames: Vec<GrayImage> = {
        let mut recv_frames = rx.iter().collect::<Vec<_>>();
        recv_frames.sort_by(|a, b| {
            a.0.cmp(&b.0)
        });
        recv_frames.iter().map(|frame| {
            frame.1.clone()
        }).collect::<Vec<_>>()
    };

    Ok(frames)
}





pub fn get_size(frames: &Vec<GrayImage>) -> Result<(u32, u32), Box<dyn Error>> {
    let (width, height) = (frames[0].width(), frames[0].height());

    if frames.iter().any(|frame| {
        frame.width() != width || frame.height() != height
    }) {
        panic!("Frames width & heights do not match.");
    }

    Ok((width, height))
}





fn encode_video(format: Format, frames: &Vec<GrayImage>, fps: u32) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buffer: Vec<u8> = Vec::new();
    let mut writer = ByteWriter::endian(&mut buffer, LittleEndian);

    let (width, height) = get_size(frames)?;

    writer.write(format as u8)?;
    writer.write(width as u8)?;
    writer.write(height as u8)?;
    writer.write(frames.len() as u16)?;
    writer.write(fps as u8)?;

    let frames_encoded = format.encode(frames)?;
    writer.write_bytes(&frames_encoded)?;

    Ok(buffer)
}





fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    println!("Frames: \"{}\" {}fps", &args.frames, &args.frames_fps);
    println!("Audio: \"{}\"", &args.audio);
    println!("Output: \"{}\" {}x{} {}fps", &args.output, &args.out_width, &args.out_height, &args.out_fps);



    let zip_file = File::open(&args.frames)?;
    let mut zip = zip::ZipArchive::new(zip_file)?;

    println!("Loading frames. . .");

    let frames = load_frames(&mut zip, args.frames_fps, args.out_width, args.out_height, args.out_fps)?;

    println!("Num frames: {}", frames.len());

    println!("Generating output. . .");

    let encoded = encode_video(args.format, &frames, args.out_fps)?;

    println!("Writing output.");

    let mut out_file = File::create(&args.output)?;
    out_file.write(b"Base64Stream = \"")?;
    out_file.write_all(base64::prelude::BASE64_STANDARD.encode(&encoded).as_bytes())?;
    out_file.write(b"\"")?;
    // out_file.write_all(&encoded)?;
    out_file.flush()?;



    Ok(())
}



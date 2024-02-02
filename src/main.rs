
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
use image::{imageops, EncodableLayout, GrayImage, Luma};



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

    println!("Loading frames. . .");

    let (tx, rx) = mpsc::channel();

    let mut current_time: f64 = 0.0;
    let in_frametime: f64 = 1.0 / (args.frames_fps as f64);
    let out_frametime: f64 = 1.0 / (args.out_fps as f64);

    for i in 0..zip.len() {
        current_time += in_frametime;
        if current_time > out_frametime {
            current_time -= out_frametime;
        } else {
            continue;
        }

        let mut file = zip.by_index(i)?;
        
        let image_format = image::ImageFormat::from_path(file.name()).unwrap();

        let mut data: Vec<u8> = vec![];
        let _len = file.read_to_end(&mut data)?;

        let thread_tx = tx.clone();

        thread::spawn(move || {
            let image = image::load_from_memory_with_format(data.as_bytes(), image_format).unwrap();
            let image = image.to_luma8();
            // Resizing takes a VERY VERY long time, So we use everything we can.
            let image = imageops::resize(&image, args.out_width as u32, args.out_height as u32, imageops::FilterType::Lanczos3);

            thread_tx.send((i, image)).unwrap();
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
    writer.write(args.out_fps as u8)?;

    for (index, frame) in frames.iter().enumerate() {
        
        // We encode the difference between frames.
        let last = if index > 0 {
            frames[index - 1].clone()
        } else {
            GrayImage::new(width, height)
        };

        let mut diff = GrayImage::new(width, height);
        for x in 0..width {
            for y in 0..height {
                let pixel = frame.get_pixel(x, y).0[0] >= 127;
                let last_pixel = last.get_pixel(x, y).0[0] >= 127;

                if pixel != last_pixel {
                    diff.put_pixel(x, y, Luma([255; 1]));
                }
            }
        }

        // Simplified RLE
        // Flip flops between off/on based on num pixels that were on or off.

        let mut lengths: Vec<u8> = Vec::new();
        let mut color = false;
        let mut length: u8 = 0;

        for y in 0..height {
            for x in 0..width {
                let pixel = diff.get_pixel(x, y).0[0] >= 127;

                if pixel == color {
                    if length == 0xFF {
                        lengths.push(0xFF);
                        lengths.push(0);
                        length = 1;
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
        // if color {
        //     lengths.push(0);
        // }

        // writer.write(lengths.len() as u16)?;
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



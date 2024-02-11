
use std::{env, error::Error};
mod video;
mod audio;



// TODO: Make less jank with clap.



fn main() -> Result<(), Box<dyn Error>> {

    let mut env = env::args().collect::<Vec<_>>();
    let first = env.remove(1);

    match first.to_lowercase().as_str() {
        "video" | "--video" => {
            video::main(env)?;
        },
        "audio" | "--audio" => {
            audio::main(env)?;
        },
        _ => {
            println!("Invalid arguments.");
        }
    }

    Ok(())
}



use std::{
    env::args,
    fs::read_dir,
    io::{self, Write},
    process::Command,
    thread::sleep,
    time::{Duration, Instant},
};

use image::{DynamicImage, GenericImageView};

const CHARACTERS: &str =
    " `.-':_,^=;><+!rc*/z?sLTv)J7(|Fi{C}fI31tlu[neoZ5Yxjya]2ESwqkP6h9d4VpOGbUAKXHm8RD#$Bg0MNWQ%&@";

fn main() -> Result<(), io::Error> {
    let args = args().collect::<Vec<_>>();

    let video = &args[1];
    let framerate = &args[2];
    let framerate_float = match framerate.parse::<f64>() {
        Ok(framerate_float) => framerate_float,

        Err(_) => {
            eprintln!("Invalid framerate: {}", framerate);
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid framerate",
            ));
        }
    };

    let frame_time = 1.0 / framerate_float;

    let mut to_frames_cmd = Command::new("ffmpeg");

    to_frames_cmd
        .arg("-i")
        .arg(video)
        .arg("-r")
        .arg(framerate)
        .arg("frames/output_%04d.png");

    to_frames_cmd.spawn()?.wait()?;

    println!("\x1b[2J");
    println!("Press enter to start");

    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf)?;

    drop(buf);

    let frame_count = read_dir("./frames")?.count();
    let mut error = 0.0;

    let mut i = 1;
    while i < frame_count {
        let display_start = Instant::now();
        let path = format!("frames/output_{i:04}.png");
        let frame = match image::open(&path) {
            Ok(frame) => frame,
            _ => {
                eprintln!("Error opening frame: {}", path);
                return Err(io::Error::new(io::ErrorKind::Other, "Failed to open frame"));
            }
        };

        print!("{}", asciify(&frame));

        io::stdout().flush()?;

        let elapsed = display_start.elapsed();
        let sleep_time = (frame_time) - elapsed.as_secs_f64();

        let before_sleep = Instant::now();

        sleep(Duration::from_secs_f64(if sleep_time > 0.0 {
            sleep_time
        } else {
            0.0
        }));

        let elapsed_sleep = before_sleep.elapsed();
        error += elapsed_sleep.as_secs_f64() - sleep_time;

        if error >= frame_time {
            error = 0.0;
        } else {
            i += 1;
        }
    }

    Ok(())
}

fn asciify(frame: &DynamicImage) -> String {
    let frame = frame.resize_exact(128 * 2, 72 * 2, image::imageops::FilterType::Gaussian);
    let mut ascii = String::with_capacity((frame.height() * frame.width()) as usize);

    print!("\x1b[2J");

    let width = frame.width();

    for (x, _, pixel) in frame.pixels() {
        let r = pixel[0] as f64;
        let g = pixel[1] as f64;
        let b = pixel[2] as f64;

        let px_avg = (r + g + b) / 3.0;

        let brightness = px_avg / 255.0;

        let mut char_index = CHARACTERS.len() as f64 * brightness;

        if char_index >= CHARACTERS.len() as f64 {
            char_index = CHARACTERS.len() as f64 - 1.0;
        }

        ascii = format!(
            "{ascii}{}",
            CHARACTERS.chars().nth(char_index as usize).unwrap()
        );

        if x == (width - 1) {
            ascii.push('\n');
        }
    }

    ascii
}

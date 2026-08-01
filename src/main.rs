use mpris::{PlayerFinder, Event};
use reqwest::blocking;
use std::{error::Error, fs::File, io, process::Command, env};
use imageproc::{compose::overlay, image::{DynamicImage, ImageFormat, ImageReader, imageops::FilterType}};

fn get_album_art(url: &str, path: &str) -> Result<(), Box<dyn Error>> {
    let mut response = blocking::get(url)?;
    let mut dest = File::create(path)?;
    io::copy(&mut response, &mut dest)?;
    Ok(())
}

fn resize_cover(img: &DynamicImage, target_w: u32, target_h: u32) -> DynamicImage {
    let (src_w, src_h) = (img.width(), img.height());

    let scale = (target_w as f64 / src_w as f64).max(target_h as f64 / src_h as f64);
    let new_w = (src_w as f64 * scale).ceil() as u32;
    let new_h = (src_h as f64 * scale).ceil() as u32;
    let resized = img.resize_exact(new_w, new_h, FilterType::Lanczos3);

    let x = (new_w - target_w) / 2;
    let y = (new_h - target_h) / 2;
    resized.crop_imm(x, y, target_w, target_h)
}

fn resize_contain(img: &DynamicImage, bg: &DynamicImage) -> DynamicImage {
    let (target_w, target_h) = (bg.width(), bg.height());
    let (src_w, src_h) = (img.width(), img.height());

    let scale = (target_w as f64 / src_w as f64).min(target_h as f64 / src_h as f64);
    let new_w = (src_w as f64 * scale).round() as u32;
    let new_h = (src_h as f64 * scale).round() as u32;

    let resized = img.resize(new_w, new_h, FilterType::Lanczos3);

    let x_offset = (target_w - new_w) / 2;
    let y_offset = (target_h - new_h) / 2;

    DynamicImage::ImageRgb8(overlay(&bg.to_rgb8(), &resized.to_rgb8(), x_offset, y_offset))
}

fn process_album_art(path: &str) -> Result<(), Box<dyn Error>> {
    let img = ImageReader::open(path)?.decode()?;
    let mut bg = img.clone();
    bg = bg.blur(10.0);
    let cover = resize_cover(&bg, 1920, 1080);
    let res = resize_contain(&img, &cover);
    res.save_with_format(path, ImageFormat::Jpeg)?;
    Ok(())
}

fn begin_listener(path: &str) -> Result<(), Box<dyn Error>> {
    let finder = PlayerFinder::new()?;
    if let Ok(player) = finder.find_active() {
        let mut curr_url: String = String::new();
        for event in player.events()? {
            match event {
                Ok(Event::Playing) | Ok(Event::Paused) => println!("Playback toggle"),
                Ok(Event::TrackChanged(meta)) => {
                    let url = meta.art_url().unwrap();
                    if curr_url != url {
                        if let Ok(()) = get_album_art(url, &path) {
                            if let Ok(()) = process_album_art(&path) {
                                let mut cmd = Command::new("awww");
                                let out = cmd.arg("img").arg(path).output().unwrap();
                                let stdout = String::from_utf8(out.stdout).unwrap();
                                let stderr = String::from_utf8(out.stderr).unwrap();
                                println!("out: {stdout}");
                                println!("err: {stderr}");
                                curr_url = url.to_string();
                            }
                        }
                    }
                }
                Err(_e) => continue,
                _ => (),
            }
        }
    } else {
        eprintln!("No active player detected!");
        std::process::exit(1);
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <path to temp file>", args[0]);
        std::process::exit(1);
    }

    let path: &str = args[1].as_str();

    begin_listener(path).expect("Failed to be a good app");
}

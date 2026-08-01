use mpris::{PlayerFinder, Event};
use reqwest::blocking;
use std::{error::Error, fs::File, io, process::Command};
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

    println!("Original: {src_w}px {src_h}px");
    println!("Resized: {new_w}px {new_h}px");
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

fn begin_listener() -> Result<(), Box<dyn Error>> {
    let finder = PlayerFinder::new()?;
    let player = finder.find_active()?;

    let path = "/home/lee/Pictures/temp.jpg";
    let mut curr_url: String = String::new();
    for event in player.events()? {
        match event {
            Ok(Event::Playing) | Ok(Event::Paused) => println!("Playback toggle"),
            Ok(Event::TrackChanged(meta)) => {
                let url = meta.art_url().unwrap();
                if curr_url != url {
                    if let Ok(()) = get_album_art(url, path) {
                        if let Ok(()) = process_album_art(path) {
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
    Ok(())
}

fn main() {
    let finder: PlayerFinder = PlayerFinder::new().unwrap();
    let player = finder.find_active().unwrap();

    println!("Listening to changes on: {}", player.identity());

    begin_listener().expect("Failed to be a good app");
}

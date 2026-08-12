use imageproc::{
    compose::overlay,
    image::{DynamicImage, EncodableLayout, ImageFormat, imageops::FilterType, load_from_memory},
};
use mpris::{Event, PlayerFinder};
use reqwest::blocking;
use std::{env, error::Error, process, time::Duration};

const TEMP_IMAGE: &str = "pbj-wayland.jpg";

fn get_album_art(url: &str) -> Result<DynamicImage, Box<dyn Error>> {
    let response = blocking::get(url)?;
    let img = load_from_memory(response.bytes()?.as_bytes())?;
    Ok(img)
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

    DynamicImage::ImageRgb8(overlay(
        &bg.to_rgb8(),
        &resized.to_rgb8(),
        x_offset,
        y_offset,
    ))
}

fn process_album_art(img: &DynamicImage) -> Result<(), Box<dyn Error>> {
    let mut bg = img.clone();
    bg = bg.blur(10.0);
    let cover = resize_cover(&bg, 1920, 1080);
    let res = resize_contain(&img, &cover);
    res.save_with_format(env::temp_dir().join(TEMP_IMAGE), ImageFormat::Jpeg)?;
    Ok(())
}

fn update_wallpaper(url: &str) {
    if let Ok(img) = get_album_art(url) {
        if let Ok(()) = process_album_art(&img) {
            match process::Command::new("awww")
                .arg("img")
                .arg(env::temp_dir().join(TEMP_IMAGE))
                .output()
            {
                Ok(_) => (),
                Err(_) => (),
            }
        }
    }
}

fn handle_event(e: &Event) {
    match e {
        Event::Playing | Event::Paused => {
            println!("Playback toggle");
        }
        Event::TrackChanged(meta) => {
            if let Some(url) = meta.art_url() {
                println!("Found album art: {url}");
                update_wallpaper(url);
            } else {
                println!("No album art for active player");
            }
        }
        Event::PlayerShutDown => println!("Player closed"),
        _ => (),
    }
}

fn begin_listener() {
    if let Ok(finder) = PlayerFinder::new() {
        loop {
            if let Ok(player) = finder.find_active() {
                println!("Player found! Listening on: {}", player.bus_name_trimmed());
                match player.events() {
                    Ok(event_queue) => {
                        for event in event_queue {
                            if let Ok(e) = event {
                                handle_event(&e);
                            } else {
                                continue;
                            }
                        }
                    }
                    Err(e) => eprintln!("Error accessing player events: {e:?}"),
                }
            } else {
                eprintln!("No player found, trying again...")
            }
            std::thread::sleep(Duration::from_secs(2));
        }
    }
}

fn main() {
    begin_listener();
}

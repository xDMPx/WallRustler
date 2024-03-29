use rand::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Wallpaper {
    path: std::path::PathBuf,
    count: usize,
}

const COUNT_FACTOR: f64 = 1.001;

fn main() {
    swww_init().unwrap();

    let wallpapers_dir_path = std::env::args()
        .last()
        .map(|path_str| std::path::PathBuf::from(path_str))
        .unwrap();

    let mut wallpapers_state_path = wallpapers_dir_path.clone();
    wallpapers_state_path.push("state.bin");

    let mut wallpapers: Vec<Wallpaper> = if let Ok(state) = std::fs::read(&wallpapers_state_path) {
        println!("Using previous state");
        serde_binary::from_vec(state, serde_binary::binary_stream::Endian::Little).unwrap()
    } else {
        let wallpapers = wallpapers_dir_path.read_dir().unwrap();
        let wallpapers = wallpapers.filter_map(|dir_entry| dir_entry.ok());
        let wallpapers = wallpapers.filter(|dir_entry| {
            dir_entry
                .path()
                .extension()
                .map_or(false, |extension| is_img_file(extension))
        });
        let wallpapers = wallpapers.map(|wallpaper| Wallpaper {
            path: wallpaper.path(),
            count: 0,
        });
        wallpapers.collect()
    };

    loop {
        wallpapers = mean_centering_counts(wallpapers);
        let wallpaper = pick_random_wallpaper(&mut wallpapers);
        set_wallpaper(wallpaper);
        let state =
            serde_binary::to_vec(&wallpapers, serde_binary::binary_stream::Endian::Little).unwrap();
        std::fs::write(&wallpapers_state_path, state).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(15 * 60));
    }
}

fn pick_random_wallpaper(wallpapers: &mut Vec<Wallpaper>) -> &std::path::Path {
    let total_count_w: f64 = wallpapers
        .iter()
        .map(|wallpaper| wallpaper.count as f64)
        .fold(0.0, |acc, count: f64| acc + COUNT_FACTOR.powf(-count));

    let rand_num = get_random_num(total_count_w);
    let mut cum_count_w: f64 = 0.0;
    let wallpaper = wallpapers
        .iter_mut()
        .skip_while(|wallpaper| {
            cum_count_w += COUNT_FACTOR.powf(-(wallpaper.count as f64));
            cum_count_w < rand_num
        })
        .next()
        .unwrap();

    wallpaper.count += 1;
    &wallpaper.path
}

fn set_wallpaper(wallpaper: &std::path::Path) {
    std::process::Command::new("swww")
        .arg("img")
        .arg(wallpaper)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

fn is_img_file(extension: &std::ffi::OsStr) -> bool {
    match extension.to_string_lossy().to_string().as_str() {
        "jpg" => true,
        "jpeg" => true,
        "png" => true,
        "gif" => true,
        "pnm" => true,
        "tga" => true,
        "tiff" => true,
        "webp" => true,
        "bmp" => true,
        "farbfeld" => true,
        _ => false,
    }
}

fn get_random_num(to: f64) -> f64 {
    let mut rng = rand_hc::Hc128Rng::from_entropy();
    rng.gen_range(0.0..to)
}

fn swww_init() -> Result<(), std::io::Error> {
    let output = std::process::Command::new("pgrep")
        .arg("-f")
        .arg("swww")
        .output()?;

    if !output.status.success() {
        std::process::Command::new("swww")
            .arg("init")
            .spawn()
            .map(|mut child| {
                child
                    .wait()
                    .map_err(|child_error| eprintln!("{:?}", child_error))
                    .ok()
            })?;
    }

    Ok(())
}

fn mean_centering_counts(mut wallpapers: Vec<Wallpaper>) -> Vec<Wallpaper> {
    if let Some(min) = wallpapers.iter().map(|w| w.count).min() {
        if min != 0 {
            wallpapers.iter_mut().for_each(|w| w.count -= min);
        }
    }
    wallpapers
}

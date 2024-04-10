#[cfg_attr(target_os = "windows", path = "windows.rs")]
#[cfg_attr(not(target_os = "windows"), path = "linux.rs")]
pub mod wallpaper;

use rand::prelude::*;
use serde::{Deserialize, Serialize};

const COUNT_FACTOR: f64 = 1.001;

#[derive(Serialize, Deserialize, Debug)]
pub struct Wallpaper {
    pub file_name: std::ffi::OsString,
    pub count: usize,
}

#[derive(Debug, PartialEq)]
pub enum Option {
    Path(std::path::PathBuf),
    PrintState,
}

#[derive(Debug)]
pub enum Error {
    InvalidOption(String),
    InvalidOptionsStructure,
}

pub fn process_args() -> Result<Vec<Option>, Error> {
    let mut options = vec![];
    let mut args = std::env::args().skip(1).rev();
    if let Some(wallpapers_dir_path) = args.next() {
        let wallpapers_dir_path = std::path::PathBuf::from(wallpapers_dir_path);
        if !wallpapers_dir_path.is_dir() {
            return Err(Error::InvalidOptionsStructure);
        }
        options.push(Option::Path(wallpapers_dir_path));
        for arg in args {
            let arg = match arg.as_str() {
                "--print-state" => Ok(Option::PrintState),
                _ => Err(Error::InvalidOption(arg)),
            };
            options.push(arg?);
        }
        return Ok(options);
    } else {
        return Err(Error::InvalidOptionsStructure);
    }
}

pub fn pick_random_wallpaper(
    wallpaper_dir_path: &std::path::Path,
    wallpapers: &mut Vec<Wallpaper>,
) -> std::path::PathBuf {
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

    wallpaper_dir_path.join(wallpaper.file_name.clone())
}

pub fn sync_wallpapers(
    wallpaper_dir_path: &std::path::Path,
    mut wallpapers: Vec<Wallpaper>,
) -> Vec<Wallpaper> {
    let wallpapers_names = get_wallpapers_from_path(&wallpaper_dir_path);

    let old_wallpapers_names: Vec<&std::ffi::OsString> = wallpapers
        .iter()
        .map(|wallpaper_name| &wallpaper_name.file_name)
        .collect();

    let mut new_wallpapers: Vec<Wallpaper> = wallpapers_names
        .iter()
        .filter(|wallpaper_name| !old_wallpapers_names.contains(wallpaper_name))
        .map(|wallpaper_name| Wallpaper {
            file_name: wallpaper_name.clone(),
            count: 1,
        })
        .collect();
    new_wallpapers
        .iter()
        .for_each(|wallpaper| println!("Pushing {}", wallpaper.file_name.to_string_lossy()));

    let removed_wallpapers: Vec<usize> = old_wallpapers_names
        .iter()
        .filter(|wallpaper_name| !wallpapers_names.contains(wallpaper_name))
        .filter_map(|wallpaper_name| {
            old_wallpapers_names
                .iter()
                .position(|old_wallpapers_name| old_wallpapers_name == wallpaper_name)
        })
        .collect();

    for wallpaper_index in removed_wallpapers {
        println!(
            "Poping {}",
            wallpapers_names[wallpaper_index].to_string_lossy()
        );
        wallpapers.swap_remove(wallpaper_index);
    }

    wallpapers.append(&mut new_wallpapers);

    wallpapers
}

pub fn mean_centering_counts(mut wallpapers: Vec<Wallpaper>) -> Vec<Wallpaper> {
    if let Some(min) = wallpapers.iter().map(|w| w.count).min() {
        if min != 0 {
            wallpapers.iter_mut().for_each(|w| w.count -= min);
        }
    }
    wallpapers
}

pub fn get_wallpapers_from_path(wallpaper_dir_path: &std::path::Path) -> Vec<std::ffi::OsString> {
    let wallpapers = wallpaper_dir_path.read_dir().unwrap();
    let wallpapers = wallpapers.filter_map(|dir_entry| dir_entry.ok());
    let wallpapers = wallpapers.filter(|dir_entry| {
        dir_entry
            .path()
            .extension()
            .map_or(false, |extension| is_img_file(extension))
    });
    let wallpapers = wallpapers.map(|dir_entry| dir_entry.file_name());

    wallpapers.collect()
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

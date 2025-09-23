#[cfg_attr(target_os = "windows", path = "windows.rs")]
#[cfg_attr(not(target_os = "windows"), path = "linux.rs")]
pub mod wallpaper;

impl Default for crate::wallpaper::WallSetter {
    fn default() -> Self {
        Self::new()
    }
}

use rand::prelude::*;
use serde::{Deserialize, Serialize};
#[cfg(target_os = "linux")]
use wallpaper::WallSetterProgram;

const COUNT_FACTOR: f64 = 1.001;

#[derive(Serialize, Deserialize, Debug)]
pub struct Wallpaper {
    pub file_name: String,
    pub count: usize,
}

#[derive(Debug, PartialEq)]
pub enum Option {
    Path(std::path::PathBuf),
    PrintState,
    PrintHelp,
    Interval(u64),
    #[cfg(target_os = "linux")]
    Program(WallSetterProgram),
    #[cfg(target_os = "windows")]
    HideTerminalWindow,
}

#[derive(Debug)]
pub enum Error {
    InvalidOption(String),
    UnavailableOption(String),
    InvalidOptionsStructure,
}

pub fn process_args() -> Result<Vec<Option>, Error> {
    let mut options = vec![];
    let mut args: Vec<String> = std::env::args().skip(1).collect();

    let last_arg = args.pop().ok_or(Error::InvalidOptionsStructure)?;
    if last_arg != "--help" {
        let wallpapers_dir_path = last_arg;
        let wallpapers_dir_path = std::path::PathBuf::from(wallpapers_dir_path);
        if !wallpapers_dir_path.is_dir() {
            return Err(Error::InvalidOptionsStructure);
        }
        options.push(Option::Path(wallpapers_dir_path));
    } else {
        args.push(last_arg);
    }

    for arg in args {
        let arg = match arg.as_str() {
            "--print-state" => Ok(Option::PrintState),
            "--help" => Ok(Option::PrintHelp),
            s if s.starts_with("--interval=") => {
                if let Some(Ok(min)) = s.split_once('=').map(|(_, s)| s.parse::<u64>()) {
                    if min > 0 {
                        Ok(Option::Interval(min))
                    } else {
                        Err(Error::InvalidOption(arg))
                    }
                } else {
                    Err(Error::InvalidOption(arg))
                }
            }
            #[cfg(target_os = "linux")]
            s if s.starts_with("--program=") => {
                if s.ends_with("swww") {
                    Ok(Option::Program(WallSetterProgram::SWWW))
                } else if s.ends_with("plasma-apply-wallpaperimage") {
                    if let Ok(val) = std::env::var("XDG_CURRENT_DESKTOP") {
                        if val == "KDE" {
                            Ok(Option::Program(WallSetterProgram::PLASMA))
                        } else {
                            Err(Error::UnavailableOption(
                                "plasma-apply-wallpaperimage".to_owned(),
                            ))
                        }
                    } else {
                        Err(Error::UnavailableOption(
                            "plasma-apply-wallpaperimage".to_owned(),
                        ))
                    }
                } else if s.ends_with("hyprpaper") {
                    #[allow(unused_mut, unused_assignments)]
                    let mut option = Err(Error::InvalidOption(arg));
                    #[cfg(all(feature = "hyprpaper", target_os = "linux"))]
                    {
                        if let Ok(_) = std::env::var("HYPRLAND_INSTANCE_SIGNATURE") {
                            option = Ok(Option::Program(WallSetterProgram::HYPRPAPER));
                        } else {
                            option = Err(Error::UnavailableOption("hyprpaper".to_owned()))
                        }
                    }
                    option
                } else {
                    Err(Error::InvalidOption(arg))
                }
            }
            #[cfg(target_os = "windows")]
            "--hidden" => Ok(Option::HideTerminalWindow),
            _ => Err(Error::InvalidOption(arg)),
        };
        options.push(arg?);
    }

    Ok(options)
}

pub fn print_help() {
    println!("Usage: {} [OPTIONS] DIRECTORY", env!("CARGO_PKG_NAME"));
    println!("       {} --print-state DIRECTORY", env!("CARGO_PKG_NAME"));
    println!("       {} --help", env!("CARGO_PKG_NAME"));
    println!("Options:");
    println!("\t --help");
    println!("\t --interval=<u64>");
    #[cfg(target_os = "windows")]
    println!("\t --hidden");
    #[cfg(not(all(feature = "hyprpaper", target_os = "linux")))]
    println!("\t --program=<swww|plasma-apply-wallpaperimage>");
    #[cfg(all(feature = "hyprpaper", target_os = "linux"))]
    println!("\t --program=<swww|hyprpaper|plasma-apply-wallpaperimage>");
}

pub fn pick_random_wallpaper(
    wallpaper_dir_path: &std::path::Path,
    wallpapers: &mut [Wallpaper],
) -> std::path::PathBuf {
    let total_count_w: f64 = wallpapers
        .iter()
        .map(|wallpaper| wallpaper.count as f64)
        .fold(0.0, |acc, count: f64| acc + COUNT_FACTOR.powf(-count));

    let rand_num = get_random_num(total_count_w);
    let mut cum_count_w: f64 = 0.0;
    let wallpaper = wallpapers
        .iter_mut()
        .find(|wallpaper| {
            cum_count_w += COUNT_FACTOR.powf(-(wallpaper.count as f64));
            cum_count_w >= rand_num
        })
        .unwrap();

    wallpaper.count += 1;

    wallpaper_dir_path.join(wallpaper.file_name.clone())
}

pub fn sync_wallpapers(
    wallpaper_dir_path: &std::path::Path,
    mut wallpapers: Vec<Wallpaper>,
) -> Vec<Wallpaper> {
    let wallpapers_names = get_wallpapers_paths_from_path(wallpaper_dir_path);

    let old_wallpapers_names: Vec<&String> = wallpapers
        .iter()
        .map(|wallpaper_name| &wallpaper_name.file_name)
        .collect();

    let mut new_wallpapers: Vec<Wallpaper> = wallpapers_names
        .iter()
        .filter(|wallpaper_name| !old_wallpapers_names.contains(wallpaper_name))
        .map(|wallpaper_name| Wallpaper {
            file_name: wallpaper_name.clone(),
            count: 0,
        })
        .collect();
    new_wallpapers
        .iter()
        .for_each(|wallpaper| println!("Pushing {}", wallpaper.file_name));

    let removed_wallpapers_names: Vec<String> = old_wallpapers_names
        .iter()
        .filter(|wallpaper_name| !wallpapers_names.contains(wallpaper_name))
        .map(|wallpaper_name| wallpaper_name.to_string())
        .collect();

    for to_remove in removed_wallpapers_names {
        if let Some(to_remove_index) = wallpapers
            .iter()
            .position(|wallpaper| wallpaper.file_name == to_remove)
        {
            println!("Popping {}", to_remove);
            wallpapers.swap_remove(to_remove_index);
        }
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

pub fn find_wallpaper_path(options: &Vec<Option>) -> std::option::Option<&std::path::PathBuf> {
    let wallpapers_dir_path = options.iter().find_map(|option| match option {
        Option::Path(path) => Some(path),
        _ => None,
    });

    wallpapers_dir_path
}

pub fn retrieve_wallpapers(path: &std::path::PathBuf) -> Vec<Wallpaper> {
    let mut wallpapers_state_path = path.clone();
    wallpapers_state_path.push("state.bin");
    let wallpapers: Vec<Wallpaper> = if let Ok(state) = std::fs::read(&wallpapers_state_path) {
        println!("Using previous state");
        serde_binary::from_vec(state, serde_binary::binary_stream::Endian::Little).unwrap()
    } else {
        let wallpapers_paths = get_wallpapers_paths_from_path(path);
        let wallpapers = wallpapers_paths
            .into_iter()
            .map(|wallpaper_path| Wallpaper {
                file_name: wallpaper_path,
                count: 0,
            });
        wallpapers.collect()
    };

    wallpapers
}

pub fn get_wallpapers_paths_from_path(wallpaper_dir_path: &std::path::Path) -> Vec<String> {
    let wallpapers = wallpaper_dir_path.read_dir().unwrap();
    let wallpapers = wallpapers.filter_map(|dir_entry| dir_entry.ok());
    let wallpapers =
        wallpapers.filter(|dir_entry| dir_entry.path().extension().is_some_and(is_img_file));
    let wallpapers = wallpapers.map(|dir_entry| {
        dir_entry
            .file_name()
            .into_string()
            .unwrap_or_else(|_| panic!("Invalid Unicode file name: {:?}", dir_entry))
    });

    wallpapers.collect()
}

fn is_img_file(extension: &std::ffi::OsStr) -> bool {
    let ext_str = extension.to_string_lossy().to_string();

    matches!(
        ext_str.as_str(),
        "jpg" | "jpeg" | "png" | "gif" | "pnm" | "tga" | "tiff" | "webp" | "bmp" | "farbfeld"
    )
}

fn get_random_num(to: f64) -> f64 {
    let mut rng = rand_hc::Hc128Rng::from_entropy();
    rng.gen_range(0.0..to)
}

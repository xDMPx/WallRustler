#![windows_subsystem = "windows"]

#[allow(unused_imports)]
use std::env;
use wallrustler::wallpaper::WallSetter;
use wallrustler::{
    get_wallpapers_from_path, mean_centering_counts, pick_random_wallpaper, print_help,
    process_args, sync_wallpapers, Error, Option, Wallpaper,
};

#[cfg(all(feature = "hyprpaper", target_os = "linux"))]
use wallrustler::wallpaper::WallSetterProgram;

fn main() {
    #[allow(unused_mut)]
    let mut wall_setter = WallSetter::new();

    let options = process_args()
        .map_err(|err| {
            match err {
                Error::InvalidOption(option) => eprintln!("Provided option {option} is invalid"),
                Error::InvalidOptionsStructure => eprintln!("Invalid input"),
            }
            print_help();
            std::process::exit(-1);
        })
        .unwrap();
    if options.contains(&Option::PrintHelp) {
        print_help();
        std::process::exit(-1);
    }
    #[cfg(all(feature = "hyprpaper", target_os = "linux"))]
    if options.contains(&Option::Program(WallSetterProgram::HYPRPAPER)) {
        println!("Using hyprpaper");
        wall_setter.set_program(wallrustler::wallpaper::WallSetterProgram::HYPRPAPER);
    }

    let wallpapers_dir_path = options
        .iter()
        .find_map(|option| match option {
            Option::Path(path) => Some(path),
            _ => None,
        })
        .unwrap();

    if !wall_setter.is_running() {
        wall_setter.init();
    } else {
        println!("Killing already running instance");
        wall_setter.kill().unwrap();
        wall_setter.init();
    }

    let mut wallpapers_state_path = wallpapers_dir_path.clone();
    wallpapers_state_path.push("state.bin");

    let mut wallpapers: Vec<Wallpaper> = if let Ok(state) = std::fs::read(&wallpapers_state_path) {
        println!("Using previous state");
        serde_binary::from_vec(state, serde_binary::binary_stream::Endian::Little).unwrap()
    } else {
        let wallpapers_paths = get_wallpapers_from_path(&wallpapers_dir_path);
        let wallpapers = wallpapers_paths
            .into_iter()
            .map(|wallpaper_path| Wallpaper {
                file_name: wallpaper_path,
                count: 0,
            });
        wallpapers.collect()
    };

    if options.contains(&Option::PrintState) {
        wallpapers = sync_wallpapers(&wallpapers_dir_path, wallpapers);
        let states: Vec<(String, usize)> = wallpapers
            .iter()
            .map(|wallpaper| (wallpaper.file_name.to_owned(), wallpaper.count))
            .collect();
        let max_len = states.iter().map(|(name, _)| name.len()).max().unwrap();
        for (name, count) in states {
            println!("{:<max_len$}: {count}", name);
        }
        return;
    }

    loop {
        wallpapers = sync_wallpapers(&wallpapers_dir_path, wallpapers);
        wallpapers = mean_centering_counts(wallpapers);
        let wallpaper = pick_random_wallpaper(&wallpapers_dir_path, &mut wallpapers);
        wall_setter.set_wallpaper(&wallpaper).unwrap();
        let state =
            serde_binary::to_vec(&wallpapers, serde_binary::binary_stream::Endian::Little).unwrap();
        std::fs::write(&wallpapers_state_path, state).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1 * 60));
    }
}

#![windows_subsystem = "windows"]

#[allow(unused_imports)]
use std::env;
use wallrustler::wallpaper::WallSetter;
use wallrustler::{
    find_wallpaper_path, mean_centering_counts, pick_random_wallpaper, print_help, process_args,
    retrieve_wallpapers, sync_wallpapers, Error, Option,
};

#[cfg(target_os = "linux")]
use wallrustler::wallpaper::WallSetterProgram;

fn main() {
    #[allow(unused_mut)]
    let mut wall_setter = WallSetter::new();

    let mut interval = 15 * 60;

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
    if options.contains(&Option::PrintState) {
        let wallpapers_dir_path = find_wallpaper_path(&options).unwrap();
        let mut wallpapers = retrieve_wallpapers(wallpapers_dir_path);
        wallpapers = sync_wallpapers(wallpapers_dir_path, wallpapers);

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

    if let Some(m) = options.iter().find_map(|o| match o {
        Option::Interval(min) => Some(min),
        _ => None,
    }) {
        interval = m * 60;
    }

    #[cfg(target_os = "linux")]
    if let Ok(val) = env::var("XDG_CURRENT_DESKTOP") {
        if val == "KDE" {
            println!("KDE detected, switching to plasma-apply-wallpaperimage as wallpaper setting program\nThis behavior can be changed by using --program option");
            wall_setter.set_program(WallSetterProgram::PLASMA);
        }
    }

    #[cfg(target_os = "linux")]
    if let Some(p) = options.iter().find_map(|o| match o {
        Option::Program(program) => Some(program),
        _ => None,
    }) {
        println!("Using {p:?}");
        wall_setter.set_program(*p);
    }

    let wallpapers_dir_path = find_wallpaper_path(&options).unwrap();

    if !wall_setter.is_running() {
        wall_setter.init();
    } else {
        println!("Killing already running instance");
        wall_setter.kill().unwrap();
        wall_setter.init();
    }

    let mut wallpapers = retrieve_wallpapers(wallpapers_dir_path);
    wallpapers = sync_wallpapers(wallpapers_dir_path, wallpapers);

    let mut wallpapers_state_path = wallpapers_dir_path.clone();
    wallpapers_state_path.push("state.bin");
    loop {
        wallpapers = sync_wallpapers(wallpapers_dir_path, wallpapers);
        wallpapers = mean_centering_counts(wallpapers);
        let wallpaper = pick_random_wallpaper(wallpapers_dir_path, &mut wallpapers);
        wall_setter.set_wallpaper(&wallpaper).unwrap();
        let state =
            serde_binary::to_vec(&wallpapers, serde_binary::binary_stream::Endian::Little).unwrap();
        std::fs::write(&wallpapers_state_path, state).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(interval));
    }
}

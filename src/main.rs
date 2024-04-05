use wallrustler::wallpaper::{init, set_wallpaper};
use wallrustler::{
    get_wallpapers_from_path, mean_centering_counts, pick_random_wallpaper, sync_wallpapers,
    Wallpaper,
};

fn main() {
    init();

    let wallpapers_dir_path = std::env::args()
        .skip(1)
        .map(|arg| std::path::PathBuf::from(arg))
        .find(|path| path.is_dir())
        .unwrap();

    let mut wallpapers_state_path = wallpapers_dir_path.clone();
    wallpapers_state_path.push("state.bin");

    let mut wallpapers: Vec<Wallpaper> = if let Ok(state) = std::fs::read(&wallpapers_state_path) {
        println!("Using previous state");
        serde_binary::from_vec(state, serde_binary::binary_stream::Endian::Little).unwrap()
    } else {
        let wallpapers_paths = get_wallpapers_from_path(&wallpapers_dir_path);
        let wallpapers = wallpapers_paths.iter().map(|wallpaper_path| Wallpaper {
            path: wallpaper_path.to_path_buf(),
            count: 0,
        });
        wallpapers.collect()
    };

    if std::env::args()
        .find(|arg| arg == "--print-state")
        .is_some()
    {
        wallpapers = sync_wallpapers(&wallpapers_dir_path, wallpapers);
        let states: Vec<(String, usize)> = wallpapers
            .iter()
            .map(|wallpaper| {
                (
                    wallpaper
                        .path
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string(),
                    wallpaper.count,
                )
            })
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
        let wallpaper = pick_random_wallpaper(&mut wallpapers);
        set_wallpaper(wallpaper);
        let state =
            serde_binary::to_vec(&wallpapers, serde_binary::binary_stream::Endian::Little).unwrap();
        std::fs::write(&wallpapers_state_path, state).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(15 * 60));
    }
}

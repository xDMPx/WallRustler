use rand::prelude::*;

fn main() {
    let wallpaper_path = std::env::args()
        .last()
        .map(|path_str| std::path::PathBuf::from(path_str))
        .unwrap();

    let wallpapers = wallpaper_path.read_dir().unwrap();
    let wallpapers = wallpapers.filter_map(|dir_entry| dir_entry.ok());
    let wallpapers = wallpapers.filter(|dir_entry| {
        dir_entry
            .path()
            .extension()
            .map_or(false, |extension| is_img_file(extension))
    });
    let wallpapers: Vec<std::fs::DirEntry> = wallpapers.collect();
    let wallpaper = &wallpapers[get_random_num(wallpapers.len())];
    std::process::Command::new("swww")
        .arg("img")
        .arg(wallpaper.path())
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

fn is_img_file(extension: &std::ffi::OsStr) -> bool {
    match extension.to_string_lossy().to_string().as_str() {
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

fn get_random_num(to: usize) -> usize {
    let mut rng = rand_hc::Hc128Rng::from_entropy();
    rng.gen_range(0..to)
}

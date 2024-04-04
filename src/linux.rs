pub fn init() {
    if is_running_under_wayland() {
        swww_init().unwrap();
    }
}

pub fn set_wallpaper(wallpaper: &std::path::Path) {
    if is_running_under_wayland() {
        set_wallpaper_wayland(wallpaper);
    } else {
        set_wallpaper_x11(wallpaper);
    }
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

fn is_running_under_wayland() -> bool {
    let wayland = std::env::var("WAYLAND_DISPLAY");
    wayland.is_ok()
}

fn set_wallpaper_wayland(wallpaper: &std::path::Path) {
    std::process::Command::new("swww")
        .arg("img")
        .arg(wallpaper)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

fn set_wallpaper_x11(wallpaper: &std::path::Path) {
    std::process::Command::new("feh")
        .arg("--bg-fill")
        .arg(wallpaper)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

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

pub fn is_running() -> bool {
    let output = std::process::Command::new("pgrep")
        .arg("-c")
        .arg(env!("CARGO_PKG_NAME"))
        .output();

    output
        .map(|output| {
            output.status.success()
                && std::string::String::from_utf8(output.stdout)
                    .map(|x| !x.starts_with("1"))
                    .unwrap_or(false)
        })
        .unwrap_or(false)
}

pub fn kill() -> Result<(), std::io::Error> {
    let output = std::process::Command::new("pkill")
        .arg("-o")
        .arg(env!("CARGO_PKG_NAME"))
        .output()?;

    if !output.status.success() {
        eprintln!("{:?}", output.stderr);
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("{:?}", output),
        ));
    }

    let output = std::process::Command::new("pkill")
        .arg("-o")
        .arg("swww-daemon")
        .output()?;

    if !output.status.success() {
        eprintln!("{:?}", output.stderr);
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("{:?}", output),
        ));
    }

    Ok(())
}

fn swww_init() -> Result<(), std::io::Error> {
    let output = std::process::Command::new("pgrep")
        .arg("-f")
        .arg("swww")
        .output()?;

    if !output.status.success() {
        std::process::Command::new("swww-daemon")
            .spawn()
            .map(|mut child| {
                child
                    .try_wait()
                    .map_err(|child_error| eprintln!("{:?}", child_error))
                    .ok()
            })?;
    }
    std::thread::sleep(std::time::Duration::from_secs(1));

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

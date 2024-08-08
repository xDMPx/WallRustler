pub fn init() {
    if is_running_under_wayland() {
        swww_init().unwrap();
    }
}

pub fn set_wallpaper(wallpaper: &std::path::Path) -> Result<(), std::io::Error> {
    if is_running_under_wayland() {
        set_wallpaper_wayland(wallpaper)?;
        std::thread::sleep(std::time::Duration::from_secs(10));
        kill_swww_daemon()?;
        swww_daemon_init()?;
    } else {
        set_wallpaper_x11(wallpaper)?;
    }

    Ok(())
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

    kill_swww_daemon()?;

    Ok(())
}

fn kill_swww_daemon() -> Result<(), std::io::Error> {
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
        swww_daemon_init()?;
    }

    Ok(())
}

fn swww_daemon_init() -> Result<(), std::io::Error> {
    std::process::Command::new("swww-daemon")
        .spawn()
        .map(|mut child| {
            child
                .try_wait()
                .map_err(|child_error| eprintln!("{:?}", child_error))
                .ok()
        })?;
    std::thread::sleep(std::time::Duration::from_secs(2));

    Ok(())
}

fn is_running_under_wayland() -> bool {
    let wayland = std::env::var("WAYLAND_DISPLAY");
    wayland.is_ok()
}

fn set_wallpaper_wayland(wallpaper: &std::path::Path) -> Result<(), std::io::Error> {
    std::process::Command::new("swww")
        .arg("img")
        .arg(wallpaper)
        .spawn()?
        .wait()?;

    Ok(())
}

fn set_wallpaper_x11(wallpaper: &std::path::Path) -> Result<(), std::io::Error> {
    std::process::Command::new("feh")
        .arg("--bg-fill")
        .arg(wallpaper)
        .spawn()?
        .wait()?;

    Ok(())
}

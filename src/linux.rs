pub struct WallSetter {
    child: Option<std::process::Child>,
}

impl WallSetter {
    pub fn new() -> WallSetter {
        WallSetter { child: None }
    }

    pub fn init(&mut self) {
        if self.is_running_under_wayland() {
            self.swww_init().unwrap();
        }
    }

    pub fn set_wallpaper(&mut self, wallpaper: &std::path::Path) -> Result<(), std::io::Error> {
        if self.is_running_under_wayland() {
            self.set_wallpaper_wayland(wallpaper)?;
            std::thread::sleep(std::time::Duration::from_secs(10));
            self.kill_swww_daemon()?;
            self.swww_daemon_init()?;
        } else {
            self.set_wallpaper_x11(wallpaper)?;
        }

        Ok(())
    }

    pub fn is_running(&self) -> bool {
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

    pub fn kill(&mut self) -> Result<(), std::io::Error> {
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

        self.kill_swww_daemon()?;

        Ok(())
    }

    fn kill_swww_daemon(&mut self) -> Result<(), std::io::Error> {
        if let Some(child) = self.child.as_mut() {
            child.kill()?;
            child.wait()?;
            self.child = None;
        } else {
            let output = std::process::Command::new("pkill")
                .arg("swww-daemon")
                .output()?;

            if !output.status.success() {
                eprintln!("{:?}", output.stderr);
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("{:?}", output),
                ));
            }
        }

        Ok(())
    }

    fn swww_init(&mut self) -> Result<(), std::io::Error> {
        let output = std::process::Command::new("pgrep")
            .arg("-f")
            .arg("swww")
            .output()?;

        if !output.status.success() {
            self.swww_daemon_init()?;
        }

        Ok(())
    }

    fn swww_daemon_init(&mut self) -> Result<(), std::io::Error> {
        std::thread::sleep(std::time::Duration::from_secs(2));
        self.child = Some(std::process::Command::new("swww-daemon").spawn()?);
        std::thread::sleep(std::time::Duration::from_secs(2));

        Ok(())
    }

    fn is_running_under_wayland(&self) -> bool {
        let wayland = std::env::var("WAYLAND_DISPLAY");
        wayland.is_ok()
    }

    fn set_wallpaper_wayland(&self, wallpaper: &std::path::Path) -> Result<(), std::io::Error> {
        std::process::Command::new("swww")
            .arg("img")
            .arg(wallpaper)
            .spawn()?
            .wait()?;

        Ok(())
    }

    fn set_wallpaper_x11(&self, wallpaper: &std::path::Path) -> Result<(), std::io::Error> {
        std::process::Command::new("feh")
            .arg("--bg-fill")
            .arg(wallpaper)
            .spawn()?
            .wait()?;

        Ok(())
    }
}

use core::ffi::c_void;
use std::os::windows::ffi::OsStrExt;
use std::thread::sleep;
use std::time::Duration;
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::System::Console::GetConsoleWindow;
use windows_sys::Win32::UI::WindowsAndMessaging::{GetWindow, ShowWindow, GW_OWNER, SW_HIDE};

pub struct WallSetter {
    hidden: bool,
}

impl WallSetter {
    pub fn new() -> WallSetter {
        WallSetter { hidden: false }
    }

    pub fn enable_hide_terminal_window(&mut self) {
        self.hidden = true;
    }

    pub fn init(&self) {
        if self.hidden {
            unsafe {
                // Based on: https://stackoverflow.com/a/78943791
                let hwnd: HWND = GetConsoleWindow();
                if hwnd == std::ptr::null_mut() {
                    return;
                }
                sleep(Duration::from_millis(200));
                let owner: HWND = GetWindow(hwnd, GW_OWNER);
                if owner == std::ptr::null_mut() {
                    // Windows 10/Console Host: hide the console window itself
                    ShowWindow(hwnd, SW_HIDE);
                } else {
                    // Windows 11 / Windows Terminal: hide the owner window
                    ShowWindow(owner, SW_HIDE);
                }
            }
        }
    }

    pub fn set_wallpaper(&self, wallpaper: &std::path::Path) -> Result<(), std::io::Error> {
        self.set_wallpaper_windows(wallpaper)
    }

    pub fn is_running(&self) -> bool {
        let output = std::process::Command::new("tasklist")
            .arg("/fo")
            .arg("csv")
            .arg("/nh")
            .arg("/fi")
            .arg(format!("IMAGENAME eq {}.exe", env!("CARGO_PKG_NAME")))
            .output();

        output
            .map(|output| {
                output.status.success()
                    && std::string::String::from_utf8_lossy(&output.stdout)
                        .to_string()
                        .lines()
                        .count()
                        > 1
            })
            .unwrap_or(false)
    }

    pub fn kill(&self) -> Result<(), std::io::Error> {
        let pid = self.get_running_pid()?;
        let output = std::process::Command::new("taskkill")
            .arg("/f")
            .arg("/t")
            .arg("/PID")
            .arg(pid.to_string())
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

    fn get_running_pid(&self) -> Result<usize, std::io::Error> {
        let output = std::process::Command::new("tasklist")
            .arg("/fo")
            .arg("csv")
            .arg("/nh")
            .arg("/fi")
            .arg(format!("IMAGENAME eq {}.exe", env!("CARGO_PKG_NAME")))
            .output()?;

        let out = std::string::String::from_utf8_lossy(&output.stdout);
        let pid: String = out
            .lines()
            .next()
            .map(|line| {
                line.split_once("\",\"")
                    .map(|split| split.1)
                    .unwrap_or("")
                    .split_once("\"")
                    .map(|split| split.0)
                    .unwrap_or("")
            })
            .unwrap_or("")
            .to_string();

        if pid.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("tasklist invalid out: {}", out),
            ));
        }

        if let Ok(pid) = pid.parse() {
            Ok(pid)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("tasklist invalid pid: {}", pid),
            ))
        }
    }

    fn set_wallpaper_windows(&self, wallpaper: &std::path::Path) -> Result<(), std::io::Error> {
        let path = std::ffi::OsStr::new(wallpaper)
            .encode_wide()
            .chain(Some(0))
            .collect::<Vec<u16>>();

        unsafe {
            windows_sys::Win32::UI::WindowsAndMessaging::SystemParametersInfoW(
                20,
                0,
                path.as_ptr() as *mut c_void,
                3,
            );
        }

        Ok(())
    }
}

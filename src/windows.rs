use core::ffi::c_void;
use std::os::windows::ffi::OsStrExt;

pub fn init() {}

pub fn set_wallpaper(wallpaper: &std::path::Path) {
    set_wallpaper_windows(wallpaper);
}

fn set_wallpaper_windows(wallpaper: &std::path::Path) {
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
}

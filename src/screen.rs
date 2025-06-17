#[cfg(target_os = "macos")]
mod macos_screen {
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGMainDisplayID() -> u32;
        fn CGDisplayPixelsWide(display: u32) -> usize;
        fn CGDisplayPixelsHigh(display: u32) -> usize;
    }

    pub fn get_main_screen_size() -> (usize, usize) {
        unsafe {
            let display_id = CGMainDisplayID();
            let width = CGDisplayPixelsWide(display_id);
            let height = CGDisplayPixelsHigh(display_id);
            (width, height)
        }
    }
}

#[cfg(target_os = "windows")]
mod windows_screen {
    use winapi::um::winuser::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

    pub fn get_main_screen_size() -> (usize, usize) {
        unsafe {
            let width = GetSystemMetrics(SM_CXSCREEN) as usize;
            let height = GetSystemMetrics(SM_CYSCREEN) as usize;
            (width, height)
        }
    }
}


#[cfg(any(target_os = "macos", target_os = "windows"))]
pub fn get_screen_size() -> (usize, usize) {
    #[cfg(target_os = "macos")]
    {
        macos_screen::get_main_screen_size()
    }
    #[cfg(target_os = "windows")]
    {
        windows_screen::get_main_screen_size()
    }
}

#[test]
#[cfg(any(target_os = "macos", target_os = "windows"))]
fn main() {
    let (width, height) = macos_screen::get_main_screen_size();
    println!("macOS 屏幕尺寸: {}x{}", width, height);
}

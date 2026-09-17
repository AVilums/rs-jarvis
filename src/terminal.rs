#[cfg(windows)]
pub fn initialize(host_window: isize) -> anyhow::Result<()> {
    use anyhow::bail;
    use windows_sys::Win32::{
        Foundation::{GENERIC_READ, GENERIC_WRITE, INVALID_HANDLE_VALUE},
        Storage::FileSystem::{CreateFileW, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING},
        System::Console::{
            AllocConsole, GetConsoleWindow, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
            SetConsoleTitleW, SetStdHandle,
        },
        UI::WindowsAndMessaging::PostMessageW,
    };

    if unsafe { AllocConsole() } == 0 {
        bail!("could not create the Jarvis terminal");
    }

    let input_name: Vec<u16> = "CONIN$\0".encode_utf16().collect();
    let output_name: Vec<u16> = "CONOUT$\0".encode_utf16().collect();
    unsafe {
        let input = CreateFileW(
            input_name.as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null(),
            OPEN_EXISTING,
            0,
            std::ptr::null_mut(),
        );
        let output = CreateFileW(
            output_name.as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null(),
            OPEN_EXISTING,
            0,
            std::ptr::null_mut(),
        );
        if input == INVALID_HANDLE_VALUE || output == INVALID_HANDLE_VALUE {
            bail!("could not connect to the Jarvis terminal");
        }
        SetStdHandle(STD_INPUT_HANDLE, input);
        SetStdHandle(STD_OUTPUT_HANDLE, output);
        SetStdHandle(STD_ERROR_HANDLE, output);

        let title: Vec<u16> = "Jarvis\0".encode_utf16().collect();
        SetConsoleTitleW(title.as_ptr());
        let console = GetConsoleWindow();
        if host_window != 0 && !console.is_null() {
            PostMessageW(
                host_window as _,
                crate::host::WM_TERMINAL_READY,
                console as usize,
                0,
            );
        }
    }

    Ok(())
}

#[cfg(windows)]
pub fn hide() {
    use windows_sys::Win32::{
        System::Console::GetConsoleWindow,
        UI::WindowsAndMessaging::{SW_HIDE, ShowWindow},
    };

    unsafe {
        let window = GetConsoleWindow();
        if !window.is_null() {
            ShowWindow(window, SW_HIDE);
        }
    }
}

#[cfg(not(windows))]
pub fn hide() {}

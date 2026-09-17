use std::{
    process::{Child, Command},
    ptr,
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicIsize, Ordering},
    },
};

use anyhow::{Context, Result, bail};
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM},
    System::LibraryLoader::GetModuleHandleW,
    UI::{
        Input::KeyboardAndMouse::{MOD_NOREPEAT, MOD_SHIFT, RegisterHotKey},
        Shell::{
            NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW, Shell_NotifyIconW,
        },
        WindowsAndMessaging::{
            AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu,
            DestroyWindow, DispatchMessageW, GetCursorPos, GetMessageW, IDI_APPLICATION, IsWindow,
            LoadIconW, MF_STRING, MSG, PostQuitMessage, RegisterClassW, SW_RESTORE,
            SetForegroundWindow, ShowWindow, TPM_RETURNCMD, TPM_RIGHTBUTTON, TrackPopupMenu,
            TranslateMessage, WM_APP, WM_DESTROY, WM_HOTKEY, WM_LBUTTONDBLCLK, WM_LBUTTONUP,
            WM_RBUTTONUP, WNDCLASSW,
        },
    },
};

const HOTKEY_ID: i32 = 0x4a52;
const J_KEY: u32 = b'J' as u32;
const TRAY_ID: u32 = 1;
const WM_TRAY: u32 = WM_APP + 1;
pub const WM_TERMINAL_READY: u32 = WM_APP + 2;
const MENU_OPEN: usize = 1;
const MENU_EXIT: usize = 2;

static TERMINAL_WINDOW: AtomicIsize = AtomicIsize::new(0);
static TERMINAL_PROCESS: OnceLock<Mutex<Option<Child>>> = OnceLock::new();

pub fn run() -> Result<()> {
    let class_name = wide("JarvisTrayHost");
    let instance = unsafe { GetModuleHandleW(ptr::null()) };
    if instance.is_null() {
        bail!("could not get the application module handle");
    }

    let class = WNDCLASSW {
        lpfnWndProc: Some(window_proc),
        hInstance: instance,
        lpszClassName: class_name.as_ptr(),
        ..unsafe { std::mem::zeroed() }
    };
    if unsafe { RegisterClassW(&class) } == 0 {
        bail!("could not register the tray window class");
    }

    let window = unsafe {
        CreateWindowExW(
            0,
            class_name.as_ptr(),
            class_name.as_ptr(),
            0,
            0,
            0,
            0,
            0,
            ptr::null_mut(),
            ptr::null_mut(),
            instance,
            ptr::null(),
        )
    };
    if window.is_null() {
        bail!("could not create the tray window");
    }

    add_tray_icon(window)?;
    if unsafe { RegisterHotKey(window, HOTKEY_ID, MOD_SHIFT | MOD_NOREPEAT, J_KEY) } == 0 {
        remove_tray_icon(window);
        bail!("could not register Shift+J; another application may already use it");
    }

    spawn_terminal(window)?;

    let mut message: MSG = unsafe { std::mem::zeroed() };
    while unsafe { GetMessageW(&mut message, ptr::null_mut(), 0, 0) } > 0 {
        unsafe {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }

    Ok(())
}

fn add_tray_icon(window: HWND) -> Result<()> {
    let mut data = tray_data(window);
    data.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    data.uCallbackMessage = WM_TRAY;
    data.hIcon = unsafe { LoadIconW(ptr::null_mut(), IDI_APPLICATION) };
    let tip: Vec<u16> = "Jarvis\0".encode_utf16().collect();
    data.szTip[..tip.len()].copy_from_slice(&tip);

    if unsafe { Shell_NotifyIconW(NIM_ADD, &data) } == 0 {
        bail!("could not add Jarvis to the notification area");
    }
    Ok(())
}

fn remove_tray_icon(window: HWND) {
    unsafe {
        Shell_NotifyIconW(NIM_DELETE, &tray_data(window));
    }
}

fn tray_data(window: HWND) -> NOTIFYICONDATAW {
    NOTIFYICONDATAW {
        cbSize: size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: window,
        uID: TRAY_ID,
        ..unsafe { std::mem::zeroed() }
    }
}

fn spawn_terminal(host_window: HWND) -> Result<()> {
    let mut process = TERMINAL_PROCESS
        .get_or_init(|| Mutex::new(None))
        .lock()
        .expect("terminal process mutex was poisoned");

    if let Some(child) = process.as_mut()
        && child
            .try_wait()
            .context("could not inspect the terminal process")?
            .is_none()
    {
        return Ok(());
    }

    TERMINAL_WINDOW.store(0, Ordering::Release);
    let executable = std::env::current_exe().context("could not find the Jarvis executable")?;
    *process = Some(
        Command::new(executable)
            .arg("--terminal")
            .arg((host_window as usize).to_string())
            .spawn()
            .context("could not launch the Jarvis terminal")?,
    );
    Ok(())
}

fn show_terminal(window: HWND) {
    let terminal = TERMINAL_WINDOW.load(Ordering::Acquire) as HWND;
    if !terminal.is_null() && unsafe { IsWindow(terminal) } != 0 {
        unsafe {
            ShowWindow(terminal, SW_RESTORE);
            SetForegroundWindow(terminal);
        }
    } else {
        let _ = spawn_terminal(window);
    }
}

fn show_menu(window: HWND) {
    unsafe {
        let menu = CreatePopupMenu();
        if menu.is_null() {
            return;
        }
        let open = wide("Open Jarvis");
        let exit = wide("Exit");
        AppendMenuW(menu, MF_STRING, MENU_OPEN, open.as_ptr());
        AppendMenuW(menu, MF_STRING, MENU_EXIT, exit.as_ptr());

        let mut point = POINT { x: 0, y: 0 };
        GetCursorPos(&mut point);
        SetForegroundWindow(window);
        let selected = TrackPopupMenu(
            menu,
            TPM_RETURNCMD | TPM_RIGHTBUTTON,
            point.x,
            point.y,
            0,
            window,
            ptr::null(),
        );
        DestroyMenu(menu);

        match selected as usize {
            MENU_OPEN => show_terminal(window),
            MENU_EXIT => {
                DestroyWindow(window);
            }
            _ => {}
        }
    }
}

unsafe extern "system" fn window_proc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_HOTKEY => {
            show_terminal(window);
            0
        }
        WM_TERMINAL_READY => {
            TERMINAL_WINDOW.store(wparam as isize, Ordering::Release);
            0
        }
        WM_TRAY => {
            match lparam as u32 {
                WM_LBUTTONUP | WM_LBUTTONDBLCLK => show_terminal(window),
                WM_RBUTTONUP => show_menu(window),
                _ => {}
            }
            0
        }
        WM_DESTROY => {
            remove_tray_icon(window);
            if let Some(process) = TERMINAL_PROCESS.get()
                && let Ok(mut process) = process.lock()
                && let Some(mut child) = process.take()
            {
                let _ = child.kill();
            }
            unsafe { PostQuitMessage(0) };
            0
        }
        _ => unsafe { DefWindowProcW(window, message, wparam, lparam) },
    }
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}

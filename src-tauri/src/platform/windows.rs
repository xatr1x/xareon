use std::collections::HashMap;
use std::ffi::OsString;
use std::mem::{size_of, zeroed};
use std::os::windows::ffi::OsStringExt;

use windows_sys::Win32::Foundation::{CloseHandle, BOOL, HWND, LPARAM};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
    TH32CS_SNAPPROCESS,
};
use windows_sys::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW,
    GetWindowThreadProcessId, IsIconic, IsWindowVisible,
};

use crate::domain::automatic_tracking::RunningProcess;

#[derive(Debug, Clone)]
pub struct ProcessObservation {
    pub process: RunningProcess,
    pub is_foreground: bool,
    pub is_minimized: bool,
}

struct WindowInfo { title: Option<String>, visible: bool, minimized: bool }

pub fn observe_processes() -> Vec<ProcessObservation> {
    let foreground = unsafe { GetForegroundWindow() };
    let mut foreground_pid = 0;
    if !foreground.is_null() { unsafe { GetWindowThreadProcessId(foreground, &mut foreground_pid) }; }
    let windows = enumerate_windows();
    let mut result = Vec::new();
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot.is_null() || snapshot as isize == -1 { return result; }
    let mut entry: PROCESSENTRY32W = unsafe { zeroed() };
    entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;
    let mut ok = unsafe { Process32FirstW(snapshot, &mut entry) };
    while ok != 0 {
        if let Some(path) = process_path(entry.th32ProcessID) {
            let window = windows.get(&entry.th32ProcessID);
            let name = path.rsplit(['\\', '/']).next().unwrap_or(&path).to_string();
            result.push(ProcessObservation {
                process: RunningProcess {
                    pid: entry.th32ProcessID,
                    executable_path: path,
                    executable_name: name,
                    window_title: window.and_then(|w| w.title.clone()),
                    has_visible_window: window.is_some_and(|w| w.visible),
                },
                is_foreground: entry.th32ProcessID == foreground_pid,
                is_minimized: window.is_some_and(|w| w.minimized),
            });
        }
        ok = unsafe { Process32NextW(snapshot, &mut entry) };
    }
    unsafe { CloseHandle(snapshot) };
    result
}

pub fn last_input_tick() -> u32 {
    let mut info = LASTINPUTINFO { cbSize: size_of::<LASTINPUTINFO>() as u32, dwTime: 0 };
    if unsafe { GetLastInputInfo(&mut info) } != 0 { info.dwTime } else { 0 }
}

fn process_path(pid: u32) -> Option<String> {
    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if handle.is_null() { return None; }
    let mut buffer = vec![0u16; 32_768];
    let mut length = buffer.len() as u32;
    let ok = unsafe { QueryFullProcessImageNameW(handle, 0, buffer.as_mut_ptr(), &mut length) };
    unsafe { CloseHandle(handle) };
    (ok != 0).then(|| OsString::from_wide(&buffer[..length as usize]).to_string_lossy().into_owned())
}

fn enumerate_windows() -> HashMap<u32, WindowInfo> {
    unsafe extern "system" fn callback(hwnd: HWND, data: LPARAM) -> BOOL {
        if IsWindowVisible(hwnd) == 0 { return 1; }
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == 0 { return 1; }
        let length = GetWindowTextLengthW(hwnd);
        let title = if length > 0 {
            let mut buffer = vec![0u16; length as usize + 1];
            let read = GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32);
            (read > 0).then(|| OsString::from_wide(&buffer[..read as usize]).to_string_lossy().into_owned())
        } else { None };
        let map = &mut *(data as *mut HashMap<u32, WindowInfo>);
        map.entry(pid).or_insert(WindowInfo { title, visible: true, minimized: IsIconic(hwnd) != 0 });
        1
    }
    let mut map = HashMap::new();
    unsafe { EnumWindows(Some(callback), &mut map as *mut _ as LPARAM) };
    map
}

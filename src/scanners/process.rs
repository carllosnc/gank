#[cfg(target_os = "windows")]
pub fn is_process_running(target_name: &str) -> bool {
    use std::ffi::c_void;

    const TH32CS_SNAPPROCESS: u32 = 0x00000002;
    const MAX_PATH: usize = 260;

    #[allow(non_snake_case)]
    #[repr(C)]
    struct PROCESSENTRY32W {
        dwSize: u32,
        cntUsage: u32,
        th32ProcessID: u32,
        th32DefaultHeapID: usize,
        th32ModuleID: u32,
        cntThreads: u32,
        th32ParentProcessID: u32,
        pcPriClassBase: i32,
        dwFlags: u32,
        szExeFile: [u16; MAX_PATH],
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn CreateToolhelp32Snapshot(dwFlags: u32, th32ProcessID: u32) -> *mut c_void;
        fn Process32FirstW(hSnapshot: *mut c_void, lppe: *mut PROCESSENTRY32W) -> i32;
        fn Process32NextW(hSnapshot: *mut c_void, lppe: *mut PROCESSENTRY32W) -> i32;
        fn CloseHandle(hObject: *mut c_void) -> i32;
    }

    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot.is_null() || snapshot == -1isize as *mut c_void {
        return false;
    }

    let mut entry = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        cntUsage: 0,
        th32ProcessID: 0,
        th32DefaultHeapID: 0,
        th32ModuleID: 0,
        cntThreads: 0,
        th32ParentProcessID: 0,
        pcPriClassBase: 0,
        dwFlags: 0,
        szExeFile: [0; MAX_PATH],
    };

    let target = target_name.to_lowercase();
    let mut found = false;

    if unsafe { Process32FirstW(snapshot, &mut entry) } != 0 {
        loop {
            let len = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(MAX_PATH);
            let name = String::from_utf16_lossy(&entry.szExeFile[..len]).to_lowercase();
            if name == target {
                found = true;
                break;
            }
            if unsafe { Process32NextW(snapshot, &mut entry) } == 0 {
                break;
            }
        }
    }

    unsafe { CloseHandle(snapshot) };
    found
}

#[cfg(not(target_os = "windows"))]
pub fn is_process_running(_target_name: &str) -> bool {
    false
}

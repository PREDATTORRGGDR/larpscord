//! Обнаружение запущенных процессов Windows (для flavor Discord и,
//! в будущем, авто-переключения профилей).

/// Возвращает имена образов запущенных процессов (без путей).
#[cfg(windows)]
pub fn list_process_names() -> Vec<String> {
    use std::mem::size_of;
    #[repr(C)]
    struct ProcessEntry32 {
        dw_size: u32,
        cnt_usage: u32,
        th32_process_id: u32,
        th32_default_heap_id: usize,
        th32_module_id: u32,
        cnt_threads: u32,
        th32_parent_process_id: u32,
        pc_pri_class_base: i32,
        dw_flags: u32,
        sz_exe_file: [u8; 260],
    }
    extern "system" {
        fn CreateToolhelp32Snapshot(dw_flags: u32, th32_process_id: u32) -> isize;
        fn Process32First(h: isize, lppe: *mut ProcessEntry32) -> i32;
        fn Process32Next(h: isize, lppe: *mut ProcessEntry32) -> i32;
        fn CloseHandle(h: isize) -> i32;
    }
    const TH32CS_SNAPPROCESS: u32 = 0x2;

    let mut names = Vec::new();
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap == -1 {
            return names;
        }
        let mut entry: ProcessEntry32 = std::mem::zeroed();
        entry.dw_size = size_of::<ProcessEntry32>() as u32;
        if Process32First(snap, &mut entry) != 0 {
            loop {
                let len = entry.sz_exe_file.iter().position(|&c| c == 0).unwrap_or(260);
                let name = String::from_utf8_lossy(&entry.sz_exe_file[..len]).to_string();
                names.push(name);
                if Process32Next(snap, &mut entry) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snap);
    }
    names
}

#[cfg(not(windows))]
pub fn list_process_names() -> Vec<String> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(windows)]
    fn lists_processes() {
        let names = super::list_process_names();
        assert!(!names.is_empty());
        assert!(names.iter().any(|n| n.to_lowercase().contains("explorer") || n.to_lowercase().contains("system")));
    }
}

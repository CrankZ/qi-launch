// 已安装程序软件来源（从注册表读取）

use crate::shortcut::get_file_metadata;
use crate::sources::scanner::IconScanner;
use crate::types::*;
use rayon::prelude::*;
use std::error::Error;
use std::path::Path;
use windows::core::PCWSTR;
use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::System::Registry::*;

pub struct InstalledProgramsScanner;

impl IconScanner for InstalledProgramsScanner {
    fn id(&self) -> &str {
        "installed_programs"
    }
    fn name(&self) -> &str {
        "已安装程序"
    }
    fn description(&self) -> &str {
        "从注册表读取的已安装程序"
    }
    fn icon(&self) -> &str {
        "📦"
    }
    fn scan(&self, method: Option<&str>) -> Result<Vec<DesktopIcon>, Box<dyn Error>> {
        get_installed_programs_icons(method)
    }
}

/// 获取已安装程序图标
pub fn get_installed_programs_icons(
    method: Option<&str>,
) -> std::result::Result<Vec<DesktopIcon>, Box<dyn std::error::Error>> {
    let scan_start = std::time::Instant::now();

    let mut programs = Vec::new();

    // 读取 64 位程序
    read_uninstall_keys(
        HKEY_LOCAL_MACHINE,
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        &mut programs,
    )?;

    // 读取 32 位程序（在 64 位系统上）
    read_uninstall_keys(
        HKEY_LOCAL_MACHINE,
        "SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        &mut programs,
    )?;

    // 读取当前用户安装的程序
    read_uninstall_keys(
        HKEY_CURRENT_USER,
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        &mut programs,
    )?;

    let scan_duration = scan_start.elapsed();
    println!(
        "🔍 [扫描阶段] 已安装程序扫描完成, 找到 {} 个注册表项, 耗时: {:.3}s",
        programs.len(),
        scan_duration.as_secs_f64()
    );

    // 并行处理程序信息
    let prepare_start = std::time::Instant::now();
    let results: Vec<_> = programs
        .par_iter()
        .filter_map(|(name, icon_path, exe_path)| {
            match create_icon_from_program(name, icon_path, exe_path, method) {
                Ok(icon) => Some(icon),
                Err(e) => {
                    eprintln!("处理程序 {} 失败: {}", name, e);
                    None
                }
            }
        })
        .collect();
    let prepare_duration = prepare_start.elapsed();
    println!(
        "🧩 [准备阶段] 已安装程序扫描结束, 等待后续统一提取图标, 已准备 {} 个条目, 耗时: {:.3}s",
        results.len(),
        prepare_duration.as_secs_f64()
    );
    Ok(results)
}

/// 读取注册表卸载项
fn read_uninstall_keys(
    hkey: HKEY,
    subkey: &str,
    programs: &mut Vec<(String, String, String)>,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    unsafe {
        let subkey_wide: Vec<u16> = subkey.encode_utf16().chain(std::iter::once(0)).collect();
        let mut key_handle = HKEY::default();

        if RegOpenKeyExW(
            hkey,
            PCWSTR(subkey_wide.as_ptr()),
            Some(0),
            KEY_READ,
            &mut key_handle,
        ) == ERROR_SUCCESS
        {
            let mut index = 0;
            println!("成功打开注册表键: {:?}", subkey);
            loop {
                let mut name_buffer = [0u16; 256];
                let mut name_len = name_buffer.len() as u32;

                let result = RegEnumKeyExW(
                    key_handle,
                    index,
                    Some(windows::core::PWSTR(name_buffer.as_mut_ptr())),
                    &mut name_len,
                    None,
                    None,
                    None,
                    None,
                );

                if result != ERROR_SUCCESS {
                    if result != windows::Win32::Foundation::WIN32_ERROR(259) {
                        // ERROR_NO_MORE_ITEMS
                        println!("枚举注册表子键 {} 失败: {:?}", subkey, result);
                    }
                    break;
                }

                let name = String::from_utf16_lossy(&name_buffer[..name_len as usize]);
                let full_subkey = format!("{}\\{}", subkey, name);

                if let Ok((display_name, icon_path, exe_path)) =
                    read_program_info(hkey, &full_subkey)
                {
                    if !display_name.is_empty() {
                        programs.push((display_name, icon_path, exe_path));
                    }
                }

                index += 1;
            }
            let _ = RegCloseKey(key_handle);
        } else {
            println!("无法打开注册表键: {:?}", subkey);
        }
    }

    Ok(())
}

/// 读取程序信息
fn read_program_info(
    hkey: HKEY,
    subkey: &str,
) -> std::result::Result<(String, String, String), Box<dyn std::error::Error>> {
    unsafe {
        let subkey_wide: Vec<u16> = subkey.encode_utf16().chain(std::iter::once(0)).collect();
        let mut key_handle = HKEY::default();

        if RegOpenKeyExW(
            hkey,
            PCWSTR(subkey_wide.as_ptr()),
            Some(0),
            KEY_READ,
            &mut key_handle,
        ) != ERROR_SUCCESS
        {
            return Err("无法打开注册表键".into());
        }

        let display_name = read_registry_string(key_handle, "DisplayName").unwrap_or_default();
        let icon_path = read_registry_string(key_handle, "DisplayIcon").unwrap_or_default();
        let exe_path = read_registry_string(key_handle, "InstallLocation")
            .or_else(|| read_registry_string(key_handle, "UninstallString"))
            .unwrap_or_default();

        let _ = RegCloseKey(key_handle);

        Ok((display_name, icon_path, exe_path))
    }
}

/// 读取注册表字符串值
fn read_registry_string(key: HKEY, value_name: &str) -> Option<String> {
    unsafe {
        let value_wide: Vec<u16> = value_name
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let mut buffer = [0u16; 512];
        let mut buffer_size = (buffer.len() * 2) as u32;
        let mut value_type = REG_NONE;

        if RegQueryValueExW(
            key,
            PCWSTR(value_wide.as_ptr()),
            None,
            Some(&mut value_type),
            Some(buffer.as_mut_ptr() as *mut u8),
            Some(&mut buffer_size),
        ) == ERROR_SUCCESS
            && value_type == REG_SZ
        {
            let len = buffer_size as usize / 2;
            let result = String::from_utf16_lossy(&buffer[..len.saturating_sub(1)]);
            if !result.is_empty() {
                return Some(result);
            }
        }
    }
    None
}

/// 从程序信息创建图标
fn create_icon_from_program(
    name: &str,
    icon_path: &str,
    exe_path: &str,
    _method: Option<&str>,
) -> std::result::Result<DesktopIcon, Box<dyn std::error::Error>> {
    // 解析图标路径和索引
    let (mut actual_icon_path, icon_index) = parse_icon_path(icon_path, exe_path);

    if actual_icon_path.is_empty() && !name.is_empty() {
        // 如果没有图标路径，但有程序名，我们也保留它
        actual_icon_path = exe_path.to_string();
    }

    if name.is_empty() {
        return Err("程序名为空".into());
    }

    let meta_path = std::path::Path::new(&actual_icon_path);
    let file_meta = get_file_metadata(meta_path);

    Ok(DesktopIcon {
        name: name.to_string(),
        icon_base64: String::new(),
        target_path: actual_icon_path.clone(),
        file_path: actual_icon_path.clone(),
        icon_width: 32,
        icon_height: 32,
        icon_source_path: Some(actual_icon_path),
        icon_source_index: Some(icon_index),
        created_time: file_meta.created_time,
        modified_time: file_meta.modified_time,
        accessed_time: file_meta.accessed_time,
        file_size: file_meta.file_size,
        file_type: file_meta.file_type,
        description: None,
        arguments: None,
        working_directory: None,
        hotkey: None,
        show_command: None,
        source_name: Some("已安装程序".to_string()),
    })
}

/// 解析图标路径（可能包含索引，如 "path.exe,0"）
fn parse_icon_path(icon_path: &str, exe_path: &str) -> (String, i32) {
    if !icon_path.is_empty() {
        // DisplayIcon 可能的格式: "C:\path\file.exe", "C:\path\file.exe,0", "C:\path\icon.ico"
        if let Some(comma_pos) = icon_path.rfind(',') {
            let path_part = icon_path[..comma_pos].trim().trim_matches('"');
            let index_part = icon_path[comma_pos + 1..].trim();
            if let Ok(index) = index_part.parse::<i32>() {
                return (path_part.to_string(), index);
            }
        }
        // 没有索引，直接返回路径
        let cleaned_path = icon_path.trim().trim_matches('"').to_string();
        if Path::new(&cleaned_path).exists() {
            return (cleaned_path, 0);
        }
    }

    // 如果图标路径无效，尝试使用 exe_path
    if !exe_path.is_empty() {
        let cleaned_exe = exe_path.trim().trim_matches('"').to_string();
        // 从 UninstallString 中提取 exe 路径
        if cleaned_exe.to_lowercase().ends_with(".exe") {
            return (cleaned_exe, 0);
        }
        // 可能包含参数，提取第一个 .exe
        if let Some(exe_pos) = cleaned_exe.to_lowercase().find(".exe") {
            let exe_part = &cleaned_exe[..exe_pos + 4];
            return (exe_part.trim_matches('"').to_string(), 0);
        }
    }

    (String::new(), 0)
}

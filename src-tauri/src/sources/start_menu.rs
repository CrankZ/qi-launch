// 开始菜单软件来源

use crate::path::*;
use crate::shortcut::{
    get_file_metadata, get_localized_name, get_shortcut_full_info, hotkey_to_string,
    show_command_to_string,
};
use crate::sources::scanner::IconScanner;
use crate::types::*;
use rayon::prelude::*;
use std::error::Error;
use std::path::Path;

pub struct StartMenuScanner;

impl IconScanner for StartMenuScanner {
    fn id(&self) -> &str {
        "start_menu"
    }
    fn name(&self) -> &str {
        "用户开始菜单"
    }
    fn description(&self) -> &str {
        "当前用户开始菜单中的应用"
    }
    fn icon(&self) -> &str {
        "📋"
    }
    fn scan(&self, method: Option<&str>) -> Result<Vec<DesktopIcon>, Box<dyn Error>> {
        get_start_menu_icons(method)
    }
}

pub struct CommonStartMenuScanner;

impl IconScanner for CommonStartMenuScanner {
    fn id(&self) -> &str {
        "common_start_menu"
    }
    fn name(&self) -> &str {
        "公共开始菜单"
    }
    fn description(&self) -> &str {
        "所有用户共享的开始菜单应用"
    }
    fn icon(&self) -> &str {
        "🗂️"
    }
    fn scan(&self, method: Option<&str>) -> Result<Vec<DesktopIcon>, Box<dyn Error>> {
        get_common_start_menu_icons(method)
    }
}

/// 获取用户开始菜单图标
pub fn get_start_menu_icons(
    method: Option<&str>,
) -> std::result::Result<Vec<DesktopIcon>, Box<dyn std::error::Error>> {
    let programs_path = get_start_menu_programs_path()?;
    scan_folder_recursive(&programs_path, method, "用户开始菜单")
}

/// 获取公共开始菜单图标
pub fn get_common_start_menu_icons(
    method: Option<&str>,
) -> std::result::Result<Vec<DesktopIcon>, Box<dyn std::error::Error>> {
    let programs_path = get_common_start_menu_programs_path()?;
    scan_folder_recursive(&programs_path, method, "公共开始菜单")
}

/// 递归扫描文件夹（开始菜单有子文件夹）
fn scan_folder_recursive(
    folder_path: &Path,
    _method: Option<&str>,
    source_name: &str,
) -> std::result::Result<Vec<DesktopIcon>, Box<dyn std::error::Error>> {
    println!("递归扫描 {} 文件夹: {:?}", source_name, folder_path);

    if !folder_path.exists() {
        println!("{} 路径不存在", source_name);
        return Ok(Vec::new());
    }

    let scan_start = std::time::Instant::now();
    let mut all_files = Vec::new();
    collect_files_recursive(folder_path, &mut all_files)?;
    let scan_duration = scan_start.elapsed();

    println!(
        "🔍 [扫描阶段] {} 递归扫描完成, 找到 {} 个文件, 耗时: {:.3}s",
        source_name,
        all_files.len(),
        scan_duration.as_secs_f64()
    );

    // 并行处理所有文件
    let extract_start = std::time::Instant::now();
    let results: Vec<_> = all_files
        .par_iter()
        .filter_map(|path| match process_item(path, source_name) {
            Ok(icon) => Some(icon),
            Err(e) => {
                eprintln!("{} 处理失败 {:?}: {}", source_name, path, e);
                None
            }
        })
        .collect();
    let extract_duration = extract_start.elapsed();

    println!(
        "🧩 [准备阶段] {} 已准备 {} 个条目, 耗时: {:.3}s",
        source_name,
        results.len(),
        extract_duration.as_secs_f64()
    );
    Ok(results)
}

/// 递归收集所有文件
fn collect_files_recursive(
    dir: &Path,
    files: &mut Vec<std::path::PathBuf>,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                // 递归处理子目录
                collect_files_recursive(&path, files)?;
            } else if path.is_file() {
                // 收集所有文件，不再仅限于快捷方式，统一由前端过滤
                files.push(path);
            }
        }
    }
    Ok(())
}

/// 处理单个文件项
fn process_item(path: &Path, source_name: &str) -> Result<DesktopIcon, Box<dyn std::error::Error>> {
    let file_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();

    let file_path = path.to_string_lossy().to_string();

    // 获取文件元数据
    let file_meta = get_file_metadata(path);

    // 获取快捷方式完整信息
    let shortcut_info = match get_shortcut_full_info(path) {
        Ok(info) => info,
        Err(e) => {
            eprintln!("解析快捷方式失败 {}: {}", file_path, e);
            return Err(e);
        }
    };

    // 使用 Shell API 获取本地化显示名称，如果失败则使用文件名
    let display_name = get_localized_name(path).unwrap_or(file_name);

    Ok(DesktopIcon {
        name: display_name,
        icon_base64: String::new(),
        target_path: shortcut_info.target_path,
        file_path,
        icon_width: 32,
        icon_height: 32,
        icon_source_path: Some(shortcut_info.icon_path),
        icon_source_index: Some(shortcut_info.icon_index),

        // 时间信息
        created_time: file_meta.created_time,
        modified_time: file_meta.modified_time,
        accessed_time: file_meta.accessed_time,

        // 文件信息
        file_size: file_meta.file_size,
        file_type: file_meta.file_type,

        // 快捷方式专属信息
        description: if shortcut_info.description.is_empty() {
            None
        } else {
            Some(shortcut_info.description)
        },
        arguments: if shortcut_info.arguments.is_empty() {
            None
        } else {
            Some(shortcut_info.arguments)
        },
        working_directory: if shortcut_info.working_directory.is_empty() {
            None
        } else {
            Some(shortcut_info.working_directory)
        },
        hotkey: hotkey_to_string(shortcut_info.hotkey),
        show_command: Some(show_command_to_string(shortcut_info.show_command)),
        source_name: Some(source_name.to_string()),
    })
}

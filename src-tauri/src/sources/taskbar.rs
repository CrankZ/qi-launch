// 任务栏固定软件来源

use crate::shortcut::{
    get_file_metadata, get_shortcut_full_info, hotkey_to_string, resolve_shortcut,
    show_command_to_string,
};
use crate::sources::scanner::IconScanner;
use crate::types::*;
use rayon::prelude::*;
use std::env;
use std::error::Error;
use std::path::{Path, PathBuf};

pub struct TaskbarScanner;

impl IconScanner for TaskbarScanner {
    fn id(&self) -> &str {
        "taskbar_pinned"
    }
    fn name(&self) -> &str {
        "任务栏固定项"
    }
    fn description(&self) -> &str {
        "任务栏固定的应用程序"
    }
    fn icon(&self) -> &str {
        "📌"
    }
    fn scan(&self, method: Option<&str>) -> Result<Vec<DesktopIcon>, Box<dyn Error>> {
        get_taskbar_pinned_icons(method)
    }
}

/// 获取任务栏固定的图标
pub fn get_taskbar_pinned_icons(
    method: Option<&str>,
) -> std::result::Result<Vec<DesktopIcon>, Box<dyn std::error::Error>> {
    // 扫描 User Pinned 目录，包含 TaskBar、ImplicitAppShortcuts 等所有子目录
    if let Ok(user_pinned_path) = get_user_pinned_path() {
        return scan_taskbar_folder(&user_pinned_path, method, "任务栏及常用项");
    }

    // 如果目录不存在，返回空列表
    Ok(Vec::new())
}

/// 获取 User Pinned 路径
fn get_user_pinned_path() -> std::result::Result<PathBuf, Box<dyn std::error::Error>> {
    let appdata = env::var("APPDATA").map_err(|_| "无法获取 APPDATA 环境变量")?;

    // User Pinned 目录包含 TaskBar、ImplicitAppShortcuts、StartMenu 等子目录
    let user_pinned = PathBuf::from(appdata)
        .join("Microsoft")
        .join("Internet Explorer")
        .join("Quick Launch")
        .join("User Pinned");

    if user_pinned.exists() {
        Ok(user_pinned)
    } else {
        Err("User Pinned 路径不存在".into())
    }
}

/// 扫描任务栏固定文件夹
fn scan_taskbar_folder(
    folder_path: &Path,
    _method: Option<&str>,
    source_name: &str,
) -> std::result::Result<Vec<DesktopIcon>, Box<dyn std::error::Error>> {
    println!("扫描 {} 文件夹: {:?}", source_name, folder_path);

    if !folder_path.exists() {
        println!("{} 路径不存在", source_name);
        return Ok(Vec::new());
    }

    let mut icons = Vec::new();
    let mut all_files = Vec::new();

    let scan_start = std::time::Instant::now();
    // 递归收集所有 .lnk 文件（任务栏文件夹可能有子目录）
    collect_lnk_files_recursive(folder_path, &mut all_files)?;
    let scan_duration = scan_start.elapsed();

    println!(
        "🔍 [扫描阶段] {} 递归扫描完成, 找到 {} 个文件, 耗时: {:.3}s",
        source_name,
        all_files.len(),
        scan_duration.as_secs_f64()
    );

    // 并行处理所有文件
    let prepare_start = std::time::Instant::now();
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
    let prepare_duration = prepare_start.elapsed();
    println!(
        "🧩 [准备阶段] {} 扫描结束, 等待后续统一提取图标, 已准备 {} 个条目, 耗时: {:.3}s",
        source_name,
        results.len(),
        prepare_duration.as_secs_f64()
    );

    icons.extend(results);
    Ok(icons)
}

/// 递归收集 .lnk 文件
fn collect_lnk_files_recursive(
    dir: &Path,
    files: &mut Vec<PathBuf>,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                // 递归处理子目录
                collect_lnk_files_recursive(&path, files)?;
            } else if path.is_file() {
                // 收集所有文件，统一由前端过滤
                files.push(path);
            }
        }
    }
    Ok(())
}

/// 处理单个快捷方式
fn process_item(
    path: &Path,
    source_name: &str,
) -> std::result::Result<DesktopIcon, Box<dyn std::error::Error>> {
    let file_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();

    let file_path = path.to_string_lossy().to_string();
    let file_meta = get_file_metadata(path);

    let shortcut_info = match get_shortcut_full_info(path) {
        Ok(info) => info,
        Err(e) => {
            eprintln!("解析快捷方式失败 {}: {}", file_path, e);
            let (target, icon, idx) =
                resolve_shortcut(path).unwrap_or((file_path.clone(), file_path.clone(), 0));
            return Ok(DesktopIcon {
                name: file_name,
                icon_base64: String::new(),
                target_path: target,
                file_path,
                icon_width: 32,
                icon_height: 32,
                icon_source_path: Some(icon),
                icon_source_index: Some(idx),
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
                source_name: Some(source_name.to_string()),
            });
        }
    };

    Ok(DesktopIcon {
        name: file_name,
        icon_base64: String::new(),
        target_path: shortcut_info.target_path,
        file_path,
        icon_width: 32,
        icon_height: 32,
        icon_source_path: Some(shortcut_info.icon_path),
        icon_source_index: Some(shortcut_info.icon_index),
        created_time: file_meta.created_time,
        modified_time: file_meta.modified_time,
        accessed_time: file_meta.accessed_time,
        file_size: file_meta.file_size,
        file_type: file_meta.file_type,
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

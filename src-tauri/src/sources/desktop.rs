// 桌面软件来源

use crate::path::*;
use crate::shortcut::{
    get_file_metadata, get_shortcut_full_info, hotkey_to_string, resolve_shortcut,
    show_command_to_string,
};
use crate::sources::scanner::IconScanner;
use crate::types::*;
use rayon::prelude::*;
use std::error::Error;
use std::path::Path;

pub struct DesktopScanner;

impl IconScanner for DesktopScanner {
    fn id(&self) -> &str {
        "desktop"
    }
    fn name(&self) -> &str {
        "用户桌面"
    }
    fn description(&self) -> &str {
        "当前用户桌面上的应用"
    }
    fn icon(&self) -> &str {
        "🖥️"
    }
    fn scan(&self, method: Option<&str>) -> Result<Vec<DesktopIcon>, Box<dyn Error>> {
        get_desktop_icons(method)
    }
}

pub struct PublicDesktopScanner;

impl IconScanner for PublicDesktopScanner {
    fn id(&self) -> &str {
        "public_desktop"
    }
    fn name(&self) -> &str {
        "公共桌面"
    }
    fn description(&self) -> &str {
        "所有用户共享的桌面应用"
    }
    fn icon(&self) -> &str {
        "💼"
    }
    fn scan(&self, method: Option<&str>) -> Result<Vec<DesktopIcon>, Box<dyn Error>> {
        get_public_desktop_icons(method)
    }
}

/// 获取用户桌面图标
pub fn get_desktop_icons(
    method: Option<&str>,
) -> std::result::Result<Vec<DesktopIcon>, Box<dyn std::error::Error>> {
    let desktop_path = get_desktop_path()?;
    scan_folder(&desktop_path, method, "用户桌面")
}

/// 获取公共桌面图标
pub fn get_public_desktop_icons(
    method: Option<&str>,
) -> std::result::Result<Vec<DesktopIcon>, Box<dyn std::error::Error>> {
    let desktop_path = get_public_desktop_path()?;
    scan_folder(&desktop_path, method, "公共桌面")
}

/// 扫描文件夹获取图标
fn scan_folder(
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

    if let Ok(entries) = std::fs::read_dir(folder_path) {
        let scan_start = std::time::Instant::now();
        let file_paths: Vec<_> = entries
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_file())
            .map(|entry| entry.path())
            .collect();
        let scan_duration = scan_start.elapsed();

        println!(
            "🔍 [扫描阶段] {} 扫描完成, 找到 {} 个文件, 耗时: {:.3}s",
            source_name,
            file_paths.len(),
            scan_duration.as_secs_f64()
        );

        // 并行处理文件
        let extract_start = std::time::Instant::now();
        let results: Vec<_> = file_paths
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

        icons.extend(results);
    }

    Ok(icons)
}

/// 处理单个文件项
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
    let is_lnk = path.extension().and_then(|s| s.to_str()) == Some("lnk");

    if is_lnk {
        let shortcut_info = match get_shortcut_full_info(path) {
            Ok(info) => info,
            Err(e) => {
                eprintln!("{} 解析快捷方式失败 {}: {}", source_name, file_path, e);
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

        return Ok(DesktopIcon {
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
        });
    }

    Ok(DesktopIcon {
        name: file_name,
        icon_base64: String::new(),
        target_path: file_path.clone(),
        file_path: file_path.clone(),
        icon_width: 32,
        icon_height: 32,
        icon_source_path: Some(file_path),
        icon_source_index: Some(0),
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
    })
}

// 快速启动软件来源

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

pub struct QuickLaunchScanner;

impl IconScanner for QuickLaunchScanner {
    fn id(&self) -> &str {
        "quick_launch"
    }
    fn name(&self) -> &str {
        "快速启动"
    }
    fn description(&self) -> &str {
        "快速启动栏中的应用"
    }
    fn icon(&self) -> &str {
        "⚡"
    }
    fn scan(&self, method: Option<&str>) -> Result<Vec<DesktopIcon>, Box<dyn Error>> {
        get_quick_launch_icons(method)
    }
}

/// 获取快速启动栏图标
pub fn get_quick_launch_icons(
    method: Option<&str>,
) -> std::result::Result<Vec<DesktopIcon>, Box<dyn std::error::Error>> {
    let quick_launch_path = get_quick_launch_path()?;
    scan_quick_launch_folder(&quick_launch_path, method)
}

/// 获取快速启动路径
fn get_quick_launch_path() -> std::result::Result<PathBuf, Box<dyn std::error::Error>> {
    // Windows 快速启动路径
    let appdata = env::var("APPDATA").map_err(|_| "无法获取 APPDATA 环境变量")?;

    let quick_launch = PathBuf::from(appdata)
        .join("Microsoft")
        .join("Internet Explorer")
        .join("Quick Launch");

    if quick_launch.exists() {
        Ok(quick_launch)
    } else {
        Err("快速启动路径不存在".into())
    }
}

/// 扫描快速启动文件夹
fn scan_quick_launch_folder(
    folder_path: &Path,
    _method: Option<&str>,
) -> std::result::Result<Vec<DesktopIcon>, Box<dyn std::error::Error>> {
    println!("扫描快速启动文件夹: {:?}", folder_path);

    if !folder_path.exists() {
        println!("快速启动路径不存在");
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

        let source_name = "快速启动";
        println!(
            "🔍 [扫描阶段] {} 扫描完成, 找到 {} 个文件, 耗时: {:.3}s",
            source_name,
            file_paths.len(),
            scan_duration.as_secs_f64()
        );

        // 并行处理文件
        let prepare_start = std::time::Instant::now();
        let results: Vec<_> = file_paths
            .par_iter()
            .filter_map(|path| match process_item(path) {
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
    }

    Ok(icons)
}

/// 处理单个快捷方式
fn process_item(path: &Path) -> std::result::Result<DesktopIcon, Box<dyn std::error::Error>> {
    let file_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();

    let file_path = path.to_string_lossy().to_string();
    let file_meta = get_file_metadata(path);

    let shortcut_info = match get_shortcut_full_info(path) {
        Ok(info) => {
            // 过滤掉目标为 URL 的快捷方式
            if is_url(&info.target_path) {
                return Err("目标是 URL，已过滤".into());
            }
            info
        }
        Err(e) => {
            eprintln!("解析快捷方式失败 {}: {}", file_path, e);
            let (target, icon, idx) =
                resolve_shortcut(path).unwrap_or((file_path.clone(), file_path.clone(), 0));

            // 过滤掉目标为 URL 的快捷方式
            if is_url(&target) {
                return Err("目标是 URL，已过滤".into());
            }
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
                source_name: Some("快速启动".to_string()),
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
        source_name: Some("快速启动".to_string()),
    })
}

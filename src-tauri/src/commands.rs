// Tauri 命令模块

use crate::path::normalize_path;
#[cfg(target_os = "windows")]
use crate::sources::desktop::{
    get_desktop_icons as get_user_desktop_icons, get_public_desktop_icons,
};
use crate::sources::{
    fill_icons, get_all_icons, get_all_scanners, get_icons_from_source as get_source_icons,
    IconSource,
};
use crate::types::DesktopIcon;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg(target_os = "windows")]
#[tauri::command]
pub async fn get_desktop_icons() -> std::result::Result<Vec<DesktopIcon>, String> {
    println!("收到前端调用 get_desktop_icons 命令");

    let handle = tokio::task::spawn_blocking(move || -> Vec<DesktopIcon> {
        let mut all_icons = Vec::new();
        let mut seen_targets = std::collections::HashSet::new();

        // 获取用户桌面图标
        if let Ok(icons) = get_user_desktop_icons(None) {
            for icon in icons {
                let name = icon.name.trim().to_lowercase();
                let target = if cfg!(target_os = "windows") {
                    normalize_path(&icon.target_path).to_lowercase()
                } else {
                    icon.target_path.to_lowercase()
                };
                let fingerprint = format!("{}:{}", name, target);
                if seen_targets.insert(fingerprint) {
                    all_icons.push(icon);
                }
            }
        }

        // 获取公共桌面图标
        if let Ok(icons) = get_public_desktop_icons(None) {
            for icon in icons {
                let name = icon.name.trim().to_lowercase();
                let target = if cfg!(target_os = "windows") {
                    normalize_path(&icon.target_path).to_lowercase()
                } else {
                    icon.target_path.to_lowercase()
                };
                let fingerprint = format!("{}:{}", name, target);
                if seen_targets.insert(fingerprint) {
                    all_icons.push(icon);
                }
            }
        }
        fill_icons(&mut all_icons, None);
        all_icons
    });

    let all_icons: Vec<DesktopIcon> = handle.await.map_err(|e| e.to_string())?;
    println!("成功返回 {} 个图标给前端", all_icons.len());
    Ok(all_icons)
}

#[cfg(target_os = "windows")]
#[tauri::command]
pub async fn get_desktop_icons_with_method(
    method: String,
) -> std::result::Result<Vec<DesktopIcon>, String> {
    println!(
        "收到前端调用 get_desktop_icons_with_method 命令，方法: {}",
        method
    );

    let method_clone = method.clone();
    let handle = tokio::task::spawn_blocking(move || -> Vec<DesktopIcon> {
        let mut all_icons = Vec::new();
        let mut seen_targets = std::collections::HashSet::new();

        // 获取用户桌面图标
        if let Ok(icons) = get_user_desktop_icons(None) {
            for icon in icons {
                let name = icon.name.trim().to_lowercase();
                let target = if cfg!(target_os = "windows") {
                    normalize_path(&icon.target_path).to_lowercase()
                } else {
                    icon.target_path.to_lowercase()
                };
                let fingerprint = format!("{}:{}", name, target);
                if seen_targets.insert(fingerprint) {
                    all_icons.push(icon);
                }
            }
        }

        // 获取公共桌面图标
        if let Ok(icons) = get_public_desktop_icons(None) {
            for icon in icons {
                let name = icon.name.trim().to_lowercase();
                let target = if cfg!(target_os = "windows") {
                    normalize_path(&icon.target_path).to_lowercase()
                } else {
                    icon.target_path.to_lowercase()
                };
                let fingerprint = format!("{}:{}", name, target);
                if seen_targets.insert(fingerprint) {
                    all_icons.push(icon);
                }
            }
        }
        fill_icons(&mut all_icons, Some(&method));
        all_icons
    });

    let all_icons: Vec<DesktopIcon> = handle.await.map_err(|e| e.to_string())?;
    println!(
        "使用 {} 方式成功返回 {} 个图标给前端",
        method_clone,
        all_icons.len()
    );
    Ok(all_icons)
}

/// 获取所有可用的图标提取方法列表（Windows）
#[cfg(target_os = "windows")]
#[tauri::command]
pub fn get_available_icon_methods() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
          "id": "smart",
          "name": "智能方式",
          "description": "先用系统 ImageList 获取，失败再用 PrivateExtractIcons；不再尝试其它方式",
          "maxSize": 512
        }),
        serde_json::json!({
          "id": "high_res",
          "name": "PrivateExtractIcons 提取",
          "description": "使用 PrivateExtractIconsW 接口提取图标 (快，最高 512px)",
          "maxSize": 512
        }),
        serde_json::json!({
          "id": "imagelist",
          "name": "系统 ImageList (JUMBO)",
          "description": "使用 Windows 系统列表 JUMBO 接口 (最快，固定 256px)",
          "maxSize": 256
        }),
        serde_json::json!({
          "id": "shell",
          "name": "Shell 文件关联接口",
          "description": "通过 SHGetFileInfo 获取文件关联图标 (极快，最高 256px)",
          "maxSize": 256
        }),
        serde_json::json!({
          "id": "pe_resource",
          "name": "PE 图标组智能提取",
          "description": "解析 PE 资源中的图标组并选择最佳尺寸 (较快，支持 1024px)",
          "maxSize": 1024
        }),
        serde_json::json!({
          "id": "thumbnail",
          "name": "资源管理器缩略图",
          "description": "使用 IShellItemImageFactory 获取缩略图 (慢，支持 1024px)",
          "maxSize": 1024
        }),
    ]
}

/// 获取所有可用的图标提取方法列表（macOS）
#[cfg(target_os = "macos")]
#[tauri::command]
pub fn get_available_icon_methods() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
          "id": "native",
          "name": "原生 API",
          "description": "使用 macOS 原生接口提取，支持圆角和系统效果"
        }),
        serde_json::json!({
          "id": "icns",
          "name": "ICNS 提取",
          "description": "直接从应用包内的 .icns 文件提取原始图标"
        }),
    ]
}

// ========== 软件来源命令 ==========

/// 从指定来源获取图标
#[tauri::command]
pub async fn get_icons_from_source(
    source: String,
    method: Option<String>,
) -> std::result::Result<Vec<DesktopIcon>, String> {
    println!(
        "[Backend] 收到 get_icons_from_source 命令, 来源: {}, 方式: {:?}",
        source, method
    );
    let start = std::time::Instant::now();

    let source_clone = source.clone();
    let method_clone = method.clone();
    let handle =
        tokio::task::spawn_blocking(move || -> std::result::Result<Vec<DesktopIcon>, String> {
            let icon_source = IconSource::from_str(&source)
                .ok_or_else(|| format!("无效的软件来源: {}", source))?;

            let mut icons =
                get_source_icons(icon_source, method.as_deref()).map_err(|e| e.to_string())?;
            fill_icons(&mut icons, method.as_deref());
            Ok(icons)
        });

    let result: std::result::Result<Vec<DesktopIcon>, String> =
        handle.await.map_err(|e| e.to_string())?;

    let duration = start.elapsed();
    match &result {
        Ok(icons) => println!(
            "[Backend] 从 {} 成功返回 {} 个图标, 方式: {:?}, 耗时: {:.3}s",
            source_clone,
            icons.len(),
            method_clone,
            duration.as_secs_f64()
        ),
        Err(e) => eprintln!(
            "[Backend] 从 {} 返回错误: {}, 方式: {:?}, 耗时: {:.3}s",
            source_clone,
            e,
            method_clone,
            duration.as_secs_f64()
        ),
    }
    result
}

/// 获取所有来源的图标（去重汇总）
#[tauri::command]
pub async fn get_all_source_icons(
    method: Option<String>,
) -> std::result::Result<Vec<DesktopIcon>, String> {
    println!(
        "[Backend] 收到 get_all_source_icons 命令, 方式: {:?}",
        method
    );
    let start = std::time::Instant::now();

    let method_clone = method.clone();
    let handle =
        tokio::task::spawn_blocking(move || -> std::result::Result<Vec<DesktopIcon>, String> {
            get_all_icons(method.as_deref()).map_err(|e| e.to_string())
        });

    let result: std::result::Result<Vec<DesktopIcon>, String> =
        handle.await.map_err(|e| e.to_string())?;

    let duration = start.elapsed();
    match &result {
        Ok(icons) => println!(
            "[Backend] 汇总所有来源成功返回 {} 个图标, 方式: {:?}, 耗时: {:.3}s",
            icons.len(),
            method_clone,
            duration.as_secs_f64()
        ),
        Err(e) => eprintln!(
            "[Backend] 汇总所有来源返回错误: {}, 方式: {:?}, 耗时: {:.3}s",
            e,
            method_clone,
            duration.as_secs_f64()
        ),
    }
    result
}

/// 从多个来源获取图标（支持多选）
#[tauri::command]
pub async fn get_icons_from_multiple_sources(
    sources: Vec<String>,
    method: Option<String>,
) -> std::result::Result<Vec<DesktopIcon>, String> {
    println!(
        "[Backend] 收到 get_icons_from_multiple_sources 命令, 来源: {:?}, 方式: {:?}",
        sources, method
    );
    let start = std::time::Instant::now();

    let _sources_clone = sources.clone();
    let method_clone = method.clone();
    let handle =
        tokio::task::spawn_blocking(move || -> std::result::Result<Vec<DesktopIcon>, String> {
            // 定义来源优先级函数
            let get_priority = |source_id: &str| -> i32 {
                match source_id {
                    "uwp_apps" => 100,
                    "taskbar_pinned" => 90,
                    "start_menu" => 85,
                    "common_start_menu" => 80,
                    "desktop" => 75,
                    "public_desktop" => 70,
                    "quick_launch" => 65,
                    "appdata_programs" => 60,
                    "installed_programs" => 50,
                    "program_files" => 40,
                    "program_files_x86" => 35,
                    "applications" => 90,        // macOS
                    "system_applications" => 80, // macOS
                    "user_applications" => 85,   // macOS
                    _ => 50,
                }
            };

            // 如果在 macOS 下 sources 全全是 Windows 来源，则执行全量扫描
            let actual_sources = sources.clone();
            #[cfg(target_os = "macos")]
            {
                let has_mac_source = actual_sources.iter().any(|s| {
                    s == "applications"
                        || s == "system_applications"
                        || s == "user_applications"
                        || s == "spotlight"
                        || s == "core_services"
                });

                if !has_mac_source {
                    println!("[Backend] 检测到 macOS 环境但来源不匹配，自动执行全量扫描");
                    return get_all_icons(method.as_deref()).map_err(|e| e.to_string());
                }
            }

            // 第一步：汇总来源（去重）
            let unique_sources: std::collections::HashSet<&str> =
                actual_sources.iter().map(|s| s.as_str()).collect();
            let mut unique_sources: Vec<&str> = unique_sources.into_iter().collect();
            unique_sources.sort_unstable();

            // 第二步：提取所有图标到列表
            use rayon::prelude::*;
            let all_icons: Vec<(DesktopIcon, i32, &str)> = unique_sources
                .par_iter()
                .flat_map_iter(|&source_str| {
                    let icon_source = match IconSource::from_str(source_str) {
                        Some(s) => s,
                        None => {
                            eprintln!("[Backend] 无效的软件来源: {}", source_str);
                            return Vec::new();
                        }
                    };

                    let priority = get_priority(source_str);

                    match get_source_icons(icon_source, method.as_deref()) {
                        Ok(icons) => icons
                            .into_iter()
                            .map(|icon| (icon, priority, source_str))
                            .collect(),
                        Err(e) => {
                            eprintln!("[Backend] 获取来源 {} 失败: {}", source_str, e);
                            Vec::new()
                        }
                    }
                })
                .collect();

            // 第三步：去重并合并
            let mut non_uwp_map: std::collections::HashMap<String, (DesktopIcon, i32)> =
                std::collections::HashMap::new();
            let mut uwp_map: std::collections::HashMap<String, (DesktopIcon, i32)> =
                std::collections::HashMap::new();

            for (icon, priority, _source_str) in all_icons {
                let name = icon.name.trim().to_lowercase();
                let target = if cfg!(target_os = "windows") {
                    normalize_path(&icon.target_path).to_lowercase()
                } else {
                    icon.target_path.to_lowercase()
                };
                let is_uwp = icon.file_type.as_deref() == Some("UWP App");
                let fingerprint = format!("{}:{}", name, target);

                if is_uwp {
                    match uwp_map.entry(fingerprint) {
                        std::collections::hash_map::Entry::Vacant(entry) => {
                            entry.insert((icon, priority));
                        }
                        std::collections::hash_map::Entry::Occupied(mut entry) => {
                            let (existing_icon, existing_priority) = entry.get_mut();
                            let should_replace = if priority > *existing_priority {
                                true
                            } else if priority == *existing_priority {
                                existing_icon.icon_base64.is_empty() && !icon.icon_base64.is_empty()
                            } else {
                                false
                            };
                            if should_replace {
                                *existing_icon = icon;
                                *existing_priority = priority;
                            }
                        }
                    }
                } else {
                    match non_uwp_map.entry(fingerprint) {
                        std::collections::hash_map::Entry::Vacant(entry) => {
                            entry.insert((icon, priority));
                        }
                        std::collections::hash_map::Entry::Occupied(mut entry) => {
                            let (existing_icon, existing_priority) = entry.get_mut();
                            let should_replace = if priority > *existing_priority {
                                true
                            } else if priority == *existing_priority {
                                existing_icon.icon_base64.is_empty() && !icon.icon_base64.is_empty()
                            } else {
                                false
                            };
                            if should_replace {
                                *existing_icon = icon;
                                *existing_priority = priority;
                            }
                        }
                    }
                }
            }

            // 跨类型去重：移除UWP中与非UWP同名的
            let non_uwp_names: std::collections::HashSet<String> = non_uwp_map
                .keys()
                .map(|s| s.split(':').next().unwrap_or("").to_string())
                .collect();

            // 合并结果
            let mut result_icons: Vec<DesktopIcon> =
                non_uwp_map.into_iter().map(|(_, (icon, _))| icon).collect();

            for (fingerprint, (icon, _)) in uwp_map {
                let name = fingerprint.split(':').next().unwrap_or("");
                if !non_uwp_names.contains(name) {
                    result_icons.push(icon);
                }
            }

            // 按名称排序
            result_icons.sort_by(|a, b| a.name.cmp(&b.name));

            fill_icons(&mut result_icons, method.as_deref());
            Ok(result_icons)
        });

    let result: std::result::Result<Vec<DesktopIcon>, String> =
        handle.await.map_err(|e| e.to_string())?;

    let duration = start.elapsed();
    match &result {
        Ok(icons) => println!(
            "[Backend] 从多个来源成功返回 {} 个图标, 方式: {:?}, 耗时: {:.3}s",
            icons.len(),
            method_clone,
            duration.as_secs_f64()
        ),
        Err(e) => eprintln!(
            "[Backend] 从多个来源返回错误: {}, 方式: {:?}, 耗时: {:.3}s",
            e,
            method_clone,
            duration.as_secs_f64()
        ),
    }
    result
}

/// 获取可用的软件来源列表（Windows）
#[cfg(target_os = "windows")]
#[tauri::command]
pub fn get_available_sources() -> Vec<serde_json::Value> {
    let mut sources = vec![];

    let scanners = get_all_scanners();
    for scanner in scanners {
        sources.push(serde_json::json!({
          "id": scanner.id(),
          "name": scanner.name(),
          "description": scanner.description(),
          "icon": scanner.icon()
        }));
    }

    sources
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn reveal_file(path: String) -> Result<(), String> {
    println!("[Backend] 收到 reveal_file 命令, 路径: {}", path);

    let contents_path = if path.ends_with(".app") {
        format!("{}/Contents", path)
    } else {
        path
    };

    std::process::Command::new("open")
        .arg(&contents_path)
        .output()
        .map_err(|e| format!("执行 open 命令失败: {}", e))?;

    Ok(())
}

/// 获取可用的软件来源列表（macOS）
#[cfg(target_os = "macos")]
#[tauri::command]
pub fn get_available_sources() -> Vec<serde_json::Value> {
    let mut sources = vec![];

    let scanners = get_all_scanners();
    for scanner in scanners {
        sources.push(serde_json::json!({
          "id": scanner.id(),
          "name": scanner.name(),
          "description": scanner.description(),
          "icon": scanner.icon()
        }));
    }

    sources
}

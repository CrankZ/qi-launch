// 软件来源模块

pub mod scanner;

// Windows 专用来源
#[cfg(target_os = "windows")]
pub mod appdata;
#[cfg(target_os = "windows")]
pub mod desktop;
#[cfg(target_os = "windows")]
pub mod installed_programs;
#[cfg(target_os = "windows")]
pub mod program_files;
#[cfg(target_os = "windows")]
pub mod quick_launch;
#[cfg(target_os = "windows")]
pub mod start_menu;
#[cfg(target_os = "windows")]
pub mod taskbar;
#[cfg(target_os = "windows")]
pub mod uwp;

// macOS 专用来源
#[cfg(target_os = "macos")]
pub mod macos;

use crate::path::normalize_path;
use crate::sources::scanner::IconScanner;
use crate::types::{DesktopIcon, IconData};
use rayon::prelude::*;

#[cfg(target_os = "windows")]
pub fn fill_icons(icons: &mut [DesktopIcon], method: Option<&str>) {
    use crate::extractor::{extract_icon_as_base64, extract_icon_with_method};
    use crate::extractors::shell_icon::{extract_icon_via_shell, extract_thumbnail_icon};
    use rayon::prelude::*;

    const KEY_SEP: char = '\u{1f}';

    let start = std::time::Instant::now();
    let total = icons.len();
    let already_has_icon = icons.iter().filter(|i| !i.icon_base64.is_empty()).count();

    #[derive(Clone)]
    struct ExtractPlan {
        kind: u8,
        path: String,
        icon_index: i32,
    }

    let clean_path = |p: &str| -> String { p.trim().trim_matches('"').to_string() };
    let normalize_key_path = |p: &str| -> String {
        let p = clean_path(p);
        let p = normalize_path(&p);
        let p = p.replace('/', "\\");
        let p = p.strip_prefix(r"\\?\").unwrap_or(&p).to_string();
        p.to_lowercase()
    };

    let mut plans = std::collections::HashMap::<String, ExtractPlan>::new();
    let mut per_source: std::collections::BTreeMap<
        String,
        (usize, usize, std::collections::HashSet<String>),
    > = std::collections::BTreeMap::new();
    let mut need_extract = 0usize;
    for icon in icons.iter() {
        let source = icon
            .source_name
            .clone()
            .unwrap_or_else(|| "未知来源".to_string());
        let entry = per_source
            .entry(source)
            .or_insert_with(|| (0, 0, std::collections::HashSet::new()));
        entry.0 += 1;

        if !icon.icon_base64.is_empty() {
            continue;
        }
        entry.1 += 1;
        need_extract += 1;
        let (key, plan) = if icon.file_type.as_deref() == Some("UWP App") {
            let shell_path_raw = icon.icon_source_path.as_deref().unwrap_or(&icon.file_path);
            let plan_path = normalize_path(&clean_path(shell_path_raw));
            let key_path = normalize_key_path(shell_path_raw);
            (
                format!("uwp{KEY_SEP}{key_path}"),
                ExtractPlan {
                    kind: 1,
                    path: plan_path,
                    icon_index: 0,
                },
            )
        } else {
            let source_path_raw = icon.icon_source_path.as_deref().unwrap_or(&icon.file_path);
            let icon_index = icon.icon_source_index.unwrap_or(0);
            let plan_path = normalize_path(&clean_path(source_path_raw));
            let key_path = normalize_key_path(source_path_raw);
            (
                format!("file{KEY_SEP}{icon_index}{KEY_SEP}{key_path}"),
                ExtractPlan {
                    kind: 0,
                    path: plan_path,
                    icon_index,
                },
            )
        };
        entry.2.insert(key.clone());
        plans.entry(key).or_insert(plan);
    }

    let unique_count = plans.len();
    for (source, (total_count, need_count, keys)) in per_source {
        println!(
            "🖼️ [提取阶段] {} 已准备条目: {}, 需提取: {}, 唯一图标源(本来源): {}",
            source,
            total_count,
            need_count,
            keys.len()
        );
    }

    println!(
        "🖼️ [提取阶段] 去重后统一提取准备就绪, 总条目: {}, 需提取: {}, 唯一图标源: {}",
        total, need_extract, unique_count
    );

    let plans: Vec<(String, ExtractPlan)> = plans.into_iter().collect();
    let extracted: Vec<(String, IconData)> = plans
        .par_iter()
        .map(|(key, plan)| {
            let icon_data = if plan.kind == 1 {
                let shell_path = plan.path.as_str();
                let mut data = extract_thumbnail_icon(shell_path, 256).unwrap_or(IconData {
                    base64: String::new(),
                    width: 32,
                    height: 32,
                });
                if data.base64.is_empty() || data.width < 48 {
                    if let Ok(better) = extract_icon_via_shell(shell_path, 256) {
                        if !better.base64.is_empty() {
                            data = better;
                        }
                    }
                }
                data
            } else {
                let icon_index = plan.icon_index;
                let source_path = plan.path.as_str();
                if let Some(m) = method {
                    extract_icon_with_method(source_path, icon_index, m).unwrap_or(IconData {
                        base64: String::new(),
                        width: 32,
                        height: 32,
                    })
                } else {
                    extract_icon_as_base64(source_path, icon_index).unwrap_or(IconData {
                        base64: String::new(),
                        width: 32,
                        height: 32,
                    })
                }
            };
            (key.clone(), icon_data)
        })
        .collect();

    let mut extracted_map = std::collections::HashMap::<String, IconData>::new();
    extracted_map.reserve(extracted.len());
    for (key, data) in extracted {
        extracted_map.insert(key, data);
    }

    for icon in icons.iter_mut() {
        if !icon.icon_base64.is_empty() {
            continue;
        }
        let key = if icon.file_type.as_deref() == Some("UWP App") {
            let shell_path_raw = icon.icon_source_path.as_deref().unwrap_or(&icon.file_path);
            let key_path = normalize_key_path(shell_path_raw);
            format!("uwp{KEY_SEP}{key_path}")
        } else {
            let source_path_raw = icon.icon_source_path.as_deref().unwrap_or(&icon.file_path);
            let icon_index = icon.icon_source_index.unwrap_or(0);
            let key_path = normalize_key_path(source_path_raw);
            format!("file{KEY_SEP}{icon_index}{KEY_SEP}{key_path}")
        };
        if let Some(icon_data) = extracted_map.get(&key) {
            icon.icon_base64 = icon_data.base64.clone();
            icon.icon_width = icon_data.width;
            icon.icon_height = icon_data.height;
        }
    }

    let duration = start.elapsed();
    let filled = icons.iter().filter(|i| !i.icon_base64.is_empty()).count();
    let newly_filled = filled.saturating_sub(already_has_icon);

    println!(
        "🖼️ [提取阶段] 统一图标提取完成, 总计: {}, 已有: {}, 本次新增: {}, 最终有图标: {}, 耗时: {:.3}s",
        total,
        already_has_icon,
        newly_filled,
        filled,
        duration.as_secs_f64()
    );
}

#[cfg(target_os = "macos")]
pub fn fill_icons(icons: &mut [DesktopIcon], method: Option<&str>) {
    use crate::sources::macos::extract_icon_for_app;
    use rayon::prelude::*;
    use std::path::Path;

    const KEY_SEP: char = '\u{1f}';

    let start = std::time::Instant::now();
    let total = icons.len();
    let already_has_icon = icons.iter().filter(|i| !i.icon_base64.is_empty()).count();

    let mut unique_keys = std::collections::HashSet::<String>::new();
    let mut per_source: std::collections::BTreeMap<
        String,
        (usize, usize, std::collections::HashSet<String>),
    > = std::collections::BTreeMap::new();
    let mut need_extract = 0usize;
    for icon in icons.iter() {
        let source = icon
            .source_name
            .clone()
            .unwrap_or_else(|| "未知来源".to_string());
        let entry = per_source
            .entry(source)
            .or_insert_with(|| (0, 0, std::collections::HashSet::new()));
        entry.0 += 1;

        if !icon.icon_base64.is_empty() {
            continue;
        }
        entry.1 += 1;
        need_extract += 1;
        let app_path = icon.icon_source_path.as_deref().unwrap_or(&icon.file_path);
        let key = format!("app{KEY_SEP}{app_path}");
        entry.2.insert(key.clone());
        unique_keys.insert(key);
    }

    let unique_count = unique_keys.len();
    for (source, (total_count, need_count, keys)) in per_source {
        println!(
            "🖼️ [提取阶段] {} 已准备条目: {}, 需提取: {}, 唯一图标源(本来源): {}",
            source,
            total_count,
            need_count,
            keys.len()
        );
    }
    println!(
        "🖼️ [提取阶段] 去重后统一提取准备就绪, 总条目: {}, 需提取: {}, 唯一图标源: {}",
        total, need_extract, unique_count
    );

    let keys: Vec<String> = unique_keys.into_iter().collect();
    let extracted: Vec<(String, IconData)> = if method == Some("icns") {
        keys.par_iter()
            .map(|key| {
                let mut parts = key.split(KEY_SEP);
                let _kind = parts.next().unwrap_or_default();
                let app_path = parts.next().unwrap_or_default();
                let icon_data =
                    extract_icon_for_app(Path::new(app_path), method).unwrap_or(IconData {
                        base64: String::new(),
                        width: 32,
                        height: 32,
                    });
                (key.clone(), icon_data)
            })
            .collect()
    } else {
        keys.iter()
            .map(|key| {
                let mut parts = key.split(KEY_SEP);
                let _kind = parts.next().unwrap_or_default();
                let app_path = parts.next().unwrap_or_default();
                let icon_data =
                    extract_icon_for_app(Path::new(app_path), method).unwrap_or(IconData {
                        base64: String::new(),
                        width: 32,
                        height: 32,
                    });
                (key.clone(), icon_data)
            })
            .collect()
    };

    let mut extracted_map = std::collections::HashMap::<String, IconData>::new();
    extracted_map.reserve(extracted.len());
    for (key, data) in extracted {
        extracted_map.insert(key, data);
    }

    for icon in icons.iter_mut() {
        if !icon.icon_base64.is_empty() {
            continue;
        }
        let app_path = icon.icon_source_path.as_deref().unwrap_or(&icon.file_path);
        let key = format!("app{KEY_SEP}{app_path}");
        if let Some(icon_data) = extracted_map.get(&key) {
            icon.icon_base64 = icon_data.base64.clone();
            icon.icon_width = icon_data.width;
            icon.icon_height = icon_data.height;
        }
    }

    let duration = start.elapsed();
    let filled = icons.iter().filter(|i| !i.icon_base64.is_empty()).count();
    let newly_filled = filled.saturating_sub(already_has_icon);

    println!(
        "🖼️ [提取阶段] 统一图标提取完成, 总计: {}, 已有: {}, 本次新增: {}, 最终有图标: {}, 耗时: {:.3}s",
        total,
        already_has_icon,
        newly_filled,
        filled,
        duration.as_secs_f64()
    );
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn fill_icons(_icons: &mut [DesktopIcon], _method: Option<&str>) {}

/// 获取所有可用的扫描器
pub fn get_all_scanners() -> Vec<Box<dyn IconScanner>> {
    let mut scanners: Vec<Box<dyn IconScanner>> = Vec::new();
    #[cfg(target_os = "windows")]
    {
        scanners.push(Box::new(taskbar::TaskbarScanner));
        scanners.push(Box::new(desktop::DesktopScanner));
        scanners.push(Box::new(desktop::PublicDesktopScanner));
        scanners.push(Box::new(start_menu::StartMenuScanner));
        scanners.push(Box::new(start_menu::CommonStartMenuScanner));
        scanners.push(Box::new(uwp::UWPScanner));
        scanners.push(Box::new(appdata::AppDataScanner));
        scanners.push(Box::new(quick_launch::QuickLaunchScanner));
        scanners.push(Box::new(installed_programs::InstalledProgramsScanner));
        scanners.push(Box::new(program_files::ProgramFilesScanner));
        scanners.push(Box::new(program_files::ProgramFilesX86Scanner));
    }
    #[cfg(target_os = "macos")]
    {
        scanners.push(Box::new(macos::ApplicationsScanner));
        scanners.push(Box::new(macos::SystemApplicationsScanner));
        scanners.push(Box::new(macos::UserApplicationsScanner));
        scanners.push(Box::new(macos::CoreServicesScanner));
        scanners.push(Box::new(macos::SpotlightScanner));
        scanners.push(Box::new(macos::SystemProfilerScanner));
    }
    scanners
}

/// 软件来源枚举
#[derive(Debug, Clone, PartialEq)]
pub enum IconSource {
    Desktop,
    PublicDesktop,
    StartMenu,
    CommonStartMenu,
    InstalledPrograms,
    ProgramFiles,
    ProgramFilesX86,
    QuickLaunch,
    TaskbarPinned,
    AppDataPrograms,
    UWPApps,
    // macOS
    Applications,
    SystemApplications,
    UserApplications,
    CoreServices,
    Spotlight,
    SystemProfiler,
}

impl IconSource {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "desktop" => Some(Self::Desktop),
            "public_desktop" => Some(Self::PublicDesktop),
            "start_menu" => Some(Self::StartMenu),
            "common_start_menu" => Some(Self::CommonStartMenu),
            "installed_programs" => Some(Self::InstalledPrograms),
            "program_files" => Some(Self::ProgramFiles),
            "program_files_x86" => Some(Self::ProgramFilesX86),
            "quick_launch" => Some(Self::QuickLaunch),
            "taskbar_pinned" => Some(Self::TaskbarPinned),
            "appdata_programs" => Some(Self::AppDataPrograms),
            "uwp_apps" => Some(Self::UWPApps),
            // macOS
            "applications" => Some(Self::Applications),
            "system_applications" => Some(Self::SystemApplications),
            "user_applications" => Some(Self::UserApplications),
            "core_services" => Some(Self::CoreServices),
            "spotlight" => Some(Self::Spotlight),
            "system_profiler" => Some(Self::SystemProfiler),
            _ => None,
        }
    }
}

/// 获取指定来源的图标
pub fn get_icons_from_source(
    source: IconSource,
    method: Option<&str>,
) -> std::result::Result<Vec<DesktopIcon>, Box<dyn std::error::Error>> {
    let scanners = get_all_scanners();
    let source_id = match source {
        IconSource::Desktop => "desktop",
        IconSource::PublicDesktop => "public_desktop",
        IconSource::StartMenu => "start_menu",
        IconSource::CommonStartMenu => "common_start_menu",
        IconSource::InstalledPrograms => "installed_programs",
        IconSource::ProgramFiles => "program_files",
        IconSource::ProgramFilesX86 => "program_files_x86",
        IconSource::QuickLaunch => "quick_launch",
        IconSource::TaskbarPinned => "taskbar_pinned",
        IconSource::AppDataPrograms => "appdata_programs",
        IconSource::UWPApps => "uwp_apps",
        IconSource::Applications => "applications",
        IconSource::SystemApplications => "system_applications",
        IconSource::UserApplications => "user_applications",
        IconSource::CoreServices => "core_services",
        IconSource::Spotlight => "spotlight",
        IconSource::SystemProfiler => "system_profiler",
    };

    if let Some(scanner) = scanners.iter().find(|s| s.id() == source_id) {
        scanner.scan(method)
    } else {
        Ok(vec![])
    }
}

/// 获取所有来源的图标（并行去重）
pub fn get_all_icons(
    method: Option<&str>,
) -> std::result::Result<Vec<DesktopIcon>, Box<dyn std::error::Error>> {
    let start_all = std::time::Instant::now();
    let scanners = get_all_scanners();

    // 并行执行所有扫描器
    let mut icon_map: std::collections::HashMap<String, (DesktopIcon, i32)> =
        std::collections::HashMap::new();

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

    // 并行执行所有扫描器
    let all_results: Vec<(Vec<DesktopIcon>, String)> = scanners
        .par_iter()
        .filter(|s| s.id() != "system_profiler") // 排除极慢的扫描器，避免全量扫描时卡死
        .map(|scanner| {
            let id = scanner.id().to_string();
            println!(">>> 开始并行扫描来源: {}", scanner.name());
            let result = scanner.scan(method).unwrap_or_else(|e| {
                eprintln!("!!! 扫描器 {} 失败: {}", scanner.name(), e);
                vec![]
            });
            println!(
                "<<< 扫描器 {} 完成，找到 {} 个图标",
                scanner.name(),
                result.len()
            );
            (result, id)
        })
        .collect();

    // 按扫描器顺序汇总（保留优先级）
    for (icons, source_id) in all_results {
        let priority = get_priority(&source_id);
        for icon in icons {
            // 生成唯一指纹：名称 + 规范化后的 target_path
            // 不再包含 arguments，以解决带不同追踪参数的同名应用重复问题
            let name = icon.name.trim().to_lowercase();
            let target = if cfg!(target_os = "windows") {
                normalize_path(&icon.target_path).to_lowercase()
            } else {
                icon.target_path.to_lowercase()
            };
            let fingerprint = format!("{}:{}", name, target);

            if let Some((existing_icon, existing_priority)) = icon_map.get_mut(&fingerprint) {
                // 如果新来源优先级更高，或者优先级相同但新图标数据更完整，则替换
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
            } else {
                icon_map.insert(fingerprint, (icon, priority));
            }
        }
    }

    // 将 HashMap 转换为 Vec，并按名称排序
    let mut all_icons: Vec<DesktopIcon> = icon_map.into_iter().map(|(_, (icon, _))| icon).collect();
    all_icons.sort_by(|a, b| a.name.cmp(&b.name));

    let duration_all = start_all.elapsed();
    println!(
        "✅ [汇总阶段] 所有来源扫描去重完成, 共找到 {} 个唯一图标, 耗时: {:.3}s",
        all_icons.len(),
        duration_all.as_secs_f64()
    );

    fill_icons(&mut all_icons, method);

    Ok(all_icons)
}

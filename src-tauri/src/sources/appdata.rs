use crate::shortcut::get_file_metadata;
use crate::sources::scanner::IconScanner;
use crate::types::*;
use rayon::prelude::*;
use std::env;
use std::error::Error;
use std::path::{Path, PathBuf};

pub struct AppDataScanner;

impl IconScanner for AppDataScanner {
    fn id(&self) -> &str {
        "appdata_programs"
    }
    fn name(&self) -> &str {
        "用户程序 (AppData)"
    }
    fn description(&self) -> &str {
        "当前用户 AppData 目录中的程序 (如 VS Code, Discord 等)"
    }
    fn icon(&self) -> &str {
        "👤"
    }
    fn scan(&self, method: Option<&str>) -> Result<Vec<DesktopIcon>, Box<dyn Error>> {
        get_appdata_icons(method)
    }
}

pub fn get_appdata_icons(method: Option<&str>) -> Result<Vec<DesktopIcon>, Box<dyn Error>> {
    let mut all_icons = Vec::new();

    // 1. %LOCALAPPDATA%\Programs
    if let Ok(local_appdata) = env::var("LOCALAPPDATA") {
        let local_path = PathBuf::from(&local_appdata);

        // 扫描 Programs 目录
        let programs_path = local_path.join("Programs");
        if programs_path.exists() {
            all_icons.extend(scan_appdata_folder(
                &programs_path,
                method,
                "用户程序 (AppData/Local)",
            )?);
        }

        // 扫描 Local 根目录（有些应用直接装在这里，比如 Telegram）
        // 限制深度为 2，避免扫描太多
        all_icons.extend(scan_appdata_folder_with_depth(
            &local_path,
            method,
            2,
            "用户程序 (AppData/Local)",
        )?);
    }

    // 2. %APPDATA% (Roaming)
    if let Ok(appdata) = env::var("APPDATA") {
        let roaming_path = PathBuf::from(appdata);
        if roaming_path.exists() {
            // Roaming 目录下的程序通常在子目录中
            all_icons.extend(scan_appdata_folder_with_depth(
                &roaming_path,
                method,
                2,
                "用户程序 (AppData/Roaming)",
            )?);
        }
    }

    Ok(all_icons)
}

fn scan_appdata_folder(
    folder_path: &Path,
    method: Option<&str>,
    source_name: &str,
) -> Result<Vec<DesktopIcon>, Box<dyn Error>> {
    scan_appdata_folder_with_depth(folder_path, method, 4, source_name)
}

fn scan_appdata_folder_with_depth(
    folder_path: &Path,
    _method: Option<&str>,
    depth: usize,
    source_name: &str,
) -> Result<Vec<DesktopIcon>, Box<dyn Error>> {
    println!(
        "扫描 {} 文件夹: {:?} (深度: {})",
        source_name, folder_path, depth
    );

    let scan_start = std::time::Instant::now();
    let mut exe_files = Vec::new();
    collect_exe_files(folder_path, &mut exe_files, depth)?;
    let scan_duration = scan_start.elapsed();

    println!(
        "🔍 [扫描阶段] {} 扫描完成, 找到 {} 个文件, 耗时: {:.3}s",
        source_name,
        exe_files.len(),
        scan_duration.as_secs_f64()
    );

    // 并行处理所有 EXE 文件
    let prepare_start = std::time::Instant::now();
    let results: Vec<_> = exe_files
        .par_iter()
        .filter_map(|path| match process_exe_file(path, source_name) {
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

    Ok(results)
}

fn collect_exe_files(
    dir: &Path,
    files: &mut Vec<PathBuf>,
    max_depth: usize,
) -> Result<(), Box<dyn Error>> {
    if max_depth == 0 {
        return Ok(());
    }
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                collect_exe_files(&path, files, max_depth - 1)?;
            } else if path.is_file() {
                // 收集所有文件，统一由前端过滤
                files.push(path);
            }
        }
    }
    Ok(())
}

fn process_exe_file(path: &Path, source_name: &str) -> Result<DesktopIcon, Box<dyn Error>> {
    let file_name = path.file_stem().unwrap().to_string_lossy().to_string();
    let file_path = path.to_string_lossy().to_string();
    let file_meta = get_file_metadata(path);

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

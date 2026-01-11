// Program Files 软件来源

use crate::path::*;
use crate::shortcut::get_file_metadata;
use crate::sources::scanner::IconScanner;
use crate::types::*;
use rayon::prelude::*;
use std::error::Error;
use std::path::Path;

pub struct ProgramFilesScanner;

impl IconScanner for ProgramFilesScanner {
    fn id(&self) -> &str {
        "program_files"
    }
    fn name(&self) -> &str {
        "Program Files"
    }
    fn description(&self) -> &str {
        "Program Files 目录中的程序"
    }
    fn icon(&self) -> &str {
        "📁"
    }
    fn scan(&self, method: Option<&str>) -> Result<Vec<DesktopIcon>, Box<dyn Error>> {
        get_program_files_icons(method)
    }
}

pub struct ProgramFilesX86Scanner;

impl IconScanner for ProgramFilesX86Scanner {
    fn id(&self) -> &str {
        "program_files_x86"
    }
    fn name(&self) -> &str {
        "Program Files (x86)"
    }
    fn description(&self) -> &str {
        "Program Files (x86) 目录中的程序"
    }
    fn icon(&self) -> &str {
        "📂"
    }
    fn scan(&self, method: Option<&str>) -> Result<Vec<DesktopIcon>, Box<dyn Error>> {
        get_program_files_x86_icons(method)
    }
}

/// 获取 Program Files 中的程序图标
pub fn get_program_files_icons(
    method: Option<&str>,
) -> std::result::Result<Vec<DesktopIcon>, Box<dyn std::error::Error>> {
    let program_files_path = get_program_files_path()?;
    scan_program_folder(&program_files_path, method, "Program Files")
}

/// 获取 Program Files (x86) 中的程序图标
pub fn get_program_files_x86_icons(
    method: Option<&str>,
) -> std::result::Result<Vec<DesktopIcon>, Box<dyn std::error::Error>> {
    let program_files_x86_path = get_program_files_x86_path()?;
    scan_program_folder(&program_files_x86_path, method, "Program Files (x86)")
}

/// 扫描 Program Files 文件夹
fn scan_program_folder(
    folder_path: &Path,
    _method: Option<&str>,
    source_name: &str,
) -> std::result::Result<Vec<DesktopIcon>, Box<dyn std::error::Error>> {
    println!("扫描 {} 文件夹: {:?}", source_name, folder_path);

    if !folder_path.exists() {
        println!("{} 路径不存在", source_name);
        return Ok(Vec::new());
    }

    let scan_start = std::time::Instant::now();
    let mut exe_files = Vec::new();
    collect_exe_files(folder_path, &mut exe_files, 4)?; // 增加深度到4
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
            Err(_) => None,
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

/// 收集可执行文件（限制递归深度，避免扫描太多文件）
fn collect_exe_files(
    dir: &Path,
    files: &mut Vec<std::path::PathBuf>,
    max_depth: usize,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    if max_depth == 0 {
        return Ok(());
    }

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();

            // 跳过一些系统文件夹
            if let Some(name) = path.file_name() {
                let name_str = name.to_string_lossy().to_lowercase();
                if name_str.starts_with("windows")
                    || name_str == "system32"
                    || name_str == "syswow64"
                    || name_str.starts_with("$")
                {
                    continue;
                }
            }

            if path.is_dir() {
                // 递归处理子目录
                collect_exe_files(&path, files, max_depth - 1)?;
            } else if path.is_file() {
                // 收集所有文件，统一由前端过滤
                files.push(path);
            }
        }
    }
    Ok(())
}

/// 处理单个可执行文件
fn process_exe_file(
    path: &Path,
    source_name: &str,
) -> Result<DesktopIcon, Box<dyn std::error::Error>> {
    let file_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();

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

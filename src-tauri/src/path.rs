// 路径处理模块

#[cfg(target_os = "windows")]
use std::env;
#[cfg(target_os = "windows")]
use std::path::PathBuf;
#[cfg(target_os = "windows")]
use windows::{
    core::GUID,
    Win32::Foundation::HANDLE,
    Win32::System::Com::CoTaskMemFree,
    Win32::UI::Shell::{
        FOLDERID_CommonPrograms, FOLDERID_Desktop, FOLDERID_ProgramFiles, FOLDERID_ProgramFilesX86,
        FOLDERID_Programs, FOLDERID_Public, FOLDERID_PublicDesktop, FOLDERID_System,
        FOLDERID_SystemX86, FOLDERID_UserProfiles, FOLDERID_Windows, SHGetKnownFolderPath,
        KF_FLAG_DEFAULT,
    },
};

/// 将 GUID 格式的路径规范化为物理路径
/// 例如：{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\services.msc -> C:\Windows\System32\services.msc
#[cfg(target_os = "windows")]
pub fn normalize_path(path: &str) -> String {
    if !path.starts_with('{') {
        return path.to_string();
    }

    let end_bracket = match path.find('}') {
        Some(pos) => pos,
        None => return path.to_string(),
    };

    let guid_str = &path[1..end_bracket];
    let remaining = &path[end_bracket + 1..].trim_start_matches('\\');

    let folder_id = match guid_str.to_uppercase().as_str() {
        "1AC14E77-02E7-4E5D-B744-2EB1AE5198B7" => Some(&FOLDERID_System),
        "F38BF404-1D43-42F2-9305-67DE0B28FC23" => Some(&FOLDERID_Windows),
        "D65231B0-B2F1-4857-A4CE-A8E7C6EA7D27" => Some(&FOLDERID_SystemX86),
        "7C5A40EF-A0FB-4BFC-874A-C0F2E0B9FA8E" => Some(&FOLDERID_ProgramFiles),
        "50233421-DB61-45A4-9A2F-F5962B859194" => Some(&FOLDERID_ProgramFilesX86),
        "AE054212-3519-4430-83ED-D70627221F3C" => Some(&FOLDERID_UserProfiles),
        "DFDF76A2-C82A-4D63-906A-5644AC457385" => Some(&FOLDERID_Public),
        _ => None,
    };

    if let Some(fid) = folder_id {
        if let Ok(base_path) = get_known_folder_path(fid) {
            let mut full_path = base_path;
            if !remaining.is_empty() {
                full_path.push(remaining);
            }
            return full_path.to_string_lossy().to_string();
        }
    }

    path.to_string()
}

#[cfg(not(target_os = "windows"))]
pub fn normalize_path(path: &str) -> String {
    path.to_string()
}

#[cfg(target_os = "windows")]
fn get_known_folder_path(
    folder_id: &GUID,
) -> std::result::Result<PathBuf, Box<dyn std::error::Error>> {
    unsafe {
        let path = SHGetKnownFolderPath(folder_id, KF_FLAG_DEFAULT, Some(HANDLE::default()))?;
        let desktop_path = path.to_string()?;
        CoTaskMemFree(Some(path.as_ptr() as *const _));
        Ok(PathBuf::from(desktop_path))
    }
}

#[cfg(target_os = "windows")]
pub fn get_public_desktop_path() -> std::result::Result<PathBuf, Box<dyn std::error::Error>> {
    get_known_folder_path(&FOLDERID_PublicDesktop)
}

#[cfg(target_os = "windows")]
pub fn get_desktop_path() -> std::result::Result<PathBuf, Box<dyn std::error::Error>> {
    println!("正在获取桌面路径...");
    let result = get_known_folder_path(&FOLDERID_Desktop);
    match &result {
        Ok(path) => println!("成功获取桌面路径: {:?}", path),
        Err(e) => eprintln!("获取桌面路径失败: {}", e),
    }
    result
}

// 获取用户开始菜单程序路径
#[cfg(target_os = "windows")]
pub fn get_start_menu_programs_path() -> std::result::Result<PathBuf, Box<dyn std::error::Error>> {
    get_known_folder_path(&FOLDERID_Programs)
}

// 获取公共开始菜单程序路径
#[cfg(target_os = "windows")]
pub fn get_common_start_menu_programs_path(
) -> std::result::Result<PathBuf, Box<dyn std::error::Error>> {
    get_known_folder_path(&FOLDERID_CommonPrograms)
}

// 获取 Program Files 路径
#[cfg(target_os = "windows")]
pub fn get_program_files_path() -> std::result::Result<PathBuf, Box<dyn std::error::Error>> {
    env::var("ProgramFiles")
        .map(PathBuf::from)
        .map_err(|e| format!("获取 ProgramFiles 路径失败: {}", e).into())
}

// 获取 Program Files (x86) 路径
#[cfg(target_os = "windows")]
pub fn get_program_files_x86_path() -> std::result::Result<PathBuf, Box<dyn std::error::Error>> {
    env::var("ProgramFiles(x86)")
        .map(PathBuf::from)
        .map_err(|e| format!("获取 ProgramFiles(x86) 路径失败: {}", e).into())
}

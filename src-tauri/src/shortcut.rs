// 快捷方式解析模块

use crate::extractors::utils::ComInit;
use std::fs;
use std::path::Path;
use std::time::SystemTime;
use windows::{
    core::{Interface, PCWSTR},
    Win32::Storage::FileSystem::{FILE_FLAGS_AND_ATTRIBUTES, WIN32_FIND_DATAW},
    Win32::System::Com::{CoCreateInstance, IPersistFile, CLSCTX_INPROC_SERVER, STGM_READ},
    Win32::UI::Shell::{
        IShellLinkW, SHGetFileInfoW, ShellLink, SHFILEINFOW, SHGFI_DISPLAYNAME, SLGP_UNCPRIORITY,
    },
};

/// 文件元数据信息
pub struct FileMetadata {
    pub created_time: Option<String>,
    pub modified_time: Option<String>,
    pub accessed_time: Option<String>,
    pub file_size: Option<u64>,
    pub file_type: Option<String>,
}

/// 快捷方式完整信息结构
pub struct ShortcutInfo {
    pub target_path: String,
    pub icon_path: String,
    pub icon_index: i32,
    pub description: String,
    pub arguments: String,
    pub working_directory: String,
    pub hotkey: u16,
    pub show_command: i32,
}

/// 获取快捷方式的完整信息
pub fn get_shortcut_full_info(lnk_path: &Path) -> Result<ShortcutInfo, Box<dyn std::error::Error>> {
    unsafe {
        let _com = ComInit::new(windows::Win32::System::Com::COINIT_MULTITHREADED);

        let shell_link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;
        let persist_file: IPersistFile = shell_link.cast()?;

        let wide_path: Vec<u16> = lnk_path
            .to_string_lossy()
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        persist_file.Load(PCWSTR(wide_path.as_ptr()), STGM_READ)?;

        // 获取目标路径
        let mut target_path = [0u16; 260];
        let mut find_data = WIN32_FIND_DATAW::default();
        shell_link.GetPath(&mut target_path, &mut find_data, SLGP_UNCPRIORITY.0 as u32)?;

        // 获取图标位置
        let mut icon_path = [0u16; 260];
        let mut icon_index = 0i32;
        shell_link.GetIconLocation(&mut icon_path, &mut icon_index)?;

        // 获取描述
        let mut description = [0u16; 260];
        let _ = shell_link.GetDescription(&mut description);

        // 获取启动参数
        let mut arguments = [0u16; 260];
        let _ = shell_link.GetArguments(&mut arguments);

        // 获取工作目录
        let mut working_dir = [0u16; 260];
        let _ = shell_link.GetWorkingDirectory(&mut working_dir);

        // 获取快捷键
        let hotkey = shell_link.GetHotkey().unwrap_or_else(|_| 0);

        // 获取显示命令
        let show_cmd = shell_link.GetShowCmd().map(|cmd| cmd.0).unwrap_or(1);

        let target = String::from_utf16_lossy(&target_path)
            .trim_end_matches('\0')
            .to_string();

        let icon = String::from_utf16_lossy(&icon_path)
            .trim_end_matches('\0')
            .to_string();
        let desc = String::from_utf16_lossy(&description)
            .trim_end_matches('\0')
            .to_string();
        let args = String::from_utf16_lossy(&arguments)
            .trim_end_matches('\0')
            .to_string();
        let work_dir = String::from_utf16_lossy(&working_dir)
            .trim_end_matches('\0')
            .to_string();

        let final_icon_path = if icon.is_empty() {
            target.clone()
        } else {
            icon
        };

        Ok(ShortcutInfo {
            target_path: target,
            icon_path: final_icon_path,
            icon_index,
            description: desc,
            arguments: args,
            working_directory: work_dir,
            hotkey,
            show_command: show_cmd,
        })
    }
}

/// 将快捷键值转换为可读字符串
pub fn hotkey_to_string(hotkey: u16) -> Option<String> {
    if hotkey == 0 {
        return None;
    }

    let key = (hotkey & 0xFF) as u8;
    let modifiers = (hotkey >> 8) as u8;

    let mut parts = Vec::new();

    if modifiers & 0x01 != 0 {
        parts.push("Shift".to_string());
    }
    if modifiers & 0x02 != 0 {
        parts.push("Ctrl".to_string());
    }
    if modifiers & 0x04 != 0 {
        parts.push("Alt".to_string());
    }

    // 添加按键
    if key >= 0x41 && key <= 0x5A {
        // A-Z
        parts.push(format!("{}", key as char));
    } else if key >= 0x30 && key <= 0x39 {
        // 0-9
        parts.push(format!("{}", key as char));
    } else if key >= 0x70 && key <= 0x87 {
        // F1-F24
        parts.push(format!("F{}", key - 0x6F));
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join("+"))
    }
}

/// 将显示命令转换为可读字符串
pub fn show_command_to_string(show_cmd: i32) -> String {
    match show_cmd {
        1 => "正常窗口".to_string(),
        2 => "最小化".to_string(),
        3 => "最大化".to_string(),
        7 => "最小化无激活".to_string(),
        _ => format!("未知({})", show_cmd),
    }
}

/// 获取文件的元数据信息
pub fn get_file_metadata(file_path: &Path) -> FileMetadata {
    let mut metadata = FileMetadata {
        created_time: None,
        modified_time: None,
        accessed_time: None,
        file_size: None,
        file_type: None,
    };

    // 获取文件扩展名
    if let Some(ext) = file_path.extension() {
        metadata.file_type = Some(ext.to_string_lossy().to_string());
    }

    // 获取文件元数据
    if let Ok(meta) = fs::metadata(file_path) {
        // 文件大小
        metadata.file_size = Some(meta.len());

        // 创建时间
        if let Ok(created) = meta.created() {
            metadata.created_time = Some(system_time_to_string(created));
        }

        // 修改时间
        if let Ok(modified) = meta.modified() {
            metadata.modified_time = Some(system_time_to_string(modified));
        }

        // 访问时间
        if let Ok(accessed) = meta.accessed() {
            metadata.accessed_time = Some(system_time_to_string(accessed));
        }
    }

    metadata
}

/// 将 SystemTime 转换为可读字符串
fn system_time_to_string(time: SystemTime) -> String {
    use chrono::{DateTime, Local};
    let datetime: DateTime<Local> = time.into();
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn resolve_shortcut(
    lnk_path: &Path,
) -> Result<(String, String, i32), Box<dyn std::error::Error>> {
    unsafe {
        let _com = ComInit::new(windows::Win32::System::Com::COINIT_MULTITHREADED);

        let shell_link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;
        let persist_file: IPersistFile = shell_link.cast()?;

        let wide_path: Vec<u16> = lnk_path
            .to_string_lossy()
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        persist_file.Load(PCWSTR(wide_path.as_ptr()), STGM_READ)?;

        let mut target_path = [0u16; 260];
        let mut find_data = WIN32_FIND_DATAW::default();
        shell_link.GetPath(&mut target_path, &mut find_data, SLGP_UNCPRIORITY.0 as u32)?;

        let mut icon_path = [0u16; 260];
        let mut icon_index = 0i32;
        shell_link.GetIconLocation(&mut icon_path, &mut icon_index)?;

        let target = String::from_utf16_lossy(&target_path)
            .trim_end_matches('\0')
            .to_string();
        let icon = String::from_utf16_lossy(&icon_path)
            .trim_end_matches('\0')
            .to_string();

        let final_icon_path = if icon.is_empty() {
            target.clone()
        } else {
            icon
        };

        Ok((target, final_icon_path, icon_index))
    }
}

/// 获取文件的本地化显示名称
pub fn get_localized_name(file_path: &Path) -> Option<String> {
    unsafe {
        // 初始化 COM
        let _com = ComInit::new(windows::Win32::System::Com::COINIT_MULTITHREADED);

        let wide_path: Vec<u16> = file_path
            .to_string_lossy()
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let mut shfi = SHFILEINFOW::default();
        let result = SHGetFileInfoW(
            PCWSTR(wide_path.as_ptr()),
            FILE_FLAGS_AND_ATTRIBUTES::default(),
            Some(&mut shfi),
            size_of::<SHFILEINFOW>() as u32,
            SHGFI_DISPLAYNAME,
        );

        if result != 0 {
            let display_name = String::from_utf16_lossy(&shfi.szDisplayName)
                .trim_end_matches('\0')
                .to_string();

            if !display_name.is_empty() {
                return Some(display_name);
            }
        }

        None
    }
}

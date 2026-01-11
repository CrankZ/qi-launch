// Shell图标提取方式 - 使用Shell API获取高质量图标

use crate::types::*;
use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Gdi::*, Win32::System::Com::*,
    Win32::UI::Shell::*, Win32::UI::WindowsAndMessaging::*,
};

/// 使用Shell的高级API提取大尺寸图标
/// 这个方法结合了多种Shell API来获取最高质量的图标
pub fn extract_icon_via_shell(
    file_path: &str,
    size: u32,
) -> std::result::Result<IconData, Box<dyn std::error::Error>> {
    let start = std::time::Instant::now();
    println!(
        "🖼️ [提取阶段] shell 开始提取图标: {}, 尺寸: {}",
        file_path, size
    );

    let result: std::result::Result<IconData, Box<dyn std::error::Error>> = unsafe {
        'extract: {
            let _com = super::utils::ComInit::new(COINIT_MULTITHREADED);

            let wide_path: Vec<u16> = file_path.encode_utf16().chain(std::iter::once(0)).collect();

            let mut shfi = SHFILEINFOW::default();
            let flags = if size >= 256 {
                SHGFI_ICON.0 | SHGFI_LARGEICON.0
            } else if size >= 48 {
                SHGFI_ICON.0 | SHGFI_LARGEICON.0
            } else {
                SHGFI_ICON.0 | SHGFI_SMALLICON.0
            };

            extern "system" {
                fn SHGetFileInfoW(
                    pszPath: PCWSTR,
                    dwFileAttributes: u32,
                    psfi: *mut SHFILEINFOW,
                    cbFileInfo: u32,
                    uFlags: u32,
                ) -> usize;
            }

            let result = SHGetFileInfoW(
                PCWSTR(wide_path.as_ptr()),
                0,
                &mut shfi as *mut SHFILEINFOW,
                std::mem::size_of::<SHFILEINFOW>() as u32,
                flags,
            );

            let hicon = shfi.hIcon;
            if result != 0 && !hicon.is_invalid() {
                let icon_data = match super::utils::convert_hicon_to_base64(hicon) {
                    Ok(v) => v,
                    Err(e) => {
                        let _ = DestroyIcon(hicon);
                        break 'extract Err(e);
                    }
                };
                let _ = DestroyIcon(hicon);

                if !icon_data.base64.is_empty() {
                    break 'extract Ok(icon_data);
                }
            }

            break 'extract Err("Shell API 无法提取图标".into());
        }
    };

    let duration = start.elapsed();
    match &result {
        Ok(icon_data) => println!(
            "🖼️ [提取阶段] shell 提取成功: {} ({}x{}), 耗时: {:.3}s",
            file_path,
            icon_data.width,
            icon_data.height,
            duration.as_secs_f64()
        ),
        Err(e) => println!(
            "🖼️ [提取阶段] shell 提取失败: {} ({}), 耗时: {:.3}s",
            file_path,
            e,
            duration.as_secs_f64()
        ),
    }

    result
}

/// 获取文件的缩略图/预览图标
/// 使用 IShellItemImageFactory 获取真正的高质量缩略图
pub fn extract_thumbnail_icon(
    file_path: &str,
    size: u32,
) -> std::result::Result<IconData, Box<dyn std::error::Error>> {
    let start = std::time::Instant::now();
    println!(
        "🖼️ [提取阶段] thumbnail 开始提取图标: {}, 尺寸: {}",
        file_path, size
    );

    let result: std::result::Result<IconData, Box<dyn std::error::Error>> = unsafe {
        'extract: {
            let _com = super::utils::ComInit::new(COINIT_MULTITHREADED);

            let wide_path: Vec<u16> = file_path.encode_utf16().chain(std::iter::once(0)).collect();

            let shell_item: IShellItem =
                match SHCreateItemFromParsingName(PCWSTR(wide_path.as_ptr()), None) {
                    Ok(v) => v,
                    Err(e) => break 'extract Err(Box::new(e)),
                };

            let factory: IShellItemImageFactory = match shell_item.cast() {
                Ok(v) => v,
                Err(e) => break 'extract Err(Box::new(e)),
            };

            let hbitmap = match factory.GetImage(
                SIZE {
                    cx: size as i32,
                    cy: size as i32,
                },
                SIIGBF_ICONONLY | SIIGBF_BIGGERSIZEOK,
            ) {
                Ok(v) => v,
                Err(e) => break 'extract Err(Box::new(e)),
            };

            if hbitmap.is_invalid() {
                break 'extract Err("无法获取缩略图位图".into());
            }

            let icon_data = match super::utils::convert_hbitmap_to_base64(hbitmap) {
                Ok(v) => v,
                Err(e) => {
                    let _ = DeleteObject(hbitmap.into());
                    break 'extract Err(e);
                }
            };
            let _ = DeleteObject(hbitmap.into());

            if !icon_data.base64.is_empty() {
                break 'extract Ok(icon_data);
            }

            break 'extract Err("缩略图提取为空".into());
        }
    };

    let duration = start.elapsed();
    match &result {
        Ok(icon_data) => println!(
            "🖼️ [提取阶段] thumbnail 提取成功: {} ({}x{}), 耗时: {:.3}s",
            file_path,
            icon_data.width,
            icon_data.height,
            duration.as_secs_f64()
        ),
        Err(e) => println!(
            "🖼️ [提取阶段] thumbnail 提取失败: {} ({}), 耗时: {:.3}s",
            file_path,
            e,
            duration.as_secs_f64()
        ),
    }

    result
}

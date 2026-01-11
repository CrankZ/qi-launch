// 高分辨率图标提取

use crate::types::*;
use windows::{core::*, Win32::UI::WindowsAndMessaging::*};

// 外部函数声明
extern "system" {
    fn PrivateExtractIconsW(
        szFileName: PCWSTR,
        nIconIndex: i32,
        cxIcon: i32,
        cyIcon: i32,
        phicon: *mut HICON,
        piconid: *mut u32,
        nIcons: u32,
        flags: u32,
    ) -> u32;
}

// 高分辨率图标提取 - 使用 PrivateExtractIconsW 获取指定尺寸图标
pub fn extract_high_resolution_icon(
    file_path: &str,
    icon_index: i32,
    target_size: Option<u32>,
) -> std::result::Result<IconData, Box<dyn std::error::Error>> {
    let start = std::time::Instant::now();
    let size = target_size.unwrap_or(512);
    println!(
        "🖼️ [提取阶段] high_res 开始提取图标: {}, 索引: {}, 尺寸: {}",
        file_path, icon_index, size
    );

    let result: std::result::Result<IconData, Box<dyn std::error::Error>> = unsafe {
        let wide_path: Vec<u16> = file_path.encode_utf16().chain(std::iter::once(0)).collect();

        let mut icons: [HICON; 1] = [HICON::default(); 1];
        let mut icon_ids: [u32; 1] = [0; 1];

        let count = PrivateExtractIconsW(
            PCWSTR(wide_path.as_ptr()),
            icon_index,
            size as i32,
            size as i32,
            icons.as_mut_ptr(),
            icon_ids.as_mut_ptr(),
            1,
            0,
        );

        let hicon = icons[0];
        if count > 0 && !hicon.is_invalid() {
            let icon_data = super::utils::convert_hicon_to_base64(hicon)?;
            let _ = DestroyIcon(hicon);

            if !icon_data.base64.is_empty() {
                Ok(icon_data)
            } else {
                Err("PrivateExtractIcons 提取为空".into())
            }
        } else {
            Err("PrivateExtractIcons 提取失败".into())
        }
    };

    let duration = start.elapsed();
    match &result {
        Ok(icon_data) => println!(
            "🖼️ [提取阶段] high_res 提取成功: {} ({}x{}), 耗时: {:.3}s",
            file_path,
            icon_data.width,
            icon_data.height,
            duration.as_secs_f64()
        ),
        Err(e) => println!(
            "🖼️ [提取阶段] high_res 提取失败: {} ({}), 耗时: {:.3}s",
            file_path,
            e,
            duration.as_secs_f64()
        ),
    }

    result
}

// 系统图标缓存提取方式

use crate::constants::{IID_IIMAGELIST, SHIL_JUMBO as CUSTOM_SHIL_JUMBO};
use crate::types::*;
use windows::{core::*, Win32::UI::Shell::*, Win32::UI::WindowsAndMessaging::*};

// 外部函数声明
extern "system" {
    fn SHGetFileInfoW(
        pszPath: PCWSTR,
        dwFileAttributes: u32,
        psfi: *mut SHFILEINFOW,
        cbFileInfo: u32,
        uFlags: u32,
    ) -> usize;

    fn SHGetImageList(
        iImageList: i32,
        riid: *const GUID,
        ppvObj: *mut *mut std::ffi::c_void,
    ) -> HRESULT;
}

// 使用 SHGetImageList 获取系统图标缓存中的更大尺寸图标
pub fn extract_system_icon_highest_resolution(
    file_path: &str,
) -> std::result::Result<IconData, Box<dyn std::error::Error>> {
    let start = std::time::Instant::now();
    println!(
        "🖼️ [提取阶段] imagelist 开始提取图标: {}, 尺寸: 256",
        file_path
    );

    let result = extract_system_icon_with_imagelist_jumbo(file_path);

    let duration = start.elapsed();
    match &result {
        Ok(icon_data) => println!(
            "🖼️ [提取阶段] imagelist 提取成功: {} ({}x{}), 耗时: {:.3}s",
            file_path,
            icon_data.width,
            icon_data.height,
            duration.as_secs_f64()
        ),
        Err(e) => println!(
            "🖼️ [提取阶段] imagelist 提取失败: {} ({}), 耗时: {:.3}s",
            file_path,
            e,
            duration.as_secs_f64()
        ),
    }

    result
}

/// 仅使用 SHGetImageList JUMBO 尺寸提取图标
fn extract_system_icon_with_imagelist_jumbo(
    file_path: &str,
) -> std::result::Result<IconData, Box<dyn std::error::Error>> {
    unsafe {
        // 初始化 COM (使用 MTA 模型更适合后台线程)
        let _com = super::utils::ComInit::new(windows::Win32::System::Com::COINIT_MULTITHREADED);

        let wide_path: Vec<u16> = file_path.encode_utf16().chain(std::iter::once(0)).collect();

        // 获取 JUMBO 尺寸的 ImageList
        let mut image_list: *mut std::ffi::c_void = std::ptr::null_mut();
        let hr = SHGetImageList(CUSTOM_SHIL_JUMBO, &IID_IIMAGELIST, &mut image_list);

        if hr.is_err() || image_list.is_null() {
            return Err("无法获取 JUMBO ImageList".into());
        }

        let image_list = image_list as *mut IImageList;

        // 获取文件的图标索引
        let mut file_info: SHFILEINFOW = std::mem::zeroed();
        let result = SHGetFileInfoW(
            PCWSTR(wide_path.as_ptr()),
            0,
            &mut file_info,
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_SYSICONINDEX.0,
        );

        if result == 0 {
            ((*(*image_list).vtable).release)(image_list);
            return Err("无法获取系统图标索引".into());
        }

        let icon_index = file_info.iIcon;

        // 从 ImageList 获取图标
        let mut icon: HICON = HICON::default();
        let hr = ((*(*image_list).vtable).get_icon)(image_list, icon_index, 0, &mut icon);

        // 释放 ImageList
        ((*(*image_list).vtable).release)(image_list);

        if hr.is_ok() && !icon.is_invalid() {
            // 进行昂贵的图像转换
            let icon_data = super::utils::convert_hicon_to_base64(icon)?;
            let _ = DestroyIcon(icon);

            if !icon_data.base64.is_empty() {
                return Ok(icon_data);
            }
        }

        Err("SHGetImageList JUMBO 提取失败".into())
    }
}

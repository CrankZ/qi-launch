// PNG/BMP资源直接提取

use super::utils::{parse_bmp_dimensions, parse_png_dimensions};
use crate::types::*;
use base64::prelude::*;
use windows::{core::*, Win32::Foundation::*, Win32::System::LibraryLoader::*};

// PNG/BMP资源直接提取 - 从PE资源中直接提取PNG/BMP格式图标
pub fn extract_png_bmp_from_pe_resource(
    file_path: &str,
) -> std::result::Result<IconData, Box<dyn std::error::Error>> {
    let start = std::time::Instant::now();
    println!("🖼️ [提取阶段] png_bmp 开始提取图标: {}", file_path);

    let result: std::result::Result<IconData, Box<dyn std::error::Error>> = unsafe {
        'extract: {
            let wide_path: Vec<u16> = file_path.encode_utf16().chain(std::iter::once(0)).collect();

            let module = match LoadLibraryExW(
                PCWSTR(wide_path.as_ptr()),
                None,
                LOAD_LIBRARY_AS_DATAFILE | LOAD_LIBRARY_AS_IMAGE_RESOURCE,
            ) {
                Ok(v) => v,
                Err(e) => break 'extract Err(Box::new(e)),
            };

            if module.is_invalid() {
                let _ = FreeLibrary(module);
                break 'extract Err("无法加载PE文件".into());
            }

            let png_types = [
                PCWSTR::from_raw(b"PNG\0".as_ptr() as *const u16),
                PCWSTR::from_raw(24 as *const u16),
                PCWSTR::from_raw(10 as *const u16),
            ];

            for &resource_type in &png_types {
                if let Ok(icon_data) = extract_resource_by_type(module, resource_type) {
                    if icon_data.len() >= 8 && &icon_data[0..8] == b"\x89PNG\r\n\x1a\n" {
                        if let Ok(original_data) =
                            super::utils::process_image_data(&icon_data, "png", false)
                        {
                            let _ = FreeLibrary(module);
                            break 'extract Ok(original_data);
                        }

                        let (width, height) = match parse_png_dimensions(&icon_data) {
                            Ok(v) => v,
                            Err(e) => {
                                let _ = FreeLibrary(module);
                                break 'extract Err(e);
                            }
                        };
                        let base64_data = BASE64_STANDARD.encode(&icon_data);
                        let _ = FreeLibrary(module);
                        break 'extract Ok(IconData {
                            base64: format!("data:image/png;base64,{}", base64_data),
                            width,
                            height,
                        });
                    }

                    if icon_data.len() >= 2 && &icon_data[0..2] == b"BM" {
                        if let Ok(original_data) =
                            super::utils::process_image_data(&icon_data, "bmp", false)
                        {
                            let _ = FreeLibrary(module);
                            break 'extract Ok(original_data);
                        }

                        let (width, height) = match parse_bmp_dimensions(&icon_data) {
                            Ok(v) => v,
                            Err(e) => {
                                let _ = FreeLibrary(module);
                                break 'extract Err(e);
                            }
                        };
                        let base64_data = BASE64_STANDARD.encode(&icon_data);
                        let _ = FreeLibrary(module);
                        break 'extract Ok(IconData {
                            base64: format!("data:image/bmp;base64,{}", base64_data),
                            width,
                            height,
                        });
                    }
                }
            }

            let _ = FreeLibrary(module);
            break 'extract Err("未找到PNG/BMP资源".into());
        }
    };

    let duration = start.elapsed();
    match &result {
        Ok(icon_data) => println!(
            "🖼️ [提取阶段] png_bmp 提取成功: {} ({}x{}), 耗时: {:.3}s",
            file_path,
            icon_data.width,
            icon_data.height,
            duration.as_secs_f64()
        ),
        Err(e) => println!(
            "🖼️ [提取阶段] png_bmp 提取失败: {} ({}), 耗时: {:.3}s",
            file_path,
            e,
            duration.as_secs_f64()
        ),
    }

    result
}

// 从PE资源中提取指定类型的资源数据
unsafe fn extract_resource_by_type(
    module: HMODULE,
    resource_type: PCWSTR,
) -> std::result::Result<Vec<u8>, Box<dyn std::error::Error>> {
    // 枚举该类型的所有资源
    let mut resource_names = Vec::new();

    // 使用EnumResourceNamesW枚举资源名称
    extern "system" fn enum_resource_names_proc(
        _module: HMODULE,
        _type: PCWSTR,
        name: PCWSTR,
        lparam: isize,
    ) -> BOOL {
        let names = lparam as *mut Vec<PCWSTR>;
        unsafe {
            (*names).push(name);
        }
        TRUE
    }

    let names_ptr = &mut resource_names as *mut Vec<PCWSTR> as isize;
    let _ = EnumResourceNamesW(
        Some(module),
        resource_type,
        Some(enum_resource_names_proc),
        names_ptr,
    );

    // 尝试加载每个资源
    for &resource_name in &resource_names {
        if let Ok(data) = super::utils::load_resource_data(module, resource_type, resource_name) {
            if !data.is_empty() {
                return Ok(data);
            }
        }
    }

    Err("未找到有效资源".into())
}

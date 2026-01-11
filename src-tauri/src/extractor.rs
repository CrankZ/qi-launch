// 图标提取模块 - 第一部分：外部函数声明和基础提取函数

use crate::extractors::*;
use crate::types::*;

pub fn extract_icon_as_base64(
    file_path: &str,
    icon_index: i32,
) -> std::result::Result<IconData, Box<dyn std::error::Error>> {
    // 默认使用 PrivateExtractIcons (high_res) 方式，512px
    extract_icon_with_method(file_path, icon_index, "high_res")
}

pub fn extract_icon_with_method(
    file_path: &str,
    icon_index: i32,
    method: &str,
) -> std::result::Result<IconData, Box<dyn std::error::Error>> {
    // 根据指定方法提取图标 (v3版本支持更高分辨率)
    let icon_data = match method {
        "smart" => {
            if let Ok(icon_data) = extract_icon_via_imagelist(file_path, icon_index) {
                icon_data
            } else {
                high_resolution::extract_high_resolution_icon(file_path, icon_index, Some(512))?
            }
        }
        "imagelist" => extract_icon_via_imagelist(file_path, icon_index)?,
        "png_bmp" => extract_png_bmp_from_pe_resource(file_path)?,
        "pe_resource" => extract_icon_from_best_group(file_path)?,
        "high_res" => {
            high_resolution::extract_high_resolution_icon(file_path, icon_index, Some(512))?
        }
        "shell" => extract_icon_via_shell(file_path, 512)?,
        "thumbnail" => extract_thumbnail_icon(file_path, 1024)?,
        _ => {
            return Err(format!("不支持的提取方法: {}", method).into());
        }
    };

    Ok(icon_data)
}

// 移除冗余的提取函数，统一使用 extract_icon_best_quality

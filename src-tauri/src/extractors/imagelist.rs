// 系统 ImageList JUMBO 提取方式 - 使用 Windows 系统列表获取 256x256 图标

use crate::extractors::*;
use crate::types::*;

/// 使用系统 ImageList (JUMBO 尺寸) 提取图标
/// 这是一个具体、单一的提取方式，不包含任何兜底逻辑
pub fn extract_icon_via_imagelist(
    file_path: &str,
    _icon_index: i32,
) -> std::result::Result<IconData, Box<dyn std::error::Error>> {
    // 仅使用系统 ImageList (JUMBO 尺寸) 提取
    let icon_data = extract_system_icon_highest_resolution(file_path)?;

    if !icon_data.base64.is_empty() {
        Ok(icon_data)
    } else {
        Err("无法获取 JUMBO 图标".into())
    }
}

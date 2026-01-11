// 图标提取器模块 - 统一各种图标获取方式

pub mod high_resolution; // 高分辨率提取
pub mod icon_group; // 图标组提取
pub mod imagelist; // 系统 ImageList (JUMBO) 方式
pub mod png_bmp; // PNG/BMP资源提取
pub mod shell_icon; // Shell API图标提取
pub mod system_cache; // 系统图标缓存
pub mod utils; // 工具函数

// 重新导出主要的提取函数
pub use icon_group::extract_icon_from_best_group;
pub use imagelist::extract_icon_via_imagelist;
pub use png_bmp::extract_png_bmp_from_pe_resource;
pub use shell_icon::{extract_icon_via_shell, extract_thumbnail_icon};
pub use system_cache::extract_system_icon_highest_resolution;

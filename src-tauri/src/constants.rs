// 常量定义

use windows::core::GUID;

// SHGetImageList 相关常量
pub const SHIL_JUMBO: i32 = 0x4; // 256x256 像素

// PE资源类型常量
pub const RT_ICON: u16 = 3; // 图标资源
pub const RT_GROUP_ICON: u16 = 14; // 图标组资源

// IImageList 接口 GUID
pub const IID_IIMAGELIST: GUID = GUID {
    data1: 0x46EB5926,
    data2: 0x582E,
    data3: 0x4017,
    data4: [0x9F, 0xDF, 0xE8, 0x99, 0x8D, 0xAA, 0x09, 0x50],
};

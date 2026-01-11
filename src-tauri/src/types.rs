// 类型定义和结构体

use serde::{Deserialize, Serialize};
#[cfg(target_os = "windows")]
use windows::core::BOOL;
#[cfg(target_os = "windows")]
use windows::{
    core::GUID, Win32::Foundation::*, Win32::Graphics::Gdi::*, Win32::UI::WindowsAndMessaging::*,
};

// HRESULT从 Foundation 模块导入
#[cfg(target_os = "windows")]
type HRESULT = windows::core::HRESULT;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IconData {
    pub base64: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopIcon {
    pub name: String,
    pub icon_base64: String,
    pub target_path: String,
    pub file_path: String,
    pub icon_width: u32,
    pub icon_height: u32,
    pub icon_source_path: Option<String>,
    pub icon_source_index: Option<i32>,

    // 时间信息
    pub created_time: Option<String>,  // 创建时间
    pub modified_time: Option<String>, // 修改时间
    pub accessed_time: Option<String>, // 访问时间

    // 文件信息
    pub file_size: Option<u64>,    // 文件大小（字节）
    pub file_type: Option<String>, // 文件类型/扩展名

    // 快捷方式专属信息
    pub description: Option<String>,       // 描述/备注
    pub arguments: Option<String>,         // 启动参数
    pub working_directory: Option<String>, // 工作目录
    pub hotkey: Option<String>,            // 快捷键
    pub show_command: Option<String>,      // 运行方式（正常、最小化、最大化）
    pub source_name: Option<String>,       // 软件来源名称（如：用户桌面、开始菜单等）
}

/// 检查路径是否为 URL
#[cfg(target_os = "windows")]
pub fn is_url(path: &str) -> bool {
    let p = path.to_lowercase();
    p.starts_with("http://") || p.starts_with("https://")
}

// 图标组目录结构
#[cfg(target_os = "windows")]
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct IconGroupDir {
    pub reserved: u16,      // 保留字段，必须为0
    pub resource_type: u16, // 资源类型，图标为1
    pub icon_count: u16,    // 图标数量
}

// 图标组目录条目
#[cfg(target_os = "windows")]
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct IconGroupDirEntry {
    pub width: u8,         // 图标宽度（0表示256）
    pub height: u8,        // 图标高度（0表示256）
    pub color_count: u8,   // 颜色数量（0表示>=256色）
    pub reserved: u8,      // 保留字段
    pub planes: u16,       // 颜色平面数
    pub bit_count: u16,    // 每像素位数
    pub bytes_in_res: u32, // 图标数据大小
    pub icon_id: u16,      // 图标资源ID
}

// IImageList 接口定义（仅在 Windows 下编译）
#[cfg(target_os = "windows")]
#[repr(C)]
pub struct IImageList {
    pub vtable: *const IImageListVtbl,
}

#[cfg(target_os = "windows")]
#[repr(C)]
pub struct IImageListVtbl {
    // IUnknown 方法
    pub query_interface: unsafe extern "system" fn(
        this: *mut IImageList,
        riid: *const GUID,
        ppv_object: *mut *mut std::ffi::c_void,
    ) -> HRESULT,
    pub add_ref: unsafe extern "system" fn(this: *mut IImageList) -> u32,
    pub release: unsafe extern "system" fn(this: *mut IImageList) -> u32,

    // IImageList 方法
    pub add: unsafe extern "system" fn(
        this: *mut IImageList,
        hbm_image: HBITMAP,
        hbm_mask: HBITMAP,
        pi: *mut i32,
    ) -> HRESULT,
    pub replace_icon: unsafe extern "system" fn(
        this: *mut IImageList,
        i: i32,
        hicon: HICON,
        pi: *mut i32,
    ) -> HRESULT,
    pub set_overlay_image:
        unsafe extern "system" fn(this: *mut IImageList, i_image: i32, i_overlay: i32) -> HRESULT,
    pub replace: unsafe extern "system" fn(
        this: *mut IImageList,
        i: i32,
        hbm_image: HBITMAP,
        hbm_mask: HBITMAP,
    ) -> HRESULT,
    pub add_masked: unsafe extern "system" fn(
        this: *mut IImageList,
        hbm_image: HBITMAP,
        cr_mask: u32,
        pi: *mut i32,
    ) -> HRESULT,
    pub draw: unsafe extern "system" fn(
        this: *mut IImageList,
        pimldp: *const std::ffi::c_void,
    ) -> HRESULT,
    pub remove: unsafe extern "system" fn(this: *mut IImageList, i: i32) -> HRESULT,
    pub get_icon: unsafe extern "system" fn(
        this: *mut IImageList,
        i: i32,
        flags: u32,
        picon: *mut HICON,
    ) -> HRESULT,
    pub get_image_info: unsafe extern "system" fn(
        this: *mut IImageList,
        i: i32,
        pimage_info: *mut std::ffi::c_void,
    ) -> HRESULT,
    pub copy: unsafe extern "system" fn(
        this: *mut IImageList,
        i_dst: i32,
        punk_src: *mut std::ffi::c_void,
        i_src: i32,
        u_flags: u32,
    ) -> HRESULT,
    pub merge: unsafe extern "system" fn(
        this: *mut IImageList,
        i1: i32,
        punk2: *mut std::ffi::c_void,
        i2: i32,
        dx: i32,
        dy: i32,
        riid: *const GUID,
        ppv: *mut *mut std::ffi::c_void,
    ) -> HRESULT,
    pub clone: unsafe extern "system" fn(
        this: *mut IImageList,
        riid: *const GUID,
        ppv: *mut *mut std::ffi::c_void,
    ) -> HRESULT,
    pub get_image_rect:
        unsafe extern "system" fn(this: *mut IImageList, i: i32, prc: *mut RECT) -> HRESULT,
    pub get_icon_size:
        unsafe extern "system" fn(this: *mut IImageList, cx: *mut i32, cy: *mut i32) -> HRESULT,
    pub set_icon_size:
        unsafe extern "system" fn(this: *mut IImageList, cx: i32, cy: i32) -> HRESULT,
    pub get_image_count: unsafe extern "system" fn(this: *mut IImageList, pi: *mut i32) -> HRESULT,
    pub set_image_count:
        unsafe extern "system" fn(this: *mut IImageList, u_new_count: u32) -> HRESULT,
    pub set_bk_color:
        unsafe extern "system" fn(this: *mut IImageList, cl_bk: u32, pcl_old: *mut u32) -> HRESULT,
    pub get_bk_color: unsafe extern "system" fn(this: *mut IImageList, pcl_bk: *mut u32) -> HRESULT,
    pub begin_drag: unsafe extern "system" fn(
        this: *mut IImageList,
        i_track: i32,
        dx_hotspot: i32,
        dy_hotspot: i32,
    ) -> HRESULT,
    pub end_drag: unsafe extern "system" fn(this: *mut IImageList) -> HRESULT,
    pub drag_enter: unsafe extern "system" fn(
        this: *mut IImageList,
        hwnd_lock: HWND,
        x: i32,
        y: i32,
    ) -> HRESULT,
    pub drag_leave: unsafe extern "system" fn(this: *mut IImageList, hwnd_lock: HWND) -> HRESULT,
    pub drag_move: unsafe extern "system" fn(this: *mut IImageList, x: i32, y: i32) -> HRESULT,
    pub set_drag_cursor_image: unsafe extern "system" fn(
        this: *mut IImageList,
        punk: *mut std::ffi::c_void,
        i_drag: i32,
        dx_hotspot: i32,
        dy_hotspot: i32,
    ) -> HRESULT,
    pub drag_show_no_lock:
        unsafe extern "system" fn(this: *mut IImageList, f_show: BOOL) -> HRESULT,
    pub get_drag_image: unsafe extern "system" fn(
        this: *mut IImageList,
        ppt: *mut POINT,
        ppt_hotspot: *mut POINT,
        riid: *const GUID,
        ppv: *mut *mut std::ffi::c_void,
    ) -> HRESULT,
    pub get_item_flags:
        unsafe extern "system" fn(this: *mut IImageList, i: i32, dw_flags: *mut u32) -> HRESULT,
    pub get_overlay_image: unsafe extern "system" fn(
        this: *mut IImageList,
        i_overlay: i32,
        pi_index: *mut i32,
    ) -> HRESULT,
}

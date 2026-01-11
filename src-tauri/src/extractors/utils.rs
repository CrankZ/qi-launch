// 图标提取辅助工具函数模块 - 通用工具函数

use crate::types::*;
use base64::prelude::*;
use std::mem::size_of;

/// COM 初始化包装器，确保正确引用计数
pub struct ComInit {
    hr: HRESULT,
}

impl ComInit {
    pub fn new(dwcoinit: windows::Win32::System::Com::COINIT) -> Self {
        unsafe {
            let hr = windows::Win32::System::Com::CoInitializeEx(None, dwcoinit);
            Self { hr }
        }
    }
}

impl Drop for ComInit {
    fn drop(&mut self) {
        if self.hr.is_ok() {
            unsafe {
                windows::Win32::System::Com::CoUninitialize();
            }
        }
    }
}
use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Gdi::*, Win32::System::LibraryLoader::*,
    Win32::UI::WindowsAndMessaging::*,
};

// 从HICON转换为base64字符串（默认不裁剪，保持原始尺寸）
pub fn convert_hicon_to_base64(
    icon: HICON,
) -> std::result::Result<IconData, Box<dyn std::error::Error>> {
    convert_hicon_to_base64_with_options(icon, false)
}

// 从HICON转换为base64字符串（可选裁剪）
pub fn convert_hicon_to_base64_with_options(
    icon: HICON,
    crop_borders: bool,
) -> std::result::Result<IconData, Box<dyn std::error::Error>> {
    unsafe {
        // 获取图标信息
        let mut icon_info = ICONINFO::default();
        if GetIconInfo(icon, &mut icon_info).is_err() {
            return Ok(IconData {
                base64: String::new(),
                width: 0,
                height: 0,
            });
        }

        // 获取位图信息
        let mut bitmap = BITMAP::default();
        GetObjectW(
            icon_info.hbmColor.into(),
            size_of::<BITMAP>() as i32,
            Some(&mut bitmap as *mut _ as *mut _),
        );

        let width = bitmap.bmWidth;
        let height = bitmap.bmHeight;

        if width <= 0 || height <= 0 {
            let _ = DeleteObject(icon_info.hbmColor.into());
            let _ = DeleteObject(icon_info.hbmMask.into());
            return Ok(IconData {
                base64: String::new(),
                width: 0,
                height: 0,
            });
        }

        // 创建设备上下文
        let hdc = GetDC(Some(HWND::default()));
        let mem_dc = CreateCompatibleDC(Some(hdc));

        // 创建DIB，确保支持透明度
        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height, // 负值表示自上而下
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
        let dib = CreateDIBSection(
            Some(mem_dc),
            &bmi,
            DIB_RGB_COLORS,
            &mut bits,
            Some(HANDLE::default()),
            0,
        )?;

        let old_bitmap = SelectObject(mem_dc, dib.into());

        // 清除背景为透明
        let pixel_count = (width * height) as usize;
        if !bits.is_null() {
            // 将所有像素初始化为透明（ARGB = 0x00000000）
            std::ptr::write_bytes(bits as *mut u8, 0, pixel_count * 4);
        }

        // 绘制图标，不使用背景画刷
        let _ = DrawIconEx(mem_dc, 0, 0, icon, 0, 0, 0, None, DI_NORMAL);

        // 读取像素数据
        let mut pixel_data = vec![0u8; pixel_count * 4];

        if !bits.is_null() {
            std::ptr::copy_nonoverlapping(
                bits as *const u8,
                pixel_data.as_mut_ptr(),
                pixel_data.len(),
            );
        }

        // 转换 BGRA 到 RGBA 并进行智能透明度处理
        // 1. 检测是否全透明（某些旧图标 Alpha 通道全 0）
        // 2. 检测是否是预乘 Alpha（解决黑边问题）
        let mut all_alpha_zero = true;
        let mut has_alpha_content = false;
        let mut definitely_straight = false;
        let mut looks_premultiplied = false;

        for i in (0..pixel_data.len()).step_by(4) {
            let a = pixel_data[i + 3];
            if a > 0 {
                all_alpha_zero = false;
                if a < 255 {
                    has_alpha_content = true;
                    let b = pixel_data[i];
                    let g = pixel_data[i + 1];
                    let r = pixel_data[i + 2];

                    if r > a || g > a || b > a {
                        definitely_straight = true;
                    } else if r < a || g < a || b < a {
                        looks_premultiplied = true;
                    }
                }
            }
        }

        // 决定是否需要反预乘
        let should_unpremultiply = looks_premultiplied && !definitely_straight && has_alpha_content;

        for i in (0..pixel_data.len()).step_by(4) {
            let mut b = pixel_data[i];
            let mut g = pixel_data[i + 1];
            let mut r = pixel_data[i + 2];
            let mut a = pixel_data[i + 3];

            if all_alpha_zero {
                // 如果全透明，通常是旧式图标，将 Alpha 设为 255
                a = 255;
            } else if should_unpremultiply && a > 0 && a < 255 {
                // 反预乘处理：Color = Color_pre / (Alpha / 255)
                let alpha_f = a as f32 / 255.0;
                r = (r as f32 / alpha_f).min(255.0) as u8;
                g = (g as f32 / alpha_f).min(255.0) as u8;
                b = (b as f32 / alpha_f).min(255.0) as u8;
            }

            pixel_data[i] = r; // R
            pixel_data[i + 1] = g; // G
            pixel_data[i + 2] = b; // B
            pixel_data[i + 3] = a; // A
        }

        let icon_data =
            process_pixel_data_to_icon_data(width as u32, height as u32, pixel_data, crop_borders)?;

        // 清理资源
        let _ = SelectObject(mem_dc, old_bitmap);
        let _ = DeleteObject(dib.into());
        let _ = DeleteDC(mem_dc);
        let _ = ReleaseDC(Some(HWND::default()), hdc);
        let _ = DeleteObject(icon_info.hbmColor.into());
        let _ = DeleteObject(icon_info.hbmMask.into());

        Ok(icon_data)
    }
}

/// 内部通用的像素数据处理函数
fn process_pixel_data_to_icon_data(
    width: u32,
    height: u32,
    pixel_data: Vec<u8>,
    crop_borders: bool,
) -> std::result::Result<IconData, Box<dyn std::error::Error>> {
    use image::{ImageBuffer, ImageFormat, Rgba};
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(width, height, pixel_data)
        .ok_or_else(|| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to create image buffer",
            )) as Box<dyn std::error::Error>
        })?;

    let (final_img, final_width, final_height) = if crop_borders {
        // 裁剪空白区域
        let cropped_img = crop_transparent_borders(&img);
        let cropped_width = cropped_img.width();
        let cropped_height = cropped_img.height();
        (cropped_img, cropped_width, cropped_height)
    } else {
        // 保持原始尺寸
        (img, width, height)
    };

    let mut png_data = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut png_data);
    final_img
        .write_to(&mut cursor, ImageFormat::Png)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    // 转换为base64
    let base64_string = BASE64_STANDARD.encode(&png_data);
    Ok(IconData {
        base64: format!("data:image/png;base64,{}", base64_string),
        width: final_width,
        height: final_height,
    })
}

// 从 HBITMAP 转换为 base64 字符串
// 从HBITMAP转换为base64字符串（默认不裁剪，保持原始尺寸）
pub fn convert_hbitmap_to_base64(
    hbitmap: HBITMAP,
) -> std::result::Result<IconData, Box<dyn std::error::Error>> {
    convert_hbitmap_to_base64_with_options(hbitmap, false)
}

// 从 HBITMAP 转换为 base64 字符串（可选裁剪）
pub fn convert_hbitmap_to_base64_with_options(
    hbitmap: HBITMAP,
    crop_borders: bool,
) -> std::result::Result<IconData, Box<dyn std::error::Error>> {
    unsafe {
        let mut bitmap = BITMAP::default();
        if GetObjectW(
            hbitmap.into(),
            size_of::<BITMAP>() as i32,
            Some(&mut bitmap as *mut _ as *mut _),
        ) == 0
        {
            return Err("无法获取位图信息".into());
        }

        let width = bitmap.bmWidth;
        let height = bitmap.bmHeight;

        // 创建设备上下文
        let hdc = GetDC(None);
        let mem_dc = CreateCompatibleDC(Some(hdc));

        // 创建 DIBSection 来读取像素
        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
        let dib = CreateDIBSection(Some(mem_dc), &bmi, DIB_RGB_COLORS, &mut bits, None, 0)?;

        let old_bitmap = SelectObject(mem_dc, dib.into());

        // 使用 GetDIBits 获取位图数据，这比 BitBlt 更可靠地保留 Alpha 通道
        if GetDIBits(
            hdc,
            hbitmap,
            0,
            height as u32,
            Some(bits),
            &mut bmi,
            DIB_RGB_COLORS,
        ) == 0
        {
            // 如果 GetDIBits 失败，尝试备用的 BitBlt 方法
            let hdc_src = CreateCompatibleDC(Some(hdc));
            let old_src = SelectObject(hdc_src, hbitmap.into());
            BitBlt(mem_dc, 0, 0, width, height, Some(hdc_src), 0, 0, SRCCOPY)?;
            SelectObject(hdc_src, old_src);
            let _ = DeleteDC(hdc_src);
        }

        // 转换为 PNG
        let pixel_count = (width * height) as usize;
        let mut pixel_data = vec![0u8; pixel_count * 4];
        if !bits.is_null() {
            std::ptr::copy_nonoverlapping(
                bits as *const u8,
                pixel_data.as_mut_ptr(),
                pixel_count * 4,
            );
        }

        // 转换 BGRA 到 RGBA 并进行智能透明度处理
        // 1. 检测是否全透明（某些旧图标 Alpha 通道全 0）
        // 2. 检测是否是预乘 Alpha（解决黑边问题）
        let mut all_alpha_zero = true;
        let mut has_alpha_content = false;
        let mut definitely_straight = false;
        let mut looks_premultiplied = false;

        for i in (0..pixel_data.len()).step_by(4) {
            let a = pixel_data[i + 3];
            if a > 0 {
                all_alpha_zero = false;
                if a < 255 {
                    has_alpha_content = true;
                    let b = pixel_data[i];
                    let g = pixel_data[i + 1];
                    let r = pixel_data[i + 2];

                    if r > a || g > a || b > a {
                        definitely_straight = true;
                    } else if r < a || g < a || b < a {
                        looks_premultiplied = true;
                    }
                }
            }
        }

        // 决定是否需要反预乘
        let should_unpremultiply = looks_premultiplied && !definitely_straight && has_alpha_content;

        for i in (0..pixel_data.len()).step_by(4) {
            let mut b = pixel_data[i];
            let mut g = pixel_data[i + 1];
            let mut r = pixel_data[i + 2];
            let mut a = pixel_data[i + 3];

            if all_alpha_zero {
                // 如果全透明，通常是旧式图标，将 Alpha 设为 255
                a = 255;
            } else if should_unpremultiply && a > 0 && a < 255 {
                // 反预乘处理：Color = Color_pre / (Alpha / 255)
                let alpha_f = a as f32 / 255.0;
                r = (r as f32 / alpha_f).min(255.0) as u8;
                g = (g as f32 / alpha_f).min(255.0) as u8;
                b = (b as f32 / alpha_f).min(255.0) as u8;
            }

            pixel_data[i] = r; // R
            pixel_data[i + 1] = g; // G
            pixel_data[i + 2] = b; // B
            pixel_data[i + 3] = a; // A
        }

        let icon_data =
            process_pixel_data_to_icon_data(width as u32, height as u32, pixel_data, crop_borders)?;

        // 释放资源
        let _ = SelectObject(mem_dc, old_bitmap);
        let _ = DeleteObject(dib.into());
        let _ = DeleteDC(mem_dc);
        let _ = ReleaseDC(None, hdc);

        Ok(icon_data)
    }
}

/// 裁剪图像的透明边框，去除空白区域
pub fn crop_transparent_borders(
    img: &image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();

    // 如果图像太小，直接返回原图
    if width <= 1 || height <= 1 {
        return img.clone();
    }

    // 找到非透明像素的边界
    let mut min_x = width;
    let mut max_x = 0;
    let mut min_y = height;
    let mut max_y = 0;

    let mut has_content = false;

    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            // 检查alpha通道，如果不是完全透明的像素
            if pixel[3] > 0 {
                has_content = true;
                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_y = min_y.min(y);
                max_y = max_y.max(y);
            }
        }
    }

    // 如果没有找到任何非透明内容，返回1x1的透明图像
    if !has_content {
        let mut empty_img = image::ImageBuffer::new(1, 1);
        empty_img.put_pixel(0, 0, image::Rgba([0, 0, 0, 0]));
        return empty_img;
    }

    // 计算裁剪后的尺寸
    let crop_width = max_x - min_x + 1;
    let crop_height = max_y - min_y + 1;

    // 创建裁剪后的图像
    let mut cropped_img = image::ImageBuffer::new(crop_width, crop_height);

    for y in 0..crop_height {
        for x in 0..crop_width {
            let src_x = min_x + x;
            let src_y = min_y + y;
            let pixel = img.get_pixel(src_x, src_y);
            cropped_img.put_pixel(x, y, *pixel);
        }
    }

    cropped_img
}

/// 处理图像数据，支持PNG、BMP、ICO等格式，可选是否裁剪
pub fn process_image_data(
    image_data: &[u8],
    format: &str,
    crop_borders: bool,
) -> std::result::Result<IconData, Box<dyn std::error::Error>> {
    use image::ImageFormat;
    use std::io::Cursor;

    // 解析图像格式
    let image_format = match format.to_lowercase().as_str() {
        "png" => ImageFormat::Png,
        "bmp" => ImageFormat::Bmp,
        "jpg" | "jpeg" => ImageFormat::Jpeg,
        "ico" => ImageFormat::Ico,
        _ => return Err("不支持的图像格式".into()),
    };

    // 从字节数据加载图像
    let cursor = Cursor::new(image_data);
    let dynamic_img = image::load(cursor, image_format)?;

    // 转换为RGBA格式
    let rgba_img = dynamic_img.to_rgba8();

    // 如果需要，裁剪透明边框
    let final_img = if crop_borders {
        crop_transparent_borders(&rgba_img)
    } else {
        rgba_img
    };

    // 转换回PNG格式
    let mut png_data = Vec::new();
    {
        let mut cursor = Cursor::new(&mut png_data);
        final_img.write_to(&mut cursor, ImageFormat::Png)?;
    }

    // 编码为base64
    let base64_string = BASE64_STANDARD.encode(&png_data);

    Ok(IconData {
        base64: format!("data:image/png;base64,{}", base64_string),
        width: final_img.width(),
        height: final_img.height(),
    })
}

// 从图标数据创建HICON
pub fn create_hicon_from_data(
    icon_data: &[u8],
) -> std::result::Result<HICON, Box<dyn std::error::Error>> {
    unsafe {
        // 使用CreateIconFromResourceEx创建图标，保持原始尺寸
        let hicon = CreateIconFromResourceEx(
            &icon_data,
            true,            // 是图标（不是光标）
            0x00030000,      // 版本
            0,               // 宽度，0表示使用原始尺寸
            0,               // 高度，0表示使用原始尺寸
            LR_DEFAULTCOLOR, // 标志
        )?;

        if hicon.is_invalid() {
            return Err("无法从数据创建图标".into());
        }

        Ok(hicon)
    }
}

// 加载指定的资源数据
pub unsafe fn load_resource_data(
    module: HMODULE,
    resource_type: PCWSTR,
    resource_name: PCWSTR,
) -> std::result::Result<Vec<u8>, Box<dyn std::error::Error>> {
    let resource_info = FindResourceW(Some(module), resource_name, resource_type);
    if resource_info.is_invalid() {
        return Err("资源未找到".into());
    }

    let resource_handle = LoadResource(Some(module), resource_info)?;
    if resource_handle.is_invalid() {
        return Err("无法加载资源".into());
    }

    let resource_data = LockResource(resource_handle);
    if resource_data.is_null() {
        return Err("无法锁定资源".into());
    }

    let resource_size = SizeofResource(Some(module), resource_info);
    if resource_size == 0 {
        return Err("资源大小为0".into());
    }

    let data_slice = std::slice::from_raw_parts(resource_data as *const u8, resource_size as usize);
    Ok(data_slice.to_vec())
}

// 解析PNG文件头获取尺寸信息
pub fn parse_png_dimensions(
    png_data: &[u8],
) -> std::result::Result<(u32, u32), Box<dyn std::error::Error>> {
    if png_data.len() < 24 {
        return Err("PNG数据太短".into());
    }

    // PNG文件头: 8字节签名 + IHDR块
    // IHDR块: 4字节长度 + "IHDR" + 4字节宽度 + 4字节高度 + ...
    if &png_data[0..8] != b"\x89PNG\r\n\x1a\n" {
        return Err("不是有效的PNG文件".into());
    }

    if &png_data[12..16] != b"IHDR" {
        return Err("PNG文件格式错误".into());
    }

    let width = u32::from_be_bytes([png_data[16], png_data[17], png_data[18], png_data[19]]);
    let height = u32::from_be_bytes([png_data[20], png_data[21], png_data[22], png_data[23]]);

    Ok((width, height))
}

// 解析BMP文件头获取尺寸信息
pub fn parse_bmp_dimensions(
    bmp_data: &[u8],
) -> std::result::Result<(u32, u32), Box<dyn std::error::Error>> {
    if bmp_data.len() < 26 {
        return Err("BMP数据太短".into());
    }

    // BMP文件头: 2字节签名"BM" + 12字节文件头 + DIB头
    if &bmp_data[0..2] != b"BM" {
        return Err("不是有效的BMP文件".into());
    }

    // DIB头中的宽度和高度 (从偏移18开始)
    let width = u32::from_le_bytes([bmp_data[18], bmp_data[19], bmp_data[20], bmp_data[21]]);
    let height_i32 = i32::from_le_bytes([bmp_data[22], bmp_data[23], bmp_data[24], bmp_data[25]]);

    Ok((width, height_i32.abs() as u32)) // 高度可能为负数，取绝对值
}

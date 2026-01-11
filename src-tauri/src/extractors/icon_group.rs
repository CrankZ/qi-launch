// 图标组提取方式 - 从第一个或最佳图标组提取主图标

use crate::constants::{RT_GROUP_ICON as CUSTOM_RT_GROUP_ICON, RT_ICON as CUSTOM_RT_ICON};
use crate::types::*;
use windows::{
    core::*, Win32::Foundation::*, Win32::System::LibraryLoader::*,
    Win32::UI::WindowsAndMessaging::*,
};

// 新增：从最佳图标组提取图标（智能选择主图标）
pub fn extract_icon_from_best_group(
    file_path: &str,
) -> std::result::Result<IconData, Box<dyn std::error::Error>> {
    let start = std::time::Instant::now();
    println!("🖼️ [提取阶段] pe_resource 开始提取图标: {}", file_path);

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
                break 'extract Err("无法加载PE文件".into());
            }

            let mut icon_groups: Vec<(PCWSTR, u32, u32)> = Vec::new();

            struct EnumContext {
                groups: *mut Vec<(PCWSTR, u32, u32)>,
                module: HMODULE,
            }

            extern "system" fn enum_all_icon_groups_proc(
                _module: HMODULE,
                _type: PCWSTR,
                name: PCWSTR,
                lparam: isize,
            ) -> BOOL {
                let context = lparam as *mut EnumContext;
                unsafe {
                    let groups = (*context).groups;
                    let module = (*context).module;

                    if let Ok(group_data) = super::utils::load_resource_data(
                        module,
                        PCWSTR::from_raw(super::super::constants::RT_GROUP_ICON as *const u16),
                        name,
                    ) {
                        if let Ok((max_size, icon_count)) = analyze_icon_group(&group_data) {
                            (*groups).push((name, max_size, icon_count));
                        }
                    }
                }
                TRUE // 继续枚举所有图标组
            }

            let mut context = EnumContext {
                groups: &mut icon_groups as *mut Vec<(PCWSTR, u32, u32)>,
                module,
            };
            let _ = EnumResourceNamesW(
                Some(module),
                PCWSTR::from_raw(CUSTOM_RT_GROUP_ICON as *const u16),
                Some(enum_all_icon_groups_proc),
                &mut context as *mut EnumContext as isize,
            );

            if icon_groups.is_empty() {
                let _ = FreeLibrary(module);
                break 'extract Err("未找到任何图标组资源".into());
            }

            // 智能选择最佳图标组（增强打分规则）
            let compute_score = |group_data: &[u8]| -> u32 {
                if group_data.len() < std::mem::size_of::<IconGroupDir>() {
                    return 0;
                }
                let group_dir =
                    std::ptr::read_unaligned(group_data.as_ptr() as *const IconGroupDir);
                if group_dir.resource_type != 1 || group_dir.icon_count == 0 {
                    return 0;
                }
                let entries_start = std::mem::size_of::<IconGroupDir>();
                let entry_size = std::mem::size_of::<IconGroupDirEntry>();
                let mut has256 = false;
                let mut has48 = false;
                let mut has32 = false;
                let mut has16 = false;
                let mut bpp32_count = 0u32;
                let mut max_size_local = 0u32;
                use std::collections::HashSet;
                let mut sizes: HashSet<u32> = HashSet::new();

                for i in 0..group_dir.icon_count as usize {
                    let entry_offset = entries_start + i * entry_size;
                    if entry_offset + entry_size > group_data.len() {
                        break;
                    }
                    let entry = std::ptr::read_unaligned(
                        group_data.as_ptr().add(entry_offset) as *const IconGroupDirEntry
                    );
                    let w = if entry.width == 0 {
                        256
                    } else {
                        entry.width as u32
                    };
                    let h = if entry.height == 0 {
                        256
                    } else {
                        entry.height as u32
                    };
                    let size = w.max(h);
                    max_size_local = max_size_local.max(size);
                    sizes.insert(size);
                    if size == 256 {
                        has256 = true;
                    }
                    if size == 48 {
                        has48 = true;
                    }
                    if size == 32 {
                        has32 = true;
                    }
                    if size == 16 {
                        has16 = true;
                    }
                    if entry.bit_count >= 32 {
                        bpp32_count += 1;
                    }
                }

                let mut score = 0u32;
                if has256 {
                    score += 1000;
                }
                if has48 {
                    score += 50;
                }
                if has32 {
                    score += 30;
                }
                if has16 {
                    score += 10;
                }
                score += (sizes.len() as u32) * 20;
                score += bpp32_count * 5;
                score += max_size_local * 2;
                score += group_dir.icon_count as u32;
                score
            };

            // 对每个组重新计算得分
            let mut scored_groups: Vec<(PCWSTR, u32, u32, u32)> = Vec::new(); // (name, score, max_size, icon_count)
            for (name, max_size, icon_count) in icon_groups.iter() {
                if let Ok(data) = super::utils::load_resource_data(
                    module,
                    PCWSTR::from_raw(CUSTOM_RT_GROUP_ICON as *const u16),
                    *name,
                ) {
                    let s = compute_score(&data);
                    scored_groups.push((*name, s, *max_size, *icon_count));
                } else {
                    scored_groups.push((*name, 0, *max_size, *icon_count));
                }
            }

            // 按得分、最大尺寸、图标数量排序
            scored_groups.sort_by(|a, b| {
                let sc = b.1.cmp(&a.1);
                if sc != std::cmp::Ordering::Equal {
                    return sc;
                }
                let size_cmp = b.2.cmp(&a.2);
                if size_cmp != std::cmp::Ordering::Equal {
                    return size_cmp;
                }
                b.3.cmp(&a.3)
            });

            // 尝试从最佳图标组提取图标
            let mut target_resource: Option<(Vec<u8>, u32, u32)> = None;

            for (_i, (group_name, _score, _max_size, _icon_count)) in
                scored_groups.iter().enumerate()
            {
                if let Ok(group_data) = super::utils::load_resource_data(
                    module,
                    PCWSTR::from_raw(CUSTOM_RT_GROUP_ICON as *const u16),
                    *group_name,
                ) {
                    if let Ok(res) = get_best_icon_resource_from_group(&group_data, module) {
                        target_resource = Some(res);
                        break;
                    }
                }
            }

            let _ = FreeLibrary(module);

            if let Some((icon_data, _width, _height)) = target_resource {
                if let Ok(original_data) =
                    super::utils::process_image_data(&icon_data, "ico", false)
                {
                    break 'extract Ok(original_data);
                }

                if let Ok(hicon) = super::utils::create_hicon_from_data(&icon_data) {
                    let result = super::utils::convert_hicon_to_base64(hicon);
                    let _ = DestroyIcon(hicon);
                    if let Ok(icon_result) = result {
                        break 'extract Ok(icon_result);
                    }
                }
            }

            break 'extract Err("所有图标组都无法提取图标".into());
        }
    };

    let duration = start.elapsed();
    match &result {
        Ok(icon_data) => println!(
            "🖼️ [提取阶段] pe_resource 提取成功: {} ({}x{}), 耗时: {:.3}s",
            file_path,
            icon_data.width,
            icon_data.height,
            duration.as_secs_f64()
        ),
        Err(e) => println!(
            "🖼️ [提取阶段] pe_resource 提取失败: {} ({}), 耗时: {:.3}s",
            file_path,
            e,
            duration.as_secs_f64()
        ),
    }

    result
}

// 分析图标组，返回最大尺寸和图标数量
pub fn analyze_icon_group(
    group_data: &[u8],
) -> std::result::Result<(u32, u32), Box<dyn std::error::Error>> {
    if group_data.len() < std::mem::size_of::<IconGroupDir>() {
        return Err("图标组数据太小".into());
    }

    let group_dir = unsafe { std::ptr::read_unaligned(group_data.as_ptr() as *const IconGroupDir) };

    if group_dir.resource_type != 1 {
        return Err("不是有效的图标组资源".into());
    }

    let icon_count = group_dir.icon_count as u32;
    let mut max_size = 0u32;

    let entries_start = std::mem::size_of::<IconGroupDir>();
    let entry_size = std::mem::size_of::<IconGroupDirEntry>();

    for i in 0..icon_count {
        let entry_offset = entries_start + (i as usize * entry_size);
        if entry_offset + entry_size > group_data.len() {
            break;
        }

        let entry = unsafe {
            std::ptr::read_unaligned(
                group_data.as_ptr().add(entry_offset) as *const IconGroupDirEntry
            )
        };

        let width = if entry.width == 0 {
            256
        } else {
            entry.width as u32
        };
        let height = if entry.height == 0 {
            256
        } else {
            entry.height as u32
        };
        let size = std::cmp::max(width, height);

        if size > max_size {
            max_size = size;
        }
    }

    Ok((max_size, icon_count))
}

// 从图标组数据中找到最佳图标资源的字节数据
pub fn get_best_icon_resource_from_group(
    group_data: &[u8],
    module: HMODULE,
) -> std::result::Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error>> {
    unsafe {
        if group_data.len() < std::mem::size_of::<IconGroupDir>() {
            return Err("图标组数据太小".into());
        }

        // 解析图标组目录
        let group_dir = std::ptr::read_unaligned(group_data.as_ptr() as *const IconGroupDir);

        if group_dir.icon_count == 0 {
            return Err("图标组中没有图标".into());
        }

        let entries_offset = std::mem::size_of::<IconGroupDir>();
        let entry_size = std::mem::size_of::<IconGroupDirEntry>();

        if group_data.len() < entries_offset + (group_dir.icon_count as usize * entry_size) {
            return Err("图标组数据不完整".into());
        }

        let _icon_count = group_dir.icon_count;

        // 收集所有图标信息并找到最高分辨率的图标
        let mut best_entry: Option<IconGroupDirEntry> = None;
        let mut best_area = 0u32;
        let mut best_width = 0u32;
        let mut best_height = 0u32;

        for i in 0..group_dir.icon_count {
            let entry_offset = entries_offset + (i as usize * entry_size);
            let entry = std::ptr::read_unaligned(
                group_data[entry_offset..].as_ptr() as *const IconGroupDirEntry
            );

            // 计算实际尺寸（0表示256）
            let width = if entry.width == 0 {
                256
            } else {
                entry.width as u32
            };
            let height = if entry.height == 0 {
                256
            } else {
                entry.height as u32
            };
            let area = width * height;

            let bit_count = entry.bit_count;

            // 选择规则：
            // 1. 面积更大（更高分辨率）优先
            // 2. 如果面积相同，位深度更高优先
            // 3. 256x256 是特殊的高清尺寸，必须被正确识别并优先选择
            let mut is_better = false;
            if let Some(_current_best) = best_entry {
                if area > best_area {
                    is_better = true;
                } else if area == best_area {
                    if bit_count > best_entry.unwrap().bit_count {
                        is_better = true;
                    }
                }
            } else {
                is_better = true;
            }

            if is_better {
                best_entry = Some(entry);
                best_area = area;
                best_width = width;
                best_height = height;
            }
        }

        if let Some(entry) = best_entry {
            // 加载实际的图标数据
            let icon_resource = super::utils::load_resource_data(
                module,
                PCWSTR::from_raw(CUSTOM_RT_ICON as *const u16),
                PCWSTR(entry.icon_id as *const u16),
            )?;

            Ok((icon_resource, best_width, best_height))
        } else {
            Err("未找到合适的图标资源".into())
        }
    }
}

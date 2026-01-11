use crate::sources::scanner::IconScanner;
use crate::types::*;
use std::error::Error;
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Com::COINIT_MULTITHREADED;
use windows::Win32::UI::Shell::Common::STRRET;
use windows::Win32::UI::Shell::{
    FOLDERID_AppsFolder, IEnumIDList, IShellFolder, SHGetDesktopFolder, SHGetKnownFolderIDList,
    KF_FLAG_DEFAULT,
};

pub struct UWPScanner;

impl IconScanner for UWPScanner {
    fn id(&self) -> &str {
        "uwp_apps"
    }
    fn name(&self) -> &str {
        "应用商店应用 (UWP)"
    }
    fn description(&self) -> &str {
        "从 Microsoft Store 安装的 UWP 应用"
    }
    fn icon(&self) -> &str {
        "🛍️"
    }
    fn scan(&self, method: Option<&str>) -> Result<Vec<DesktopIcon>, Box<dyn Error>> {
        get_uwp_icons(method)
    }
}

pub fn get_uwp_icons(_method: Option<&str>) -> Result<Vec<DesktopIcon>, Box<dyn Error>> {
    let scan_start = std::time::Instant::now();
    let mut app_items = Vec::new();

    unsafe {
        // 初始化 COM
        let _com = crate::extractors::utils::ComInit::new(COINIT_MULTITHREADED);

        // 获取 AppsFolder 的 PIDL
        let apps_folder_pidl =
            SHGetKnownFolderIDList(&FOLDERID_AppsFolder, KF_FLAG_DEFAULT.0 as u32, None).map_err(
                |e| {
                    println!("获取 AppsFolder PIDL 失败: {:?}", e);
                    e
                },
            )?;

        // 获取桌面文件夹
        let desktop_folder: IShellFolder = SHGetDesktopFolder().map_err(|e| {
            println!("获取桌面文件夹失败: {:?}", e);
            e
        })?;

        // 绑定到 AppsFolder
        let apps_folder: IShellFolder = desktop_folder
            .BindToObject(apps_folder_pidl, None)
            .map_err(|e| {
                println!("绑定到 AppsFolder 失败: {:?}", e);
                e
            })?;

        // 枚举对象
        let mut enum_id_list: Option<IEnumIDList> = None;
        let enum_flags = windows::Win32::UI::Shell::SHCONTF_FOLDERS.0
            | windows::Win32::UI::Shell::SHCONTF_NONFOLDERS.0
            | windows::Win32::UI::Shell::SHCONTF_INCLUDEHIDDEN.0;

        apps_folder
            .EnumObjects(HWND::default(), enum_flags as u32, &mut enum_id_list)
            .ok()
            .map_err(|e| {
                println!("EnumObjects 失败: {:?}", e);
                e
            })?;

        let enum_id_list = enum_id_list.ok_or_else(|| {
            println!("IEnumIDList 为空");
            "Failed to get IEnumIDList"
        })?;
        let mut item_pidl_vec: [*mut windows::Win32::UI::Shell::Common::ITEMIDLIST; 1] =
            [std::ptr::null_mut()];
        let mut fetched = 0;

        while enum_id_list
            .Next(&mut item_pidl_vec, Some(&mut fetched))
            .is_ok()
            && fetched > 0
        {
            let item_pidl = item_pidl_vec[0];
            if item_pidl.is_null() {
                continue;
            }

            // 获取显示名称
            let mut str_ret = STRRET::default();
            if apps_folder
                .GetDisplayNameOf(
                    item_pidl,
                    windows::Win32::UI::Shell::SHGDN_NORMAL,
                    &mut str_ret,
                )
                .is_ok()
            {
                let display_name = match str_ret.uType {
                    0 => {
                        // STRRET_WSTR
                        let s = str_ret.Anonymous.pOleStr.to_string()?;
                        windows::Win32::System::Com::CoTaskMemFree(Some(
                            str_ret.Anonymous.pOleStr.as_ptr() as *const _,
                        ));
                        s
                    }
                    _ => "Unknown".to_string(),
                };

                // 获取解析名称 (AUMID)
                let mut str_ret_parsing = STRRET::default();
                if apps_folder
                    .GetDisplayNameOf(
                        item_pidl,
                        windows::Win32::UI::Shell::SHGDN_FORPARSING,
                        &mut str_ret_parsing,
                    )
                    .is_ok()
                {
                    let parsing_name = match str_ret_parsing.uType {
                        0 => {
                            // STRRET_WSTR
                            let s = str_ret_parsing.Anonymous.pOleStr.to_string()?;
                            windows::Win32::System::Com::CoTaskMemFree(Some(
                                str_ret_parsing.Anonymous.pOleStr.as_ptr() as *const _,
                            ));
                            s
                        }
                        _ => String::new(),
                    };

                    if !parsing_name.is_empty() {
                        app_items.push((display_name, parsing_name));
                    }
                }
            }
            windows::Win32::System::Com::CoTaskMemFree(Some(item_pidl as *const _));
        }
        windows::Win32::System::Com::CoTaskMemFree(Some(apps_folder_pidl as *const _));
    }

    let scan_duration = scan_start.elapsed();
    println!(
        "🔍 [扫描阶段] UWP 应用扫描完成, 找到 {} 个应用, 耗时: {:.3}s",
        app_items.len(),
        scan_duration.as_secs_f64()
    );

    use rayon::prelude::*;

    let prepare_start = std::time::Instant::now();
    let icons: Vec<DesktopIcon> = app_items
        .into_par_iter()
        .map(|(display_name, parsing_name)| {
            let shell_path = format!("shell:AppsFolder\\{}", parsing_name);

            DesktopIcon {
                name: display_name,
                icon_base64: String::new(),
                target_path: parsing_name.clone(),
                file_path: parsing_name,
                icon_width: 32,
                icon_height: 32,
                icon_source_path: Some(shell_path),
                icon_source_index: None,
                created_time: None,
                modified_time: None,
                accessed_time: None,
                file_size: None,
                file_type: Some("UWP App".to_string()),
                description: None,
                arguments: None,
                working_directory: None,
                hotkey: None,
                show_command: None,
                source_name: Some("应用商店应用 (UWP)".to_string()),
            }
        })
        .collect();
    let prepare_duration = prepare_start.elapsed();
    println!(
        "🧩 [准备阶段] UWP 扫描结束, 等待后续统一提取图标, 已准备 {} 个条目, 耗时: {:.3}s",
        icons.len(),
        prepare_duration.as_secs_f64()
    );

    Ok(icons)
}

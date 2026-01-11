// macOS 应用扫描与图标提取

use crate::sources::scanner::IconScanner;
use crate::types::{DesktopIcon, IconData};
use base64::prelude::*;
use icns::{IconFamily, IconType};
use plist::Value as PlistValue;
use rayon::prelude::*;
use std::error::Error;
use std::fs;
use std::io::{BufReader, Cursor};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// macOS 原生 API 支持
use objc2::runtime::AnyObject;
use objc2::{msg_send, ClassType};
use objc2_app_kit::{NSBitmapImageRep, NSImage, NSWorkspace};
use objc2_foundation::{NSData, NSString, NSPoint, NSRect, NSSize};

use std::sync::{Mutex, OnceLock};

static SYSTEM_LOCALE: OnceLock<String> = OnceLock::new();
static ICON_CACHE: OnceLock<Mutex<std::collections::HashMap<String, IconData>>> = OnceLock::new();

fn get_system_locale() -> String {
    SYSTEM_LOCALE
        .get_or_init(|| {
            println!("正在初始化系统语言设置...");
            // 优先使用 macOS 的语言设置
            if let Ok(output) = std::process::Command::new("defaults")
                .args(["read", "-g", "AppleLocale"])
                .output()
            {
                if output.status.success() {
                    if let Ok(locale) = String::from_utf8(output.stdout) {
                        let locale = locale.trim().to_lowercase();
                        if !locale.is_empty() {
                            return locale;
                        }
                    }
                }
            }

            // 备选方案：使用 LANG 环境变量
            std::env::var("LANG").unwrap_or_default().to_lowercase()
        })
        .clone()
}

fn get_localized_lproj_variants() -> Vec<String> {
    let locale = get_system_locale();
    let mut variants = Vec::new();

    // 1. 原始 locale (e.g., zh_CN.lproj)
    variants.push(format!("{}.lproj", locale));

    // 2. 处理下划线/横杠 (zh_CN -> zh-CN)
    let dash_locale = locale.replace('_', "-");
    variants.push(format!("{}.lproj", dash_locale));

    // 3. 语言部分 (zh_CN -> zh)
    if let Some(lang) = locale
        .split('_')
        .next()
        .or_else(|| locale.split('-').next())
    {
        variants.push(format!("{}.lproj", lang));
    }

    // 4. 常见的中文变体
    if locale.contains("zh") {
        variants.push("zh_CN.lproj".to_string());
        variants.push("zh-Hans.lproj".to_string());
        variants.push("zh_TW.lproj".to_string());
        variants.push("zh-Hant.lproj".to_string());
        variants.push("Chinese.lproj".to_string());
    }

    // 5. 英文保底
    variants.push("en.lproj".to_string());
    variants.push("English.lproj".to_string());
    variants.push("Base.lproj".to_string());

    variants
}

/// 从 macOS 系统获取原生图标 (带圆角和阴影)
fn extract_icon_native(app_path: &Path) -> Result<IconData, Box<dyn Error>> {
    objc2::rc::autoreleasepool(|_| unsafe {
        let path_str = app_path.to_string_lossy();
        let c_path = std::ffi::CString::new(path_str.as_bytes())?;
        let ns_path: *mut NSString =
            msg_send![NSString::class(), stringWithUTF8String: c_path.as_ptr()];
        if ns_path.is_null() {
            return Err("无法创建 NSString".into());
        }

        let workspace: *mut NSWorkspace = msg_send![NSWorkspace::class(), sharedWorkspace];

        let icon: *mut NSImage = msg_send![workspace, iconForFile: ns_path];
        if icon.is_null() {
            return Err("无法获取 NSImage".into());
        }

        let side = 96.0;
        let size = NSSize {
            width: side,
            height: side,
        };
        let _: () = msg_send![icon, setSize: size];

        let mut rect = NSRect {
            origin: NSPoint { x: 0.0, y: 0.0 },
            size,
        };
        let nil: *mut AnyObject = std::ptr::null_mut();
        let cg_image: *mut std::ffi::c_void =
            msg_send![icon, CGImageForProposedRect: &mut rect, context: nil, hints: nil];

        let png_data: *mut NSData = if !cg_image.is_null() {
            let bitmap_rep: *mut NSBitmapImageRep = msg_send![NSBitmapImageRep::class(), alloc];
            let bitmap_rep: *mut NSBitmapImageRep =
                msg_send![bitmap_rep, initWithCGImage: cg_image];
            let data: *mut NSData =
                msg_send![bitmap_rep, representationUsingType: 4usize, properties: nil];
            let _: () = msg_send![bitmap_rep, release];
            data
        } else {
            let tiff_data: *mut NSData = msg_send![icon, TIFFRepresentation];
            if tiff_data.is_null() {
                return Err("无法获取图片数据".into());
            }
            let bitmap_rep: *mut NSBitmapImageRep =
                msg_send![NSBitmapImageRep::class(), imageRepWithData: tiff_data];
            if bitmap_rep.is_null() {
                return Err("无法转换为位图".into());
            }
            msg_send![bitmap_rep, representationUsingType: 4usize, properties: nil]
        };

        if png_data.is_null() {
            return Err("无法转换为 PNG".into());
        }

        let length: usize = msg_send![png_data, length];
        let bytes: *const u8 = msg_send![png_data, bytes];
        let slice = std::slice::from_raw_parts(bytes, length);

        let base64 = BASE64_STANDARD.encode(slice);

        Ok(IconData {
            base64: format!("data:image/png;base64,{}", base64),
            width: side as u32,
            height: side as u32,
        })
    })
}

pub struct ApplicationsScanner;

impl IconScanner for ApplicationsScanner {
    fn id(&self) -> &str {
        "applications"
    }
    fn name(&self) -> &str {
        "系统应用程序"
    }
    fn description(&self) -> &str {
        "扫描 /Applications 下的 .app"
    }
    fn icon(&self) -> &str {
        "🍎"
    }
    fn scan(&self, method: Option<&str>) -> Result<Vec<DesktopIcon>, Box<dyn Error>> {
        let mut icons = get_applications_icons(method)?;
        for icon in &mut icons {
            icon.source_name = Some("系统应用程序".to_string());
        }
        Ok(icons)
    }
}

pub struct SystemApplicationsScanner;

impl IconScanner for SystemApplicationsScanner {
    fn id(&self) -> &str {
        "system_applications"
    }
    fn name(&self) -> &str {
        "核心应用程序"
    }
    fn description(&self) -> &str {
        "扫描 /System/Applications 下的 .app"
    }
    fn icon(&self) -> &str {
        "⚙️"
    }
    fn scan(&self, method: Option<&str>) -> Result<Vec<DesktopIcon>, Box<dyn Error>> {
        let mut icons = get_system_applications_icons(method)?;
        for icon in &mut icons {
            icon.source_name = Some("核心应用程序".to_string());
        }
        Ok(icons)
    }
}

pub struct UserApplicationsScanner;

impl IconScanner for UserApplicationsScanner {
    fn id(&self) -> &str {
        "user_applications"
    }
    fn name(&self) -> &str {
        "用户应用程序"
    }
    fn description(&self) -> &str {
        "扫描 ~/Applications 下的 .app"
    }
    fn icon(&self) -> &str {
        "👤"
    }
    fn scan(&self, method: Option<&str>) -> Result<Vec<DesktopIcon>, Box<dyn Error>> {
        let mut icons = get_user_applications_icons(method)?;
        for icon in &mut icons {
            icon.source_name = Some("用户应用程序".to_string());
        }
        Ok(icons)
    }
}

pub struct SpotlightScanner;

impl IconScanner for SpotlightScanner {
    fn id(&self) -> &str {
        "spotlight"
    }
    fn name(&self) -> &str {
        "Spotlight 搜索"
    }
    fn description(&self) -> &str {
        "使用 mdfind 搜索全盘应用 (快且全)"
    }
    fn icon(&self) -> &str {
        "🔍"
    }
    fn scan(&self, method: Option<&str>) -> Result<Vec<DesktopIcon>, Box<dyn Error>> {
        let mut icons = get_spotlight_applications_icons(method)?;
        for icon in &mut icons {
            icon.source_name = Some("Spotlight 搜索".to_string());
        }
        Ok(icons)
    }
}

pub struct SystemProfilerScanner;

impl IconScanner for SystemProfilerScanner {
    fn id(&self) -> &str {
        "system_profiler"
    }
    fn name(&self) -> &str {
        "系统信息报告"
    }
    fn description(&self) -> &str {
        "使用 system_profiler 获取完整列表 (极慢)"
    }
    fn icon(&self) -> &str {
        "📋"
    }
    fn scan(&self, method: Option<&str>) -> Result<Vec<DesktopIcon>, Box<dyn Error>> {
        let mut icons = get_system_profiler_icons(method)?;
        for icon in &mut icons {
            icon.source_name = Some("系统信息报告".to_string());
        }
        Ok(icons)
    }
}

pub struct CoreServicesScanner;

impl IconScanner for CoreServicesScanner {
    fn id(&self) -> &str {
        "core_services"
    }
    fn name(&self) -> &str {
        "系统核心服务"
    }
    fn description(&self) -> &str {
        "扫描 /System/Library/CoreServices"
    }
    fn icon(&self) -> &str {
        "🛠️"
    }
    fn scan(&self, method: Option<&str>) -> Result<Vec<DesktopIcon>, Box<dyn Error>> {
        let mut icons = get_core_services_icons(method)?;
        for icon in &mut icons {
            icon.source_name = Some("系统核心服务".to_string());
        }
        Ok(icons)
    }
}

/// 从系统 /Applications 扫描应用
pub fn get_applications_icons(
    method: Option<&str>,
) -> std::result::Result<Vec<DesktopIcon>, Box<dyn std::error::Error>> {
    let dir = PathBuf::from("/Applications");
    scan_applications_dir(&dir, method)
}

/// 从核心系统 /System/Applications 扫描应用
pub fn get_system_applications_icons(
    method: Option<&str>,
) -> std::result::Result<Vec<DesktopIcon>, Box<dyn std::error::Error>> {
    let dir = PathBuf::from("/System/Applications");
    scan_applications_dir(&dir, method)
}

/// 从用户 ~/Applications 扫描应用
pub fn get_user_applications_icons(
    method: Option<&str>,
) -> std::result::Result<Vec<DesktopIcon>, Box<dyn std::error::Error>> {
    let mut dir = dirs::home_dir().ok_or("无法获取用户主目录")?;
    dir.push("Applications");
    scan_applications_dir(&dir, method)
}

/// 从系统核心服务目录扫描
pub fn get_core_services_icons(
    method: Option<&str>,
) -> std::result::Result<Vec<DesktopIcon>, Box<dyn std::error::Error>> {
    let dir = PathBuf::from("/System/Library/CoreServices");
    scan_applications_dir(&dir, method)
}

/// 使用 system_profiler 获取所有应用
pub fn get_system_profiler_icons(
    _method: Option<&str>,
) -> std::result::Result<Vec<DesktopIcon>, Box<dyn std::error::Error>> {
    use std::process::Command;
    // 使用 xml 格式解析会更准，但这里先用简单文本解析路径
    let output = Command::new("system_profiler")
        .args(["SPApplicationsDataType", "-detailLevel", "mini"])
        .output()?;

    if !output.status.success() {
        return Err("system_profiler 执行失败".into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut results = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.starts_with("Location:") {
            let path_str = line.replace("Location:", "").trim().to_string();
            if path_str.ends_with(".app") && !seen.contains(&path_str) {
                let path = Path::new(&path_str);
                if path.exists() {
                    if let Some(icon) = build_desktop_icon_from_app(path, _method) {
                        seen.insert(path_str);
                        results.push(icon);
                    }
                }
            }
        }
    }

    Ok(results)
}

/// 使用 mdfind (Spotlight) 扫描所有应用
pub fn get_spotlight_applications_icons(
    _method: Option<&str>,
) -> std::result::Result<Vec<DesktopIcon>, Box<dyn std::error::Error>> {
    use std::process::Command;
    let output = Command::new("mdfind")
        .arg("kMDItemContentType == 'com.apple.application-bundle'")
        .output()?;

    if !output.status.success() {
        return Err("mdfind 命令执行失败".into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut results = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // 排除已知的标准路径，避免重复扫描（虽然上层可能有去重，但这里可以先做一层）
    let skip_prefixes = ["/Applications/", "/System/Applications/"];

    for line in stdout.lines() {
        let path_str = line.trim();
        if path_str.is_empty() || !path_str.ends_with(".app") {
            continue;
        }

        let path = Path::new(path_str);

        // 简单去重和过滤
        if seen.contains(path_str) {
            continue;
        }

        let mut should_skip = false;
        for prefix in skip_prefixes {
            if path_str.starts_with(prefix) {
                should_skip = true;
                break;
            }
        }

        if should_skip {
            continue;
        }

        if let Some(icon) = build_desktop_icon_from_app(path, _method) {
            seen.insert(path_str.to_string());
            results.push(icon);
        }
    }

    Ok(results)
}

fn scan_applications_dir(
    dir: &Path,
    _method: Option<&str>,
) -> std::result::Result<Vec<DesktopIcon>, Box<dyn std::error::Error>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    // 收集所有可能的 .app 路径
    let paths: Vec<PathBuf> = WalkDir::new(dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|e| e.path().to_path_buf())
        .filter(|path| {
            path.extension().and_then(|s| s.to_str()).unwrap_or("") == "app" && path.is_dir()
        })
        .collect();

    // 并行处理每个 .app 路径提取图标
    let results: Vec<DesktopIcon> = paths
        .into_par_iter()
        .filter_map(|path| build_desktop_icon_from_app(&path, _method))
        .collect();

    Ok(results)
}

pub(crate) fn extract_icon_for_app(app_path: &Path, method: Option<&str>) -> Option<IconData> {
    let method_key = method.unwrap_or("native");
    let cache_key = format!("{}\u{1f}{}", method_key, app_path.to_string_lossy());
    if let Some(cache) = ICON_CACHE.get() {
        if let Ok(cache) = cache.lock() {
            if let Some(cached) = cache.get(&cache_key) {
                return Some(cached.clone());
            }
        }
    }

    let plist_path = app_path.join("Contents").join("Info.plist");

    let icon_data = match method {
        Some("icns") => {
            let dict = PlistValue::from_file(&plist_path).ok()?.into_dictionary()?;
            let icon_name = dict
                .get("CFBundleIconFile")
                .or_else(|| dict.get("CFBundleIconName"))
                .and_then(|v| v.as_string())
                .unwrap_or_default();

            let mut icns_path = app_path.join("Contents/Resources").join(&icon_name);
            if !icns_path.extension().is_some() {
                icns_path.set_extension("icns");
            }
            extract_icon_from_icns(&icns_path).ok()
        }
        _ => extract_icon_native(app_path).ok(),
    }?;

    if icon_data.base64.is_empty() {
        return None;
    }

    let cache = ICON_CACHE.get_or_init(|| Mutex::new(std::collections::HashMap::new()));
    if let Ok(mut cache) = cache.lock() {
        if cache.len() > 1024 {
            cache.clear();
        }
        cache.insert(cache_key, icon_data.clone());
    }

    Some(icon_data)
}

fn build_desktop_icon_from_app(app_path: &Path, _method: Option<&str>) -> Option<DesktopIcon> {
    let plist_path = app_path.join("Contents").join("Info.plist");

    // 1. 获取名称
    let name = read_app_name(&plist_path, app_path)?;

    let icon_source_path = app_path.to_string_lossy().to_string();
    let metadata = fs::metadata(app_path).ok();

    Some(DesktopIcon {
        name,
        icon_base64: String::new(),
        target_path: icon_source_path.clone(),
        file_path: icon_source_path.clone(),
        icon_width: 32,
        icon_height: 32,
        icon_source_path: Some(icon_source_path),
        icon_source_index: None,
        created_time: file_time_str(
            metadata
                .as_ref()
                .map(|m| m.created())
                .unwrap_or(Err(std::io::ErrorKind::NotFound.into())),
        ),
        modified_time: file_time_str(
            metadata
                .as_ref()
                .map(|m| m.modified())
                .unwrap_or(Err(std::io::ErrorKind::NotFound.into())),
        ),
        accessed_time: file_time_str(
            metadata
                .as_ref()
                .map(|m| m.accessed())
                .unwrap_or(Err(std::io::ErrorKind::NotFound.into())),
        ),
        file_size: None,
        file_type: Some("app".to_string()),
        description: None,
        arguments: None,
        working_directory: None,
        hotkey: None,
        show_command: None,
        source_name: None,
    })
}

/// 从 ICNS 文件提取图标 (旧方法)
fn extract_icon_from_icns(
    icns_path: &Path,
) -> std::result::Result<IconData, Box<dyn std::error::Error>> {
    let file = BufReader::new(fs::File::open(icns_path)?);
    let family = IconFamily::read(file)?;

    let mut types: Vec<IconType> = family.available_icons().iter().cloned().collect();
    types.sort_by_key(|t| t.pixel_width() * t.pixel_height());
    types.reverse();

    for icon_type in types {
        if let Ok(image) = family.get_icon_with_type(icon_type) {
            let mut cursor = Cursor::new(Vec::<u8>::new());
            image.write_png(&mut cursor)?;
            let buf = cursor.into_inner();
            let base64 = BASE64_STANDARD.encode(buf);
            return Ok(IconData {
                base64: format!("data:image/png;base64,{}", base64),
                width: icon_type.pixel_width(),
                height: icon_type.pixel_height(),
            });
        }
    }

    Err("无法从ICNS解码图标".into())
}

fn read_app_name(plist_path: &Path, app_path: &Path) -> Option<String> {
    let dict = PlistValue::from_file(plist_path).ok()?.into_dictionary()?;
    let resources_path = app_path.join("Contents/Resources");
    let name = std::cell::RefCell::new(None);

    // 1. 优先从本地化字符串文件读取名称 (这通常是最准确且较快的)
    let lproj_variants = get_localized_lproj_variants();
    for variant in &lproj_variants {
        let strings_path = resources_path.join(variant).join("InfoPlist.strings");
        if strings_path.try_exists().unwrap_or(false) {
            if let Some(localized_name) = extract_name_from_strings_file(&strings_path) {
                name.borrow_mut().replace(localized_name);
                break;
            }
        }
    }

    // 2. 尝试通过 mdls 获取本地化名称
    if name.borrow().is_none() {
        if let Some(localized_name) = get_localized_name_via_mdls(app_path) {
            name.borrow_mut().replace(localized_name);
        }
    }

    // 3. 尝试 plist 中的 CFBundleDisplayName 或 CFBundleName
    if name.borrow().is_none() {
        if let Some(plist_name) = dict
            .get("CFBundleDisplayName")
            .or_else(|| dict.get("CFBundleName"))
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
        {
            name.borrow_mut().replace(plist_name);
        }
    }

    Some(name.into_inner().unwrap_or_else(|| {
        app_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string()
    }))
}

/// 使用 mdls 获取本地化名称
fn get_localized_name_via_mdls(path: &Path) -> Option<String> {
    use std::process::Command;
    // 尝试获取显示名称
    let output = Command::new("mdls")
        .args([
            "-name",
            "kMDItemDisplayName",
            "-raw",
            &path.to_string_lossy(),
        ])
        .output()
        .ok()?;

    if output.status.success() {
        let name = String::from_utf8_lossy(&output.stdout)
            .trim()
            .trim_matches('"')
            .to_string();
        if !name.is_empty() && name != "(null)" {
            return Some(name);
        }
    }
    None
}

/// 从 InfoPlist.strings 文件中提取 CFBundleDisplayName 或 CFBundleName
fn extract_name_from_strings_file(path: &Path) -> Option<String> {
    // 尝试读取文件内容
    let content = fs::read(path).ok()?;

    // 1. 尝试作为标准的 Plist (二进制或 XML) 解析
    if let Ok(plist) = PlistValue::from_reader(Cursor::new(&content)) {
        if let Some(dict) = plist.as_dictionary() {
            if let Some(name) = dict
                .get("CFBundleDisplayName")
                .or_else(|| dict.get("CFBundleName"))
                .and_then(|v| v.as_string())
            {
                return Some(name.to_string());
            }
        }
    }

    // 2. 尝试作为 UTF-16 LE 编码的文本解析 (常见的 InfoPlist.strings 格式)
    let utf16_content = content
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect::<Vec<u16>>();

    if let Ok(text) = String::from_utf16(&utf16_content) {
        // 简单的正则/字符串匹配查找 "CFBundleDisplayName" = "xxx";
        for line in text.lines() {
            if line.contains("CFBundleDisplayName") || line.contains("CFBundleName") {
                if let Some(start) = line.find('=') {
                    let val = &line[start + 1..]
                        .trim()
                        .trim_matches(';')
                        .trim()
                        .trim_matches('"');
                    if !val.is_empty() {
                        return Some(val.to_string());
                    }
                }
            }
        }
    }

    // 3. 尝试作为普通 UTF-8 文本解析
    if let Ok(text) = String::from_utf8(content) {
        for line in text.lines() {
            if line.contains("CFBundleDisplayName") || line.contains("CFBundleName") {
                if let Some(start) = line.find('=') {
                    let val = &line[start + 1..]
                        .trim()
                        .trim_matches(';')
                        .trim()
                        .trim_matches('"');
                    if !val.is_empty() {
                        return Some(val.to_string());
                    }
                }
            }
        }
    }

    None
}

fn file_time_str(time_res: Result<std::time::SystemTime, std::io::Error>) -> Option<String> {
    use std::time::UNIX_EPOCH;
    match time_res {
        Ok(t) => t
            .duration_since(UNIX_EPOCH)
            .ok()
            .map(|d| d.as_secs().to_string()),
        Err(_) => None,
    }
}

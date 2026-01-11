use crate::types::DesktopIcon;
use std::error::Error;

/// 图标扫描器 Trait
pub trait IconScanner: Send + Sync {
    /// 扫描器的唯一标识符
    fn id(&self) -> &str;

    /// 扫描器的显示名称
    fn name(&self) -> &str;

    /// 扫描器的描述
    fn description(&self) -> &str;

    /// 扫描器的图标（Emoji）
    fn icon(&self) -> &str;

    /// 执行扫描
    fn scan(&self, method: Option<&str>) -> Result<Vec<DesktopIcon>, Box<dyn Error>>;
}

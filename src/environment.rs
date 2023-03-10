use crate::configure::init_configure;

/// 初始化运行环境
/// 负责创建各种目录, 文件等等可能不存在的资源
pub fn init_environment() -> anyhow::Result<()> {
    init_configure()?;
    Ok(())
}
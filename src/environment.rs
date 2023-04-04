use std::fs::create_dir_all;
use std::future::Future;

use crate::configure::{get_config, init_configure};
use crate::log::init_tracing_subscriber;

/// 初始化运行环境
/// 负责创建各种目录, 文件等等可能不存在的资源
pub fn init_environment() -> anyhow::Result<impl Future<Output = anyhow::Result<()>> + Sized> {
    let handle = init_tracing_subscriber();
    init_configure()?;

    create_dir_all(&get_config().general.app_path)?;
    create_dir_all(get_config().general.cache_dir())?;

    Ok(handle)
}

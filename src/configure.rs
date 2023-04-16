use std::fs::File;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use arc_swap::{ArcSwap, Guard};
use figment::providers::{Env, Format, Json, Serialized, Toml};
use figment::Figment;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};

static CONFIG: OnceCell<ArcSwap<Config>> = OnceCell::new();

pub fn init_configure() -> anyhow::Result<()> {
    // 读取.env文件到环境变量
    if let Ok(path) = dotenv::dotenv() {
        tracing::info!("load {}", path.display());
    }
    let config = reload()?;
    // 在跑tests的时候可能会有多个test用到config，所以简单的无视掉重复初始化
    // 但是在正式运行的时候不应该发生这种情况
    #[cfg(test)]
    CONFIG.set(ArcSwap::from_pointee(config)).ok();
    #[cfg(not(test))]
    CONFIG
        .set(ArcSwap::from_pointee(config))
        .expect("CONFIG意外的重复初始化");
    Ok(())
}

#[inline]
pub fn get_config() -> Guard<Arc<Config>> {
    // SAFETY: 在程序一开始就应该已经调用`init_configure`
    unsafe { CONFIG.get_unchecked() }.load()
}

/// 从以下途径读取配置文件(如果有的话)
/// 后加载的会覆盖先加载的
fn reload() -> anyhow::Result<Config> {
    Figment::new()
        // 一个默认的配置文件，里面应该包含所有配置项合理的默认值
        .merge(Toml::string(include_str!("default.toml")))
        .merge(Json::file("_config_auto.json"))
        .merge(Toml::file("config.toml"))
        .merge(Env::prefixed("CP_"))
        .extract::<Config>()
        .map_err(Into::into)
}

/// 将一个新的配置合并到现有的配置文件中
/// 并且可选的持久化到文件中(写入`_config_auto.json`)
/// 以达到运行从webui界面更改配置的效果
fn merge<T>(target: T, persistence: bool) -> anyhow::Result<()>
where
    T: Serialize,
{
    let config = Figment::new()
        .merge(Serialized::defaults(Config::clone(&get_config())))
        .merge(Serialized::defaults(target))
        .extract::<Config>()?;

    if persistence {
        serde_json::to_writer(File::create("_config_auto.json")?, &config)?;
    }

    // SAFETY: 在程序一开始就应该已经调用`init_configure`
    unsafe { CONFIG.get_unchecked() }.store(Arc::new(config));

    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub http: HttpConfig,
    pub general: GeneralConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HttpConfig {
    pub bind: SocketAddr,
    #[serde(with = "humantime_serde")]
    pub system_info_refresh_limit: Duration,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GeneralConfig {
    #[serde(default = "default_app_path")]
    pub app_path: PathBuf,
}

#[inline(always)]
fn default_app_path() -> PathBuf {
    dirs::home_dir().expect("找不到home dir").join(".cat_panel")
}

impl GeneralConfig {
    pub fn cache_dir(&self) -> PathBuf {
        self.app_path.join("cache")
    }

    pub fn components_dir(&self) -> PathBuf {
        self.app_path.join("components")
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Ok;
    use serde_json::json;

    use crate::configure::{get_config, merge};

    #[cp_macros::test]
    async fn test_merge() -> anyhow::Result<()> {
        assert_eq!(get_config().http.bind.port(), 8686);

        merge(
            json!({
                "http": {
                    "bind": "127.0.0.0:65535",
                },
            }),
            false,
        )?;
        assert_eq!(get_config().http.bind.port(), 65535);

        Ok(())
    }
}

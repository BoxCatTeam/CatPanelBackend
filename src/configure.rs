use std::fs::File;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use arc_swap::{ArcSwap, Guard};
use figment::providers::{Env, Format, Json, Serialized, Toml};
use figment::Figment;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};

static CONFIG: OnceCell<ArcSwap<Config>> = OnceCell::new();

pub fn init_configure() -> anyhow::Result<()> {
    if let Ok(path) = dotenv::dotenv() {
        tracing::info!("load {}", path.display());
    }
    let config = reload()?;
    CONFIG
        .set(ArcSwap::from_pointee(config))
        .expect("CONFIG意外的重复初始化");
    Ok(())
}

#[inline]
pub fn get_config() -> Guard<Arc<Config>> {
    // SAFELY: 在程序一开始就应该已经调用`init_configure`
    unsafe { CONFIG.get_unchecked() }.load()
}

fn reload() -> anyhow::Result<Config> {
    Figment::new()
        .merge(Toml::string(include_str!("default.toml")))
        .merge(Json::file("_config_auto.json"))
        .merge(Toml::file("config.toml"))
        .merge(Env::prefixed("CP_"))
        .extract::<Config>()
        .map_err(Into::into)
}

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

    // SAFELY: 在程序一开始就应该已经调用`init_configure`
    unsafe { CONFIG.get_unchecked() }.store(Arc::new(config));

    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub http: HttpConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HttpConfig {
    pub bind: SocketAddr,
    pub system_info_refresh_limit: Duration,
}

#[cfg(test)]
mod tests {
    use anyhow::Ok;
    use serde_json::json;

    use crate::configure::{get_config, init_configure, merge};

    #[test]
    fn test_merge() -> anyhow::Result<()> {
        init_configure()?;
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

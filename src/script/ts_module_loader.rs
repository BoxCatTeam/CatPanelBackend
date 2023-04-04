use std::path::PathBuf;
use std::pin::Pin;

use anyhow::{anyhow, bail};
use deno_ast::{MediaType, ParseParams, SourceTextInfo};
use deno_core::{
    resolve_import, ModuleLoader, ModuleSource, ModuleSourceFuture, ModuleSpecifier, ModuleType,
    ResolutionKind,
};
use futures::FutureExt;
use once_cell::sync::Lazy;
use rust_embed::RustEmbed;
use tokio::fs::read_to_string;
use tracing::debug;

use crate::configure::get_config;
use crate::kv::{DefaultStore, KVStore};

static REMOTE_CACHE: Lazy<DefaultStore> = Lazy::new(|| {
    DefaultStore::new(get_config().general.cache_dir().join("remote_script")).unwrap()
});

#[derive(RustEmbed)]
#[folder = "components"]
struct ComponentScriptEmbed;

pub struct TypescriptModuleLoader;

impl ModuleLoader for TypescriptModuleLoader {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        _kind: ResolutionKind,
    ) -> anyhow::Result<ModuleSpecifier> {
        Ok(resolve_import(specifier, referrer)?)
    }

    fn load(
        &self,
        module_specifier: &ModuleSpecifier,
        _maybe_referrer: Option<ModuleSpecifier>,
        _is_dyn_import: bool,
    ) -> Pin<Box<ModuleSourceFuture>> {
        let module_specifier = module_specifier.clone();
        async move {
            let (path, code) = match module_specifier.scheme().to_ascii_lowercase().as_str() {
                "file" => {
                    let path = module_specifier.to_file_path().unwrap();
                    let code = read_to_string(&path).await?;
                    (path, code)
                }
                "http" | "https" => {
                    let path = PathBuf::from(module_specifier.path());

                    let key = path.display().to_string();
                    let code = if let Some(data) = REMOTE_CACHE.get(&key)? {
                        debug!("get from cache: {}", &key);
                        String::from_utf8(data)?
                    } else {
                        debug!("cache miss, fetched at {}", module_specifier.as_str());
                        let data = reqwest::get(module_specifier.clone()).await?.text().await?;
                        REMOTE_CACHE.insert(&key, data.as_bytes())?;
                        data
                    };

                    (path, code)
                }
                "embed" => {
                    let path = module_specifier.path().trim_start_matches("/");
                    let code = String::from_utf8_lossy(
                        ComponentScriptEmbed::get(path)
                            .ok_or_else(|| anyhow!("embedded file: {} not found", path))?
                            .data
                            .as_ref(),
                    )
                    .to_string();
                    (PathBuf::from(path), code)
                }
                unsupported => bail!("Unsupported protocol {}", unsupported),
            };

            let media_type = MediaType::from_path(&path);
            let (module_type, should_transpile) = match media_type {
                MediaType::JavaScript | MediaType::Mjs | MediaType::Cjs => {
                    (ModuleType::JavaScript, false)
                }
                MediaType::Jsx => (ModuleType::JavaScript, true),
                MediaType::TypeScript
                | MediaType::Mts
                | MediaType::Cts
                | MediaType::Dts
                | MediaType::Dmts
                | MediaType::Dcts
                | MediaType::Tsx => (ModuleType::JavaScript, true),
                MediaType::Json => (ModuleType::Json, false),
                _ => bail!("Unknown extension {:?}", path.extension()),
            };

            let code = if should_transpile {
                let parsed = deno_ast::parse_module(ParseParams {
                    specifier: module_specifier.to_string(),
                    text_info: SourceTextInfo::from_string(code),
                    media_type,
                    capture_tokens: false,
                    scope_analysis: false,
                    maybe_syntax: None,
                })?;
                parsed.transpile(&Default::default())?.text
            } else {
                code
            };
            let module = ModuleSource {
                code: code.into(),
                module_type,
                module_url_specified: module_specifier.to_string(),
                module_url_found: module_specifier.to_string(),
            };
            Ok(module)
        }
        .boxed_local()
    }
}

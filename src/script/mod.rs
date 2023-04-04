use std::rc::Rc;
use std::sync::Arc;

use deno_core::{Extension, ModuleSpecifier};
use deno_runtime::deno_broadcast_channel::InMemoryBroadcastChannel;
use deno_runtime::deno_core::error::AnyError;
use deno_runtime::deno_web::BlobStore;
use deno_runtime::ops::worker_host::CreateWebWorkerArgs;
use deno_runtime::permissions::PermissionsContainer;
use deno_runtime::web_worker::{SendableWebWorkerHandle, WebWorker, WebWorkerOptions};
use deno_runtime::worker::{MainWorker, WorkerOptions};
use deno_runtime::BootstrapOptions;
use futures::task::LocalFutureObj;
use once_cell::sync::Lazy;
use serde::Serialize;
use url::Url;

use ts_module_loader::TypescriptModuleLoader;

use crate::script::ops::{cat_panel_component, Info};

mod ops;
mod ts_module_loader;

#[derive(Debug, Serialize)]
struct CpEnv {
    cp_version: &'static str,
    cp_git_hash: &'static str,
    target_os: &'static str,
    target_arch: &'static str,
    target_family: &'static str,
    target_env: &'static str,
    profile: &'static str,
}

fn extensions() -> Vec<Extension> {
    vec![cat_panel_component::init_ops_and_esm()]
}

static BOOTSTRAP_OPTIONS: Lazy<BootstrapOptions> = Lazy::new(|| BootstrapOptions {
    args: Default::default(),
    cpu_count: std::thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(1),
    debug_flag: Default::default(),
    enable_testing_features: false,
    locale: deno_core::v8::icu::get_language_tag(),
    location: None,
    no_color: !deno_runtime::colors::use_color(),
    is_tty: deno_runtime::colors::is_tty(),
    runtime_version: concat!("deno0.103.0+cp", env!("CARGO_PKG_VERSION")).to_owned(),
    ts_version: "unknown".to_owned(),
    unstable: true,
    user_agent: concat!(
        "cp/",
        env!("CARGO_PKG_VERSION"),
        "+",
        env!("GIT_COMMIT_HASH_SHORT")
    )
    .to_owned(),
    inspect: false,
});

fn create_web_worker(args: CreateWebWorkerArgs) -> (WebWorker, SendableWebWorkerHandle) {
    WebWorker::bootstrap_from_options(
        args.name,
        args.permissions,
        args.main_module.clone(),
        args.worker_id,
        WebWorkerOptions {
            bootstrap: {
                let mut opt = BOOTSTRAP_OPTIONS.clone();
                opt.location = Some(args.main_module);
                opt
            },
            extensions: extensions(),
            startup_snapshot: None,
            unsafely_ignore_certificate_errors: None,
            root_cert_store: None,
            seed: None,
            module_loader: Rc::new(TypescriptModuleLoader),
            npm_resolver: None,
            create_web_worker_cb: Arc::new(create_web_worker),
            preload_module_cb: Arc::new(web_worker_event),
            pre_execute_module_cb: Arc::new(web_worker_event),
            format_js_error_fn: None,
            source_map_getter: None,
            worker_type: args.worker_type,
            maybe_inspector_server: None,
            get_error_class_fn: Some(&|e: &AnyError| {
                deno_runtime::errors::get_error_class_name(e).unwrap_or("Error")
            }),
            blob_store: BlobStore::default(),
            broadcast_channel: InMemoryBroadcastChannel::default(),
            shared_array_buffer_store: None,
            compiled_wasm_module_store: None,
            cache_storage_dir: None,
            stdio: Default::default(),
        },
    )
}

fn web_worker_event(worker: WebWorker) -> LocalFutureObj<'static, anyhow::Result<WebWorker>> {
    LocalFutureObj::new(Box::new(async { Ok(worker) }))
}

fn create_worker(module_path: ModuleSpecifier) -> anyhow::Result<MainWorker> {
    let options = WorkerOptions {
        bootstrap: BOOTSTRAP_OPTIONS.clone(),
        extensions: extensions(),
        startup_snapshot: None,
        unsafely_ignore_certificate_errors: None,
        root_cert_store: None,
        seed: None,
        source_map_getter: None,
        format_js_error_fn: None,
        web_worker_preload_module_cb: Arc::new(web_worker_event),
        web_worker_pre_execute_module_cb: Arc::new(web_worker_event),
        create_web_worker_cb: Arc::new(create_web_worker),
        maybe_inspector_server: None,
        should_break_on_first_statement: false,
        should_wait_for_inspector_session: false,
        module_loader: Rc::new(TypescriptModuleLoader),
        npm_resolver: None,
        get_error_class_fn: Some(&|e: &AnyError| {
            deno_runtime::errors::get_error_class_name(e).unwrap_or("Error")
        }),
        cache_storage_dir: None,
        origin_storage_dir: None,
        blob_store: BlobStore::default(),
        broadcast_channel: InMemoryBroadcastChannel::default(),
        shared_array_buffer_store: None,
        compiled_wasm_module_store: None,
        stdio: Default::default(),
    };

    let permissions = PermissionsContainer::allow_all();
    let mut worker = MainWorker::bootstrap_from_options(module_path, permissions, options);
    worker.execute_script(
        "[set env]",
        format!(
            "globalThis.cp_env = JSON.parse(String.raw`{}`);",
            serde_json::to_string(&CpEnv {
                cp_version: env!("CARGO_PKG_VERSION"),
                cp_git_hash: env!("GIT_COMMIT_HASH_SHORT"),
                target_os: env!("TARGET_OS"),
                target_arch: env!("TARGET_ARCH"),
                target_family: env!("TARGET_FAMILY"),
                target_env: env!("TARGET_ENV"),
                profile: env!("PROFILE"),
                //component_dir: get_config().general.app_path.join("component"),
            })?
        ),
    )?;
    worker.execute_script("[built-in code]", include_str!("built-in.js"))?;

    Ok(worker)
}

#[cp_macros::test(log = "DEBUG")]
async fn test_php_info() -> anyhow::Result<()> {
    //let main = resolve_path("components/php/info.ts", current_dir()?.as_path())?;
    let main = Url::parse("embed:/php/info.ts")?;
    let mut worker = create_worker(main.clone())?;
    worker.execute_main_module(&main).await?;
    worker.run_event_loop(false).await?;

    dbg!(worker.js_runtime.op_state().borrow().borrow::<Info>());

    Ok(())
}

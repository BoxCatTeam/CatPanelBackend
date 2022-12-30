# CatPanelBackend

## 国内镜像
[字节跳动](https://rsproxy.cn)

## 环境&工具
- nightly rust

`rustup toolchain install nightly`

- rustfmt

`rustup component add rustfmt`

- clippy

`rustup component add clippy`

- outdated (可选)(自动检查依赖更新)

`cargo install cargo-outdated`

## 命令
- 调试运行

`cargo run`

- 编译

`cargo build --release`

- Clippy

`cargo clippy -- -D warnings`

- 格式化

`cargo fmt`

- 检查是否正确格式化

`cargo fmt --check`

- 检查依赖更新

`cargo outdated`

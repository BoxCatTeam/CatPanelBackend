# CatPanelBackend

## 文档
### 编译时间分析
[cargo timing](https://boxcatteam.github.io/CatPanelBackend/timings/cargo-timing.html)
### 代码文档
todo
### Graphql
todo

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
- 运行测试

`cargo test -- --test-threads=1`

***因为部分测试依赖不同环境变量并且共用全局的设置/日志器,
所以并行测试可能会引起奇怪的错误***

- 调试运行

`cargo run`

- 编译

`cargo build --release`

- Clippy

`cargo clippy -- -D warnings`

**开发过程中会有部分东西写了但是还没写使用部分，所以开发的时候可以改为**

`cargo clippy -- -D warnings -A unused`

- 格式化

`cargo fmt`

- 检查是否正确格式化

`cargo fmt --check`

- 检查依赖更新

`cargo outdated`

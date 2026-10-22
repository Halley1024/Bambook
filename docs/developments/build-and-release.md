# 开发环境、平台适配与打包发布

[返回技术总览](README.md)

## 001 · 配置入口

| 文件 | 用途 |
| --- | --- |
| [package.json](../../package.json) | 前端构建和 Tauri CLI 脚本 |
| [Cargo.toml](../../backend/Cargo.toml) | Rust 与本地原生依赖 |
| [tauri.conf.json](../../backend/tauri.conf.json) | 通用应用、资源和构建钩子 |
| [Windows 配置](../../backend/tauri.windows.conf.json) | Windows 覆盖项与 NSIS |
| [Linux 配置](../../backend/tauri.linux.conf.json) | Linux 覆盖项与 deb / AppImage |
| [tauri.mjs](../../scripts/tauri.mjs) | 指定 backend 与前端工具链位置，转发 Tauri CLI 参数 |
| [MuPDF 构建脚本](../../backend/native/mupdf-sys/build.rs) | Windows 本地 libclang 自动发现 |
| [原生依赖说明](../../backend/native/README.md) | MuPDF 与 Windows libclang 的维护方式 |

当前文档只覆盖 Windows 与 Linux。Linux 已有配置，不代表桌面调用和安装包已经全部验收。

## 002 · 环境准备

通用环境使用 Node.js 22+、npm、Rust 稳定工具链，并确保 `node`、`npm`、`cargo` 能从终端调用。依赖版本以仓库清单与 lock 文件为准，首次依赖安装可能需要网络。

Windows 需要 MSVC Rust 工具链、Visual Studio C++ Build Tools、Windows SDK 和 WebView2。缺少仓库本地 libclang 时，在项目根目录执行：

```powershell
python -m pip install libclang==18.1.1 --target backend/native/libclang --no-compile
```

该目录由 `.gitignore` 排除，普通新检出不应假定存在。上述恢复命令还需要 Python 与 pip。

Linux 需要 C/C++ 工具链、Make、pkg-config、Clang / libclang，以及 Tauri 所需的 GTK 3、WebKitGTK 4.1 等开发依赖。具体包名依发行版确定；当前没有经过各发行版验证的统一安装脚本。

无需 `.cargo/config.toml`：Windows 构建脚本会自动查找 `backend/native/libclang/clang/native/libclang.dll`，Linux 使用系统 libclang。需要手动指定时，设置本机实际包含 libclang 动态库的目录：

```sh
# 将占位路径替换为本机实际目录后执行。
export LIBCLANG_PATH="/实际的/libclang/目录"
```

MuPDF 通过 `native/mupdf-rs` 和 `native/mupdf-sys` 的本地源码编译并静态链接。libclang 是 bindgen 的编译期依赖，应用运行时不需要随包加载它。

## 003 · 安装依赖与开发

以下命令在项目根目录执行：

```sh
npm ci
npm run tauri -- dev
```

Tauri 通过 `beforeDevCommand` 启动 Vite，访问 `http://localhost:1420`。仅运行 `npm run dev` 可检查前端页面，但浏览器中不能完整运行原生文档能力。

Tauri 命令通过 `scripts/tauri.mjs` 设置 `TAURI_APP_PATH=backend`，并将包含 package.json 的项目根目录作为 `TAURI_FRONTEND_PATH`。`npm run tauri:dev` 复用该入口；`tauri:build:windows` 用于在 Windows 上生成 NSIS 安装包。

## 004 · 构建与验证

```sh
npm run build
cargo check --manifest-path backend/Cargo.toml
cargo test --manifest-path backend/Cargo.toml
```

前端输出到 `dist/`；桌面构建通过 `beforeBuildCommand` 自动执行前端构建。各检查的验证范围见[测试文档](testing.md)。

## 005 · 平台打包

在目标系统准备环境后执行：

```sh
npm run tauri -- build
```

Tauri 自动合并通用配置与对应平台配置。Windows 配置 NSIS；Linux 配置 deb 与 AppImage。当前未建立在单台主机上完整交叉生成所有平台安装包的流程。

不覆盖 target 或 Cargo 输出目录时，产物位于：

```text
backend/target/release/
├── bambook.exe 或 bambook
└── bundle/
    ├── nsis/                   Windows
    ├── deb/                    Linux
    └── appimage/               Linux
```

使用 `--target` 或 `CARGO_TARGET_DIR` 后实际目录会变化。打包资源由 `bundle.resources` 指定，当前包含 `resources/**/*`，不包含整个 `native/` 源码和构建工具目录。

## 006 · 当前平台差异

后端系统操作已集中到 `backend/src/platform/desktop/`。Windows 使用 `explorer.exe` 和 `rundll32.exe`，Linux 使用 `xdg-open`，定位文件在 Linux 中打开其父目录。系统数据目录入口复用同一适配器。

本轮进行了 Linux 源码编译与自动测试，没有实际启动系统文件管理器或浏览器，也没有生成和安装跨平台安装包。WebView、文件对话框、字体、窗口关闭及导出效果仍需要桌面验收。

## 007 · 发布流程与记录

1. 核对 `package.json`、Cargo 与 Tauri 配置中的应用版本，平台覆盖配置也要同步。
2. 完成与变更相应的构建、测试及桌面流程验收，记录失败项。
3. 在目标系统生成安装包，检查随包资源与[第三方声明](../../backend/resources/THIRD_PARTY_NOTICES.md)。
4. 在没有开发工具链的测试环境安装，验证打开、编辑保存、批注重开、导出和系统入口。
5. 记录安装包名称、目标平台、构建版本、校验值和已知限制，再交付对应产物。

以上是建议执行的发布流程。当前没有完整的自动发布、签名、自动更新及全平台安装包验收流水线；不能把本说明视为这些设施已经搭建。

## 008 · 常见问题定位

| 现象 | 优先检查 |
| --- | --- |
| `cargo` 找不到 | 工具链安装和终端 PATH |
| bindgen 找不到 libclang | `.cargo/config.toml`、实际文件与 `LIBCLANG_PATH` |
| MuPDF 原生构建失败 | C/C++ 工具链、Make / MSBuild、源码与完整构建日志 |
| Linux GTK / WebKit 报错 | 对应发行版开发依赖与 pkg-config 查询结果 |
| Vite 1420 端口占用 | 是否已有开发进程，当前配置启用 strictPort |
| 页面能打开但文件能力失败 | 是否在 Tauri 中运行、command 是否注册、路径与原生依赖 |
| Linux 系统入口失败 | 是否仍调用 Windows 专用程序 |

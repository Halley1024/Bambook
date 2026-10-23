# Bambook

## 001 · 软件定位

Bambook 是一款以本地文档为中心的 PDF 阅读与 Markdown 写作应用，将资料阅读、笔记整理和文档编写放在同一个桌面工作区中，适用于学习、研究和日常写作。

PDF 保留原始版式，侧重阅读和批注；Markdown 侧重编辑、预览和导出。两类文档通过统一的标签页与文档库管理。

## 002 · 软件功能

- **统一工作区**：PDF / Markdown 混合标签页、文件夹导航、文档重命名、最近访问和会话恢复。
- **PDF 阅读**：页面缩放、连续滚动、缩略图、目录导航、全文搜索和文字选择。
- **批注与书签**：高亮、下划线等文字标记、区域批注、笔记、书签及阅读位置恢复，支持笔记导出为 Markdown。
- **Markdown 写作**：源码编辑、预览与分栏显示、常用格式插入、目录导航、同步滚动、搜索以及 HTML / PDF 导出。
- **本地文档管理**：文档库、文件副本或源文件链接管理、文档包导出，以及未保存文档的关闭提示和未命名 Markdown 恢复。
- **辅助阅读**：配置兼容 Chat Completions 的本地或远程模型服务，结合 PDF 选中文字进行问答。

项目仍在开发中，以上为当前代码涵盖的主要能力，具体交互与导出效果仍在完善。PDF 批注、书签和阅读状态以附属数据保存。

## 003 · 技术栈

| 层次          | 技术                                | 用途                             |
| ------------- | ----------------------------------- | -------------------------------- |
| 前端          | React 18、TypeScript、Vite 6        | 工作区界面、交互与前端构建       |
| 桌面框架      | Tauri 2                             | 原生窗口、系统能力与前后端通信   |
| 后端          | Rust                                | 文档会话、文件访问、搜索与持久化 |
| PDF 引擎      | MuPDF（本地维护的 Rust / FFI 封装） | PDF 解析、页面渲染和文字提取     |
| Markdown 引擎 | pulldown-cmark                      | Markdown 解析与结构化文档输出    |
| 数据与服务    | Serde / JSON、Reqwest               | 本地数据存储与外部服务请求       |

## 004 · 支持平台

| 平台    | 当前状态                               | 打包格式                |
| ------- | -------------------------------------- | ----------------------- |
| Windows | 现有主要开发平台，已配置桌面构建       | NSIS 安装程序（`.exe`） |
| Linux   | 已加入平台配置，适配与安装包验证进行中 | `.deb`、`.AppImage`     |

## 005 · 开发和部署流程

### 1. 准备环境

- 通用：Node.js 22+、npm、Rust 稳定工具链，确保 `node`、`npm` 和 `cargo` 可从终端调用。
- Windows：MSVC 工具链、Visual Studio C++ Build Tools、Windows SDK 和 WebView2 Runtime。
- Linux：C/C++ 编译工具、Make、pkg-config、Clang / libclang，以及 Tauri 2 所需的 GTK 3、WebKitGTK 4.1 等系统开发依赖。

MuPDF 从仓库内的源码编译并静态链接。Windows 缺少本地 libclang 时，可在项目根目录恢复：

```powershell
python -m pip install libclang==18.1.1 --target src-tauri/native/libclang --no-compile
```

无需 `.cargo/config.toml`：Windows 上，MuPDF 构建脚本会自动使用 `src-tauri/native/libclang/clang/native/libclang.dll`；本地 DLL 不存在时，使用系统 libclang 查找机制。Linux 使用系统 libclang。手动设置的 `LIBCLANG_PATH` 优先于自动查找。

Windows 的 C/C++ 编译器与 MSBuild 由构建依赖自动定位，需安装 Visual Studio 2022 Build Tools 的“使用 C++ 的桌面开发”组件及 Windows SDK。Rust 的 `cargo`、`rustc` 则需通过安装程序加入 `PATH`，默认目录为 `%USERPROFILE%\.cargo\bin`。

### 2. 安装依赖并启动

以下命令均在项目根目录执行：

```sh
npm ci
npm run tauri -- dev
```

只调试前端界面时可运行 `npm run dev`，浏览器地址为 `http://localhost:1420`；文件访问、PDF 渲染等原生能力需在 Tauri 应用中使用。

### 3. 构建与检查

```sh
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

`npm run build` 执行 TypeScript 检查并构建前端；Cargo 命令检查和测试 Rust 后端。桌面交互与安装包仍需在目标平台验证。

### 4. 打包与分发

在对应平台完成环境准备后执行：

```sh
npm run tauri -- build
```

Tauri 会自动构建前端并合并对应平台配置：Windows 使用 `src-tauri/tauri.windows.conf.json`，Linux 使用 `src-tauri/tauri.linux.conf.json`。

默认构建产物位于 `src-tauri/target/release/bundle/`：

- Windows：`nsis/` 下的安装程序。
- Linux：`deb/` 和 `appimage/` 下的安装包。

分发前，在目标系统安装并验证文档打开、编辑保存、批注恢复与导出流程，再交付对应安装包。

详细业务规则、架构设计和开发说明参见[技术开发文档](docs/developments/README.md)。

前端源码位于 `src/`，Rust/Tauri 工程位于 `src-tauri/`，其中 Rust 源码位于 `src-tauri/src/`。package.json、Vite/TypeScript 配置与 index.html 保留在项目根目录。npm 脚本直接调用 Tauri CLI，按默认目录约定定位后端工程。

006 · 感谢

本项目采用了开源的[mupdf]()

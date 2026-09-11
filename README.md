# Bambook

## 产品介绍

Bambook 是一款面向 Windows 的轻量级桌面文档工具，使用 Tauri、React、TypeScript 与 Rust 开发。应用以统一标签页承载 PDF 和 Markdown 文档：打开 PDF 时进入原貌阅读工作区，打开 Markdown 时进入编辑与预览工作区，让阅读、检索、摘录和写作能够在同一个窗口中完成。

PDF 由 Rust 后端通过 MuPDF 解析页面、文字层和目录等结构化数据，前端负责虚拟化渲染与交互；Markdown 由后端维护文档会话、AST、目录和搜索数据，前端负责编辑、预览与导航。设置、最近文件、关闭记录、阅读状态和工作区状态使用本地 JSON 持久化，默认保存在 `%LOCALAPPDATA%\Bambook`。

[homepage][]

## 功能

### 统一文档工作区

- 根据文件类型自动切换 PDF 或 Markdown 工作区，无需手动选择模式。
- 支持 PDF 与 Markdown 混合标签页、切换、关闭、关闭其他标签和拖动排序。
- 支持拖放打开文档、未保存状态提示、最近文件、恢复最后关闭的文件及上次会话恢复。
- 未保存文档关闭或退出时提供“保存 / 不保存 / 取消”事务，并在多文档场景中逐个处理。

### PDF 阅读

- 使用 MuPDF 原貌渲染 PDF 页面，并提供透明文字层、目录和页面缩略图。
- 支持连续页阅读、50%～250% 缩放、适合阅读区缩放、水平与垂直滚动。
- 使用页面虚拟化、LRU 缓存和资源释放机制控制长文档滚动时的内存占用。
- 支持目录与缩略图导航、阅读位置恢复、全文搜索及匹配位置定位。
- 支持文字选择、复制、下划线、波浪线、删除线、高亮、笔记、颜色调整、撤销与重做等批注交互。
- PDF 批注、笔记、书签和阅读状态采用独立数据持久化；将批注真正写入 PDF 文件仍属于后续能力。

### Markdown 编辑

- 支持新建、打开、编辑、保存、另存为和磁盘文件重命名。
- 提供源码编辑、分栏和预览三种布局。
- 基于后端 Markdown AST 生成目录和结构化预览，不使用不受控 HTML 注入渲染正文。
- 支持点击目录同步定位编辑区与预览区，以及分栏模式下的双向同步滚动。
- 支持全文搜索、语法帮助和 HTML/PDF 基础导出。
- Markdown 文档不加载或保存 PDF 批注数据。

### 本地数据

- 最近文件、关闭文件、设置、工作区状态和阅读状态由 Rust 后端持久化。
- 后端可检测最近文件是否不存在、不可访问或不是普通文件，并在界面中显示失效状态。
- JSON 数据通过临时文件、磁盘同步和同目录替换进行原子写入，降低异常退出导致的数据损坏风险。

## 部署流程

### 1. 环境要求

- Windows 10/11 x64。
- Node.js 18 或更高版本，并附带 npm。
- Rust stable（MSVC 工具链）。
- Visual Studio 2022 Build Tools，安装“使用 C++ 的桌面开发”和 Windows SDK。
- Microsoft Edge WebView2 Runtime。
- Python 3，用于恢复项目未纳入 Git 的 `libclang` 构建依赖。

### 2. 获取源码并安装依赖

```powershell
git clone <repository-url> Bambook
cd Bambook
npm install
python -m pip install libclang==18.1.1 --target src-tauri/native/libclang --no-compile
```

MuPDF Rust 封装、FFI crate、MuPDF C 源码及第三方许可证已经放在 `src-tauri/native` 与 `src-tauri/resources` 中。`libclang.dll` 是 MuPDF 绑定生成时使用的构建工具，不进入 Git 仓库和最终安装包，因此首次构建前需要执行上面的恢复命令。

### 3. 本地开发

```powershell
npm run tauri:dev
```

仅调试 React 前端时可以运行：

```powershell
npm run dev
```

### 4. 构建检查

```powershell
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

### 5. 生成 Windows 安装包

```powershell
npm run tauri:build:windows
```

构建完成后，NSIS 安装包位于 `src-tauri/target/release/bundle/nsis`。`dist`、`node_modules` 和 `src-tauri/target` 均为本地生成目录，

> MuPDF 采用 AGPL-3.0 许可证。公开发布或分发 Bambook 前，请确认项目自身许可证、源码公开方式和第三方许可证声明满足相应义务。

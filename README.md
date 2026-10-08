# Bambook

## 1、软件简介

Bambook 是一款面向 Windows、MacOS 桌面平台以及Android、IOS平板平台的 PDF 阅读与 Markdown 编辑软件，使用 Tauri v2、React、TypeScript 和 Rust 开发，目前主要是实现在Windows平台下的桌面应用开发。

应用通过当前活动标签页的文件类型来决定切换是否 PDF/Markdown 模式：PDF 标签显示固定版式阅读界面，Markdown 标签显示编辑与预览界面，两类文档可以同时存在于同一组标签页中。

核心定位：

- PDF 采用原貌阅读模式，正文不可编辑；用户可以搜索、添加笔记、书签和文字批注。
- Markdown 是可编辑文档，由 Rust 后端解析为类型化 AST，前端负责编辑、预览和样式渲染。
- Rust 后端负责可信文件访问、文档解析、搜索、会话、保存、导出与持久化。
- React 前端负责布局、标签、工具栏、侧栏、选区、动画、主题和用户交互。
- 代码和领域模型使用 `Annotation`/“批注”，用户界面统一显示为“笔记”。

### 设计原则

1. 文件类型决定工作区，不保存额外的 PDF/Markdown 模式状态。
2. PDF、Markdown 业务相互隔离，共用文档标签、搜索外壳、设置、通知等基础能力。
3. React 组件不能直接承担可信文件访问或文档解析，Tauri IPC 统一封装在 repository/platform 层。
4. 后端采用 domain、application、infrastructure、commands 分层，第三方解析器类型不能越过基础设施边界。
5. PDF 页面、文字坐标、批注坐标和搜索结果使用同一套 MuPDF 页面空间。
6. Markdown 后端返回 AST，不返回供前端直接注入的拼接 HTML。
7. 未命名文档、保存、关闭和退出必须拥有完整的取消与失败处理，不能只实现按钮入口。
8. README 中勾选的功能必须经过真实业务流程验证，编译通过或菜单可见不等于功能完成。

### 默认数据位置

应用数据统一保存在：

```text
%LOCALAPPDATA%\Bambook\
├─ storage\
│  ├─ pdf-id\
│  │  ├─ document.pdf
│  │  ├─ metadata.json
│  │  ├─ annotations.json
│  │  ├─ bookmarks.json
│  │  └─ reading-state.json
│  └─ markdown-id\
│     ├─ assets\
│     ├─ metadata.json
│     └─ markdown.md
├─ themes\
├─ settings.json
├─ document-library.json
├─ recent-documents.json
├─ closed-documents.json
└─ workspace-state.json
```

- `storage` 按稳定文档 ID 保存托管文档及其附属数据。
- PDF 与 Markdown 可分别指定存储根目录；每个根目录下仍使用稳定的 `pdf-id` / `markdown-id` 子目录。
- 打开外部文档只建立内存会话，不创建文档 ID 目录；首次保存时才按设置入库。“添加文件副本”会保存副本，“仅添加链接”只记录绝对源路径。
- 链接失效后可重新选择文件；文件名不一致时必须再次确认，确认后同步更新索引与 metadata。
- 打开 PDF 时，批注、书签和阅读状态优先读取 PDF 所在目录中的同名 JSON 侧车文件；不存在时再读取 Bambook 托管目录，两处都不存在则保持空状态且不创建文件。
- `themes` 独立保存用户导入的 CSS 主题。
- HTML/PDF 导出不设置默认导出目录，每次都由用户选择目标路径。
- 所有数据路径最终都由 Rust 后端创建、校验和访问。

## 2、业务需求和软件功能

任务状态说明：

- `[x]`：当前代码已经实现，并有相应构建或测试依据。
- `[ ] 🟡`：已经部分实现，但尚未满足最终业务规则或仍需人工验收。
- `[ ]`：尚未实现。

### 2.1 统一文档工作区

- [X] 根据活动标签的文件类型显示 PDF 或 Markdown 工作区。
- [X] PDF/Markdown 混合标签页。
- [X] 标签切换、关闭、关闭其他标签和拖动排序的基础状态。
- [X] 后端识别 PDF 文件签名和 Markdown 扩展名。
- [X] PDF/Markdown 后端 Session 生命周期基础接口。
- [X] 未命名 Markdown 标签已接入，编号按已打开文档智能去重，首次保存选择路径后更新标签状态。
- [X] 标签页右键重命名。
- [X] 已保存 Markdown 的磁盘文件原子重命名。
- [X] PDF 重命名及关联数据迁移。
- [X] 标签页未保存圆点。
- [X] 文档拖放打开。
- [X] 恢复上次标签页会话（已恢复有磁盘路径的文档，未命名 Markdown 内容持久化并启动时询问恢复）。

### 2.2 关闭与退出

- [X] 统一关闭事务协调器（`features/documents/close/closeCoordinator.ts`），三个关闭入口统一走两阶段事务。
- [X] “保存 / 不保存 / 取消”三种关闭决策的完整事务流程。
- [X] 多个未保存文档逐个询问。
- [X] 询问前激活对应标签并跳转到合适视图。
- [X] 用户取消退出时，之前选择“不保存”的文档仍保留全部内容（询问阶段不做任何破坏性操作）。
- [X] 所有决策完成后统一释放后端 Session。
- [ ] 🟡 多窗口分别管理其持有的文档 Session（React 树与后端 Session 已按窗口隔离，多窗口 UI 入口与持久化按窗口隔离待完成）。
- [X] 应用崩溃后的未保存 Markdown 恢复。

### 2.3 文件夹工作区

- [ ] 在应用内部打开文件夹工作区。
- [ ] 文件树只展示支持的 PDF、Markdown 和相关资源。
- [ ] 点击文件树节点打开或激活对应标签。
- [ ] 文件创建、删除、重命名后的文件树刷新。
- [ ] PDF 左侧栏提供“文件 / 视图 / 目录”。
- [ ] Markdown 左侧栏提供“文件 / 目录”。
- [ ] 左右侧栏拖动调整宽度。
- [ ] 左右侧栏最小、最大宽度限制。
- [ ] 中间文档区域最小宽度保护。
- [ ] 侧栏宽度由 Rust 后端持久化。

> 当前菜单中的“打开文件夹”还不是完整的应用内文件夹工作区，因此不能视为已完成。

### 2.4 最近文件与关闭文件

- [X] 将最近文件从前端 `localStorage` 迁移到 Rust 后端。
- [X] 使用 `%LOCALAPPDATA%\Bambook\recent-documents.json` 持久化。
- [X] PDF 和 Markdown 按最后关闭时间排序。
- [X] 最近文件按 PDF/Markdown 分类显示。
- [X] 后端实时检测文件不存在、无法访问和路径不是文件，前端显示失效状态并禁用打开。
- [X] 重新打开最后关闭的一个文件。
- [X] 恢复成功后弹出关闭文件栈顶部记录。
- [X] 清除所有最近文件和关闭记录，但不删除磁盘文档。
- [ ] 🟡 右侧弹层数据和失效状态已完成，完整键盘操作与人工验收待完成。

### 2.5 PDF 阅读与性能

- [X] MuPDF 后端打开和认证 PDF。
- [X] MuPDF PNG 页面原貌渲染。
- [X] MuPDF 页面文字结构与目录协议。
- [X] PDF 页面虚拟化，只渲染当前页附近页面。
- [X] 页面结构、图片 LRU 缓存和 Object URL 释放。
- [X] 50%～250% 缩放。
- [X] 一键恢复到适合阅读区域的比例。
- [X] 独立水平、垂直滚动区域。
- [X] 按页面可见面积判断当前页。
- [X] 左上角视口锚点缩放。
- [ ] 🟡 大型 PDF 反复滚动的长期性能验收。
- [ ] 页面旋转。
- [ ] 单页阅读模式。
- [X] 连续页阅读模式。
- [ ] 双页阅读模式。
- [ ] 阅读区纸张底色与保护色。

### 2.6 PDF 左侧导航

- [X] 页面缩略图列表。
- [X] PDF 目录树和折叠动画。
- [X] 点击缩略图跳转到页面顶部。
- [X] 点击目录跳转到标题附近并居中显示。
- [X] 当前目录标题联动强调。
- [ ] PDF 左侧“文件 / 视图 / 目录”三项结构。
- [ ] 视图缩略图中的书签标识。
- [ ] 同页多个书签数量显示。
- [ ] PDF 内部链接热点。
- [ ] 外部链接打开确认。
- [ ] 阅读历史前进、后退。
- [X] 使用 `reading-state.json` 防抖保存并恢复页码、页内位置、缩放模式和缩放比例。

### 2.7 PDF 搜索

- [X] PDF/Markdown 通用搜索请求与结果外壳。
- [X] Unicode NFKC、大小写、空白折叠和原始位置映射。
- [X] PDF 关键词匹配和 MuPDF 字符矩形映射。
- [X] requestId、防抖、过期响应丢弃和取消协议。
- [X] PDF 搜索独立 worker 和索引缓存。
- [X] 搜索结果只传递给虚拟化范围内页面。
- [ ] 🟡 搜索结果列表、当前命中样式和居中导航需要继续人工验收。
- [ ] 500 页以上 PDF 的性能验收。
- [ ] 扫描版 PDF 无文字提示。
- [ ] OCR 搜索。

### 2.8 PDF 文字选择、批注和笔记

- [X] MuPDF 透明文字选择层。
- [ ] 🟡 复制选中文字。
- [ ] 🟡 下划线、波浪线、删除线和高亮。
- [ ] 🟡 批注颜色预设、调色盘和颜色代码。
- [ ] 🟡 批注类型转换和删除。
- [ ] 🟡 右键文字选区菜单。
- [ ] 🟡 笔记图标与右侧笔记栏。
- [ ] 🟡 笔记按 PDF 位置排序。
- [ ] 🟡 点击笔记后将对应区域滚动到视口中部。
- [X] 批注撤销与重做，提供独立工具栏按钮及 Ctrl+Z、Ctrl+Y 快捷键。
- [X] 同页多条批注笔记合并，保留多段标记坐标并合并摘录和笔记正文。
- [X] 笔记输入防抖，保存前强制刷新编辑草稿；未执行保存时不写磁盘。
- [X] 批注 schema v2、旧数组迁移、有效文件备份和损坏恢复。
- [X] 本地侧车与 Bambook 托管批注同时存在时，根据 JSON 根级 `updatedAt` 选择较新数据。
- [X] 应用内部批注 JSON 基础持久化。
- [X] 笔记 Markdown 导出基础协议。

> 上述批注交互曾出现重复批注、选区残留、点击跳顶和颜色卡顿等问题，完成重新验收前统一标为部分实现。

### 2.9 PDF 书签

- [ ] 顶部工具栏书签按钮。
- [ ] 为当前页和页内纵向位置创建书签。
- [ ] 删除和重命名书签。
- [ ] 点击书签跳转到对应页面位置。
- [ ] 在缩略图中显示书签及页码。
- [X] 将书签独立保存到文档目录的 `bookmarks.json`。
- [X] 书签变更触发标签未保存圆点，并接入统一关闭保存事务。

### 2.10 PDF 保存

- [ ] 使用 MuPDF 将应用批注真正写入 PDF annotation objects。
- [X] 首次保存 PDF 时按设置复制托管副本或记录源文件链接；仅打开不会创建 `pdf-id` 目录。
- [X] “另存为文档包”由用户选择父目录，并以原 PDF 文件名建立目录，打包源文件、metadata、批注、书签和阅读状态。
- [X] 使用稳定文档 ID 关联托管 PDF、批注、笔记、书签和阅读状态。
- [ ] `KeepOriginalAndAnnotated`：保留原始 PDF 和带批注 PDF。
- [ ] `AnnotatedOnly`：应用目录只保留带批注 PDF，不删除用户外部原文件。
- [ ] 临时文件写入、重新打开校验和原子替换。
- [ ] 加密 PDF 权限与保存失败回滚。
- [ ] PDF 保存成功后清除标签未保存圆点。

### 2.11 Markdown 文档生命周期

- [X] 后端 Markdown 内存 Session 和 revision。
- [X] 打开、编辑和保存已有 Markdown 文件。
- [X] 后端 AST、目录、UTF-16 范围和诊断。
- [ ] 🟡 新建“未命名 1、未命名 2……”标签。
- [ ] 🟡 未命名文档首次保存时选择路径。
- [ ] 🟡 Markdown 另存为。
- [ ] 已保存 Markdown 的磁盘文件原子重命名。
- [ ] 外部程序修改文件后的冲突检测。
- [ ] 自动保存与崩溃恢复。
- [ ] 标签未保存圆点。

### 2.12 Markdown 编辑、预览与搜索

- [X] Markdown 源码编辑器。
- [ ] 新增类似Tyopra这种非源码编辑器
- [X] 编辑、分栏和预览三种布局。
- [X] React AST renderer，不使用 `dangerouslySetInnerHTML` 渲染文档。
- [X] Markdown 标题目录。
- [X] 点击 Markdown 目录同时定位源码与预览标题。
- [X] Markdown 分栏模式下编辑区与预览区双向同步滚动。
- [X] Markdown 语法帮助面板。
- [ ] 🟡 文件树与当前目录联动。
- [ ] 🟡 常用 Markdown 语法插入。
- [X] Markdown 后端全文搜索协议。
- [X] Markdown 不加载或保存应用批注，批注与笔记仅归属 PDF。
- [ ] 🟡 编辑器定位和预览搜索强调。
- [ ] 图片资源管理。
- [ ] 数学公式正式渲染。
- [ ] Mermaid 图表。
- [ ] 完整脚注显示。

### 2.13 Markdown HTML/PDF 导出

- [X] HTML/PDF 导出 IPC 和 exporter 基础结构。
- [X] 基础 HTML 和 PDF 文件生成测试。
- [ ] 完整 HTML 导出业务流程验收。
- [X] 每次导出手动选择路径，不保存默认导出位置。
- [ ] CSS 主题导入、列出、选择、删除和同名处理。
- [ ] 使用 CSS 文件名去掉 `.css` 作为主题名称。
- [ ] HTML 图片默认复制到 `{文件名}_assets`。
- [ ] HTML 图片全部嵌入 Data URI 的可选策略。
- [ ] 本地图片冲突、失败清理和路径安全检查。
- [ ] PDF 中文字体完整性。
- [ ] PDF 图片嵌入。
- [ ] PDF 表格和跨页排版。
- [ ] PDF 数学公式预渲染。
- [ ] PDF 代码语法高亮。
- [ ] 用户 CSS 对 HTML/PDF 的一致应用。
- [X] PDF 强制扩展名、临时文件、重新打开验证和原子替换。

### 2.14 设置、帮助与通知

- [X] 全局 Toast 通知。
- [X] 设置 Provider 和基础 JSON 持久化。
- [X] 将数据根目录迁移到 `%LOCALAPPDATA%\Bambook`。
- [ ] PDF 保存策略设置。
- [ ] 五类业务目录设置与迁移工具。
- [ ] CSS 主题管理设置。
- [ ] 侧栏宽度设置。
- [ ] 设置数据版本迁移。
- [ ] 鸣谢。
- [ ] 更新日志。
- [ ] 留言反馈。
- [ ] 关于页面。

## 3、技术栈

### 前端

| 技术                 | 用途                         |
| -------------------- | ---------------------------- |
| React 18             | 工作区、标签、侧栏和交互视图 |
| TypeScript           | 前端类型与 DTO 映射          |
| Vite                 | 开发服务器和前端构建         |
| Lucide React         | 工具栏、菜单和状态图标       |
| CSS                  | 主题、布局、动画和打印样式   |
| Tauri JavaScript API | 窗口和桌面能力适配           |
| Tauri Dialog Plugin  | 文件、目录和保存路径选择     |

### 后端

| 技术                  | 用途                                     |
| --------------------- | ---------------------------------------- |
| Rust 2021             | 文档业务、文件访问和持久化               |
| Tauri v2              | Windows 桌面外壳与 IPC                   |
| MuPDF 0.8             | PDF 打开、解析、渲染、搜索及后续批注写入 |
| pulldown-cmark 0.13   | Markdown AST 与 HTML 基础输出            |
| serde / serde_json    | IPC DTO 和 JSON 数据                     |
| unicode-normalization | 搜索 NFKC 标准化                         |

### Windows 开发环境

- Node.js 18+
- Rust MSVC 工具链
- Visual Studio 2022 C++ Build Tools
- Windows SDK
- WebView2 Runtime
- NSIS（安装包）

### 常用命令

```powershell
npm install
npm run tauri:dev
```

```powershell
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

仅在需要验证安装包时执行：

```powershell
npm run tauri:build:windows
```

## 4、项目模块总览

### 4.1 业务模块

```text
Bambook
├─ 应用壳 App Shell
│  ├─ 顶部应用标识
│  ├─ 活动文档工具栏
│  ├─ 文件 / 设置 / 帮助菜单
│  └─ 工作区宿主
│
├─ 统一文档 Documents
│  ├─ PDF/Markdown 混合标签
│  ├─ 活动文档
│  ├─ 打开、关闭、排序和重命名
│  ├─ 未保存状态
│  ├─ 关闭事务协调器              [规划]
│  ├─ 文件夹工作区                [规划]
│  └─ 最近文件与关闭文件栈        [后端化完成]
│
├─ PDF 工作区
│  ├─ MuPDF Session
│  ├─ 页面结构和原貌渲染
│  ├─ 页面虚拟化与缓存
│  ├─ 文字选择
│  ├─ 缩略图和目录
│  ├─ 全文搜索
│  ├─ 批注与笔记
│  ├─ 书签                        [规划]
│  └─ PDF 批注写入与保存          [规划]
│
├─ Markdown 工作区
│  ├─ 内存编辑 Session
│  ├─ Markdown AST
│  ├─ 源码编辑与预览
│  ├─ 文件树和目录
│  ├─ 全文搜索
│  ├─ 文档重命名                  [规划]
│  ├─ CSS 主题                    [规划]
│  └─ HTML/PDF 完整导出           [待重构]
│
├─ 公共业务
│  ├─ Search 搜索协议
│  ├─ Annotations 批注领域
│  ├─ Settings 设置
│  ├─ Notifications 通知
│  └─ Help 帮助
│
└─ 本地持久化
   ├─ Storage 路径与目录          [基础完成]
   ├─ 最近文件                    [已后端化]
   ├─ 工作区状态                  [基础完成]
   ├─ 批注 JSON
   └─ CSS 主题                    [规划]
```

### 4.2 前端目录

```text
src/
├─ main.tsx                      React 挂载入口
├─ app/                          应用壳和 Provider 组合
│  ├─ App.tsx
│  ├─ AppProviders.tsx
│  ├─ layout/                    顶部工具栏与工作区宿主
│  └─ styles/                    应用级布局样式
├─ components/                   无业务含义的公共视图
│  ├─ forms/
│  ├─ menus/
│  └─ navigation/
├─ features/                     按业务能力组织
│  ├─ documents/                 混合标签和文档生命周期
│  ├─ pdf/                       PDF 阅读工作区
│  ├─ markdown/                  Markdown 编辑工作区
│  ├─ search/                    通用搜索协议和视图
│  ├─ annotations/               批注领域，界面显示为“笔记”
│  ├─ settings/                  设置
│  ├─ notifications/             Toast 通知
│  └─ help/                      帮助入口
├─ platform/                     Tauri、对话框、窗口和剪贴板适配
├─ shared/                       无业务归属的纯函数与类型
├─ styles/                       全局样式入口
└─ README.md                     前端架构说明
```

前端目标新增模块：

```text
features/documents/
├─ close/                        多文档关闭事务协调器
├─ recent/                       最近文件与关闭文件弹层
├─ workspace/                    应用内文件夹工作区
└─ rename/                       标签与磁盘文件重命名

features/markdown/
├─ export/                       HTML/PDF 导出 UI 和 repository
└─ themes/                       CSS 主题管理

features/pdf/
├─ bookmarks/                    PDF 书签
└─ save/                         PDF 批注写入与保存
```

### 4.3 后端目录

```text
src-tauri/
├─ src/
│  ├─ main.rs                    可执行入口
│  ├─ lib.rs                     库入口
│  ├─ bootstrap.rs               插件、state 和 command 注册
│  ├─ state.rs                   application service 依赖装配
│  ├─ commands/                  Tauri IPC 和稳定 DTO
│  ├─ application/
│  │  ├─ services/               打开、搜索、保存、导出等用例
│  │  └─ ports/                  文件、解析器、缓存和持久化抽象
│  ├─ domain/                    纯 Rust 文档领域模型
│  ├─ infrastructure/
│  │  ├─ filesystem/             文件和 JSON 访问
│  │  ├─ repositories/           设置、批注等 repository
│  │  ├─ sessions/               PDF/Markdown Session
│  │  ├─ pdf/                    MuPDF adapter 和 worker
│  │  ├─ markdown/               AST parser 和 exporter
│  │  ├─ search/                 Unicode 标准化和匹配
│  │  └─ paths/                  应用数据路径
│  └─ error/                     统一错误
├─ native/                       MuPDF 源码、Rust 包装和原生依赖
├─ resources/                    随应用分发的许可与资源
├─ capabilities/                 Tauri 权限
├─ icons/                        Windows 图标
└─ README.md                     后端架构说明
```

后端目标新增模块：

```text
application/services/
├─ storage_service.rs
├─ workspace_service.rs
├─ recent_documents_service.rs
├─ document_close_service.rs
├─ bookmark_service.rs
├─ pdf_annotation_write_service.rs
├─ markdown_export_service.rs
└─ theme_service.rs

application/ports/
├─ storage_repository.rs
├─ workspace_repository.rs
├─ recent_documents_repository.rs
├─ bookmark_repository.rs
├─ pdf_annotation_writer.rs
├─ markdown_exporter.rs
└─ theme_repository.rs

domain/
├─ storage.rs
├─ workspace.rs
├─ recent_document.rs
├─ bookmark.rs
├─ annotation.rs
├─ markdown_export.rs
└─ theme.rs
```

### 4.4 前后端职责

```text
用户操作
   ↓
React 视图与 feature state
   ↓
feature repository / platform adapter
   ↓
Tauri command + DTO
   ↓
application service
   ↓
domain + port
   ↓
infrastructure
   ├─ MuPDF
   ├─ pulldown-cmark
   ├─ Windows 文件系统
   └─ JSON repository
```

依赖规则：

- `app` 只组合布局和 Provider，不保存 PDF/Markdown 内部状态。
- PDF 和 Markdown feature 不互相导入。
- 公共 `components` 不读取 feature context。
- React 视图不直接调用 Tauri `invoke`。
- command 不直接实现 MuPDF、Markdown 解析或 JSON 文件业务。
- application 依赖抽象 port，不依赖具体文件系统和解析器。
- infrastructure 将第三方类型转换为项目自有 domain model。

更详细的分层说明见：

- [`src/README.md`](src/README.md)
- [`src-tauri/README.md`](src-tauri/README.md)

## 5、项目开发顺序

### 阶段一：基础应用壳与前后端分层（已完成）

- [X] React 应用壳和三区顶部工具栏。
- [X] PDF/Markdown feature 分离。
- [X] 公共 components、platform、shared 边界。
- [X] Rust commands/application/domain/infrastructure 分层。
- [X] Rust 同名模块入口风格，移除 `mod.rs`。
- [X] MuPDF 原生依赖和 Windows 构建接入。
- [X] Markdown AST 后端协议。

### 阶段二：统一文档工作区（基本完成）

- [X] 文件类型驱动工作区。
- [X] PDF/Markdown 混合标签。
- [X] 活动文档工具栏和侧栏切换。
- [X] 文档类型检测协议。
- [X] 未命名 Markdown 最终生命周期。
- [X] 标签未保存圆点。
- [X] 标签右键重命名。
- [ ] 多文档关闭事务协调器。

### 阶段三：PDF 渲染与性能（主体完成）

- [X] MuPDF 页面渲染、文字结构和目录。
- [X] PDF 页面虚拟化。
- [X] LRU 缓存和资源释放。
- [X] 当前页可见面积判断。
- [X] 缩放、滚动和视口稳定。
- [ ] 大型 PDF 长时间性能验收。

### 阶段四：搜索与批注（协议完成，交互待验收）

- [X] 通用搜索协议。
- [X] PDF/Markdown 搜索后端。
- [X] 防抖、版本和取消。
- [ ] 搜索 UI 全流程验收。
- [ ] PDF 选区与批注回归测试。
- [ ] 批注撤销、重做和数据版本。

### 阶段五：存储基础设施（基础完成）

- [X] 将根目录固定为 `%LOCALAPPDATA%\Bambook`。
- [X] 初始化 `storage` 五类目录和独立 `theme` 目录。
- [X] StorageService 和 StorageRepository。
- [X] 目录可写性检查和带路径上下文的错误协议。
- [X] JSON 临时文件、flush、磁盘同步和同目录原子替换协议。
- [ ] 磁盘剩余空间检查和低空间错误协议。
- [X] 设置 schema、最近文件、关闭文件和工作区状态数据迁移。

### 阶段六：文档生命周期与最近文件

- [X] 重构未命名 Markdown 编号和首次保存。
- [X] 完成标签 dirty 状态。
- [X] 完成保存/不保存/取消事务。
- [X] 退出时逐个激活未保存文档并询问。
- [X] Rust 最近文件持久化。
- [X] 关闭文件栈和恢复最后关闭文件。
- [X] 最近文件分类弹层和失效路径处理。

### 阶段七：文件夹工作区与可调整侧栏

- [ ] WorkspaceService 和目录扫描协议。
- [ ] PDF/Markdown 共用文件树。
- [ ] PDF“文件 / 视图 / 目录”侧栏。
- [ ] Markdown“文件 / 目录”侧栏。
- [ ] 左右侧栏拖动、边界限制和宽度持久化。

### 阶段八：Markdown 主题与完整导出

- [ ] CSS 主题导入、复制、命名和删除。
- [ ] HTML 两种图片策略。
- [ ] HTML 独立文件完整性。
- [ ] PDF 中文字体、图片和表格。
- [ ] 公式预渲染和代码高亮。
- [ ] 用户 CSS 的 HTML/PDF 一致渲染。
- [ ] 每次导出手动选择路径。
- [ ] 真实复杂文档导出验收。

### 阶段九：PDF 书签与导航

- [ ] 工具栏书签按钮。
- [ ] 页码和页内位置书签。
- [ ] 缩略图书签标识。
- [ ] 阅读历史前进、后退。
- [ ] 最后阅读位置持久化。
- [ ] PDF 内部和外部链接。

### 阶段十：PDF 批注写入与保存

- [ ] MuPDF Annotation 写入适配器。
- [ ] 下划线、波浪线、删除线、高亮和笔记 appearance。
- [ ] 原始 PDF 与带批注 PDF 双文件策略。
- [ ] 只保留带批注 PDF 策略。
- [ ] 保存、另存为、临时文件和原子替换。
- [ ] 加密 PDF、权限和失败回滚。

### 阶段十一：设置、帮助与发布

- [ ] 完整设置中心。
- [ ] 数据迁移和版本升级。
- [ ] 帮助、鸣谢、更新日志、反馈和关于。
- [ ] Windows 安装、升级和卸载数据策略。
- [ ] 大型 PDF、复杂 Markdown 和多窗口验收。
- [ ] NSIS 安装包与发布流程。

## 6、后续开发

在文档阅读、编辑、保存和持久化稳定后，Bambook 可以扩展为插件化本地知识与写作工具。插件不能直接进入当前 feature 内部，应通过稳定的插件协议调用文档能力。

### 6.1 插件架构

```text
Bambook Plugin Host
├─ 插件清单 manifest
├─ 权限声明
├─ 生命周期
├─ 文档只读/可写能力
├─ 选区与当前文档上下文
├─ 本地模型连接器
├─ 后台任务与取消
└─ 插件面板和命令入口
```

规划目录：

```text
src/features/plugins/
├─ components/                   插件管理和插件面板
├─ hooks/                        插件 UI hooks
├─ repositories/                 插件 IPC
├─ state/                        插件运行状态
└─ types/                        前端插件协议

src-tauri/src/
├─ domain/plugin.rs
├─ application/
│  ├─ ports/plugin_runtime.rs
│  └─ services/plugin_service.rs
├─ infrastructure/plugins/
│  ├─ manifest.rs
│  ├─ permissions.rs
│  ├─ local_llm.rs
│  └─ runtime.rs
└─ commands/plugin_commands.rs
```

### 6.2 本地 LLM 插件

- [ ] 连接 Ollama、llama.cpp 等本地模型服务。
- [ ] 用户手动配置服务地址、模型名、超时和上下文长度。
- [ ] 读取当前文档、当前选区或用户明确选择的文件。
- [ ] PDF 使用后端结构化文字，不向模型发送页面图片，除非用户明确授权。
- [ ] Markdown 使用当前 revision，防止旧响应覆盖新内容。
- [ ] 流式输出、取消和任务状态。
- [ ] 模型不可用时不影响阅读和编辑主流程。
- [ ] 请求记录和隐私开关。

### 6.3 辅助写作插件

- [ ] 续写选中文字。
- [ ] 改写、扩写和缩写。
- [ ] 调整语气和目标读者。
- [ ] 根据当前目录生成大纲。
- [ ] 根据 PDF 笔记生成 Markdown 草稿。
- [ ] 建议修改以 diff 形式展示，由用户确认后写入。
- [ ] 插件修改必须进入 Markdown 撤销历史。

### 6.4 翻译插件

- [ ] 翻译选区、段落、当前页或整份文档。
- [ ] 支持双语对照预览。
- [ ] 保留 Markdown 标题、链接、代码块和公式结构。
- [ ] PDF 翻译不修改原正文，输出到笔记或新建 Markdown。
- [ ] 术语表和固定译法。
- [ ] 长文档分段、上下文衔接和任务恢复。

### 6.5 其他候选插件

- [ ] OCR 插件：为扫描版 PDF 建立可搜索文字层。
- [ ] 文献插件：提取标题、作者、DOI 和参考文献。
- [ ] 摘要插件：按页面、章节或整份文档生成摘要。
- [ ] 知识卡片插件：从笔记生成问答卡和复习列表。
- [ ] 引用管理插件：连接 Zotero 或本地 BibTeX。
- [ ] 图表插件：渲染 Mermaid、PlantUML 等图表。
- [ ] 语法检查插件：中文标点、英文拼写和风格检查。
- [ ] 模板插件：论文、报告、会议记录和读书笔记模板。

### 6.6 插件安全与维护要求

- 插件默认没有任意文件系统、网络或进程权限。
- 文件读取必须限制在当前文档、工作区或用户明确授权的路径。
- 本地 LLM 地址也需要显式授权，不能静默连接远程服务。
- 插件写入 Markdown 前必须展示变更并由用户确认。
- 插件不能直接持有 MuPDF 指针或绕过 application service。
- 插件任务必须支持取消、超时和窗口关闭清理。
- 插件协议需要版本号、最低应用版本和兼容性声明。
- 第三方插件异常不能使 PDF/Markdown 主工作区崩溃。

项目级功能与开发状态以本 README 为准；前端和后端实现细节分别参见 [`src/README.md`](src/README.md) 与 [`src-tauri/README.md`](src-tauri/README.md)。

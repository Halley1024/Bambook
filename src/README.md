# React 前端架构

本目录保存 Bambook 的 React/TypeScript 前端。前端负责 UI 渲染、工作区交互和表现状态，不负责可信文件访问，也不应长期承担 PDF 或 Markdown 解析业务。

返回项目总览：[`../README.md`](../README.md)  
后端设计：[`../src-tauri/README.md`](../src-tauri/README.md)

## 职责边界

前端负责：

- 页面布局、主题和交互动画。
- 根据活动标签页的文件类型选择 PDF 或 Markdown 工作区。
- 统一的混合文档标签，以及工作区各自的侧栏和工具栏状态。
- 当前页、缩放、光标、选区、展开和拖动状态。
- PDF 图像层、文字选择层、链接层和批注层的组合。
- Markdown AST 节点到 React 组件的映射。
- 把用户动作转换为 repository 调用。
- 加载、成功、警告和错误反馈。

前端不负责：

- 直接访问 Windows 文件系统。
- 校验可信路径或拼接应用数据目录。
- 长期解析 PDF、Markdown、目录或搜索索引。
- 保存设置、批注或 Markdown 原文件。
- 直接暴露 MuPDF、PDF.js 或 Rust 解析器内部类型。

当前版本已经切换到 Rust 结构化解析链路：PDF 页面、文字层和目录来自 MuPDF，Markdown 预览和目录来自后端 AST。前端不再依赖 PDF.js、`marked` 或 `DOMPurify` 执行文档解析。

## 顶层目录

```text
src/
├─ main.tsx          React 挂载入口
├─ app/              应用壳、全局 Provider 和工作区选择
├─ components/       跨工作区复用的纯视图组件
├─ features/         PDF、Markdown 及共享业务功能
├─ platform/         Tauri 和浏览器能力适配
├─ shared/           无业务归属的通用函数与类型
├─ styles/           全局重置和样式聚合入口
└─ README.md          前端架构说明
```

依赖方向：

```text
main
  ↓
app
  ↓
features ─────→ components
  ↓                 ↓
platform          shared
  ↓
Tauri Commands
```

下层模块不能反向依赖上层。例如 `platform` 不能导入 React 工作区，公共 `components` 不能导入 PDF 或 Markdown feature。

## `main.tsx`

唯一的 React 挂载入口，只负责：

- 导入全局样式。
- 查找根 DOM 节点。
- 创建 React root。
- 挂载 `App`。

不要在这里加入文档状态、文件类型判断或 Tauri 调用。

## `app/`

```text
app/
├─ App.tsx
├─ AppProviders.tsx
├─ layout/
│  ├─ AppShell.tsx
│  ├─ TopToolbar.tsx
│  ├─ ApplicationIdentity.tsx
│  ├─ ActiveDocumentToolbar.tsx
│  ├─ WorkspaceHost.tsx
│  └─ AppMenuActions.tsx
└─ styles/
   ├─ app-shell.css
   ├─ toolbar.css
   └─ workspace.css
```

`app` 只负责应用级组合，不保存 PDF 或 Markdown 内部业务。

### `App.tsx` 与 `AppProviders.tsx`

- `App.tsx` 组合全局 Provider 和 `AppShell`。
- `AppProviders.tsx` 集中声明 Provider 顺序。
- 新增全局 Provider 时在这里登记，不要散落到页面组件。

应用壳不保存 `AppMode`。活动工作区始终由
`DocumentWorkspaceProvider.activeDocument.kind` 派生，避免“模式状态”和标签页状态出现双数据源。

### `layout/`

- `AppShell`：组合顶部工具栏、工作区和全局浮层。
- `TopToolbar`：只定义左、中、右三区布局。
- `ApplicationIdentity`：显示应用名称和活动文档类型，不承担切换操作。
- `ActiveDocumentToolbar`：根据活动文档类型选择 PDF 或 Markdown 工具栏。
- `WorkspaceHost`：组合统一标签栏，并根据活动文档类型挂载工作区；无文档时显示开始页。
- `AppMenuActions`：组合通用文件菜单、文档类型扩展菜单、共用设置菜单和帮助菜单。

顶部工具栏固定分为：

```text
TopToolbar
├─ 左侧：ApplicationIdentity
├─ 中间：活动文档所属 feature 的工具栏
└─ 右侧：文件、设置和帮助菜单
```

`TopToolbar` 不读取 PDF 页码或 Markdown 光标，也不直接打开文件。

## `components/`

```text
components/
├─ forms/
│  └─ SearchInput.tsx
├─ navigation/
│  ├─ Tabs.tsx
│  └─ tabs.css
└─ menus/
   ├─ DropdownMenu.tsx
   └─ dropdown-menu.css
```

公共组件必须满足：

- 至少可以被两个业务 feature 使用。
- 通过 props 接收数据和回调。
- 不导入 feature context。
- 不调用 repository 或 Tauri。
- 不理解“PDF 当前页”或“Markdown 保存”等业务含义。

不应放入 `components`：

- `PdfPage`、`PdfOutlineTree`。
- `MarkdownEditor`、`MarkdownOutline`。
- `AnnotationPanel`、`SettingsPanel`。
- 任何只能在一个 feature 内使用的包装组件。

## `features/`

```text
features/
├─ pdf/             PDF 工作区
├─ markdown/        Markdown 工作区
├─ search/          PDF/Markdown 复用的全文搜索状态与视图
├─ documents/       跨文件类型的打开、混合标签和生命周期
├─ annotations/     批注与笔记
├─ settings/        应用设置
├─ help/            PDF/Markdown 共用帮助入口
└─ notifications/   Toast 通知
```

feature 自己拥有视图、状态、hooks、repository、services、types 和样式。PDF 与 Markdown feature 不得互相导入；共享能力需要提取到具有明确职责的公共 feature 或组件。

## `features/pdf/`

### 当前结构

```text
pdf/
├─ index.ts
├─ PdfWorkspace.tsx
├─ PdfWorkspaceProvider.tsx
├─ components/
│  ├─ PdfToolbar.tsx
│  ├─ PdfFileMenu.tsx
│  ├─ PdfReader.tsx
│  ├─ PdfPage.tsx
│  ├─ PdfSelectionMenu.tsx
│  ├─ PdfSidebar.tsx
│  ├─ PdfThumbnailList.tsx
│  └─ PdfOutlineTree.tsx
├─ hooks/
│  ├─ usePdfWorkspace.ts
│  ├─ usePdfSession.ts
│  ├─ usePdfViewportController.ts
│  └─ usePdfOutline.ts
├─ state/
│  ├─ PdfWorkspaceContext.tsx
│  └─ PdfSessionContext.tsx
├─ search/
│  ├─ PdfSearchProvider.tsx
│  ├─ hooks/usePdfSearch.ts
│  ├─ state/PdfSearchContext.tsx
│  ├─ repositories/pdfSearchRepository.ts
│  ├─ adapters/pdfSearchAdapter.ts
│  └─ components/PdfSearchOverlay.tsx
└─ styles/
   ├─ pdf-reader.css
   └─ pdf-sidebar.css
```

### 工作区状态

`PdfWorkspaceProvider` 管理 PDF 专属状态：

- 当前页和页面跳转请求；全文搜索状态由独立的 `PdfSearchProvider` 管理。
- 50% 至 250% 缩放及适合阅读区操作。
- 左右侧栏折叠状态。
- 当前文档批注。
- PDF 会话释放和批注持久化生命周期。

文档列表、活动标签、打开、关闭、重排和复制路径统一由
`DocumentWorkspaceProvider` 管理，PDF Provider 只过滤并使用其中的 PDF 文档。

`PdfWorkspace` 只组合布局：

```text
PdfSidebar
PdfReader
AnnotationPanel
```

### 阅读视口

`usePdfViewportController` 负责：

- 计算适合阅读区的宽度。
- 按页面可见面积判断当前页。
- 目录和缩略图跳转。
- 缩放时保持左上视口锚点。
- 避免缩放、翻页和滚动动画互相触发造成抖动。

只有 PDF 内容区域滚动；工具栏、标签栏和左右侧栏不能跟随 PDF 页面一起滚动。

### 当前 MuPDF 实现

`PdfSessionContext` 只暴露应用自有的 PDF 会话模型。`PdfPage` 通过 `mupdfRepository` 按需请求后端 PNG 与页面结构；透明文字层、搜索和目录都使用同一份 MuPDF 数据。

```text
features/pdf/
├─ repositories/mupdfRepository.ts
├─ state/PdfSessionContext.tsx
├─ hooks/usePdfOutline.ts
└─ components/
   ├─ PdfPage.tsx
   ├─ PdfThumbnailList.tsx
   └─ PdfOutlineTree.tsx
```

页面由四层组成：

```text
PdfPage
├─ PageBitmapLayer       MuPDF 返回的原貌页面图像
├─ TextSelectionLayer    透明结构文字，用于选择和搜索
├─ LinkLayer             页内和外部链接热点
└─ AnnotationLayer       重点、下划线和笔记几何层
```

PDF 只采用原貌阅读模式：

- 不编辑、重排或改写原始 PDF 正文。
- 应用主题只影响外壳、阅读区背景和批注颜色。
- 批注使用 document fingerprint、页码和归一化 quad/range 定位。
- `PdfDocument` 不保存整份 `Uint8Array`，只保存 sessionId、fingerprint、页数和元数据。
- `PdfSessionContext` 管理后端会话，不暴露第三方解析器类型。

## `features/markdown/`

### 当前结构

```text
markdown/
├─ index.ts
├─ MarkdownWorkspace.tsx
├─ MarkdownWorkspaceProvider.tsx
├─ components/
│  ├─ MarkdownToolbar.tsx
│  ├─ MarkdownFileMenu.tsx
│  ├─ MarkdownSidebar.tsx
│  ├─ MarkdownFileTree.tsx
│  ├─ MarkdownOutline.tsx
│  ├─ MarkdownEditor.tsx
│  ├─ MarkdownSplitView.tsx
│  ├─ MarkdownPreview.tsx
│  ├─ MarkdownAstRenderer.tsx
│  ├─ MarkdownStatusBar.tsx
│  ├─ MarkdownInspectorPanel.tsx
│  └─ MarkdownHelpPanel.tsx（兼容导出）
├─ hooks/
│  └─ useMarkdownWorkspace.ts
├─ repositories/
│  ├─ markdownRepository.ts
│  └─ structuredMarkdownRepository.ts
├─ services/
│  └─ markdownCommands.ts
├─ state/
│  └─ MarkdownWorkspaceContext.tsx
├─ search/
│  ├─ MarkdownSearchProvider.tsx
│  ├─ hooks/useMarkdownSearch.ts
│  ├─ state/MarkdownSearchContext.tsx
│  ├─ repositories/markdownSearchRepository.ts
│  └─ adapters/markdownSearchAdapter.ts
├─ types/
│  └─ markdownWorkspace.ts
└─ styles/
   ├─ markdown-editor.css
   ├─ markdown-preview.css
   └─ markdown-sidebar.css
```

### 工作区职责

- 使用统一文档工作区中的 Markdown 文档，不另建标签状态。
- 为每份文档保存独立编辑缓冲区和已保存版本。
- 计算 dirty、saving 和保存完成状态。
- 实时、源码和分栏模式切换，默认使用实时模式。
- 左侧显示当前目录文件结构和标题目录。
- 顶部工具栏提供常用格式、段落和内容插入操作。
- 右侧文档检查器展示文档统计、解析问题和链接/图片资源，默认收起。
- 通过工具栏或 `Ctrl+S` 保存。
- 搜索结果按后端 UTF-16 locator 定位编辑器，并在 AST 预览中生成 decoration。

三栏布局：

```text
MarkdownSidebar
├─ 文件：当前目录的 Markdown 文件树
└─ 目录：当前文档标题结构

MarkdownEditor / MarkdownPreview

MarkdownInspectorPanel
├─ 文档：保存状态、字词、行数和标题统计
├─ 问题：后端 diagnostics 列表与源码定位
└─ 资源：当前 AST 中的链接和图片
```

### 当前后端 AST 实现

前端已经删除 `marked`、`DOMPurify` 和 `markdownOutlineParser` 的解析职责。编辑内容通过 UTF-16 edit range 和 revision 串行提交，预览由 React AST renderer 生成：

```text
features/markdown/
├─ repositories/markdownRepository.ts
├─ state/MarkdownSessionContext.tsx
├─ hooks/useMarkdownEditor.ts
├─ renderers/
│  ├─ MarkdownNodeRenderer.tsx
│  └─ markdownNodeRegistry.ts
└─ components/
   ├─ MarkdownEditor.tsx
   ├─ MarkdownPreview.tsx
   ├─ MarkdownOutline.tsx
   └─ nodes/
```

后端返回 `source + nodes + outline + diagnostics + revision`。前端：

- 编辑器可以保留输入所需的临时源码字符串，但不解析语法。
- 约 100–200ms debounce 后向后端发送编辑批次。
- 丢弃 revision 早于当前版本的异步响应。
- `MarkdownPreview` 遍历 AST，并通过 node registry 选择 React 组件。
- 不使用后端拼接 HTML，也不使用 `dangerouslySetInnerHTML` 渲染文档。
- Markdown 主题、字体、字号、粗细和间距全部由 renderer 与 CSS 控制。
- 目录导航复用 AST outline 的 `nodeId` 与 UTF-16 offset，同时将预览标题和编辑器源码行定位到视口中部。
- `useMarkdownScrollSync` 在分栏模式下按归一化阅读进度双向同步编辑区和预览区，并使用短生命周期滚动锁阻止事件反馈循环。

## `features/search/`

```text
search/
├─ index.ts
├─ types/search.ts
├─ repositories/documentSearchRepository.ts
├─ hooks/useDocumentSearch.ts
└─ components/DocumentSearchTools.tsx
```

这是两种工作区共用的搜索外壳，只定义 query、选项、请求状态、结果列表和上一项/下一项行为：

- `useDocumentSearch` 统一实现 250ms 防抖、requestId、旧请求取消和过期响应丢弃。
- `documentSearchRepository` 是 `search_document` 与 `cancel_document_search` 的唯一前端 IPC 边界。
- `DocumentSearchTools` 是受控视图，不读取 PDF 或 Markdown Context。
- PDF locator 保存页码、页内纵坐标和矩形；Markdown locator 保存 revision、UTF-16 offset、行和列。
- 文件类型专属 Provider 负责把 locator 转为跳页、居中滚动、编辑器选区或预览 decoration。

搜索状态不得并回 `PdfWorkspaceProvider` 或 `MarkdownWorkspaceProvider`，否则输入、结果切换会让整个工作区和虚拟 PDF 页面高频重渲染。

## `features/documents/`

```text
documents/
├─ index.ts
├─ types.ts
├─ DocumentHistoryProvider.tsx
├─ DocumentWorkspaceProvider.tsx
├─ components/
│  ├─ UnifiedDocumentTabs.tsx
│  ├─ StartWorkspace.tsx
│  ├─ DocumentFileMenu.tsx
│  └─ WorkspaceFileMenu.tsx
├─ hooks/
│  ├─ useDocumentWorkspace.ts
│  ├─ useDocumentHistory.ts
│  └─ useWorkspaceTabs.ts
├─ state/
│  ├─ DocumentWorkspaceContext.tsx
│  └─ DocumentHistoryContext.tsx
└─ repositories/
   ├─ documentRepository.ts
   ├─ documentHistoryRepository.ts
   └─ workspaceStateRepository.ts
```

- `types.ts` 使用 `PdfDocument | MarkdownDocument` 可判别联合，不创建大量可选字段的万能类型。
- `useWorkspaceTabs` 只要求文档具有 `id`，负责打开、激活、关闭、关闭其他和重排。
- `DocumentWorkspaceProvider` 是打开文档列表和活动文档的唯一状态源；PDF 与 Markdown 可以混排在同一标签栏。
- `DocumentHistoryProvider` 异步读取 Rust 最近文件与关闭栈，负责记录关闭、确认恢复和清空操作，视图不直接访问持久化 API。
- `documentHistoryRepository` 仅在首次迁移时读取旧 `localStorage`；后端导入成功后立即删除旧键，后续数据全部来自 Rust JSON repository。
- 最近文件弹层挂载时重新向后端读取状态；`available` 项可以打开，`missing`、`inaccessible`、`notFile` 项显示原因并禁用打开。
- `workspaceStateRepository` 保存有磁盘路径的标签顺序、活动文档和文件夹路径；启动恢复不持久化无效 sessionId。
- `documentRepository` 负责选择受支持文件、调用后端检测类型，再调用对应打开协议并映射为严格前端类型。
- `DocumentLifecycleController` 让类型 feature 注册 `deactivate`、`canClose` 和 `close`，统一工作区不需要了解 MuPDF、dirty buffer 或 AST session。
- 批量关闭先执行全部 `canClose`，全部通过后才释放会话，避免用户取消时出现标签与后端 session 不一致。
- `DocumentFileMenu` 只放真正通用的菜单组；`PdfFileMenu` 和 `MarkdownFileMenu` 通过扩展组加入各自的导入、导出和保存操作。
- 没有活动文档时，`StartWorkspace` 提供统一打开入口。

当前已完成的文件菜单动作包括：创建未命名 Markdown 标签、打开文件、打开文件夹、在资源管理器中显示当前文件、新建窗口、关闭窗口、分类最近文件、恢复关闭文件、Markdown 保存/另存为，以及 Markdown 导出 HTML 和 PDF。未命名文档只存在于后端 session，首次保存时才选择路径；关闭标签或窗口时使用“保存/不保存/取消”三选项。批注、笔记及笔记 Markdown 导出均为 PDF 专属能力，Markdown 工作区不加载或保存批注。

`documents` 只编排打开分发与生命周期。页面结构、渲染、Markdown 编辑和搜索仍由各自 feature/repository 持有；批注只由 PDF feature 接入。

## `features/annotations/`

```text
annotations/
├─ index.ts
├─ types.ts
├─ components/AnnotationPanel.tsx
├─ hooks/useAnnotations.ts
├─ repositories/annotationRepository.ts
└─ styles.css
```

- `AnnotationPanel` 只负责笔记视图。
- `useAnnotations` 只供 PDF 工作区使用，负责按 PDF 缓存、加载、延迟保存和立即持久化。
- repository 调用批注读写和 Markdown 导出命令。
- PDF 几何批注使用归一化页面坐标，不保存 DOM 位置。

## `features/settings/`

```text
settings/
├─ index.ts
├─ SettingsProvider.tsx
├─ types.ts
├─ components/
│  ├─ SettingsPanel.tsx
│  └─ SettingsMenu.tsx
├─ hooks/useSettings.ts
├─ repositories/settingsRepository.ts
└─ styles.css
```

Provider 管理设置数据和面板开关；Panel 只显示设置；repository 负责后端通信。

## `features/help/`

```text
help/
├─ index.ts
└─ components/HelpMenu.tsx
```

帮助菜单由 PDF 和 Markdown 工作区共用。鸣谢、更新日志、留言反馈和关于页面的具体实现继续留在该 feature 内，不写入文件类型工作区或 `TopToolbar`。

## `features/notifications/`

```text
notifications/
├─ index.ts
├─ NotificationProvider.tsx
├─ useNotification.ts
├─ types.ts
├─ components/ToastViewport.tsx
└─ styles.css
```

统一处理成功、错误、警告、普通消息、队列、自动关闭和手动关闭。其他 feature 只使用公开 API：

```ts
notification.success("文档已打开");
notification.error("保存失败");
notification.warning("请先选择文档");
```

## `platform/`

```text
platform/
├─ tauriClient.ts
├─ fileDialog.ts
└─ clipboard.ts
```

- `tauriClient.ts`：唯一通用 Tauri `invoke` 封装。
- `fileDialog.ts`：文件和目录选择适配。
- `clipboard.ts`：剪贴板适配。

React 视图组件不能直接导入 `@tauri-apps/*` 或调用 `navigator.clipboard`。

## `shared/`

只保存没有业务归属的纯代码，例如数值、日期、字符串工具和通用 TypeScript 工具类型。

不应放入 shared：

- PDF 页码和坐标算法。
- Markdown 标题或 AST 解析。
- 批注保存规则。
- 设置文件路径。

这些内容分别属于对应 feature 或 Rust 后端。

## `styles/`

只保留全局入口和 reset：

```text
styles/
├─ index.css
└─ reset.css
```

具体样式跟随所属模块：

```text
app/styles/
components/navigation/tabs.css
components/menus/dropdown-menu.css
features/pdf/styles/
features/markdown/styles/
features/annotations/styles.css
features/settings/styles.css
features/notifications/styles.css
```

不要重新创建包含所有功能样式的单体文件。

## Feature 公共入口

主要 feature 通过 `index.ts` 暴露公共 API：

```ts
import { PdfWorkspace, PdfToolbar } from "../features/pdf";
```

feature 外部不要深层导入其内部组件。feature 内部可以使用相对路径互相导入。

## 状态归属

| 状态 | 归属 |
| --- | --- |
| 全部打开文档、活动文档和混合标签顺序 | `features/documents` |
| 当前工作区类型 | 不单独存储，由 `activeDocument.kind` 派生 |
| 设置面板开关和设置数据 | `features/settings` |
| PDF 页码、缩放、目录、侧栏和 PDF 会话状态 | `features/pdf` |
| Markdown 源码缓冲区、revision、dirty 和编辑视图 | `features/markdown` |
| 搜索 query、结果、当前命中和请求生命周期 | `features/search` + 文件类型专属 SearchProvider |
| PDF 批注 | `features/annotations` |
| Toast 消息 | `features/notifications` |
| 后端文档句柄和解析缓存 | Rust 后端，不进入 React state |

判断原则：哪个 feature 定义了状态的业务含义，状态就由哪个 feature 管理。

## 数据流

### 统一文档打开与分发流程

```text
StartWorkspace / DocumentFileMenu / 文档类型文件菜单
→ DocumentWorkspaceProvider.openFile
→ platform/fileDialog
→ Tauri detect_document_kind
├─ PDF      → open_pdf_document      → PdfDocument
└─ Markdown → open_markdown_document → MarkdownDocument
→ 统一 useWorkspaceTabs
→ activeDocument.kind
├─ PDF      → PdfToolbar + PdfWorkspace
└─ Markdown → MarkdownToolbar + MarkdownWorkspace
```

同一路径再次打开时不会新建重复标签，新创建的临时后端 session 会立即释放，然后激活已有标签。

### PDF 页面加载流程

```text
活动 PdfDocument(sessionId)
→ PdfWorkspaceProvider
→ PdfReader 按可见范围虚拟化
→ mupdfRepository 请求页面 structure 与 bitmap
→ PdfPage 图像/文字/搜索/批注层
```

现有 PDF 视图渲染、缩放、滚动和选区链路在本次工作区重构中保持不变。

### Markdown 编辑流程

```text
用户输入
→ debounce MarkdownEdit[]
→ apply_markdown_edits(baseRevision)
→ 新 revision + AST/outline/diagnostics
→ 更新预览
```

### 保存批注

```text
PdfWorkspace
→ useAnnotations
→ annotationRepository
→ platform/tauriClient
→ Rust save_annotations
→ JSON 文件
```

## 新文件放置规则

| 新增内容 | 位置 |
| --- | --- |
| PDF 页面、目录或覆盖层视图 | `features/pdf/components` |
| PDF 滚动、缩放或导航 Hook | `features/pdf/hooks` |
| 后端 PDF DTO 到视图模型映射 | `features/pdf/services` |
| Markdown 编辑器或 AST renderer | `features/markdown/components` / `renderers` |
| Markdown 编辑会话 Hook | `features/markdown/hooks` |
| 无业务的标签页基础视图或输入框 | `components` |
| 文档打开分发、混合标签和生命周期 | `features/documents` |
| 批注、设置、通知 | 对应 feature |
| Tauri、对话框和剪贴板 | `platform` |
| 无业务归属纯函数 | `shared` |
| 整体页面组合 | `app/layout` |

如果一个文件同时适合多个目录，通常说明它承担了多个职责，应先拆分。

## 需要避免的耦合

- 不把 PDF 或 Markdown 状态写回 `App.tsx`。
- 不让公共 components 导入 feature context。
- 不在 React 组件中直接调用 `invoke`。
- 不让 PDF 和 Markdown feature 互相导入。
- 不让 repository 渲染 UI 或保存 React state。
- 不在多个组件中重复转换 PDF 坐标或拼接路径。
- 不在前端重新实现后端已经提供的 Markdown/PDF 解析。
- 不创建包含大量可选字段的万能文档类型。
- 不把所有样式、Hook、类型或服务合并到单个文件。

## 开发与验证

只运行浏览器前端：

```powershell
npm run dev
```

TypeScript 检查和生产构建：

```powershell
npm run build
```

完整桌面开发模式从项目根目录运行：

```powershell
npm run tauri:dev
```

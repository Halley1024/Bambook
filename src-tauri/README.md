# Tauri/Rust 后端架构

> 当前实现状态（2026-08）：MuPDF 0.8.0 与 pulldown-cmark 0.13.4 已接入，不再只是规划。MuPDF 源码、安全 Rust 包装和项目本地 libclang 位于 `native/`；安装时需要携带的许可资料位于 `resources/`。PDF 采用“每个 session 一个串行 worker”，Markdown 已实现带 UTF-16 edit range 与 revision 冲突检查的 AST session。

当前新增 IPC：

- 通用文档：`detect_document_kind`，根据 PDF 文件签名或 Markdown 扩展名返回稳定的 `kind`。
- PDF：`open_pdf_document`、`authenticate_pdf_document`、`get_pdf_page_structure`、`render_pdf_page`、`close_pdf_document`。
- Markdown：`open_markdown_document`、`get_markdown_structure`、`apply_markdown_edits`、`save_markdown_document`、`close_markdown_document`。
- Markdown 文件输出：`save_markdown_document_as`、`export_markdown_html`、`export_markdown_pdf`。导出强制使用正确扩展名，先生成到同目录临时文件；HTML 结构或 MuPDF PDF 重新打开校验通过后才原子替换目标文件。
- 通用搜索：`search_document`、`cancel_document_search`，通过带 `kind` 的 target/locator 支持 PDF 与 Markdown。

`render_pdf_page` 使用 Tauri 二进制响应返回 PNG，不会把图片序列化成 JSON `number[]`。前端已经通过 feature 内的 `mupdfRepository` 与 `structuredMarkdownRepository` 使用上述协议，旧的 PDF.js/marked 文件读取命令已经删除。

本目录保存 Bambook 的 Tauri v2 桌面后端、Windows 应用配置和结构化文档解析实现。

返回项目总览：[`../README.md`](../README.md)
前端设计：[`../src/README.md`](../src/README.md)

## 后端职责

- 打开、读取、保存和关闭 PDF/Markdown 文档。
- 管理文档 session、revision 和资源生命周期。
- 使用 MuPDF 解析 PDF 元数据、目录、页面文字、链接和坐标。
- 按需渲染 PDF 原貌页面图像。
- 将 Markdown 解析为项目自有 AST、目录、源码映射和诊断。
- 执行文档搜索并返回结构化命中范围。
- 校验路径、文件类型、页码、缩放和输入长度。
- 持久化设置、批注和笔记导出结果。
- 管理页面结构和渲染缓存。
- 通过稳定 DTO 和二进制响应与 React 前端通信。

后端不负责当前页、缩放控件、侧栏开关、主题、动画和选区菜单等 UI 状态。

## 当前实现

当前 Rust 后端已经完成基础分层重构，`lib.rs` 只保留模块声明和启动入口。现有文件读取、Markdown 文件树、设置、批注和导出能力分别通过 command、application port/service、domain 与 infrastructure 协作完成。

```text
src/
├─ main.rs
├─ lib.rs                         模块声明与 run()
├─ bootstrap.rs                   Tauri 插件、state 和 command 注册
├─ state.rs                       application services 依赖装配
├─ error.rs                       error/ 的模块入口与重导出
├─ error/
│  └─ app_error.rs               统一内部错误与兼容 IPC 序列化
├─ domain.rs                      domain/ 的模块入口
├─ domain/
│  ├─ document.rs
│  ├─ markdown.rs
│  ├─ search.rs
│  ├─ annotation.rs
│  └─ settings.rs
├─ application.rs                 application/ 的模块入口
├─ application/
│  ├─ ports.rs                   ports/ 的模块入口与重导出
│  ├─ ports/                     文件、设置、批注和导出抽象
│  ├─ services.rs                services/ 的模块入口与重导出
│  └─ services/                  对应应用用例
├─ commands.rs                    commands/ 的模块入口
├─ commands/
│  ├─ document_commands.rs
│  ├─ settings_commands.rs
│  ├─ persistence_commands.rs
│  ├─ annotation_commands.rs
│  ├─ export_commands.rs
│  ├─ search_commands.rs
│  ├─ dto.rs                     dto/ 的模块入口与重导出
│  └─ dto/                       包含 search_dto.rs 等稳定 IPC 协议
├─ infrastructure.rs              infrastructure/ 的模块入口
└─ infrastructure/
   ├─ filesystem/                文件读写、JSON store、Markdown 文件树
   ├─ repositories/              设置和批注 JSON repository
   ├─ exporters/                 Markdown 笔记导出
   ├─ pdf/                       MuPDF 页面与独立搜索 worker
   ├─ markdown/                  Markdown AST 解析器
   ├─ search/                    Unicode 标准化、源码映射和匹配器
   ├─ sessions/                  Markdown/PDF 会话实现
   ├─ paths/                     AppPaths、统一存储布局与 documentId 校验
   └─ utils/                     Windows 安全文件名
```

本项目统一采用 Rust 2018 同名模块入口风格，不使用 `mod.rs`。例如 `commands.rs` 对应 `commands/`，`commands/dto.rs` 对应 `commands/dto/`。查找模块声明和 `pub(crate) use` 时，直接打开与目录同名的 `.rs` 文件。

所有可能阻塞的 command 都通过 Tauri blocking worker 执行；managed `AppState` 只保存 application services，不保存 UI 状态。现有 command 名称、参数和返回字段保持不变。

当前已实现：

| Command                         | 用途                                                           |
| ------------------------------- | -------------------------------------------------------------- |
| `detect_document_kind`        | 检测受支持文件类型，使前端无需维护独立模式状态                 |
| `open_pdf_document`           | 创建 MuPDF 会话并返回元数据、页数和目录                        |
| `get_pdf_page_structure`      | 返回页面尺寸、文字结构和链接                                   |
| `render_pdf_page`             | 使用二进制响应返回 MuPDF PNG 页面                              |
| `open_markdown_document`      | 创建 Markdown AST 编辑会话                                     |
| `apply_markdown_edits`        | 按 revision 应用 UTF-16 编辑并返回新 AST                       |
| `save_markdown_document`      | 保存指定 revision                                              |
| `save_markdown_document_as`   | 将指定 revision 保存到新 Markdown 路径，并返回更新后的会话文档 |
| `export_markdown_html`        | 将指定 revision 渲染为独立 HTML 文件                           |
| `export_markdown_pdf`         | 使用 MuPDF 将指定 revision 的 HTML 排版为 PDF                  |
| `search_document`             | 使用统一协议搜索 PDF 或 Markdown，返回文件类型专属 locator     |
| `cancel_document_search`      | 按 requestId 取消正在执行的搜索                                |
| `list_markdown_directory`     | 读取目录下的 Markdown 文件结构                                 |
| `load_settings`               | 加载应用设置                                                   |
| `save_settings`               | 保存应用设置                                                   |
| `load_annotations`            | 加载文档批注                                                   |
| `save_annotations`            | 保存文档批注                                                   |
| `export_annotations_markdown` | 导出 Markdown 笔记                                             |

后端不会把整份 PDF 发送给前端，也不会由前端解析 Markdown。

## 目标依赖方向

```text
React repositories
  ↓
Tauri commands / binary protocol
  ↓
application services
  ↓
domain models + ports
  ↑
infrastructure implementations
  ├─ MuPDF adapter
  ├─ Markdown parser adapter
  ├─ sessions and caches
  ├─ filesystem
  └─ JSON repositories
```

依赖规则：

1. command 只依赖 application，不直接访问 MuPDF 或磁盘。
2. application 只编排用例并依赖抽象 ports。
3. domain 是纯 Rust，不依赖 Tauri、MuPDF、WebView 或 Windows API。
4. infrastructure 实现 ports，可以依赖 MuPDF、解析器和文件系统。
5. 第三方库类型在 infrastructure 内转换为项目自有领域模型。
6. DTO 与 domain 分开，前端协议变化不能污染解析算法。

## 后续目标目录总览

以下目录是在当前基础分层之上，为 MuPDF、Markdown AST、文档 session 和缓存继续扩展的目标，不表示这些解析能力已经实现：

```text
src-tauri/
├─ Cargo.toml
├─ build.rs
├─ tauri.conf.json
├─ capabilities/
├─ icons/
├─ native/                    MuPDF C 边界和构建说明
├─ resources/                 随安装包分发的 DLL/许可证资源
├─ src/
│  ├─ main.rs
│  ├─ lib.rs
│  ├─ bootstrap.rs
│  ├─ state.rs
│  ├─ commands/               Tauri IPC 与 DTO
│  ├─ application/            用例服务与抽象端口
│  ├─ domain/                 纯 Rust 文档模型
│  ├─ infrastructure/         解析器、会话、缓存和持久化实现
│  └─ error/                  内部错误和 IPC 错误映射
└─ README.md
```

### 入口文件

- `main.rs`：只调用库入口。
- `lib.rs`：声明模块并暴露 `run()`。
- `bootstrap.rs`：创建实现、装配 application service、注册 managed state、commands 和页面 protocol。
- `state.rs`：定义 Tauri managed state，只保存服务对象，不保存 UI 状态。

## 统一文档外壳

PDF 是固定版式文档，Markdown 是流式语义文档。可以统一会话协议，但不能强行统一内部节点：

```rust
enum StructuredDocument {
    Pdf(PdfDocumentModel),
    Markdown(MarkdownDocumentModel),
}

struct DocumentEnvelope<T> {
    session_id: DocumentSessionId,
    revision: u64,
    fingerprint: DocumentFingerprint,
    title: String,
    content: T,
}
```

可以共享：

- `DocumentSessionId`
- `DocumentFingerprint`
- revision 和过期响应判断
- 结构化错误
- 搜索 query/options 外壳
- 目录节点公共字段
- 打开、保存、关闭的生命周期约定

必须独立：

- PDF page geometry、text quad、link 和 render key。
- Markdown block/inline AST、source range 和 diagnostics。
- 两种文档的解析器端口和缓存策略。

## 全文搜索架构

搜索复用协议与标准化算法，但保留两种文档各自的索引和定位方式：

```text
search_document / cancel_document_search
  ↓
DocumentSearchService
  ├─ PdfSearchEngine port
  │   └─ MupdfPdfSearchEngine
  │      ├─ 每个 PDF session 独立搜索 worker
  │      ├─ 按页建立文字与字符几何索引
  │      └─ Pdf locator(page, pageY, rects)
  └─ MarkdownSessionRepository
      ├─ 搜索指定 revision 的源码
      └─ Markdown locator(revision, startUtf16, endUtf16, line, column)

两条路径共同使用 infrastructure/search
  ├─ Unicode NFKC
  ├─ 可选大小写折叠
  ├─ 连续空白折叠
  └─ 标准化字符到原始源码/字符矩形的映射
```

`requestId` 同时承担响应版本和取消代次。新请求会使旧 PDF worker 搜索尽快退出；Markdown 文档 revision 变化时旧结果失效。关闭 PDF 会终止独立搜索 worker 并释放页面索引，关闭 Markdown 会清除取消令牌。搜索命令运行在 blocking task，不能占用 WebView 或 PDF 渲染 worker。

## `commands/`：Tauri 边界

```text
commands/
├─ pdf_commands.rs
├─ markdown_commands.rs
├─ annotation_commands.rs
├─ settings_commands.rs
├─ persistence_commands.rs
├─ export_commands.rs
└─ dto/
   ├─ common_dto.rs
   ├─ pdf_dto.rs
   ├─ markdown_dto.rs
   ├─ annotation_dto.rs
   ├─ settings_dto.rs
   └─ error_dto.rs
```

该目录的入口分别是同级的 `commands.rs` 和 `commands/dto.rs`。

command 负责：

- 反序列化并校验 IPC 参数。
- 从 managed state 取得 application service。
- 调用一个明确用例。
- 将领域模型转换为 camelCase DTO。
- 将内部错误转换为稳定 `CommandError`。

command 不负责：

- 打开 MuPDF document。
- 遍历 Markdown AST。
- 直接拼接设置或批注路径。
- 保存全局裸指针。
- 返回解析器第三方类型。

统一错误协议：

```ts
type CommandError = {
  code:
    | "PDF_PASSWORD_REQUIRED"
    | "PDF_INVALID"
    | "MARKDOWN_INVALID_EDIT"
    | "REVISION_CONFLICT"
    | "PAGE_OUT_OF_RANGE"
    | "IO_DENIED"
    | "INTERNAL";
  message: string;
  retryable: boolean;
  details?: Record<string, unknown>;
};
```

错误 code 是前端控制流程使用的稳定协议，`message` 用于用户反馈；不能让前端通过匹配 Rust 错误字符串判断类型。

## `application/`：用例与端口

```text
application/
├─ services/
│  ├─ pdf_document_service.rs
│  ├─ pdf_page_service.rs
│  ├─ pdf_search_service.rs
│  ├─ markdown_document_service.rs
│  ├─ markdown_parse_service.rs
│  ├─ markdown_search_service.rs
│  ├─ annotation_service.rs
│  ├─ settings_service.rs
│  └─ export_service.rs
└─ ports/
   ├─ pdf_engine.rs
   ├─ pdf_session_repository.rs
   ├─ markdown_parser.rs
   ├─ markdown_session_repository.rs
   ├─ page_cache.rs
   ├─ ocr_engine.rs
   ├─ annotation_repository.rs
   └─ settings_repository.rs
```

该目录的入口分别是同级的 `application.rs`、`application/services.rs` 和 `application/ports.rs`。

application 描述系统能做什么：

- 打开/认证/关闭 PDF。
- 获取页面结构、目录和页面图像。
- 搜索 PDF 并返回文字范围与 quad。
- 打开 Markdown 并创建编辑会话。
- 应用 Markdown 文本 edits，增加 revision 并重新解析。
- 保存指定 revision 的 Markdown。
- 加载、保存和导出批注。

`PdfEngine` 隔离 MuPDF，`MarkdownParser` 隔离具体 Markdown crate。application 测试使用 fake ports，不需要 DLL、真实 WebView 或磁盘文件。

## `domain/`：纯 Rust 领域模型

```text
domain/
├─ ids.rs
├─ document.rs
├─ page.rs
├─ geometry.rs
├─ text.rs
├─ outline.rs
├─ link.rs
├─ render.rs
├─ markdown.rs
├─ annotation.rs
└─ settings.rs
```

该目录的模块入口是同级的 `domain.rs`。

### PDF 模型

```rust
struct PdfDocumentInfo {
    session_id: DocumentSessionId,
    fingerprint: DocumentFingerprint,
    title: String,
    page_count: u32,
    metadata: PdfMetadata,
    outline: Vec<OutlineNode>,
    permissions: PdfPermissions,
}

struct PdfPageStructure {
    page_index: u32,
    size: PageSize,
    rotation: i16,
    blocks: Vec<TextBlock>,
    links: Vec<PageLink>,
}

struct TextBlock {
    bounds: Rect,
    lines: Vec<TextLine>,
}

struct TextSpan {
    text: String,
    bounds: Rect,
    baseline: Point,
    font_size: f32,
    direction: WritingDirection,
}
```

坐标统一使用项目定义的 PDF 页面空间。原点、旋转、裁切框和单位转换只在 `geometry.rs` 定义；不能在多个 commands 中重复 Y 轴翻转公式。

默认文字粒度使用 block → line → span。只有精确选区或搜索需要时才返回字符 quad，避免把每个字符都通过 IPC 发送。

### Markdown 模型

```text
MarkdownDocumentModel
└─ MarkdownNode[]
   ├─ block
   │  ├─ heading
   │  ├─ paragraph
   │  ├─ list / listItem
   │  ├─ quote
   │  ├─ table
   │  └─ codeBlock
   └─ inline
      ├─ text
      ├─ emphasis / strong
      ├─ inlineCode
      ├─ link
      └─ image
```

每个可交互节点具有：

- 稳定 `nodeId`。
- `SourceRange`。
- 明确的可判别 node kind。
- 对应类型才具有的数据字段。

不要返回万能 `MarkdownNode { heading?: ..., table?: ..., code?: ... }`。Rust enum 和 TypeScript discriminated union 必须保持对应。

## `infrastructure/`：技术实现

```text
infrastructure/
├─ mupdf/
│  ├─ ffi.rs
│  ├─ context.rs
│  ├─ engine.rs
│  ├─ document_handle.rs
│  ├─ text_extractor.rs
│  ├─ outline_extractor.rs
│  ├─ link_extractor.rs
│  ├─ renderer.rs
│  └─ conversions.rs
├─ markdown/
│  ├─ parser.rs
│  ├─ ast_converter.rs
│  ├─ source_map.rs
│  └─ sanitizer.rs
├─ sessions/
│  ├─ in_memory_pdf_sessions.rs
│  ├─ pdf_session_worker.rs
│  └─ in_memory_markdown_sessions.rs
├─ cache/
│  ├─ memory_page_cache.rs
│  ├─ disk_render_cache.rs
│  └─ cache_key.rs
├─ filesystem/
│  ├─ document_fs.rs
│  ├─ markdown_file_tree.rs
│  └─ atomic_writer.rs
├─ repositories/
│  ├─ json_annotation_repository.rs
│  └─ json_settings_repository.rs
└─ paths/
   └─ app_paths.rs
```

该目录的总入口是同级的 `infrastructure.rs`；各子目录入口同样采用 `infrastructure/<目录名>.rs`，例如 `infrastructure/markdown.rs` 与 `infrastructure/pdf.rs`。

## MuPDF 集成边界

### `native/mupdf_bridge`

```text
native/
└─ mupdf_bridge/
   ├─ include/mupdf_bridge.h
   ├─ src/mupdf_bridge.c
   └─ README.md
```

MuPDF C API 使用 `fz_try` / `fz_catch`。C 异常跳转不能穿过 Rust FFI 边界，所以桥接层负责：

- 在 C 侧捕获全部 MuPDF 异常。
- 转换为状态码和 UTF-8 错误消息。
- 暴露尺寸与所有权明确的 C ABI。
- 提供配对的 create/drop、alloc/free。
- 不向 Rust 暴露 MuPDF 内部结构体布局。

Rust `infrastructure/mupdf` 再对该 ABI 建立安全包装：

- `context.rs` 管理 context 和锁。
- `document_handle.rs` 管理打开、密码认证和关闭。
- `text_extractor.rs` 转换 structured text。
- `outline_extractor.rs` 转换目录树和目标。
- `link_extractor.rs` 区分页内链接、外部 URI 和无效目标。
- `renderer.rs` 从 display list 渲染原貌页面。
- `conversions.rs` 集中转换坐标、旋转、颜色和 UTF-8 字符串。

MuPDF 类型、裸指针和第三方错误不得离开此模块。

### PDF 只采用原貌阅读

PDF 工作区不是编辑器：

- MuPDF 页面图像是不可编辑的内容底层。
- structured text 用于选择、复制、搜索和批注定位。
- links 用于页内跳转和外部链接。
- 应用批注通过独立覆盖层显示。
- 主题只影响应用外壳、阅读背景和批注颜色。
- 不实现 PDF 正文修改、语义重排或写回原文件。

批注使用 document fingerprint、页码和归一化 quad/range 持久化，不保存 DOM selector 或当前缩放后的像素坐标。

### PDF 页面返回模型

```text
MuPDF document
├─ PdfDocumentInfo JSON
│  ├─ metadata
│  ├─ pageCount
│  └─ outline
├─ PdfPageStructure JSON
│  ├─ size / rotation
│  ├─ blocks / lines / spans
│  └─ links
└─ Page bitmap binary response
```

不要把 MuPDF 的全部矢量绘制指令转换为 JSON 让 React 重画，这会导致巨大协议、字体问题和重复实现 PDF renderer。

MuPDF structured text 不是 OCR。扫描版 PDF 仍需后续实现独立 `OcrEngine` port；OCR 输出转换为相同 PageText 模型，前端不关心文字来源。

## Markdown 解析边界

`infrastructure/markdown` 是具体 Markdown crate 的唯一入口：

- `parser.rs` 调用解析器。
- `ast_converter.rs` 转换为项目自有 AST。
- `source_map.rs` 建立 nodeId 与源码 range 映射。
- `sanitizer.rs` 校验链接 scheme、图片路径和原始 HTML。

后端返回结构化 AST，不返回拼好的 HTML：

```ts
type MarkdownDocumentDto = {
  sessionId: string;
  revision: number;
  source: string;
  nodes: MarkdownNodeDto[];
  outline: MarkdownOutlineNodeDto[];
  diagnostics: MarkdownDiagnosticDto[];
};
```

前端根据 node kind 使用 React 组件渲染，因此更换主题、字体、加粗方式或代码块样式不需要重新设计解析协议。

### Markdown 编辑协议

```text
open_markdown_document
→ sessionId + revision 1 + source + AST

apply_markdown_edits(sessionId, baseRevision, edits)
→ 校验 revision
→ 应用源码范围修改
→ 重新解析
→ revision + 1 + AST/outline/diagnostics
```

编辑 DTO 的字符偏移必须明确。建议使用 `startUtf16` / `endUtf16` 与 JavaScript 编辑器对齐，进入领域层前统一转换为 Rust UTF-8 byte range。

保存时必须指定 revision。后端只保存仍为最新的版本，否则返回 `REVISION_CONFLICT`，避免旧的异步解析结果覆盖新输入。

## 文档会话与线程模型

### PDF session

同一 MuPDF document 不能由多个线程同时访问。首版使用“每个打开 PDF 一个串行 worker”：

```text
Tauri async command
  ↓ request + oneshot response
PDF session worker
  ├─ owns MuPDF document handle
  ├─ serializes page load and parse
  ├─ builds reusable display lists
  └─ returns owned Rust domain data
```

- MuPDF 阻塞工作不能占用 Tauri 主线程。
- 同一文档句柄访问天然串行。
- 不同 PDF session 可以并行。
- display list 并行渲染只在性能测试证明需要后启用。
- 长渲染使用 MuPDF cookie 支持取消。
- 前端关闭标签后立即调用 `close_pdf_document`。

### Markdown session

每个 session 保存：

- 当前源码。
- 最新 revision。
- 当前 AST、outline 和 diagnostics。
- dirty/保存基准所需的后端状态。

编辑操作按 revision 串行应用。解析可放入 blocking worker，但提交结果前必须再次确认 session revision 没有前进。

### AppState

```rust
struct AppState {
    pdf_documents: Arc<PdfDocumentService>,
    pdf_pages: Arc<PdfPageService>,
    markdown_documents: Arc<MarkdownDocumentService>,
    markdown_parser: Arc<MarkdownParseService>,
    annotations: Arc<AnnotationService>,
    settings: Arc<SettingsService>,
}
```

AppState 不保存当前标签、页码、缩放、主题或侧栏状态。

## 缓存策略

| 缓存             | 内容                                        | 生命周期                |
| ---------------- | ------------------------------------------- | ----------------------- |
| PDF session      | MuPDF document、outline、metadata           | PDF 标签打开期间        |
| PDF 页面结构     | 尺寸、文字块和链接                          | session 内 LRU          |
| PDF render       | 页面图像                                    | 内存 LRU + 应用缓存目录 |
| Markdown session | source、revision、AST、outline、diagnostics | Markdown 标签打开期间   |

PDF render key 至少包含：

```text
document fingerprint + page index + scale bucket + rotation + devicePixelRatio + color mode
```

fingerprint 首版使用规范化路径、文件大小和修改时间；文件变化后会话和缓存失效。需要跨路径识别同一内容时再增加分块内容哈希。

不要为 50% 到 250% 的每个连续缩放值生成位图。前端缩放期间暂时缩放最近图片，停止后请求离散渲染档位。

## IPC 与二进制传输

- 元数据、目录、AST、文字结构、链接和搜索结果使用 JSON DTO。
- PDF 页面图像不能返回 `number[]`。
- 首版 `render_pdf_page` 返回 `tauri::ipc::Response` 二进制 PNG/WebP。
- 缓存成熟后可以注册只读异步自定义 protocol，让 `<img>` 直接加载。
- protocol 只接受不可猜测 session token 和经过校验的数值参数，不能接受任意磁盘路径。

Tauri event 只用于无需请求响应的通知或进度，不替代类型明确的 command。

## 目标 PDF Commands

```text
detect_document_kind(path)
open_pdf_document(path, password?)
authenticate_pdf_document(sessionId, password)
get_pdf_page_structure(sessionId, pageIndex, detail)
render_pdf_page(sessionId, pageIndex, scale, rotation, devicePixelRatio)
search_document(requestId, target, query, options)
cancel_document_search(requestId, target)
get_pdf_outline(sessionId)
close_pdf_document(sessionId)
```

| Command                    | 返回                    | 说明                                                                                     |
| -------------------------- | ----------------------- | ---------------------------------------------------------------------------------------- |
| `detect_document_kind`   | `DocumentKindDto`     | PDF 按`%PDF-` 文件签名识别；Markdown 支持 `.md`、`.markdown`、`.mdown`、`.mkd` |
| `open_pdf_document`      | `PdfDocumentDto`      | 元数据、页数、目录和 sessionId，不返回整份文件                                           |
| `get_pdf_page_structure` | `PdfPageStructureDto` | 页面尺寸、文字 block/line/span 和链接                                                    |
| `render_pdf_page`        | 二进制                  | 按需原貌渲染，不经过 JSON 数组                                                           |
| `search_document`        | `SearchResponseDto`   | 通用结果外壳；PDF 返回页码、上下文和一个或多个矩形                                       |
| `cancel_document_search` | `()`                  | 取消旧 requestId，避免旧结果覆盖新查询                                                   |
| `close_pdf_document`     | `()`                  | 释放句柄和 session 缓存                                                                  |

## 目标 Markdown Commands

```text
open_markdown_document(path)
apply_markdown_edits(sessionId, baseRevision, edits)
get_markdown_structure(sessionId, revision?)
search_document(requestId, { kind: "markdown", sessionId, revision }, query, options)
cancel_document_search(requestId, target)
save_markdown_document(sessionId, revision)
save_markdown_document_as(sessionId, revision, path)
close_markdown_document(sessionId)
```

前端 repository 是唯一知道这些命令名称和 DTO 的模块，React 组件不能直接 invoke。

## 持久化与路径

默认应用数据目录由 `AppPaths` 从 `%LOCALAPPDATA%\Bambook` 唯一派生：

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

持久化要求：

- `StorageService` 在应用状态创建前初始化全部必需目录。
- `StorageRepository` 隔离 application 层与本地文件系统实现。
- `document-library.json` 是托管文档的轻量索引；开始页面只读取该文件，不在每次显示时遍历 `storage`。
- PDF 与 Markdown 可以使用各自的存储根目录，并可选择保存托管副本或仅记录源文件绝对链接。
- 链接失效后的重新映射会同步更新 `document-library.json` 与文档目录中的 `metadata.json`。
- PDF 伴随数据采用“当前 PDF 目录侧车文件 → Bambook 托管目录 → 空状态”的读取优先级；纯读取不会创建目录或空 JSON。
- `annotations.json` 使用 schema v2 对象格式并记录根级 `updatedAt`；旧数组格式可读取并在下次保存时迁移。覆盖有效文件前生成 `.bak`，主文件损坏时按时间从备份或另一存储来源恢复。
- 索引首次创建时会从已有文档元数据执行一次迁移，此后由文档导入和打开流程增量维护。
- PDF/Markdown 打开时只建立内存会话；首次保存时才创建稳定 ID 目录，并按设置复制托管副本或记录源文件链接。
- PDF 批注、笔记、书签和阅读状态均使用同一 `pdf-id` 目录，书签与批注共同参与未保存状态和关闭事务。
- `reading-state.json` 由阅读过程自动防抖更新，不参与文档未保存提示；切换标签和关闭会执行一次尽力刷新。
- `DocumentHistoryService` 统一执行最近文件排序、Windows 路径去重、关闭栈上限和恢复确认。
- `DocumentPathInspector` 在读取历史记录时实时检查路径，返回 `available`、`missing`、`inaccessible` 或 `notFile`；状态不写入 JSON，避免文件恢复后留下过期结果。
- `WorkspaceStateService` 保存可重新打开的磁盘文档顺序、活动文档路径与文件夹路径，不持久化进程内 sessionId。
- `JsonSettingsRepository` 会将无版本号的旧设置原子迁移到当前 schema。
- `JsonDocumentHistoryRepository` 分别写入 `recent-documents.json` 和 `closed-documents.json`。
- `JsonWorkspaceStateRepository` 使用 `workspace-state.json` 保存工作区状态。
- 启动时逐个验证必需目录可写；探测文件无论成功或失败都会清理。
- 设置和批注 JSON 增加 schema version。
- JSON 先完整序列化，再写入目标文件同目录的唯一临时文件。
- 临时文件执行 flush 与 `sync_all` 后再替换目标文件；Windows 使用 `MoveFileExW` 的 replace-existing 与 write-through 标志。
- 原子替换失败时保留原目标文件，并由 guard 清理临时文件。
- 文件名过滤 Windows 非法字符、尾随空格/句点和 `CON`、`NUL` 等保留设备名。
- 路径由 `AppPaths` 统一生成，commands 不拼接路径。
- Markdown 另存为和任意目录访问必须重新校验用户选择范围。

## 安全与资源限制

- 验证文件存在、类型、文件头和大小。
- 后端再次校验 page index、scale、rotation、搜索长度和 edit range。
- PDF 密码不记录日志、不持久化。
- 限制单页像素数、缓存字节数、AST 节点数、目录深度和文件树节点数。
- LRU 与显式 close 同时使用；退出时 session manager 兜底释放句柄。
- Rust panic 和 MuPDF 错误不能 unwind 穿过 C FFI。
- 自定义 protocol 不暴露任意本地文件路径。
- Markdown 原始 HTML、链接 scheme 和图片路径必须按产品策略过滤。

## Windows 构建

### 基础环境

1. Node.js 18 或更高版本。
2. Rust MSVC 工具链，建议使用 `rustup`。
3. Visual Studio 2022 Build Tools 的 C++ 桌面开发组件。
4. Windows SDK。
5. Microsoft Edge WebView2 Runtime。

如果 `cargo metadata` 提示 program not found，确认 `%USERPROFILE%\.cargo\bin` 已加入 `PATH`。项目的 npm 脚本也会在 Windows 命令中补充该目录。

### MuPDF 原生构建

`build.rs` 只负责：

- 构建 `mupdf_bridge.c`。
- 定位固定版本的 MuPDF include/lib。
- 向 Cargo 输出 link search 和 link lib。

普通构建不能隐式联网下载 SDK。MuPDF 版本、构建选项和二进制校验值记录在 `native/mupdf_bridge/README.md`。

Windows 安装包需要验证：

- x64 MSVC 与 MuPDF 库架构一致。
- C/C++ 运行库选项一致。
- 动态链接时 DLL 被收入 Tauri bundle 并能被 exe 发现。
- 干净 Windows 虚拟机无需开发环境即可启动。
- MuPDF 版本和崩溃符号可以追踪。

### MuPDF 许可证

MuPDF 官方开源发行版采用 AGPL；嵌入非 AGPL 产品通常需要商业许可。发布安装包前必须明确本项目的许可证和分发方式，或取得合适的 MuPDF 商业许可。目录分层不能规避许可证义务。

## 开发与验证

从项目根目录运行桌面开发：

```powershell
npm run tauri:dev
```

验证 Rust：

```powershell
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

构建 Windows NSIS 安装包：

```powershell
npm run tauri:build:windows
```

构建产物：

- 可执行文件：`src-tauri/target/release/bambook.exe`
- NSIS：`src-tauri/target/release/bundle/nsis/`

## 测试分层

```text
domain unit tests
├─ 坐标、旋转、ID 和缓存键
└─ Markdown AST、source range 和目录转换

application tests + fake ports
├─ PDF 打开、分页、搜索和关闭
├─ Markdown edit、revision、解析和保存
└─ 错误映射与资源释放

infrastructure integration tests
├─ PDF fixtures 验证 MuPDF 解析与渲染
├─ Markdown fixtures 验证 AST、目录和安全规则
└─ JSON repository、路径和原子写入

command contract tests
└─ Rust DTO 与 TypeScript 字段和错误 code 一致
```

PDF fixtures 至少覆盖普通文本、复杂排版、CJK、旋转页、目录、内部链接、加密、扫描页和损坏文件。Markdown fixtures 至少覆盖标题、嵌套列表、代码块、表格、图片、原始 HTML、不安全链接和 Unicode source range。

## 已完成迁移与后续事项

为了保证当前功能始终可运行：

1. 拆分 `lib.rs`，建立 error、domain、commands、application 和 infrastructure，不改变现有协议。
2. 建立 session ID、revision、document envelope、结构化错误和 DTO 约定。
3. 引入 `MarkdownParser` port 和自有 AST，通过 fixtures 验证节点、source range、目录与安全规则。
4. 实现 Markdown session、edit revision 和保存用例；前端渲染后端 AST。
5. 移除前端 `marked`、`DOMPurify` 和 `markdownOutlineParser` 解析职责。
6. 引入 `PdfEngine` port、PDF 领域 DTO 和 fake engine。
7. 建立 C bridge 与 MuPDF safe wrapper，实现打开、元数据、页数和关闭。
8. 增加目录、页面尺寸、structured text 和链接解析，验证坐标。
9. 增加按需页面二进制渲染和缓存。
10. 前端接入 MuPDF 页面、文字层、目录和缩略图。
11. 迁移搜索、目录跳转、选区和批注，删除 `read_pdf_file`、`PDFDocumentProxy` 和 PDF.js 链路。
12. 最后评估异步 protocol、并行 display list 渲染和 OCR；不规划 PDF 正文编辑或语义重排。

后续事项包括页面缓存、链接热点层、OCR、取消长渲染和更细粒度的 Markdown 增量解析。

## 需要避免的耦合

- 不在 `lib.rs` 堆积全部 commands、模型和文件系统逻辑。
- 不让 commands 直接使用 MuPDF 或 Markdown parser。
- 不让 domain 依赖 Tauri、serde DTO 约定或 Windows API。
- 不把 MuPDF handle 放进普通并发 map 后任意跨线程访问。
- 不返回第三方 AST、MuPDF 结构体或裸指针。
- 不把页面二进制序列化为 JSON number array。
- 不在不同模块重复实现路径、文件名或坐标转换。
- 不把 UI 状态放入 AppState。
- 不把 PDF 和 Markdown 内容压缩为万能文档结构。

## 设计依据

- [MuPDF C API 概览](https://mupdf.readthedocs.io/en/1.28.0/reference/c/fitz/api-overview.html)
- [MuPDF 多线程说明](https://mupdf.readthedocs.io/en/latest/reference/c/overview.html#multi-threading)
- [MuPDF display list API](https://mupdf.readthedocs.io/en/1.28.0/_static/generated/c/html/display-list_8h.html)
- [Tauri v2：从前端调用 Rust](https://v2.tauri.app/develop/calling-rust/)
- [Tauri v2：asset protocol scope](https://v2.tauri.app/security/asset-protocol/)
- [MuPDF 官方发行页](https://mupdf.com/releases)

# 前后端 IPC 接口

[返回技术总览](README.md)

本文依据 **2026-11-26 工作区代码**整理，覆盖 [app/ipc.rs](../../src-tauri/src/app/ipc.rs) 中注册的全部 **58 个业务 command**，包括入参、成功返回值、前后端入口和主要数据结构。接口可调用性以注册表为准；下文描述来自静态代码核对，不表示完成了所有平台的运行验收。

## 001 · 通信入口与公共约定

前端统一通过 [invokeCommand](../../src/platform/tauriClient.ts) 调用 `@tauri-apps/api/core` 的 `invoke`，业务 repository / platform client 负责参数组装与模型映射。后端由 [bootstrap.rs](../../src-tauri/src/app/bootstrap.rs) 装载 `app/ipc.rs` 的 handler，再分发到各 command。

```text
React 组件 / Provider
  → repository / platform client
  → invokeCommand<TResult>(command, args)
  → app/ipc.rs 注册的 Rust command
  → workflow / service → DTO / PNG / Channel
```

| 项目 | 约定 |
| --- | --- |
| 命令名 | `snake_case`，例如 `open_pdf_document` |
| 调用参数 | 顶层使用 `camelCase`，例如 Rust 的 `session_id` 对应 `sessionId` |
| DTO 字段 | 依据 Serde 声明，通常为 `camelCase`；枚举通过 `kind` 或 `type` 区分 |
| 后端注入参数 | `State<…>`、`AppHandle` 由 Tauri 注入，前端不传 |
| 成功结果 | `AppResult<T>` 成功时直接返回 `T`，没有 `{ data, success }` 包装 |
| 无业务返回值 | 表中记为 `void`，Rust 实际为 `()`，JSON 序列化值为 `null` |
| 可空值 | Rust `Option<T>` 输出通常是 `T` 或 `null`；有 `skip_serializing_if` 的字段可能省略 |
| 可选入参 | 表中 `?` 表示可以省略；`Option<T>` 也可显式传 `null` |
| 数字类型 | 文档以 TS `number` 表示 Rust 整数和浮点数；计数、版本和索引应为非负整数，并保持在 JS 安全整数范围内 |
| 类型保障 | `invokeCommand<TResult>` 只提供静态提示，不验证运行时响应；当前没有自动生成的 TS / Rust 协议绑定 |

下文参数列是传给 `invokeCommand` 的第二个对象参数，而非位置参数。表中出现的 `Dto` 名称对应 Rust 类型，字段说明均按实际传输的 JSON 名称书写。

### 标识、页码与偏移

| 字段 / 场景 | 含义与单位 |
| --- | --- |
| `documentId` | 文档及附属数据的业务身份；导出批注文档对象中使用 `id` 字段 |
| `sessionId` | 当前进程内文档会话标识，关闭或重启后不能作为恢复依据 |
| `revision` / `baseRevision` | Markdown 内容版本；新会话从 1 开始，成功编辑后递增 |
| `requestId` | 搜索请求序号，与内容版本独立；用于取消和丢弃过期结果 |
| `path` / `sourcePath` / `dataPath` | 当前访问文件 / 来源文件 / 附属数据目录；托管导入可能改变 `path`，应采用后端返回值 |
| PDF `pageIndex`、原始 `outline[].page` | 从 **0** 开始 |
| PDF 书签、阅读状态、批注、搜索 locator 的 `page`，页面链接的 `targetPage` | 从 **1** 开始；调用页面结构和渲染接口时需减 1 |
| PDF 页面 `width` / `height`、文字和链接 `bounds` | 未按渲染 `scale` 放大的页面坐标，`bounds` 使用 `left/top/right/bottom` |
| 搜索 `rects`、批注 `rects`、书签 `x/y`、阅读位置 `pageY`、链接 `targetX/targetY` | 相对页面尺寸的归一化位置；矩形使用 `left/top/width/height` |
| Markdown 编辑和搜索 `startUtf16/endUtf16` | UTF-16 code unit 偏移，区间为 `[start, end)` |
| Markdown AST / diagnostic `range.start/end` | **UTF-8 字节偏移**，不能直接作为 JS 字符串切片位置 |
| Markdown 搜索 `line/column` | 从 1 开始；当前 `column` 按 Rust `char` 计数，不是 UTF-16 偏移 |

## 002 · 文档识别、目录与系统操作（9 个）

后端：[文档 commands](../../src-tauri/src/features/documents/commands.rs)、[桌面 commands](../../src-tauri/src/platform/desktop/commands.rs)。

前端：[documentRepository](../../src/features/documents/repositories/documentRepository.ts)、[markdownRepository](../../src/features/markdown/repositories/markdownRepository.ts)、[settingsRepository](../../src/features/settings/repositories/settingsRepository.ts)、[externalLink](../../src/platform/externalLink.ts)。

| command | 入参 | 成功返回 | 功能与注意事项 |
| --- | --- | --- | --- |
| `detect_document_kind` | `{ path: string }` | `{ kind: "pdf" 或 "markdown" }` | 读取文件头，以 `%PDF-` 识别 PDF；Markdown 按扩展名识别；不支持类型拒绝调用 |
| `list_markdown_directory` | `{ path: string }` | `DirectoryEntryDto[]` | **传 Markdown 文件路径**，从其父目录枚举 Markdown 树 |
| `list_supported_directory` | `{ path: string }` | `DirectoryEntryDto[]` | 传目录路径，枚举 PDF / Markdown 树 |
| `rename_markdown_document` | `{ path: string, newPath: string }` | `void` | 文件重命名；当前要求 `newPath` 以小写 `.md` 结尾 |
| `rename_pdf_document` | `{ path: string, newPath: string }` | `void` | 文件重命名；command 不返回更新后的会话 |
| `open_app_data_directory` | 无 | `void` | 打开 `local_data_dir()/Bambook` |
| `reveal_in_file_manager` | `{ path: string }` | `void` | 在系统文件管理器中定位文件 |
| `open_directory_in_file_manager` | `{ path: string }` | `void` | 在系统文件管理器中打开目录 |
| `open_external_url` | `{ url: string }` | `void` | 允许 `http://`、`https://`、`mailto:`，拒绝其他协议和含 NUL 的字符串 |

目录 DTO：`{ name: string, path: string, isDirectory: boolean, children: DirectoryEntryDto[] }`。支持的 Markdown 扩展名为 `.md/.markdown/.mdown/.mkd`，识别时不区分大小写。目录枚举跳过符号链接和没有匹配文件的子目录，最大递归深度为 12；不要将其作为无限深度文件系统遍历接口。

重命名 command 只调用文件层重命名；前端 [renameDocument](../../src/features/documents/repositories/documentRepository.ts) 还会执行关闭、重新打开等协调操作。不要假定直接调用重命名就能同步所有会话和文档库状态。

## 003 · PDF 会话与页面（6 个）

后端：[应用 commands](../../src-tauri/src/app/commands.rs)、[PDF commands](../../src-tauri/src/features/pdf/commands.rs)、[PDF 生命周期](../../src-tauri/src/app/workflows/pdf_lifecycle.rs)。

前端：[documentRepository](../../src/features/documents/repositories/documentRepository.ts)、[mupdfRepository](../../src/features/pdf/repositories/mupdfRepository.ts)、[pdfStorageRepository](../../src/features/pdf/repositories/pdfStorageRepository.ts)。

| command | 入参 | 成功返回 | 功能与注意事项 |
| --- | --- | --- | --- |
| `open_pdf_document` | `{ path: string, password?: string 或 null }` | `PdfDocumentDto` | 打开会话并注册搜索；检查返回的 `needsPassword` |
| `authenticate_pdf_document` | `{ sessionId: string, password: string }` | `PdfDocumentDto` | 验证已有会话并同步搜索 worker 的认证状态 |
| `close_pdf_document` | `{ sessionId: string }` | `void` | 释放 PDF 和搜索资源；不负责前端未保存确认 |
| `persist_pdf_document` | `{ sessionId: string }` | `void` | 按当前设置持久化文档；书签、批注、阅读状态通过各自接口保存 |
| `get_pdf_page_structure` | `{ sessionId: string, pageIndex: number }` | `PdfPageStructureDto` | 获取文字、矩形与链接，页索引从 0 开始 |
| `render_pdf_page` | `{ sessionId: string, pageIndex: number, scale: number }` | PNG 二进制 | `tauri::ipc::Response`；`scale` 允许范围为 `[0.25, 5.0]`，越界当前返回 `PageOutOfRange` |

### PDF 响应结构

字段依据 [pdf/dto.rs](../../src-tauri/src/features/pdf/dto.rs)，前端对应 [pdfStructure.ts](../../src/features/pdf/types/pdfStructure.ts)。以下为传输结构示意，`null` 按 Rust DTO 标明。

```ts
type PdfRect = { left: number; top: number; right: number; bottom: number };
type PdfMetadataDto = {
  title: string; author: string; subject: string;
  keywords: string; creator: string; producer: string;
};
type PdfOutlineNodeDto = {
  title: string; page: number | null; x: number | null; y: number | null;
  uri: string | null; children: PdfOutlineNodeDto[];
};
type PdfDocumentDto = {
  documentId: string; sessionId: string; path: string; sourcePath: string;
  title: string; pageCount: number; needsPassword: boolean;
  metadata: PdfMetadataDto; outline: PdfOutlineNodeDto[];
};
type PdfPageStructureDto = {
  pageIndex: number; width: number; height: number;
  blocks: Array<{
    bounds: PdfRect;
    lines: Array<{
      bounds: PdfRect;
      spans: Array<{ text: string; bounds: PdfRect; fontSize: number }>;
    }>;
  }>;
  links: Array<{
    bounds: PdfRect; uri: string;
    targetPage: number | null; targetX: number | null; targetY: number | null;
  }>;
};
```

当前 PDF 目录节点的 `x/y` 为 `null`；目录 `page` 保留引擎的 0 基页索引，前端 `usePdfOutline` 转成 1 基页码。页面内部链接的 `targetPage` 则已在后端转成 1 基页码。部分前端 TS 类型仅声明 `?: number`，但实际 JSON 可返回 `null`，接入时应同时处理。

### PNG 调用示例

```ts
// invokeCommand 从 src/platform/tauriClient 导入，按调用模块位置调整相对路径。
const raw = await invokeCommand<ArrayBuffer | Uint8Array>("render_pdf_page", {
  sessionId, pageIndex: 0, scale: 1.5,
});
const bytes = raw instanceof Uint8Array ? raw : new Uint8Array(raw);
const url = URL.createObjectURL(new Blob([bytes], { type: "image/png" }));
// 图片使用完毕后 URL.revokeObjectURL(url)。
```

业务代码优先使用 `mupdfRepository.renderPage()`，它已完成二进制归一化、缓存和并发请求复用。PNG 不是 Base64 JSON，也不包含页面文字结构。

## 004 · PDF 书签与阅读状态（4 个）

后端：[storage_commands.rs](../../src-tauri/src/features/pdf/storage_commands.rs)、[storage_dto.rs](../../src-tauri/src/features/pdf/storage_dto.rs)。前端：[pdfStorageRepository](../../src/features/pdf/repositories/pdfStorageRepository.ts)。

| command | 入参 | 成功返回 |
| --- | --- | --- |
| `load_pdf_bookmarks` | `{ documentId: string, sourcePath?: string 或 null }` | `PdfBookmarkDto[]` |
| `save_pdf_bookmarks` | `{ documentId: string, sourcePath?: string 或 null, bookmarks: PdfBookmarkDto[] }` | `void` |
| `load_pdf_reading_state` | `{ documentId: string, sourcePath?: string 或 null }` | `PdfReadingStateDto 或 null` |
| `save_pdf_reading_state` | `{ documentId: string, sourcePath?: string 或 null, readingState: PdfReadingStateDto }` | `void` |

```ts
type PdfBookmarkDto = {
  id: string; documentId: string; page: number; x: number; y: number;
  title: string; color: string; createdAt: string;
};
type PdfReadingStateDto = {
  page: number; pageY: number | null; zoomMode: string;
  scale: number; updatedAt: string;
};
```

书签 `color` 省略时默认 `#26765A`；阅读状态 `zoomMode` 的前端取值为 `custom / fit-width / fit-page`，Rust DTO 接收字符串。`save_pdf_bookmarks` 传入完整数组，不能把它当作单条追加接口。日期字段为字符串。

## 005 · Markdown 会话、编辑、保存与恢复（13 个）

后端：[应用 commands](../../src-tauri/src/app/commands.rs)、[Markdown commands](../../src-tauri/src/features/markdown/commands.rs)、[文档生命周期](../../src-tauri/src/app/workflows/document_lifecycle.rs)。

前端：[structuredMarkdownRepository](../../src/features/markdown/repositories/structuredMarkdownRepository.ts)、[documentRepository](../../src/features/documents/repositories/documentRepository.ts)。

| command | 入参 | 成功返回 | 功能与注意事项 |
| --- | --- | --- | --- |
| `create_markdown_document` | `{ path: string }` | `void` | 创建磁盘文件，不返回编辑会话；需再打开 |
| `create_untitled_markdown_document` | 无 | `MarkdownDocumentDto` | 创建内存文档，初始 `path` 为空字符串 |
| `open_markdown_document` | `{ path: string }` | `MarkdownDocumentDto` | 准备文档库路径、读取文本并解析 |
| `get_markdown_structure` | `{ sessionId: string }` | `MarkdownDocumentDto` | 获取会话当前全文及结构 |
| `apply_markdown_edits` | `{ sessionId: string, baseRevision: number, edits: MarkdownEditDto[] }` | `MarkdownDocumentDto` | 检查基础版本，应用编辑并返回完整新快照 |
| `save_markdown_document` | `{ sessionId: string, revision: number }` | `void` | 保存对应版本并协调文档库；未命名文档应先另存为 |
| `save_markdown_document_as` | `{ sessionId: string, revision: number, path: string }` | `MarkdownDocumentDto` | 另存并更新会话路径，调用方采用返回快照 |
| `close_markdown_document` | `{ sessionId: string }` | `void` | 关闭会话并清理搜索状态 |
| `export_markdown_html` | `{ sessionId: string, revision: number, path: string }` | `void` | 导出指定版本，目标使用 `.html` 扩展名 |
| `export_markdown_pdf` | `{ sessionId: string, revision: number, path: string }` | `void` | 导出指定版本，目标使用 `.pdf` 扩展名 |
| `save_untitled_markdown_recovery` | `{ content: string }` | `void` | 写入未命名草稿恢复内容，不带会话标识 |
| `load_untitled_markdown_recovery` | 无 | `string 或 null` | 读取恢复内容，无数据时为空值 |
| `clear_untitled_markdown_recovery` | 无 | `void` | 清理恢复内容 |

### Markdown DTO 与版本规则

字段依据 [markdown/dto.rs](../../src-tauri/src/features/markdown/dto.rs)，前端对应 [markdownAst.ts](../../src/features/markdown/types/markdownAst.ts)。

```ts
type SourceRangeDto = { start: number; end: number }; // UTF-8 字节
type MarkdownEditDto = { startUtf16: number; endUtf16: number; text: string };
type MarkdownOutlineNodeDto = {
  id: string; nodeId: string; level: number; title: string;
  children: MarkdownOutlineNodeDto[];
};
type MarkdownDiagnosticDto = {
  severity: string; message: string; range: SourceRangeDto | null;
};
// MarkdownNodeDto 的 type 和专用字段直接平铺在节点上，详见下表。
type MarkdownDocumentDto = {
  documentId: string; sessionId: string; revision: number;
  path: string; title: string; source: string;
  nodes: MarkdownNodeDto[]; outline: MarkdownOutlineNodeDto[];
  diagnostics: MarkdownDiagnosticDto[];
};
```

每个节点都有 `id`、`range` 和 `type`。`type` 及附加字段如下：

| type | 附加字段 |
| --- | --- |
| `paragraph`、`blockQuote`、`listItem`、`definitionList`、`definitionTitle`、`definition`、`tableHead`、`tableRow`、`tableCell`、`emphasis`、`strong`、`strikethrough`、`superscript`、`subscript`、`metadata` | `children: MarkdownNodeDto[]` |
| `heading` | `level: number, children` |
| `codeBlock` | `language: string 或 null, children` |
| `list` | `start: number 或 null, children` |
| `footnoteDefinition` | `label: string, children` |
| `table` | `alignments: string[], children` |
| `link`、`image` | `url: string, title: string, children` |
| `text`、`inlineCode`、`html` | `text: string` |
| `math` | `text: string, display: boolean` |
| `footnoteReference` | `label: string` |
| `taskListMarker` | `checked: boolean` |
| `softBreak`、`hardBreak`、`rule` | 无 |

`apply_markdown_edits` 的所有区间都基于 `baseRevision` 的源码，后端检查版本、边界、代理对和重叠；成功后返回递增的 `revision`。前端应串行提交依赖上次结果的编辑，并使用返回版本发起保存、导出和搜索。版本冲突通过 Promise rejection 返回，不会自动合并。

```ts
const updated = await invokeCommand<StructuredMarkdownDocument>(
  "apply_markdown_edits",
  {
    sessionId,
    baseRevision: revision,
    edits: [{ startUtf16: 0, endUtf16: 0, text: "# 标题\n" }],
  },
);
await invokeCommand<void>("save_markdown_document", {
  sessionId: updated.sessionId,
  revision: updated.revision,
});
```

示例中的类型来自前端 `markdownAst.ts`，省略导入。保存接口接收会话和版本，**不接收 `content`**；应先提交编辑再保存。恢复接口单独接收字符串，不能替代会话编辑与版本管理。

## 006 · 统一文档搜索（2 个）

后端：[search/commands.rs](../../src-tauri/src/features/search/commands.rs)、[search/dto.rs](../../src-tauri/src/features/search/dto.rs)。前端：[documentSearchRepository](../../src/features/search/repositories/documentSearchRepository.ts)、[search.ts](../../src/features/search/types/search.ts)。

| command | 入参 | 成功返回 |
| --- | --- | --- |
| `search_document` | `{ request: SearchRequestDto }` | `SearchResponseDto` |
| `cancel_document_search` | `{ request: { requestId: number, target: SearchTargetDto } }` | `void` |

```ts
type SearchTargetDto =
  | { kind: "pdf"; sessionId: string }
  | { kind: "markdown"; sessionId: string; revision: number };
type SearchRequestDto = {
  requestId: number; target: SearchTargetDto; query: string;
  options?: { caseSensitive?: boolean; wholeWord?: boolean; resultLimit?: number | null };
};
type SearchRectDto = { left: number; top: number; width: number; height: number };
type SearchLocatorDto =
  | { kind: "pdf"; page: number; pageY: number; rects: SearchRectDto[] }
  | { kind: "markdown"; revision: number; startUtf16: number; endUtf16: number;
      line: number; column: number };
type SearchResponseDto = {
  requestId: number; total: number; truncated: boolean; cancelled: boolean;
  matches: Array<{
    id: string; matchedText: string; beforeText: string; afterText: string;
    locationLabel: string; locator: SearchLocatorDto;
  }>;
};
```

`caseSensitive/wholeWord` 默认 `false`，`resultLimit` 默认 500，后端限制在 `[1, 5000]`。`total` 为搜索统计数量，`matches` 受上限限制，`truncated` 表示结果截断。

取消针对目标会话和请求序号，不代表通用任务取消能力。前端仍需核对 `requestId`、会话和 Markdown `revision`，忽略过期结果。当前没有 PDF 渲染或 LLM 请求对应的取消 command。

```ts
const result = await invokeCommand<SearchResponse>("search_document", {
  request: {
    requestId: 1,
    target: { kind: "markdown", sessionId, revision },
    query: "标题",
    options: { caseSensitive: false, wholeWord: false, resultLimit: 500 },
  },
});
await invokeCommand<void>("cancel_document_search", {
  request: { requestId: 1, target: { kind: "markdown", sessionId, revision } },
});
```

## 007 · 批注与批注导出（4 个）

后端：[annotations/commands.rs](../../src-tauri/src/features/annotations/commands.rs)、[export_commands.rs](../../src-tauri/src/features/annotations/export_commands.rs)、[annotations/dto.rs](../../src-tauri/src/features/annotations/dto.rs)。前端：[annotationRepository](../../src/features/annotations/repositories/annotationRepository.ts)、[types.ts](../../src/features/annotations/types.ts)。

| command | 入参 | 成功返回 |
| --- | --- | --- |
| `load_annotations` | `{ documentId: string, sourcePath?: string 或 null, settings: AppSettingsDto }` | `AnnotationDto[]` |
| `save_annotations` | `{ documentId: string, sourcePath?: string 或 null, annotations: AnnotationDto[], settings: AppSettingsDto }` | `void` |
| `migrate_pdf_annotations` | `{ fromDocumentId: string, toDocumentId: string, settings: AppSettingsDto }` | `void` |
| `export_annotations_markdown` | `{ document: ReaderDocumentDto, annotations: AnnotationDto[], settings: AppSettingsDto }` | `string`（导出文件路径） |

```ts
type ReaderDocumentDto = { id: string; title: string; path: string; kind: string };
type AnnotationDto = {
  id: string; documentId: string; documentTitle: string; type: string;
  page?: number | null; selectedText: string; note: string; hasNote?: boolean;
  color: string; createdAt: string; updatedAt: string;
  rects?: Array<{ left: number; top: number; width: number; height: number }> | null;
  areaStyle?: {
    backgroundColor: string; opacity?: number; borderStyle: string;
    borderWidth: number; radius: number;
    roundedCorners: { topLeft: boolean; topRight: boolean; bottomRight: boolean; bottomLeft: boolean };
  };
};
```

前端 `type` 取值为 `note / area / highlight / underline / squiggly / strikeout`；`borderStyle` 为 `dotted / dashed / solid / none`，Rust DTO 目前均使用字符串。`hasNote` 默认 `false`，为 `false` 时响应省略；`areaStyle` 为空时省略，`opacity` 输入省略时默认 `0.22`。`page/rects` 的空值响应为 `null`。

保存批注传入完整数组；导出返回路径而非 Markdown 正文。批注接口显式接收设置对象，路径选择依据该对象，不能只传入 `documentId`。

## 008 · 文档库、存储协调与文档包（5 个）

后端：[library/commands.rs](../../src-tauri/src/features/library/commands.rs)、[library/dto.rs](../../src-tauri/src/features/library/dto.rs)、[应用 commands](../../src-tauri/src/app/commands.rs)。前端：[documentLibraryRepository](../../src/features/documents/repositories/documentLibraryRepository.ts)。

| command | 入参 | 成功返回 | 功能与注意事项 |
| --- | --- | --- | --- |
| `load_document_library` | 无 | `StoredDocumentEntryDto[]` | 加载文档库及路径可用状态 |
| `inspect_document_relink` | `{ documentId: string, newPath: string }` | `RelinkInspectionDto` | 检查候选文件名，供前端确认使用 |
| `relink_document` | `{ documentId: string, newPath: string }` | `StoredDocumentEntryDto` | 重新关联文件并返回更新记录 |
| `reconcile_document_storage` | `{ documentId: string }` | `void` | 按当前设置协调文档存储 |
| `export_document_package` | `{ documentId: string, destination: string }` | `string`（文档包路径） | 导出文档包，并将包记录加入关闭历史 |

```ts
type StoredDocumentEntryDto = {
  id: string; title: string; kind: "pdf" | "markdown";
  path: string; sourcePath: string; dataPath: string; storageMode: string;
  createdAt: number; updatedAt: number; lastOpenedAt: number;
  pathAvailable: boolean; sourceAvailable: boolean;
};
type RelinkInspectionDto = { recordedName: string; selectedName: string; nameMatches: boolean };
```

文档库时间字段为 Unix 毫秒。`storageMode` 在协议层为字符串，存储细节见 [存储设计](storage.md)。重链接后应采用返回记录；文件名检查与真正重链接是两个独立命令。

## 009 · 文档历史与工作区（8 个）

后端：[应用 commands](../../src-tauri/src/app/commands.rs)、[history/commands.rs](../../src-tauri/src/features/history/commands.rs)、[workspace/commands.rs](../../src-tauri/src/features/workspace/commands.rs)。

前端：[documentHistoryRepository](../../src/features/documents/repositories/documentHistoryRepository.ts)、[workspaceStateRepository](../../src/features/documents/repositories/workspaceStateRepository.ts)。

| command | 入参 | 成功返回 | 功能 |
| --- | --- | --- | --- |
| `load_document_history` | 无 | `DocumentHistoryDto` | 结合文档库加载历史 |
| `record_document_closed` | `{ document: ClosedDocumentInputDto }` | `DocumentHistoryDto` | 记录关闭 |
| `record_document_accessed` | `{ document: ClosedDocumentInputDto }` | `DocumentHistoryDto` | 记录访问，复用关闭记录入参类型 |
| `confirm_reopened_closed_document` | `{ path: string }` | `DocumentHistoryDto` | 确认关闭记录已重新打开 |
| `clear_document_history` | 无 | `DocumentHistoryDto` | 清理历史并返回当前结果 |
| `import_legacy_document_history` | `{ history: DocumentHistoryDto }` | `DocumentHistoryDto` | 导入旧历史；前端用于旧版 localStorage 迁移 |
| `load_workspace_state` | 无 | `WorkspaceStateDto` | 加载工作区恢复信息 |
| `save_workspace_state` | `{ workspace: WorkspaceStateDto }` | `void` | 保存工作区恢复信息 |

字段依据 [history/dto.rs](../../src-tauri/src/features/history/dto.rs)、[workspace/dto.rs](../../src-tauri/src/features/workspace/dto.rs)。

```ts
type ClosedDocumentInputDto = {
  documentId?: string | null; path: string; title: string; kind: "pdf" | "markdown";
};
type DocumentHistoryEntryDto = ClosedDocumentInputDto & {
  lastClosedAt: number;
  availability?: "available" | "missing" | "inaccessible" | "notFile" | "unknown";
};
type DocumentHistoryDto = { recent: DocumentHistoryEntryDto[]; closed: DocumentHistoryEntryDto[] };
type WorkspaceStateDto = {
  openDocuments: Array<{ path: string; kind: "pdf" | "markdown" }>;
  activeDocumentPath: string | null;
  folderPath: string | null;
};
```

`lastClosedAt` 以 Unix 毫秒表示，最近访问记录也复用此字段。输入 `availability` 缺省为 `unknown`，输出包含该字段；`documentId` 缺省时输出为 `null`。工作区记录保存路径与类型，不保存 `sessionId`；恢复时需要重新打开文档建立会话。

## 010 · 设置（2 个）

后端：[settings/commands.rs](../../src-tauri/src/features/settings/commands.rs)、[settings/dto.rs](../../src-tauri/src/features/settings/dto.rs)。前端：[settingsRepository](../../src/features/settings/repositories/settingsRepository.ts)、[settings/types.ts](../../src/features/settings/types.ts)。

| command | 入参 | 成功返回 |
| --- | --- | --- |
| `load_settings` | 无 | `AppSettingsDto` |
| `save_settings` | `{ settings: AppSettingsDto }` | `void` |

`save_settings` 保存设置对象，不是字段级 patch。前端保存前使用 `normalizeSettingsForStorage`。字段及 Rust DTO 缺省值如下：

| 字段 | 类型 / 取值 | 缺省值 |
| --- | --- | --- |
| `annotationStorageDir`、`exportDir`、`pdfStorageDir`、`markdownStorageDir` | `string 或 null` | `null`，采用应用默认路径 |
| `pdfImportMode`、`markdownImportMode` | `copy / link` | `copy` |
| `theme` | `system / light / dark` | `system` |
| `animationsEnabled`、`confirmationRemindersEnabled` | `boolean` | `true` |
| `startupBehavior` | `restoreWorkspace / newMarkdown / startPage` | `restoreWorkspace` |
| `pdfWorkspaceLayout` | `{ leftWidth: number, rightWidth: number }` | `{ leftWidth: 280, rightWidth: 340 }` |
| `markdownWorkspaceLayout` | 同上 | `{ leftWidth: 280, rightWidth: 310 }` |
| `translationProvider` | `string` | `baidu` |
| `translationApiUrl` | `string` | `https://fanyi-api.baidu.com/api/trans/vip/translate` |
| `translationApiKey`、`translationSecretId`、`translationSecretKey` | `string` | 空字符串 |
| `translationTargetLanguage` | `string` | `zh` |
| `allowThirdPartyTranslation` | `boolean` | `false` |
| `llmBaseUrl` | `string` | `http://127.0.0.1:8080` |
| `llmModel` | `string` | `local-model` |
| `llmApiKey` | `string` | 空字符串 |
| `llmTimeoutSeconds` | `number`，秒 | `120` |
| `llmTemperature` | `number` | `0.2` |

Rust 反序列化仍接受 `localLlmBaseUrl/localLlmModel/localLlmApiKey/localLlmTimeoutSeconds/localLlmTemperature` 作为对应 `llm*` 字段的旧别名；新调用使用表中的名称。

## 011 · 翻译、LLM 与 MCP（5 个）

后端：[翻译 commands](../../src-tauri/src/features/assistant/translation/commands.rs)、[LLM commands](../../src-tauri/src/features/assistant/llm/commands.rs)、[MCP commands](../../src-tauri/src/features/assistant/mcp/commands.rs)。

前端：[translationClient](../../src/platform/translationClient.ts)、[llmClient](../../src/platform/llmClient.ts)。`list_mcp_tools` 当前没有前端调用点。

| command | 入参 | 成功返回 |
| --- | --- | --- |
| `translate_text` | `{ config: TranslationConfigDto, request: TranslationRequestDto }` | `TranslationResultDto` |
| `test_translation_connection` | `{ config: TranslationConfigDto }` | `{ provider: string, message: string }` |
| `ask_llm` | `{ config: LlmConfigDto, request: LlmChatRequestDto, onEvent: Channel<LlmStreamEventDto> }` | `LlmChatResponseDto`，同时推送 Channel 事件 |
| `test_llm_connection` | `{ config: LlmConfigDto }` | `{ provider: string, message: string }` |
| `list_mcp_tools` | 无 | `Array<{ name: string, description: string, inputSchema: unknown }>` |

### 翻译 DTO

字段依据 [translation/dto.rs](../../src-tauri/src/features/assistant/translation/dto.rs)。

```ts
type TranslationConfigDto = {
  provider: string; apiUrl: string; appId: string;
  secretKey: string; allowThirdParty: boolean;
};
type TranslationRequestDto = { text: string; sourceLanguage: string; targetLanguage: string };
type TranslationResultDto = {
  translatedText: string; detectedLanguage: string | null;
  characterCount: number; usageCount: number; provider: string;
};
```

前端将设置中的 `translationApiKey` 映射到 `appId`，将 `allowThirdPartyTranslation` 映射到 `allowThirdParty`，源语言固定传 `auto`。`translationSecretId` 当前未被该客户端传入。当前装配的翻译 provider 为百度。

### LLM DTO 与 Channel

字段依据 [llm/dto.rs](../../src-tauri/src/features/assistant/llm/dto.rs)。

```ts
type LlmConfigDto = {
  baseUrl?: string; model?: string; apiKey?: string; timeoutSeconds?: number;
};
type LlmChatRequestDto = {
  messages: Array<{ role: "system" | "user" | "assistant"; content: string }>;
  temperature?: number;
};
type LlmChatResponseDto = {
  content: string; model: string;
  promptTokens: number | null; completionTokens: number | null; totalTokens: number | null;
};
type LlmStreamEventDto =
  | { type: "delta"; content: string }
  | { type: "reasoning"; content: string }
  | { type: "complete" };
```

LLM config 的 DTO 默认值为 `baseUrl=http://127.0.0.1:8080`、`model=local-model`、`apiKey=""`、`timeoutSeconds=120`，请求 `temperature` 默认 `0.2`。

```ts
import { Channel } from "@tauri-apps/api/core";
// invokeCommand 从 src/platform/tauriClient 导入；其余变量由调用方提供。
const onEvent = new Channel<LlmStreamEventDto>();
let answer = "";
let reasoning = "";
onEvent.onmessage = (event) => {
  if (event.type === "delta") answer += event.content;
  else if (event.type === "reasoning") reasoning += event.content;
  // complete 代表服务成功完成；错误由下面的 Promise rejection 处理。
};
const result = await invokeCommand<LlmChatResponseDto>("ask_llm", {
  config: { baseUrl, model, apiKey, timeoutSeconds: 120 },
  request: { messages: [{ role: "user", content: "请概括这段文字" }], temperature: 0.2 },
  onEvent,
});
answer = result.content; // 当前实现返回累积后的完整正文，不能再次追加导致重复。
```

`delta` 是回答正文，`reasoning` 是思考内容，应分开累积；`complete` 没有 `content`。成功时 command 发送 `complete` 后返回最终响应；服务错误或 Channel 发送失败时 Promise 拒绝，没有专用 `error` 事件。当前实际装配的 OpenAI compatible 客户端会在最终 `content` 中返回完整正文，并附带模型名与可用 token 用量；只返回思考而没有正文时会拒绝调用。

当前 [wiring.rs](../../src-tauri/src/app/wiring.rs) 为 MCP 装配 `NoopMcpToolRegistry`，因此 `list_mcp_tools` 返回空数组，尚无工具执行 command。`assistant/agent` 等占位文件不在当前模块装配和 IPC 注册集合中，不应作为可用接口。

## 012 · 错误与调用边界

[AppError](../../src-tauri/src/shared/error.rs) 在 Rust 内部为枚举，但序列化为**字符串**。业务失败通常通过 `await` 抛出字符串；Tauri 自身参数反序列化或命令分发失败也会拒绝调用。当前没有统一的 `{ code, message, details }` 错误对象。

```ts
try {
  await invokeCommand<void>("save_markdown_document", { sessionId, revision });
} catch (error) {
  const message = String(error);
  // 交由前端通知 / 错误状态展示。
}
```

| 失败场景 | 后端错误示例 / 当前表现 |
| --- | --- |
| 会话已失效 | `MarkdownSessionNotFound`、`PdfSessionNotFound` |
| Markdown 版本不匹配 | `RevisionConflict`，拒绝编辑、保存、搜索或导出 |
| 编辑位置非法 / 重叠 | `InvalidMarkdownEdit` |
| PDF 未解锁 / 密码错误 | `PdfPasswordRequired`、`PdfInvalidPassword`；打开后的锁定状态另见 `needsPassword` |
| PDF 页码或缩放越界 | `PageOutOfRange` |
| 文件不可读写 / 存储失败 | `Io`、`StorageIo` |
| 不支持类型 / 非法导出扩展名 | `UnsupportedDocumentType`、`InvalidMarkdownPath`、`InvalidExportExtension` |
| 外链协议被拒绝 | `UnsafeExternalLink` |
| 模型 / 翻译请求失败 | `Llm`、`Translation` |

表中枚举名仅用于定位 Rust 源码，**不会作为结构化错误码发给前端**。不要依赖某段错误文本作为长期稳定的协议判断；引入错误码需要同时迁移后端序列化与前端消费者。

以下平台调用不属于上述 58 个自定义业务 command：

| 前端入口 | 通信方式 |
| --- | --- |
| [fileDialog.ts](../../src/platform/fileDialog.ts) | `@tauri-apps/plugin-dialog` 的 `open/save`；取消时前端包装返回 `null` |
| [window.ts](../../src/platform/window.ts) 及标题栏 / 关闭协调器 | Tauri 窗口 API；新窗口完成 / 失败使用 `tauri://created`、`tauri://error` 事件 |
| [clipboard.ts](../../src/platform/clipboard.ts) | `navigator.clipboard.writeText` |

Tauri 核心和插件的授权配置见 [capabilities/default.json](../../src-tauri/capabilities/default.json)。业务模型流使用 `Channel`，不是全局 `listen/emit` 事件总线。

## 013 · 接口变更检查

1. 修改 command 签名与 DTO，明确字段命名、默认值、可空性和单位。
2. 在 [app/ipc.rs](../../src-tauri/src/app/ipc.rs) 注册；只有定义 `#[tauri::command]` 不会自动暴露接口。
3. 同步 repository / platform client、TS 类型和所有调用者，检查版本、会话和缓存失效逻辑。
4. 对序列化、版本冲突、Unicode 偏移、PDF 页码及二进制 / Channel 路径执行与变更相符的验证。
5. 更新本文清单及相关模块文档。本文统计为：文档与系统 9、PDF 会话与页面 6、PDF 存储 4、Markdown 13、搜索 2、批注 4、文档库 5、历史与工作区 8、设置 2、助手 5，共 58 个。

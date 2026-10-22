# 前后端通信约定

[返回技术总览](README.md)

## 001 · 调用入口

前端通用入口是 [invokeCommand](../../frontend/platform/tauriClient.ts)，其内部调用 Tauri `invoke`。业务 repository 管理具体命令和模型映射。Rust command 必须在 [app/ipc.rs](../../backend/src/app/ipc.rs) 的 `generate_handler!` 中注册。

当前没有自动生成的 TypeScript / Rust 协议绑定。`invokeCommand<TResult>` 的泛型只提供静态提示，不会在运行时验证响应。

## 002 · 名称与类型

- command 名称使用 `snake_case`，例如 `open_pdf_document`。
- JavaScript 参数使用 `camelCase`，例如 `sessionId`、`baseRevision`、`pageIndex`。
- DTO 通常使用 `#[serde(rename_all = "camelCase")]`，带类型标记的枚举按各 DTO 的 `kind` 或 `type` 区分。
- 业务模型和 DTO 在各自的 `features/<功能>/` 中定义和转换；不要把 MuPDF 类型或裸指针传到前端。
- `sessionId` 仅在当前进程有效；Markdown 的 `revision` 与搜索 `requestId` 是两种不同序号。

## 003 · 主要接口分组

以下表格列出常用入口，完整注册集合以 `bootstrap.rs` 为准。

| 分组 | command 示例 | 前端入口 |
| --- | --- | --- |
| 文档识别与目录 | `detect_document_kind`、`list_supported_directory` | `features/documents/repositories/` |
| PDF 会话 | `open_pdf_document`、`authenticate_pdf_document`、`close_pdf_document` | `mupdfRepository` |
| PDF 页面 | `get_pdf_page_structure`、`render_pdf_page` | `mupdfRepository` |
| Markdown 会话 | `open_markdown_document`、`create_untitled_markdown_document`、`close_markdown_document` | 文档与结构化 Markdown repository |
| Markdown 编辑 | `get_markdown_structure`、`apply_markdown_edits` | `structuredMarkdownRepository` |
| Markdown 保存与导出 | `save_markdown_document`、`save_markdown_document_as`、`export_markdown_html`、`export_markdown_pdf` | `structuredMarkdownRepository` |
| 搜索 | `search_document`、`cancel_document_search` | 通用搜索与业务适配层 |
| PDF 附属数据 | `load_annotations`、`save_annotations`、`load_pdf_bookmarks`、`save_pdf_reading_state` | 批注和 PDF 存储 repository |
| 文档库与恢复 | `load_document_library`、`relink_document`、`export_document_package`、`load_workspace_state` | 文档库与工作区 repository |
| 设置 | `load_settings`、`save_settings`、`open_app_data_directory` | `settingsRepository` |
| 外部服务 | `ask_llm`、`test_llm_connection`、`translate_text`、`list_mcp_tools` | `platform` 客户端 |

## 004 · JSON 请求与响应

以 [structuredMarkdownRepository](../../frontend/features/markdown/repositories/structuredMarkdownRepository.ts) 的编辑调用为例：

```ts
const updated = await invokeCommand<StructuredMarkdownDocument>(
  "apply_markdown_edits",
  {
    sessionId,
    baseRevision: revision,
    edits: [{ startUtf16: 0, endUtf16: 0, text: "# 标题\n" }],
  },
);
```

响应包含 `documentId`、`sessionId`、`revision`、`path`、`title`、`source`、`nodes`、`outline` 和 `diagnostics`。参数和字段定义参见 [Markdown DTO](../../backend/src/features/markdown/dto.rs)。代码片段省略类型与函数导入。

编辑区间使用 UTF-16 code unit，结束位置不包含在范围内。后端转换为 Rust UTF-8 字节位置并检查非法边界及重叠。不能使用 JavaScript 字符串长度直接解释 Rust 字节偏移。

## 005 · 搜索协议

请求对象包含 `requestId`、`target`、`query` 和 `options`。`target` 以 `kind` 区分：PDF 指定 `sessionId`；Markdown 还携带 `revision`。

响应返回 `requestId`、`total`、`truncated`、`cancelled` 和 `matches`。PDF locator 包含页码、页内位置和矩形；Markdown locator 包含 revision、UTF-16 范围、行与列。字段以 [search_dto.rs](../../backend/src/features/search/dto.rs) 为准。

取消接口用于使旧查询失效，前端仍需检查请求序号。搜索取消不表示通用的任务取消能力，PDF 页面渲染与模型请求没有因此自动获得取消协议。

## 006 · 页面二进制与模型流式事件

`render_pdf_page` 返回 `tauri::ipc::Response` 包装的 PNG 字节，前端转为 `Uint8Array`，再生成 Blob / Object URL。页面结构单独使用 JSON 传输。

`ask_llm` 接收 `config`、`request` 和 `onEvent` Channel：

- `{ type: "delta", content }`：追加一段回答。
- `{ type: "complete" }`：成功结束。
- command Promise 拒绝：请求或流读取失败，前端设置错误状态。

当前底层客户端返回的最终结果 `content` 为空字符串，正文从 delta 累积；最终结果主要携带模型名与可用的 token 使用量。不能在完成后用空的最终 `content` 覆盖已经显示的回答。

## 007 · 错误约定

[AppError](../../backend/src/shared/error.rs) 在 Rust 内部是枚举，但序列化为字符串。前端目前通过 Promise rejection 和 `String(error)` 显示消息，没有统一的 `{ code, message, details }` 响应。

新增稳定错误码属于后续协议迁移，需要同时修改后端序列化、前端判断和测试。当前不要依赖字符串中的某个英文片段作为长期业务分支。

## 008 · 变更检查

接口变更需同时检查 Rust 签名、DTO、command 注册、前端 repository、TypeScript 类型与所有调用者。新增字段应明确默认值，变更页码或字符单位应覆盖边界场景。当前 DTO 测试覆盖部分序列化约定，完整运行路径仍需人工验证。

# 模型服务与辅助阅读

[返回技术总览](../README.md)

## 001 · 职责与入口

当前辅助阅读基于文档选区或用户提供的上下文，调用兼容 Chat Completions 的服务，流式显示问答结果。模型推理由用户配置的服务执行，应用自身不加载模型权重。

| 入口 | 用途 |
| --- | --- |
| [AssistantSidebar](../../../src/components/ai/AssistantSidebar.tsx) | 上下文、模型选择、对话、快捷提问和流式显示 |
| [PdfWorkspace](../../../src/features/pdf/PdfWorkspace.tsx) | 将 PDF 选中文字带入助手 |
| [llmClient](../../../src/platform/llmClient.ts) | 组装请求和接收 Tauri Channel |
| [llm_commands.rs](../../../src-tauri/src/features/assistant/llm/commands.rs) | command 与流式事件转换 |
| [LlmService](../../../src-tauri/src/features/assistant/llm/service.rs) | 地址、消息和参数校验 |
| [llm_provider.rs](../../../src-tauri/src/features/assistant/llm/adapters/openai_compatible.rs) | HTTP 请求和 SSE 响应解析 |

## 002 · 问答流程

1. 用户在 PDF 中选中文字，通过“询问大模型”更新助手上下文并展开侧栏。
2. 助手允许编辑上下文，输入问题或点击总结、解释、翻译快捷操作。
3. 前端将系统提示、历史消息和包含上下文的问题组成请求，传入所选模型名。
4. 后端校验参数并向服务发送流式请求。
5. Rust 解析 SSE `data:` 行，经 Channel 发送 delta；前端通过 animation frame 合并更新界面。
6. 成功发送 complete；异常由 command Promise 拒绝，前端保留已显示内容并标记错误。

助手“翻译”快捷操作也是模型提问，与配置中的百度翻译服务不是同一调用链。

## 003 · 服务地址与请求限制

当前默认地址为 `http://127.0.0.1:8080`，模型名 `local-model`。请求地址追加 `/v1/chat/completions`；配置已以 `/v1` 结尾时不重复追加。连接测试访问 `/v1/models`，仅证明该接口可访问，不保证指定模型的对话请求成功。

后端允许本机回环 HTTP 和远程 HTTPS，远程 HTTP 被拒绝。API Key 非空时使用 Bearer 鉴权。

| 校验项 | 当前值或范围 |
| --- | --- |
| 请求超时 | 5～600 秒，默认 120 秒 |
| 消息数量 | 1～32 条 |
| temperature | 0～2，默认 0.2 |
| 消息正文总量 | 最多 96,000 UTF-8 字节 |
| 流缓冲 | 最多 1 MiB |
| 累计输出 | 最多 2 MiB |
| 请求输出 token 上限 | 8,192 |

前端上下文预算用 JavaScript 字符串长度估算，后端用 UTF-8 字节数校验；预算环不是模型 tokenizer，中文与 emoji 可能导致实际额度不同。

## 004 · 状态与存储

模型服务设置随 `settings.json` 持久化，密钥目前也存于本机设置。助手的可选模型列表使用 `localStorage` 的 `bambook.assistant.models`；模型条目的“本地 / API”标签不等于每条都拥有独立的服务地址和凭据，实际请求仍读取统一设置。

对话与上下文主要保存在组件内存中，尚未形成按 documentId 隔离的完整会话存储。切换文档、切换工作区与卸载后的上下文归属需要验证，不能保证对话自动随文档保存。

## 005 · 翻译、MCP 与 Agent 的状态

- 百度翻译已有配置界面、客户端、command 和 provider；当前 PDF 的 `onTranslateSelection` 是空回调，尚未接通独立的选区翻译流程。
- `McpService` 已装配并注册 `list_mcp_tools`，但使用 `NoopMcpToolRegistry`，返回空列表；没有实际发现、启动或调用 MCP server。
- Agent 相关源文件存在，但位于 `features/assistant/agent`，未进入 assistant 模块导出、IPC 注册及应用装配，不构成已经可调用的 Agent 功能。
- 当前问答不自动建立文档向量索引，没有完整 RAG、工具调用和模型修改文档的闭环。

## 006 · 待完善与验证

需要补齐显式网络取消、按文档的对话归属、凭据管理和错误恢复。挂载标记只阻止界面更新，不能据此认为网络已取消。

验证使用测试服务或用户配置的服务，覆盖连接失败、鉴权失败、无模型、超时、分块 UTF-8、流中断、空回答、额度超限和切换文档。日志与测试结果只记录必要状态，不包含真实密钥或文档正文。

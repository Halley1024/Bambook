# 后端架构

[返回技术总览](README.md) · [重构记录与完整目录](backend-refactor.md)

## 001 · 功能模块与边界

Rust 后端在单个 crate 内按功能组织。`app` 负责启动、装配和跨模块流程，`features` 保存业务功能，`platform` 提供系统与 IO 能力，`shared` 保存公共错误和基础工具。

| 模块 | 入口与职责 |
| --- | --- |
| `app` | [bootstrap](../../backend/src/app/bootstrap.rs) 启动；[wiring](../../backend/src/app/wiring.rs) 装配；[ipc](../../backend/src/app/ipc.rs) 注册 commands；[workflows](../../backend/src/app/workflows/) 协调跨功能流程 |
| `documents` | 文件签名与扩展名识别、目录浏览、文件重命名；不维护编辑或渲染会话 |
| `library` | 文档身份、托管副本与链接、索引、重新关联、文档包与旧存储迁移 |
| `pdf` | MuPDF 展示与搜索适配器、页面模型、书签、阅读状态及其 JSON 格式 |
| `markdown` | 源码、AST、UTF-16 编辑、revision、文件读写、恢复、导出和搜索适配器 |
| `annotations` | 批注保存、备份恢复、迁移、Markdown 导出和批注路径策略 |
| `search` | 搜索请求与结果协议、提供者接口、查询分发、取消和通用匹配算法 |
| `history` / `workspace` | 最近访问与关闭记录 / 工作区持久化 |
| `settings` | 设置默认值、校验和版本化 JSON |
| `assistant` | LLM、翻译、MCP 子模块；Agent 占位源码仍未激活 |
| `platform` | 纯路径拼接、原子文件与 JSON 读写、后台任务调度、Windows / Linux 系统打开操作 |
| `shared` | 保持原 IPC 错误格式的 `AppError` / `AppResult`、通用文件名处理 |

功能内部使用 `commands.rs`、`dto.rs`、`model.rs` / `ast.rs`、`service.rs`、`ports/` 与 `adapters/`。文件随实际复杂度拆分，不要求每个功能都有相同文件数。

各功能的 `mod.rs` 导出内部 Rust 接口，具体实现子模块默认私有。普通 command 只获取 `State<Arc<本模块服务>>`；跨功能 command 位于 `app/commands.rs`，调用 workflow。业务模块不能依赖 `app`，底层公共模块不能依赖业务模块。

## 002 · 依赖装配

[AppState::new](../../backend/src/app/wiring.rs) 创建基础 `AppPaths` 与文档库 `LibraryPaths`，初始化同一份 `LocalLibraryRepository`，然后将 `Arc` 注入各服务。`AppState::register` 托管各功能服务及应用协调状态。

| 使用方 | 注入内容 |
| --- | --- |
| `DocumentService` | 仅负责发现、目录和重命名的 `FileDocumentRepository` |
| `LibraryService` | `LocalLibraryRepository`；其内部按 catalog、package、migration 拆分 |
| `PdfDocumentService` | `MupdfPdfEngine`、共享 `LibraryService` |
| `PdfPersistenceService` | `JsonPdfDataRepository`；通过 `DocumentDataLocator` 获取通用附属数据位置 |
| `MarkdownDocumentService` | `FileMarkdownRepository`、解析器、导出器、`InMemoryMarkdownSessions` |
| `DocumentSearchService` | 搜索模块定义的 `PdfSearchEngine`、`MarkdownSearchProvider` 接口 |
| `MarkdownSearchAdapter` | 与编辑服务共享的 Markdown session store |
| 其余服务 | 本功能的 JSON 仓库、导出器或 HTTP provider |

PDF 展示与搜索仍使用不同 MuPDF 实例和 worker，避免长搜索阻塞页面展示。书签与阅读位置的具体格式已归 PDF，文档库只提供通用数据位置。

## 003 · 跨功能流程

- [pdf_lifecycle](../../backend/src/app/workflows/pdf_lifecycle.rs)：打开、认证、关闭、持久化；搜索注册失败时清理已打开的展示会话。关闭搜索失败时仍尝试关闭展示会话。
- [document_lifecycle](../../backend/src/app/workflows/document_lifecycle.rs)：Markdown 托管路径准备、打开、保存入库、另存、关闭搜索状态，以及存储方式协调。
- [history](../../backend/src/app/workflows/history.rs)：组合文档库与历史信息，文档包导出后登记历史。

文件重命名仍保持既有协议与行为：由 documents service 校验后调用文件适配器 rename。本轮没有扩展为同步改写所有历史、会话和索引的事务。

## 004 · 并发与持久化

同步解析、渲染和磁盘操作在 blocking 任务中运行；公共调度入口是 [run_blocking](../../backend/src/platform/tasks.rs)，部分功能 command 保留原有直接调度。Reqwest 网络操作继续异步执行。

MuPDF `Document` 留在专属 worker 内，前端不接收第三方句柄。发送 Close 不代表已等待线程退出，也不保证可以中断正在执行的单页渲染。

Markdown 仍按读快照、校验 revision、编辑、解析、替换会话运行。revision 校验和 replace 不是公开原子 compare-and-swap，不能宣称任意并发写入安全。搜索适配器使用同一份会话，保留 UTF-16 定位和 requestId 取消规则。

单个文件使用原子替换；文档内容、metadata、文档库与附属文件的多步写入没有统一回滚事务。格式、备份与侧车读取策略见[存储设计](storage.md)。

`platform/paths.rs` 不读取 JSON 或设置；文档库位置查询归 `library/paths.rs`，批注路径策略归 `annotations/paths.rs`。

## 005 · 新增功能与验证

1. 在 `features/<功能>/` 定义模型、DTO、服务和必要端口，实现本功能适配器。
2. 在 `mod.rs` 声明模块并导出需要的内部接口；跨模块调用只使用这些接口。
3. 在 `app/wiring.rs` 装配；跨功能流程进入 `app/workflows/`。
4. 在 `app/ipc.rs` 注册 command，保持前后端协议一致。
5. 执行 `python scripts/check_backend.py`、Cargo 检查和测试；同步更新业务文档。

[check_backend.py](../../scripts/check_backend.py) 检查依赖方向，并对照重构前冻结清单核对 58 个 command 的前端参数与返回类型，以及 88 个序列化类型的字段和 serde 声明。它是源码契约检查，不能代替实际序列化和桌面运行测试。

Agent 目录保留原来的未接入源码，没有因为目录迁移而自动进入运行链路。详见[辅助阅读模块](modules/assistant.md)。

# 后端功能模块重构记录

[返回技术总览](README.md) · [后端职责与依赖](backend.md)

日期：2026-10-22。范围：`backend/src` 的功能模块组织、依赖装配、跨模块用例、公共模块、相关文档与契约检查。维持单个 Cargo crate；没有调整依赖清单和原生 MuPDF 源码。

## 001 · 重构前备份

完整目录副本位于项目根目录下：

```text
.codex-build-check/backups/src-tauri-20261022-before-feature-refactor/
```

该目录约 6.1 GiB，包含源码、Cargo 清单与锁文件、Tauri 配置、native、资源和复制时的 target 缓存。源码、配置、原生依赖与资源已在重构前使用 `diff -qr --exclude=target` 比对一致；构建缓存可以重新生成，不作为源码一致性依据。备份目录被已有 `.codex-build-check/` 忽略规则覆盖。

重构前用户已有的未提交后端修改包含在备份中。本轮未重置这些修改，也未创建 Git 提交；真实应用数据目录未用于自动测试。

需要恢复时，先停止应用和构建进程，并确认以下临时保存目录不存在。在项目根目录执行：

```sh
mv backend .codex-build-check/src-tauri-after-feature-refactor
cp -a .codex-build-check/backups/src-tauri-20261022-before-feature-refactor backend
```

这是整个后端目录的恢复方式，会保留重构后的目录供比较。不要把旧源码直接覆盖合并进新目录，否则会同时保留两套模块。

## 002 · 实际源码目录

功能内部的 `mod.rs` 声明并导出接口；实现子模块默认私有，供应用装配使用的构造类型显式导出。简单功能直接托管 `Arc<Service>`，没有为每个功能额外创建仅包装服务的 state 文件。Agent 目录保留未接入状态。

```text
backend/src/
├── app/
│   ├── workflows/
│   │   ├── document_lifecycle.rs
│   │   ├── history.rs
│   │   ├── mod.rs
│   │   ├── pdf_lifecycle.rs
│   │   └── tests.rs
│   ├── bootstrap.rs
│   ├── commands.rs
│   ├── ipc.rs
│   ├── mod.rs
│   ├── state.rs
│   └── wiring.rs
├── features/
│   ├── annotations/
│   │   ├── adapters/
│   │   │   ├── json_repository.rs
│   │   │   ├── markdown_exporter.rs
│   │   │   └── mod.rs
│   │   ├── ports/
│   │   │   ├── annotation_exporter.rs
│   │   │   ├── annotation_repository.rs
│   │   │   └── mod.rs
│   │   ├── commands.rs
│   │   ├── dto.rs
│   │   ├── export.rs
│   │   ├── export_commands.rs
│   │   ├── mod.rs
│   │   ├── model.rs
│   │   ├── paths.rs
│   │   └── service.rs
│   ├── assistant/
│   │   ├── agent/
│   │   │   ├── adapters/
│   │   │   │   ├── llama_cpp.rs
│   │   │   │   └── mod.rs
│   │   │   ├── commands.rs
│   │   │   ├── mod.rs
│   │   │   ├── model.rs
│   │   │   ├── ports.rs
│   │   │   └── service.rs
│   │   ├── llm/
│   │   │   ├── adapters/
│   │   │   │   ├── mod.rs
│   │   │   │   └── openai_compatible.rs
│   │   │   ├── ports/
│   │   │   │   ├── llm_provider.rs
│   │   │   │   └── mod.rs
│   │   │   ├── commands.rs
│   │   │   ├── dto.rs
│   │   │   ├── mod.rs
│   │   │   ├── model.rs
│   │   │   └── service.rs
│   │   ├── mcp/
│   │   │   ├── adapters/
│   │   │   │   ├── mod.rs
│   │   │   │   └── noop_registry.rs
│   │   │   ├── ports/
│   │   │   │   ├── mcp_tool_registry.rs
│   │   │   │   └── mod.rs
│   │   │   ├── commands.rs
│   │   │   ├── dto.rs
│   │   │   ├── mod.rs
│   │   │   ├── model.rs
│   │   │   └── service.rs
│   │   ├── translation/
│   │   │   ├── adapters/
│   │   │   │   ├── baidu.rs
│   │   │   │   └── mod.rs
│   │   │   ├── ports/
│   │   │   │   ├── mod.rs
│   │   │   │   └── translation_provider.rs
│   │   │   ├── commands.rs
│   │   │   ├── dto.rs
│   │   │   ├── mod.rs
│   │   │   ├── model.rs
│   │   │   └── service.rs
│   │   └── mod.rs
│   ├── documents/
│   │   ├── adapters/
│   │   │   ├── directory_reader.rs
│   │   │   ├── file_repository.rs
│   │   │   └── mod.rs
│   │   ├── ports/
│   │   │   ├── document_repository.rs
│   │   │   └── mod.rs
│   │   ├── commands.rs
│   │   ├── dto.rs
│   │   ├── formats.rs
│   │   ├── mod.rs
│   │   ├── model.rs
│   │   └── service.rs
│   ├── history/
│   │   ├── adapters/
│   │   │   ├── file_inspector.rs
│   │   │   ├── json_repository.rs
│   │   │   └── mod.rs
│   │   ├── ports/
│   │   │   ├── document_history_repository.rs
│   │   │   ├── document_path_inspector.rs
│   │   │   └── mod.rs
│   │   ├── commands.rs
│   │   ├── dto.rs
│   │   ├── mod.rs
│   │   ├── model.rs
│   │   └── service.rs
│   ├── library/
│   │   ├── adapters/
│   │   │   ├── local_storage/
│   │   │   │   ├── catalog.rs
│   │   │   │   ├── migration.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── package.rs
│   │   │   │   └── tests.rs
│   │   │   └── mod.rs
│   │   ├── ports/
│   │   │   ├── document_data_locator.rs
│   │   │   ├── library_repository.rs
│   │   │   └── mod.rs
│   │   ├── commands.rs
│   │   ├── dto.rs
│   │   ├── mod.rs
│   │   ├── model.rs
│   │   ├── paths.rs
│   │   └── service.rs
│   ├── markdown/
│   │   ├── adapters/
│   │   │   ├── exporter.rs
│   │   │   ├── file_repository.rs
│   │   │   ├── memory_sessions.rs
│   │   │   ├── mod.rs
│   │   │   ├── parser.rs
│   │   │   └── sanitizer.rs
│   │   ├── ports/
│   │   │   ├── file_repository.rs
│   │   │   ├── markdown_exporter.rs
│   │   │   ├── markdown_parser.rs
│   │   │   ├── markdown_session_repository.rs
│   │   │   └── mod.rs
│   │   ├── ast.rs
│   │   ├── commands.rs
│   │   ├── dto.rs
│   │   ├── mod.rs
│   │   ├── search.rs
│   │   └── service.rs
│   ├── pdf/
│   │   ├── adapters/
│   │   │   ├── json_data.rs
│   │   │   ├── mod.rs
│   │   │   ├── mupdf_engine.rs
│   │   │   ├── mupdf_search.rs
│   │   │   └── search_index.rs
│   │   ├── ports/
│   │   │   ├── data_repository.rs
│   │   │   ├── mod.rs
│   │   │   └── pdf_engine.rs
│   │   ├── bookmarks.rs
│   │   ├── commands.rs
│   │   ├── dto.rs
│   │   ├── mod.rs
│   │   ├── model.rs
│   │   ├── persistence.rs
│   │   ├── reading_state.rs
│   │   ├── service.rs
│   │   ├── storage_commands.rs
│   │   └── storage_dto.rs
│   ├── search/
│   │   ├── ports/
│   │   │   ├── markdown_search.rs
│   │   │   ├── mod.rs
│   │   │   └── pdf_search_engine.rs
│   │   ├── commands.rs
│   │   ├── dto.rs
│   │   ├── matcher.rs
│   │   ├── mod.rs
│   │   ├── model.rs
│   │   ├── normalizer.rs
│   │   └── service.rs
│   ├── settings/
│   │   ├── adapters/
│   │   │   ├── json_repository.rs
│   │   │   └── mod.rs
│   │   ├── ports/
│   │   │   ├── mod.rs
│   │   │   └── settings_repository.rs
│   │   ├── commands.rs
│   │   ├── dto.rs
│   │   ├── mod.rs
│   │   ├── model.rs
│   │   └── service.rs
│   ├── workspace/
│   │   ├── adapters/
│   │   │   ├── json_repository.rs
│   │   │   └── mod.rs
│   │   ├── ports/
│   │   │   ├── mod.rs
│   │   │   └── workspace_state_repository.rs
│   │   ├── commands.rs
│   │   ├── dto.rs
│   │   ├── mod.rs
│   │   ├── model.rs
│   │   └── service.rs
│   └── mod.rs
├── platform/
│   ├── desktop/
│   │   ├── commands.rs
│   │   ├── linux.rs
│   │   ├── mod.rs
│   │   └── windows.rs
│   ├── filesystem/
│   │   ├── atomic_file.rs
│   │   ├── json_file.rs
│   │   └── mod.rs
│   ├── mod.rs
│   ├── paths.rs
│   └── tasks.rs
├── shared/
│   ├── error.rs
│   ├── filename.rs
│   └── mod.rs
├── lib.rs
└── main.rs
```

`native/`、`resources/`、平台配置、capabilities、图标与构建文件维持原位置。

## 003 · 职责迁移

| 原位置或职责 | 新归属 |
| --- | --- |
| 顶层 bootstrap / state | `app/bootstrap.rs`、`app/state.rs`、`app/wiring.rs`、`app/ipc.rs` |
| 分散的 command、DTO、domain、service、port、实现 | 收拢至对应 `features/<功能>/` |
| DocumentRepository 中 Markdown 文件读写与恢复 | `markdown/ports/file_repository.rs`、`markdown/adapters/file_repository.rs` |
| 通用目录结构原名 MarkdownFileEntry | `documents::DirectoryEntry`；外部 JSON 字段维持一致 |
| StorageService / StorageRepository | `library::LibraryService` / `LibraryRepository` |
| 文档库索引、包导出、旧数据迁移 | `library/adapters/local_storage/catalog.rs`、`package.rs`、`migration.rs` |
| 书签和阅读位置模型、JSON 格式、持久化服务 | `pdf/bookmarks.rs`、`reading_state.rs`、`adapters/json_data.rs`、`persistence.rs` |
| PDF 展示与搜索会话协调 | `app/workflows/pdf_lifecycle.rs` |
| Markdown 托管打开、保存入库与关闭协调 | `app/workflows/document_lifecycle.rs` |
| 文档库与历史信息组合、包导出后的历史登记 | `app/workflows/history.rs` |
| Markdown 搜索与会话读取 | `markdown/search.rs`；搜索模块仅依赖提供者接口 |
| 基础文件与路径工具 | `platform/filesystem/`、`platform/paths.rs` |
| 文档库位置查询、批注路径策略 | `library/paths.rs`、`annotations/paths.rs` |
| 系统打开操作 | `platform/desktop/`；Windows explorer/rundll32，Linux xdg-open |
| 公共错误和文件名处理 | `shared/error.rs`、`shared/filename.rs` |

旧的 commands、application、domain、infrastructure 顶层目录与兼容模块声明已删除。

文件重命名保持原有文件操作语义：Markdown 仍校验 `.md` 目标扩展名，PDF 保留既有行为；本次没有添加跨历史、会话和文档库的重命名事务。单文件原子写入也不等于跨文件事务。

## 004 · 保持的协议与资源约束

- 58 个已注册 command 的名称、前端参数和返回类型保持兼容。新增的 Tauri State 注入不构成前端请求参数。
- 对照原目录冻结的 88 个序列化类型，字段、类型与 serde 声明保持一致。清单位于 `backend/tests/fixtures/backend-contract.json`。
- PDF 展示与搜索仍由独立 MuPDF worker 承担。搜索注册失败清理展示会话；搜索关闭失败仍尝试关闭展示会话。
- Markdown 编辑与搜索共享同一份 session store，保留 revision、UTF-16 定位和取消规则。
- 维持文档库、批注、书签、阅读状态、设置、工作区和恢复文件的现有磁盘路径与格式。
- 前端业务代码与样式没有因后端目录重构而调整。

## 005 · 验证结果

在当前 Linux 工作区执行：

| 检查 | 结果 |
| --- | --- |
| 重构前 cargo check | 通过 |
| 重构前 cargo test | 原有 68 项通过 |
| `python scripts/check_backend.py` | 依赖边界通过；58 个 command、88 个序列化类型的声明与原目录匹配 |
| `cargo check --manifest-path backend/Cargo.toml --offline` | 通过，无编译警告 |
| `cargo test --manifest-path backend/Cargo.toml --offline` | 73 项通过：原有 68 项与新增 5 项流程测试 |
| `cargo build --manifest-path backend/Cargo.toml --offline` | 通过 |
| `npm run build` | TypeScript 检查与 Vite 生产构建通过 |
| `git diff --check` | 通过 |

新增流程测试覆盖普通 PDF 打开/渲染/搜索/关闭、加密 PDF 的展示与搜索认证、搜索注册失败清理、搜索关闭失败清理，以及 Markdown 编辑后搜索、取消、保存入库和重新打开。

原有测试继续覆盖旧设置与工作区 JSON、批注数组与备份恢复、副本/链接、书签与阅读位置、文档包、Unicode 搜索及真实 HTML/PDF 导出。测试在临时目录与固定样例上执行；未读取或迁移用户的真实文档库。

没有启动桌面窗口进行人工交互、实际打开系统文件管理器或浏览器，也没有构建安装包。Windows 编译、跨平台安装和完整 UI 验收仍需在目标环境完成。

## 006 · 顶层目录重命名

同日将顶层前端源码目录改为 `frontend/`，Tauri 工程改为 `backend/`。Rust 标准源码目录保留为 `backend/src/`；package.json、index.html、Vite 和 TypeScript 配置仍在项目根目录。

已更新 HTML 入口、TypeScript include、Vite 忽略规则、Git 忽略规则、后端检查脚本、IDE 设置、已有 CMake 缓存中的工程目录名及全部开发文档链接。3352 个移动文件逐项核对，业务源码内容保持一致，仅原生依赖 README 更新了项目路径。历史备份保留原名。

新增 [Tauri 启动脚本](../../scripts/tauri.mjs)，明确指定后端工程位置和包含 package.json 的前端工具链目录，继续支持 `npm run tauri -- dev`。变量含义参见 [Tauri CLI 2.11.4 的路径实现](https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.11.4/crates/tauri-cli/src/helpers/app_paths.rs)。

移动后的构建缓存曾包含旧工程目录的绝对权限文件路径；已清理 tauri、tauri-plugin-dialog、tauri-plugin-fs 的缓存并重新生成。Linux 下验证结果：

- `cargo check --manifest-path backend/Cargo.toml --offline` 通过。
- `cargo test --manifest-path backend/Cargo.toml --offline`：73 项通过。
- `npm run build` 通过；Vite 在不监听端口的模式下成功转换 HTML 和 `/frontend/main.tsx`。
- `CARGO_NET_OFFLINE=true npm run tauri -- build --debug --no-bundle` 通过，生成 `backend/target/debug/bambook`。
- 后端协议检查通过，所有开发文档的本地链接均可解析。

本次仍未运行桌面窗口或生成安装包。

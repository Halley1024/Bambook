# 测试分层与人工验收

[返回技术总览](README.md)

## 001 · 当前验证能力

项目已有 Rust 内联单元与集成性质测试，分布于 features、platform 和 app/workflows 中。前端 `package.json` 没有 `test` 脚本；当前不能把 `npm run build` 解释为前端业务测试。

本轮后端重构验证包含原有 68 个测试及新增 5 个跨模块流程测试；具体结果见[重构记录](backend-refactor.md)。桌面交互和安装包需另行验收。

## 002 · 基础命令

从项目根目录运行，原生依赖准备见[开发与发布](build-and-release.md)：

```sh
python scripts/check_backend.py
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

需要查看测试清单或缩小范围时：

```sh
cargo test --manifest-path src-tauri/Cargo.toml -- --list
cargo test --manifest-path src-tauri/Cargo.toml applies_javascript_utf16_ranges_without_splitting_unicode
cargo test --manifest-path src-tauri/Cargo.toml writes_real_html_and_pdf_files
```

名称过滤仍需要编译相应测试目标，并不绕过 MuPDF、Tauri 或 libclang 依赖。

## 003 · 已有测试覆盖位置

| 范围 | 示例文件 | 验证内容 |
| --- | --- | --- |
| 应用用例 | [Markdown 服务](../../src-tauri/src/features/markdown/service.rs) | UTF-16 区间与代理对边界 |
| 历史记录 | [历史服务](../../src-tauri/src/features/history/service.rs) | 排序、路径检查与恢复确认 |
| PDF 引擎 | [MuPDF 引擎](../../src-tauri/src/features/pdf/adapters/mupdf_engine.rs) | 打开、认证、提取与渲染 |
| 搜索 | [normalizer](../../src-tauri/src/features/search/normalizer.rs)、[matcher](../../src-tauri/src/features/search/matcher.rs) | Unicode 标准化、匹配与映射 |
| Markdown | [解析器](../../src-tauri/src/features/markdown/adapters/parser.rs)、[导出器](../../src-tauri/src/features/markdown/adapters/exporter.rs) | AST、链接过滤与实际文件生成 |
| 存储 | [托管仓库](../../src-tauri/src/features/library/adapters/local_storage/mod.rs) | 副本、链接、索引及文档包相关规则 |
| 文件替换 | [atomic_file](../../src-tauri/src/platform/filesystem/atomic_file.rs) | 生成失败时保留目标文件 |
| 跨功能流程 | [workflow tests](../../src-tauri/src/app/workflows/tests.rs) | PDF 失败清理、加密认证、Markdown 搜索保存重开 |
| 协议 | [冻结协议清单](../../src-tauri/tests/fixtures/backend-contract.json) | 部分字段序列化和输入转换 |

现有测试数量和实际执行结果以当前测试输出为准。Rust 层测试不能替代 Tauri 窗口、选区、滚动或安装包的人工验收。

## 004 · 人工验收矩阵

以下是建议执行的场景，不是已完成清单。

| 场景 | 操作与预期 |
| --- | --- |
| 混合文档 | 打开 PDF 与 Markdown，切换、排序和关闭标签，内容与工具栏对应 |
| 批量关闭 | 多个 dirty 文档逐个询问，中途取消后未释放文档仍可继续编辑 |
| 保存失败 | 选择不可写测试目录，保留错误提示和编辑内容，不误清 dirty |
| Markdown 时序 | 快速输入后立即保存 / 导出，输出包含最后输入；旧 revision 被拒绝 |
| 未命名恢复 | 分别验证单文档、多个未命名标签、切换活动标签与重启，记录现有限制 |
| PDF 坐标 | 多级缩放、跨行与中文选择、搜索跳转后，标记仍对应原文 |
| PDF 特殊输入 | 加密、错误密码、扫描、旋转页、复杂字体和损坏文件得到明确结果 |
| 持久化 | 副本 / 链接、源文件移动、重连、保存重开和文档包异地打开 |
| 备份恢复 | 主批注损坏、有有效备份、多个来源时间不同，读取符合存储规则 |
| 模型服务 | 连接、鉴权、超时、流中断和非 ASCII 分块，回答与错误状态正确 |
| 设置布局 | 改主题、调整尺寸、显式保存并重启，PDF / Markdown 设置独立 |
| 平台安装 | 安装、启动、系统打开目录 / 外链、卸载和重新安装，记录数据保留行为 |

## 005 · 样例和性能

优先使用不含真实隐私的固定样例，测试在临时目录完成。PDF 样例覆盖文字、中文、链接、目录、旋转、加密和扫描；Markdown 覆盖 emoji、嵌套结构、表格、图片、原始 HTML 与不安全链接。

性能验收记录文件页数和大小、硬件、系统、首次打开耗时、连续滚动与缩放后的内存，以及重复打开关闭是否持续增长。当前没有统一性能门槛，不要用单次流畅体验代表大型文档验收完成。

## 006 · 结果记录

在 `docs/logs/` 中按日期记录版本或提交、操作系统、验证命令、人工样例、实际结果与未解决问题。将“编译通过”“Rust 测试通过”“桌面流程通过”“安装包验收通过”分别记录。后续自动化优先覆盖关闭协调器、编辑同步和跨模块持久化等容易丢失用户数据的流程。

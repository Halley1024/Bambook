# 文件组织、持久化与恢复

[返回技术总览](README.md)

## 001 · 数据根目录

[bootstrap.rs](../../src-tauri/src/app/bootstrap.rs) 使用 Tauri `local_data_dir()` 再追加 `Bambook`，由 [AppPaths](../../src-tauri/src/platform/paths.rs) 提供基础路径，由 [LibraryPaths](../../src-tauri/src/features/library/paths.rs) 派生托管布局。Windows 对应 `%LOCALAPPDATA%\Bambook`；Linux 使用系统解析出的本地数据目录，不写死 Windows 路径。

下图是逻辑布局，文档子目录及部分文件按需要产生，不会在每次打开时全部创建：

```text
<local_data_dir>/Bambook/
├── settings.json
├── document-library.json
├── recent-documents.json
├── closed-documents.json
├── workspace-state.json
├── themes/
└── storage/
    ├── pdf-<id>/
    │   ├── document.pdf         副本模式下的 PDF
    │   ├── metadata.json
    │   ├── annotations.json
    │   ├── bookmarks.json
    │   └── reading-state.json
    ├── markdown-<id>/
    │   ├── markdown.md          副本模式下的 Markdown
    │   ├── metadata.json
    │   └── assets/
    └── _recovery/
        └── untitled.md
```

PDF 与 Markdown 可配置各自存储根目录，批注还保留独立目录配置。实际附属数据目录优先参考文档库记录的 `dataPath`。`themes/` 目录存在不代表已经实现完整 CSS 主题管理。

## 002 · 文档身份与路径

新外部文档通常使用规范化路径的 SHA-256 派生 `pdf-...` 或 `markdown-...` ID；已入库文档优先复用索引身份。它不是仅凭内容确定的全局身份，路径变化、重新链接和包导出都需要核对身份维护规则。

| 字段 | 含义 |
| --- | --- |
| `id` / `documentId` | 连接文档库和附属数据 |
| `path` / `storagePath` | 当前用于打开或持久化的文件路径 |
| `sourcePath` | 来源文件路径 |
| `dataPath` | metadata、批注等附属数据目录 |
| `storageMode` | `managed-copy` 或 `linked-file` |

文档库还保存标题、类型以及创建、更新和最近打开时间；索引时间字段以 Unix 毫秒表示。其他 JSON 中的时间字段可能使用文本或数值，不能假定所有文件格式相同。

## 003 · 打开与入库

[LocalLibraryRepository](../../src-tauri/src/features/library/adapters/local_storage/mod.rs) 区分 `prepare_pdf` / `prepare_markdown` 与 `persist_pdf` / `persist_markdown`。

准备阶段规范化路径、查询已有记录并返回打开路径；对于未入库外部文件，不为它立即创建完整文档目录。首次显式保存再根据设置复制文件或记录链接，并写 metadata 和文档库索引。

- **副本模式**：托管目录保留文档副本，后续可优先打开副本。
- **链接模式**：文档仍位于来源路径，托管目录保存索引和附属数据。
- **链接恢复**：检查用户重新选择的文件，必要时确认名称不一致，再同步索引和 metadata。
- **文档包**：输出文档文件、metadata 与附属数据到独立目录；实现会处理包内身份与路径，不能简单把旧绝对路径原样复制。

保存内容与入库是多个步骤。Markdown 保存可能已经成功写入内容，但随后入库失败；这种失败不等于原文件未改变。

## 004 · PDF 附属数据

### 批注

[JsonAnnotationRepository](../../src-tauri/src/features/annotations/adapters/json_repository.rs) 当前写入 `schemaVersion: 4`，根对象包含 `updatedAt` 和 `annotations`，同时兼容旧数组载荷。

读取候选来自来源文件所在目录的 `annotations.json`、配置或索引确定的托管批注文件，以及旧存储位置。各位置主文件读取失败时尝试 `.bak`，再按候选 `updatedAt` 选择较新数据；旧数组使用文件修改时间。

保存前，有效旧文件复制为 `.bak`，损坏旧文件保留为带时间的 `.corrupt-...` 文件，再原子写入新内容。来源目录已有侧车文件时会同步更新，普通保存不会无条件新建来源侧车文件。

### 书签与阅读位置

`bookmarks.json` 与 `reading-state.json` 优先读取来源目录已有侧车文件，否则读取托管数据目录。保存会更新已持久化目录和已有侧车文件；首次打开尚未入库且无侧车时，并不保证已经产生跨启动保存记录。

书签参与文档 dirty 状态，阅读位置通过自动保存路径处理。三类文件采用各自格式和读取规则，不能都套用批注的“按更新时间比较”策略。

侧车使用固定名称 `annotations.json`、`bookmarks.json`、`reading-state.json`，没有附加 PDF 文件名。同一目录放置多份 PDF 并共享这些文件的隔离行为需要专项验证。

## 005 · 原子写入与恢复边界

[atomic_file.rs](../../src-tauri/src/platform/filesystem/atomic_file.rs) 的步骤是：同目录创建唯一临时文件 → 写入 → `sync_all` → 替换目标。Windows 使用 `MoveFileExW`，其他平台使用 rename 并同步父目录；guard 尽力清理残留临时文件。

这保护单文件生成和替换过程，不提供跨 metadata、索引和内容的整体事务。替换后的同步失败也不能解释为“磁盘一定保留旧版本”。

工作区 JSON 保存可重新打开的磁盘文档、活动路径与文件夹。未命名 Markdown 恢复使用单一 `untitled.md`：当前前端仅针对活动未命名文档防抖写入，且恢复询问前会发起清理；它不是多文档、全过程无损的崩溃快照。

## 006 · 变更与验证

- 格式变更明确 schema、默认值、旧数据读取和升级行为；设置当前 schema 为 `9`，与批注版本独立。
- 使用临时目录验证只读打开、首次保存、副本与链接、源文件失效、重新链接以及文档包重新打开。
- 用损坏 JSON、有效备份和不同更新时间验证批注候选选择。
- 不直接在真实文档库上验证破坏性恢复场景；具体场景见[测试文档](testing.md)。

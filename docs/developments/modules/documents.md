# 文档库、标签页与文档生命周期

[返回技术总览](../README.md)

## 001 · 职责与代码入口

文档模块统一管理 PDF / Markdown 的身份、打开入口、标签、历史记录和关闭协调。页面渲染、编辑缓冲和具体保存逻辑由业务工作区承担。

| 入口 | 用途 |
| --- | --- |
| [DocumentWorkspaceProvider](../../../src/features/documents/DocumentWorkspaceProvider.tsx) | 文档操作、dirty 状态、生命周期注册和工作区恢复 |
| [useWorkspaceTabs](../../../src/features/documents/hooks/useWorkspaceTabs.ts) | 标签列表与活动标签状态 |
| [closeCoordinator](../../../src/features/documents/close/closeCoordinator.ts) | 批量关闭的准备、提交和释放 |
| [StartWorkspace](../../../src/features/documents/components/StartWorkspace.tsx) | 最近访问和文档库 |
| [DocumentHistoryProvider](../../../src/features/documents/DocumentHistoryProvider.tsx) | 历史加载、刷新和操作 |
| [AppShell](../../../src/app/layout/AppShell.tsx) | 系统窗口关闭事件接入 |
| [DocumentService](../../../src-tauri/src/features/documents/service.rs) | 文件识别、目录读取与文件重命名 |
| [DocumentHistoryService](../../../src-tauri/src/features/history/service.rs) | 历史排序、路径状态和关闭栈 |

## 002 · 核心模型

前端 `ReaderDocument` 是 `PdfDocument | MarkdownDocument`。公共字段包括 `id`、`title` 和 `path`，各类型附加自己的 `sessionId`、元数据或 AST。

文档层保存标签顺序与活动文档，不直接持有 PDF 句柄。`DocumentLifecycleController` 由业务模块注册，使统一关闭流程可以调用各自的询问、保存和释放逻辑。

## 003 · 打开与目录导航

1. 文件选择、拖放、最近记录或文件树提供路径。
2. 调用后端识别类型：PDF 检查文件签名，Markdown 检查支持的扩展名。
3. 打开相应会话并映射为 `ReaderDocument`，加入或激活标签。
4. `WorkspaceHost` 根据活动文档类型选择工作区。
5. 历史记录与文档库通过各自接口加载或更新，不能把“最近打开”当作“已保存入库”。

文件夹工作区展示支持的文档和目录，节点点击转入统一打开路径。它是按请求读取目录的应用视图，不能默认等同于实时文件系统监听器。

## 004 · 保存与关闭

标签 dirty 圆点由文档层展示，具体业务负责更新：Markdown 对比内容与已保存基线；PDF 检查批注、书签等变化。仅切换标签不会自动保存所有编辑。

`runCloseTransaction` 的当前流程：

```mermaid
flowchart LR
    A[收集候选文档] --> B[逐个 prepare]
    B --> C{全部准备完成?}
    C -- 否 --> X[取消后续关闭]
    C -- 是 --> D[逐个执行 commit]
    D --> E[逐个 release]
    E --> F[记录关闭历史并更新标签]
```

prepare 返回 `ready` 或 `cancelled`，ready 可携带延迟执行的 commit。这样后续文档取消询问时，前面选择“不保存”的内容仍未被释放。

commit 返回 `false` 会阻止释放阶段；抛错由调用方处理。若较早的 commit 已写入磁盘、后续 commit 失败，已有写入不会回滚。release 阶段也按文档顺序执行，不能宣称整个批次具备数据库事务原子性。

关闭当前标签、关闭其他标签、关闭全部标签和窗口退出均需要复用协调路径。`AppShell` 拦截窗口关闭请求，准备成功后调用原生窗口销毁。

## 005 · 工作区与恢复

设置决定启动时恢复工作区、创建 Markdown 或显示开始页。工作区持久化磁盘文档路径和类型、顺序、活动路径及文件夹路径，不保存 sessionId。

恢复会逐个重新打开；失效文件需要提示或跳过，不能因为一项失败就把整个工作区当作成功。未命名文档的内容恢复由 Markdown 模块另行处理。

文档库支持副本与链接、链接失效后重新选择文件、名称不一致确认，以及文档包导出。重命名需要同时核对文件、会话、标签、历史和索引；磁盘 rename 成功不足以证明所有关联数据都已更新。

## 006 · 当前限制与验证

- 当前宿主显示活动文档的单工作区；旧多窗格注释代码不构成已支持能力。
- 新窗口存在创建入口，但共享 JSON 的并发写入、窗口间历史与设置隔离仍需验收。
- 多个未命名文档的恢复不完整，详见 [Markdown](markdown.md) 与[存储](../storage.md)。

修改生命周期后，至少验证：混合标签切换；多个 dirty 文档逐个询问；中途取消；保存路径选择取消；保存失败后保留标签；退出后重新启动；源文件移动或删除后的历史状态。

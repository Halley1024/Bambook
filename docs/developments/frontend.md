# 前端架构

[返回技术总览](README.md)

## 001 · 目录职责

| 目录 | 当前职责 | 新代码放置原则 |
| --- | --- | --- |
| `src/app/` | 应用壳、工具栏、工作区宿主、Provider 组合 | 组织跨业务布局，业务细节交给 feature |
| `src/features/` | 文档、PDF、Markdown、批注、搜索、设置等 | 同一业务的组件、hooks、状态、类型和 repository 就近维护 |
| `src/components/` | 菜单、标签、表单、提示、内容展示及助手 | 复用组件放在这里；助手目前仍包含服务调用和本地存储逻辑 |
| `src/platform/` | Tauri invoke、对话框、窗口、剪贴板、服务客户端 | 集中适配桌面运行环境 |
| `src/shared/` | LRU 缓存、尺寸管理和纯工具 | 避免反向依赖具体业务 |
| `src/styles/` | 全局样式入口 | 业务样式尽量由所属 feature 管理 |

这是一套基于 feature 的组织方式，不要求每个模块拥有完全相同的子目录。业务目前存在跨 feature 引用，调整时应依据实际数据归属，而不是机械移动文件。

## 002 · 挂载与 Provider

入口：[main.tsx](../../src/main.tsx) → [App.tsx](../../src/app/App.tsx) → [AppProviders.tsx](../../src/app/AppProviders.tsx)。

当前 Provider 从外到内为：

```text
NotificationProvider
└── SettingsProvider
    └── DocumentHistoryProvider
        └── DocumentWorkspaceProvider
            └── PdfWorkspaceProvider
                └── PdfSearchProvider
                    └── MarkdownWorkspaceProvider
                        └── MarkdownSearchProvider
```

顺序体现依赖：文档生命周期使用设置和历史记录，PDF / Markdown 工作区接入统一文档管理，搜索再读取对应工作区的会话信息。调整顺序前检查相关 hook 的 Provider 要求。

## 003 · 状态归属

| 状态 | 主要拥有者 | 说明 |
| --- | --- | --- |
| 文档列表、活动标签、文件夹、关闭弹窗 | `DocumentWorkspaceProvider` 与 `useWorkspaceTabs` | 两类文档的共同生命周期 |
| 最近访问、已关闭记录 | `DocumentHistoryProvider` | 数据由 Rust 持久化，前端读取并刷新 |
| PDF 缩放、页码、选区、批注操作 | `PdfWorkspaceProvider` 及 PDF hooks | 不与 Markdown 编辑状态共用 |
| Markdown 内容、选择、撤销历史 | `MarkdownWorkspaceProvider` | 按文档维护缓冲和后端结构 |
| 查询、结果、当前命中、请求序号 | 通用搜索 hook 和业务搜索 Provider | 保留 PDF / Markdown 各自定位协议 |
| 设置和布局 | `SettingsProvider` | `ready` 区分默认值与加载完成 |
| 提示消息 | `NotificationProvider` | 统一用户反馈 |
| 助手对话 | `AssistantSidebar` 组件状态 | 目前不属于统一文档持久化模型 |

同一数据应确定明确拥有者。比如标签 dirty 状态由文档层展示，但是否修改由 PDF / Markdown 业务层判断；不能只在关闭按钮中猜测。

## 004 · 交互与数据流

一般业务路径为：组件 → 工作区 hook / Provider → feature repository → `platform/tauriClient` → Rust command。repository 负责命令名称、参数和 DTO 到前端模型的适配。

当前模型、翻译客户端位于 `platform`，助手组件直接调用模型客户端。它是现有实现的一个边界例外，后续可在助手业务独立后收敛。

### 异步处理

- Markdown 对每个文档建立同步 Promise 队列，避免前端连续输入产生乱序提交；保存前等待最新结构。
- 搜索使用 requestId、延迟发送和取消接口，旧响应不应覆盖新查询。
- PDF 结构和位图请求在 repository 中去重并缓存；页面卸载或请求变化时释放 Object URL。
- 设置写入通过 Promise 链排队；布局拖动先改变内存状态，再由显式保存路径持久化。
- 组件卸载标志只能防止过期结果更新界面，不必然取消 Rust worker 或网络请求。

## 005 · 工作区与扩展

[WorkspaceHost.tsx](../../src/app/layout/WorkspaceHost.tsx) 根据活动文档选择 PDF / Markdown 工作区，无活动文档时显示开始页或文件夹工作区。两个业务 Provider 都在应用树内，页面切换不等于服务或会话已经释放。

新增业务交互时：

1. 确定状态由统一文档层还是具体业务层拥有。
2. 为需要 Rust 的操作增加 repository / platform 入口，并核对 [IPC](ipc.md)。
3. 若产生未保存状态，接入 lifecycle 的准备、提交与释放机制。
4. 确定切换标签、关闭、失败、卸载时的清理行为。
5. 修改对应模块文档，并按[测试与验收](testing.md)检查完整路径。

## 006 · 当前限制

没有配置前端自动测试脚本，`npm run build` 只能检查类型并打包。模型列表仍使用 `localStorage`；不能把“所有状态已经迁移到 Rust JSON”作为架构约定。源码中的废弃注释实现和未挂载组件需要与实际入口区分。

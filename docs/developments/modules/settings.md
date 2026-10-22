# 设置与工作区布局

[返回技术总览](../README.md)

## 001 · 职责与入口

设置模块负责默认值、启动行为、主题、动画、目录、模型和翻译配置，并保存两类工作区各自的侧栏尺寸。

| 入口 | 用途 |
| --- | --- |
| [SettingsProvider](../../../frontend/features/settings/SettingsProvider.tsx) | 加载、内存更新、写入队列与界面状态 |
| [types.ts](../../../frontend/features/settings/types.ts) | 前端设置类型和默认值 |
| [SettingsPanel](../../../frontend/features/settings/components/SettingsPanel.tsx) | 配置界面及连接测试 |
| [useWorkspacePanelWidths](../../../frontend/shared/useWorkspacePanelWidths.ts) | 侧栏拖动、折叠和尺寸计算 |
| [SettingsService](../../../backend/src/features/settings/service.rs) | 用例入口 |
| [JsonSettingsRepository](../../../backend/src/features/settings/adapters/json_repository.rs) | JSON、默认字段与版本迁移 |

## 002 · 设置分组

| 分组 | 主要字段与默认行为 |
| --- | --- |
| 文档存储 | PDF / Markdown 独立目录，导入模式默认为 `copy`；保留独立批注和导出目录字段 |
| 外观 | `theme: system`，可选 light / dark；动画默认开启 |
| 启动 | `restoreWorkspace`，另有 newMarkdown / startPage |
| 工作区布局 | PDF 默认左 280、右 340；Markdown 默认左 280、右 310 |
| 翻译 | 百度服务地址、凭据、目标语言；第三方翻译开关默认关闭 |
| 模型 | 服务地址、模型名、密钥、超时和 temperature |

这些字段并非都有同等完整的业务支持。存在目录设置或恢复字段，不代表已经实现数据迁移工具或所有入口都读取它。

## 003 · 加载与写入

Provider 初始使用 `defaultSettings`，随后从 Rust 加载并合并、归一化，完成后设置 `ready`。启动恢复应等待 ready，避免用尚未加载的默认启动行为覆盖工作区。

普通更新先合并 `settingsRef.current` 并更新界面，再通过 Promise 链写入 JSON。写入失败显示通知，界面内存值不自动回滚；用户看到新值不等于磁盘已成功保存。

布局更新通过 `updateWorkspaceLayout` 先写内存，`persistSettings` 再将当前尺寸提升为持久化快照。普通设置写入会使用已保存的布局快照，避免拖动过程的临时尺寸被无意保存。

## 004 · 主题与布局

主题通过 `document.documentElement.dataset.theme` 应用，system 模式监听系统深色设置；动画通过 `dataset.animations` 切换。它们影响应用界面，不自动成为 Markdown 导出的 CSS。

布局由共享尺寸 hook 结合容器宽度和业务设置处理。PDF 与 Markdown 独立保存尺寸；拖动、折叠、窗口缩小时要保证中间内容区仍可使用。加载时会对非法尺寸使用回退值。

## 005 · 持久化与迁移

当前设置 schema 为 `9`，存放于数据根目录 `settings.json`。仓库通过默认字段与旧版迁移读取历史设置，并使用统一 JSON 写入工具保存。

新增字段需同步修改前端类型与默认值、后端 domain、DTO、存储结构、转换和迁移逻辑。凭据当前跟随 JSON 存储，不能在文档中写成“已接入系统凭据库”。

恢复默认设置应按实际实现说明：当前保留 `annotationStorageDir` 和 `exportDir`，其他设置按默认值重置；它不会自动移动或删除已有托管文档。

## 006 · 验证重点与待完善

覆盖首次启动、旧设置升级、损坏 JSON、写入失败、系统主题变化、连续更新与重启后读取。布局检查拖动后的显式保存、未保存尺寸、窗口缩小和 PDF / Markdown 互不串用。

目录配置变化后的完整迁移、跨窗口写入协调、密钥存储与更细的配置错误提示仍需完善。设置文件版本与批注、文档库的版本分别演进，不能共用一个 schema 数字解释所有文件。

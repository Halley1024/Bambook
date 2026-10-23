# PDF 阅读、搜索、批注与书签

[返回技术总览](../README.md)

## 001 · 职责与入口

PDF 模块负责原貌阅读、文字定位、导航及应用附属数据。PDF 正文渲染与批注覆盖层分离，保存批注不会把文字标记写入 PDF annotation objects。书签与阅读位置的模型、服务和 JSON 适配器归 `features/pdf`，文档库仅提供通用附属数据位置。

| 入口 | 用途 |
| --- | --- |
| [PdfWorkspaceProvider](../../../src/features/pdf/PdfWorkspaceProvider.tsx) | 阅读状态、导航、标记与生命周期协调 |
| [PdfReader](../../../src/features/pdf/components/PdfReader.tsx) | 页面组织、可见范围与交互 |
| [PdfPage](../../../src/features/pdf/components/PdfPage.tsx) | 页面位图、文字层、批注和书签覆盖 |
| [mupdfRepository](../../../src/features/pdf/repositories/mupdfRepository.ts) | 页面请求、去重和 LRU 缓存 |
| [PdfDocumentService](../../../src-tauri/src/features/pdf/service.rs) | 会话与托管身份协调 |
| [MupdfPdfEngine](../../../src-tauri/src/features/pdf/adapters/mupdf_engine.rs) | 专属 worker 中打开、认证、提取和渲染 |
| [MupdfPdfSearchEngine](../../../src-tauri/src/features/pdf/adapters/mupdf_search.rs) | 独立文字索引与搜索 worker |
| [useAnnotations](../../../src/features/annotations/hooks/useAnnotations.ts) | 批注编辑、历史与保存 |

## 002 · 文档、结构与图像

打开返回文档身份、会话、路径、页数、密码状态、元数据与目录。页面结构返回尺寸、文字块、行、span 和链接；页面图像通过单独的二进制 PNG 接口返回。

`app/workflows/pdf_lifecycle.rs` 在展示会话建立后注册搜索会话；搜索注册失败会尝试关闭展示会话。加密文档认证后，需要同步更新展示与搜索两套句柄。

前端通过可见范围控制页面重工作，仅对所需页面加载文字和图像。repository 目前缓存 24 项页面结构，位图缓存最多 10 项、64 MiB，并复用相同 key 的未完成请求。位图 key 为 session、pageIndex 和取三位小数的 scale；不要将未来可能加入的旋转或颜色模式当作现有 key。

页面 Blob URL 随 effect 清理释放；关闭 repository 会清除该 session 的已缓存数据。关闭期间仍在途的请求及反复缩放、滚动的资源占用需要持续验证。

## 003 · 页码与坐标

| 数据 | 当前约定 |
| --- | --- |
| `pageIndex` | 传给页面结构与渲染接口的零基索引 |
| `pageNumber` / 用户页码 | 前端页面从 1 开始，调用引擎时减 1 |
| 页面文字几何 | 来自 MuPDF 页面空间，结合页面宽高映射到显示区域 |
| 批注矩形 | `left`、`top`、`width`、`height` 相对页面归一化，显示时换算为百分比 |
| 书签和导航位置 | 页码与页内位置；按各类型和转换函数处理，避免混用原始像素 |

文字层、搜索高亮和用户标记必须共享一致的页面尺寸与变换。设备像素比影响位图采样质量，不应改变持久化批注的位置。不能将缩放后的 DOM 像素直接写入批注 JSON。

## 004 · 搜索

通用 `DocumentSearchService` 按 target 分发。PDF 搜索 worker 使用独立 `Document`，提取并缓存页面文字索引；Unicode NFKC、大小写与空白处理需要保留到原始位置的映射。

搜索返回上下文、页码、页内位置和一个或多个矩形，前端负责列表、当前命中与导航。requestId 和 generation 使旧查询可协作取消，不能视为任意 MuPDF 调用都可即时中断。

扫描件没有文字层时不会自动进行 OCR；需要 OCR 的能力仍应单独设计和验证。

## 005 · 批注、书签与保存

批注支持文字标记及区域样式，包含 documentId、页码、摘录、正文、颜色、时间与矩形。编辑和撤销历史在前端维护，显式保存再通过 `AnnotationService` 写入 JSON。笔记可通过导出服务生成 Markdown。

书签通过独立 hooks、菜单和 repository 管理，变更参与 dirty 状态。阅读状态保存当前页、页内位置、缩放模式与比例，走单独的自动更新路径。

保存先协调托管文档身份，再处理附属数据；文档包导出携带 PDF 与数据文件。批注 schema、侧车选择和备份详见[存储设计](../storage.md)。

## 006 · 当前限制与验证

- 批注独立于 PDF 原文，外部 PDF 软件不会自动显示 Bambook JSON 中的标记。
- 当前右侧挂载 `AssistantSidebar`；仓库里存在 `AnnotationPanel` 不代表它仍在当前 PDF 工作区显示。
- 选区翻译回调目前为空，不能当作已经接通的 PDF 功能；模型问答有实际入口。
- 大型文档性能、复杂排版选择、旋转页坐标、字体和跨平台效果仍需运行验证。

验证至少覆盖普通文本、中文、跨行选择、复杂版式、加密、扫描和损坏 PDF；并检查缩放后标记不漂移、目录导航、搜索过期响应、撤销重做、保存重开与书签恢复。

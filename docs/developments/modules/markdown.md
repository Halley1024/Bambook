# Markdown 编辑、解析与导出

[返回技术总览](../README.md)

## 001 · 职责与入口

Markdown 模块管理源码输入、结构化解析、目录、搜索、保存和导出。PDF 批注与笔记不附着到 Markdown 文档。

| 入口 | 用途 |
| --- | --- |
| [MarkdownWorkspaceProvider](../../../src/features/markdown/MarkdownWorkspaceProvider.tsx) | 编辑缓冲、同步队列、保存与撤销历史 |
| [MarkdownEditor](../../../src/features/markdown/components/MarkdownEditor.tsx) | textarea 源码输入、选区与快捷键 |
| [MarkdownSplitView](../../../src/features/markdown/components/MarkdownSplitView.tsx) | 源码、预览与分栏布局 |
| [MarkdownAstRenderer](../../../src/features/markdown/components/MarkdownAstRenderer.tsx) | AST 节点的 React 渲染 |
| [structuredMarkdownRepository](../../../src/features/markdown/repositories/structuredMarkdownRepository.ts) | 结构、编辑、保存和导出接口 |
| [MarkdownDocumentService](../../../src-tauri/src/features/markdown/service.rs) | 会话、revision、编辑应用与导出协调 |
| [parser.rs](../../../src-tauri/src/features/markdown/adapters/parser.rs) | pulldown-cmark 事件到项目 AST 的转换 |
| [exporter.rs](../../../src-tauri/src/features/markdown/adapters/exporter.rs) | 独立 HTML 与 PDF 导出链路 |

## 002 · 内容模型与版本

后端 `MarkdownDocumentModel` 保存 documentId、sessionId、revision、路径、标题、源码、nodes、outline 与 diagnostics。AST 是项目自有结构，不把解析器的事件类型直接传给 React。

前端每个文档维护当前 buffer、saved buffer、后端 structure 和撤销 / 重做栈。输入可先于后端解析完成，dirty 状态依据当前内容与已保存基线判断。

编辑 DTO 使用 `startUtf16`、`endUtf16` 和 `text`；后端检查 baseRevision，转换 UTF-16 到字节区间，拒绝非法边界和重叠编辑，再解析并增加 revision。当前实现重新解析整份源码，前端常规同步也发送整段替换，不能称为增量解析器。

## 003 · 打开、编辑与显示

1. 打开文件时准备存储身份并建立 Markdown session；新建未命名文档使用空路径与初始 revision。
2. textarea 输入更新前端 buffer 和撤销历史。
3. 防抖后进入按文档串行的 Promise 链，使用当前后端 revision 提交编辑。
4. 返回的新 structure 更新 AST、目录、诊断和前端文档状态。
5. React 渲染 AST，目录导航通过节点与源码范围定位；分栏视图通过滚动同步 hook 联动。

视图枚举是 `realtime`、`source` 和 `split`。当前 `realtime` 隐藏 textarea，仅显示 AST 预览；它不是可直接编辑渲染文本的所见即所得编辑器。数学公式、图表等也需要区分“插入语法”和“正式渲染”。

## 004 · 保存、另存与关闭

保存先同步最新 buffer，再使用该 revision 写入磁盘，成功后更新 saved buffer。空路径文档先选择保存路径，取消选择应保留原标签与内容。

后端保存后还会执行存储入库逻辑；如果入库失败，需要保留失败提示，不能推断此前内容写入也被撤销。

当前“另存为”交互包含文档包导出路径，底层另有 `save_markdown_document_as` 用于选择文件目标、更新会话路径。维护时应根据实际调用区分文件另存与文档包导出。

关闭控制器先检查内容变化，收集保存或丢弃决定；提交保存后才释放 session。保存前刷新同步队列，避免较旧 AST 对应的源码覆盖最新输入。

## 005 · HTML / PDF 导出

导出携带 sessionId、revision 和用户选择的路径，后端先校验当前版本与扩展名。

- HTML：重新解析 source 生成完整 HTML，包含固定导出 CSS，以原子写入保存。
- PDF：将同一 HTML 交给 MuPDF，按 A4 尺寸 layout，转换并保存；重新打开检查页数后替换目标文件。

导出不直接使用前端 React DOM，因此应用主题、预览 CSS、图片和字体效果不能默认与导出一致。当前导出使用 `Options::all()` 重新处理源码，未复用 AST 预览的全部过滤规则；原始 HTML、链接和资源处理应专项检查。

## 006 · 恢复与当前限制

未命名恢复目前只自动备份活动未命名文档到一个恢复文件，不能恢复任意数量的未保存标签。启动读取后在询问前发起清理，恢复过程中出错也需要单独验证。

外部修改冲突检测、资源复制、完整中文字体与分页、正式公式渲染和自定义导出主题尚不能作为完整能力承诺。revision 校验保护版本匹配，但后端“读出—修改—替换”并非通用并发事务。

## 007 · 验证重点

使用中文、emoji、多层列表、表格、代码块、图片与不安全链接等样例。覆盖连续输入后立即保存、切换文档、撤销后保存、旧 revision 被拒绝、选择路径取消，以及 HTML / PDF 输出可重新打开。导出生成成功和版面正确应分别记录。

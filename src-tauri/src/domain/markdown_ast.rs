#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SourceRange {
    pub(crate) start: usize, // 源码范围起点，使用 UTF-8 字节偏移。
    pub(crate) end: usize, // 源码范围终点，使用 UTF-8 字节偏移。
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MarkdownDocumentModel {
    pub(crate) document_id: String, // 文档在应用中的标识。
    pub(crate) session_id: String, // 当前 Markdown 编辑会话的标识。
    pub(crate) revision: u64, // 当前内容的版本号。
    pub(crate) path: String, // Markdown 文件路径。
    pub(crate) title: String, // 文档显示标题。
    pub(crate) source: String, // 完整的 Markdown 源码。
    pub(crate) nodes: Vec<MarkdownNode>, // 解析得到的顶层 AST 节点。
    pub(crate) outline: Vec<MarkdownOutlineNode>, // 根据标题生成的目录树。
    pub(crate) diagnostics: Vec<MarkdownDiagnostic>, // 解析或安全检查产生的诊断信息。
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ParsedMarkdown {
    pub(crate) nodes: Vec<MarkdownNode>, // 解析得到的顶层 AST 节点。
    pub(crate) outline: Vec<MarkdownOutlineNode>, // 根据标题生成的目录树。
    pub(crate) diagnostics: Vec<MarkdownDiagnostic>, // 解析或安全检查产生的诊断信息。
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MarkdownNode {
    pub(crate) id: String, // AST 节点标识。
    pub(crate) range: SourceRange, // 节点对应的源码范围。
    pub(crate) kind: MarkdownNodeKind, // 节点类型及其内容。
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum MarkdownNodeKind {
    Paragraph {
        children: Vec<MarkdownNode>, // 段落中的内联节点。
    },
    Heading {
        level: u8, // 标题级别。
        children: Vec<MarkdownNode>, // 标题中的内联节点。
    },
    BlockQuote {
        children: Vec<MarkdownNode>, // 引用块中的子节点。
    },
    CodeBlock {
        language: Option<String>, // 代码块标注的语言。
        children: Vec<MarkdownNode>, // 代码块中的内容节点。
    },
    List {
        start: Option<u64>, // 有序列表的起始序号；无序列表为空。
        children: Vec<MarkdownNode>, // 列表项节点。
    },
    ListItem {
        children: Vec<MarkdownNode>, // 列表项中的子节点。
    },
    FootnoteDefinition {
        label: String, // 脚注定义的标签。
        children: Vec<MarkdownNode>, // 脚注定义中的子节点。
    },
    DefinitionList {
        children: Vec<MarkdownNode>, // 定义列表中的子节点。
    },
    DefinitionTitle {
        children: Vec<MarkdownNode>, // 被定义术语中的子节点。
    },
    Definition {
        children: Vec<MarkdownNode>, // 术语解释中的子节点。
    },
    Table {
        alignments: Vec<String>, // 各表格列的对齐方式。
        children: Vec<MarkdownNode>, // 表格中的表头和行节点。
    },
    TableHead {
        children: Vec<MarkdownNode>, // 表头中的子节点。
    },
    TableRow {
        children: Vec<MarkdownNode>, // 表格行中的单元格节点。
    },
    TableCell {
        children: Vec<MarkdownNode>, // 单元格中的内容节点。
    },
    Emphasis {
        children: Vec<MarkdownNode>, // 斜体内容中的子节点。
    },
    Strong {
        children: Vec<MarkdownNode>, // 加粗内容中的子节点。
    },
    Strikethrough {
        children: Vec<MarkdownNode>, // 删除线内容中的子节点。
    },
    Superscript {
        children: Vec<MarkdownNode>, // 上标内容中的子节点。
    },
    Subscript {
        children: Vec<MarkdownNode>, // 下标内容中的子节点。
    },
    Link {
        url: String, // 链接目标地址。
        title: String, // 链接的提示标题。
        children: Vec<MarkdownNode>, // 链接显示内容的子节点。
    },
    Image {
        url: String, // 图片资源地址。
        title: String, // 图片的提示标题。
        children: Vec<MarkdownNode>, // 图片替代文本对应的子节点。
    },
    Metadata {
        children: Vec<MarkdownNode>, // 元数据块中的子节点。
    },
    Text {
        text: String, // 普通文本内容。
    },
    InlineCode {
        text: String, // 行内代码文本。
    },
    Html {
        text: String, // 原始 HTML 文本。
    },
    Math {
        text: String, // 数学公式源码。
        display: bool, // 是否作为独立块公式显示。
    },
    FootnoteReference {
        label: String, // 被引用的脚注标签。
    },
    SoftBreak,
    HardBreak,
    Rule,
    TaskListMarker {
        checked: bool, // 任务列表项是否已勾选。
    },
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MarkdownOutlineNode {
    pub(crate) id: String, // 目录项标识。
    pub(crate) node_id: String, // 对应标题 AST 节点的标识。
    pub(crate) level: u8, // 标题级别。
    pub(crate) title: String, // 目录项显示标题。
    pub(crate) children: Vec<MarkdownOutlineNode>, // 下级目录项。
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MarkdownDiagnostic {
    pub(crate) severity: String, // 诊断级别标识。
    pub(crate) message: String, // 诊断说明。
    pub(crate) range: Option<SourceRange>, // 对应的源码范围；无法定位时为空。
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MarkdownEdit {
    pub(crate) start_utf16: usize, // 编辑起点的 UTF-16 偏移。
    pub(crate) end_utf16: usize, // 编辑终点的 UTF-16 偏移。
    pub(crate) text: String, // 用于替换指定范围的新文本。
}

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    ops::Range,
};

use pulldown_cmark::{Alignment, CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag};

use crate::{
    application::ports::MarkdownParser,
    domain::markdown_ast::{
        MarkdownDiagnostic, MarkdownNode, MarkdownNodeKind, MarkdownOutlineNode, ParsedMarkdown,
        SourceRange,
    },
    error::AppResult,
};

use super::sanitizer::sanitize_link;

#[derive(Default)]
pub(crate) struct PulldownMarkdownParser;

impl MarkdownParser for PulldownMarkdownParser {
    fn parse(&self, source: &str) -> AppResult<ParsedMarkdown> {
        let mut roots = Vec::new();
        let mut stack: Vec<Frame> = Vec::new();
        let parser = Parser::new_ext(source, Options::all()).into_offset_iter();

        for (event, range) in parser {
            match event {
                Event::Start(tag) => stack.push(Frame::new(tag, range)),
                Event::End(_) => {
                    if let Some(mut frame) = stack.pop() {
                        frame.range.end = range.end;
                        append_node(&mut roots, &mut stack, frame.finish(source));
                    }
                }
                Event::Text(text) => append_leaf(
                    &mut roots,
                    &mut stack,
                    source,
                    range,
                    MarkdownNodeKind::Text {
                        text: text.into_string(),
                    },
                ),
                Event::Code(text) => append_leaf(
                    &mut roots,
                    &mut stack,
                    source,
                    range,
                    MarkdownNodeKind::InlineCode {
                        text: text.into_string(),
                    },
                ),
                Event::Html(text) | Event::InlineHtml(text) => append_leaf(
                    &mut roots,
                    &mut stack,
                    source,
                    range,
                    MarkdownNodeKind::Html {
                        text: text.into_string(),
                    },
                ),
                Event::InlineMath(text) => append_leaf(
                    &mut roots,
                    &mut stack,
                    source,
                    range,
                    MarkdownNodeKind::Math {
                        text: text.into_string(),
                        display: false,
                    },
                ),
                Event::DisplayMath(text) => append_leaf(
                    &mut roots,
                    &mut stack,
                    source,
                    range,
                    MarkdownNodeKind::Math {
                        text: text.into_string(),
                        display: true,
                    },
                ),
                Event::FootnoteReference(label) => append_leaf(
                    &mut roots,
                    &mut stack,
                    source,
                    range,
                    MarkdownNodeKind::FootnoteReference {
                        label: label.into_string(),
                    },
                ),
                Event::SoftBreak => append_leaf(
                    &mut roots,
                    &mut stack,
                    source,
                    range,
                    MarkdownNodeKind::SoftBreak,
                ),
                Event::HardBreak => append_leaf(
                    &mut roots,
                    &mut stack,
                    source,
                    range,
                    MarkdownNodeKind::HardBreak,
                ),
                Event::Rule => append_leaf(
                    &mut roots,
                    &mut stack,
                    source,
                    range,
                    MarkdownNodeKind::Rule,
                ),
                Event::TaskListMarker(checked) => append_leaf(
                    &mut roots,
                    &mut stack,
                    source,
                    range,
                    MarkdownNodeKind::TaskListMarker { checked },
                ),
            }
        }

        while let Some(mut frame) = stack.pop() {
            frame.range.end = source.len();
            append_node(&mut roots, &mut stack, frame.finish(source));
        }

        let headings = collect_headings(&roots);
        let mut index = 0;
        let outline = nest_outline(&headings, &mut index, 0);
        Ok(ParsedMarkdown {
            nodes: roots,
            outline,
            diagnostics: Vec::<MarkdownDiagnostic>::new(),
        })
    }
}

struct Frame {
    range: Range<usize>,
    container: Container,
    children: Vec<MarkdownNode>,
}

impl Frame {
    fn new(tag: Tag<'_>, range: Range<usize>) -> Self {
        let container = match tag {
            Tag::Paragraph => Container::Paragraph,
            Tag::Heading { level, .. } => Container::Heading(heading_level(level)),
            Tag::BlockQuote(_) => Container::BlockQuote,
            Tag::CodeBlock(kind) => Container::CodeBlock(match kind {
                CodeBlockKind::Indented => None,
                CodeBlockKind::Fenced(info) => info
                    .split_whitespace()
                    .next()
                    .filter(|value| !value.is_empty())
                    .map(str::to_string),
            }),
            Tag::List(start) => Container::List(start),
            Tag::Item => Container::ListItem,
            Tag::FootnoteDefinition(label) => Container::FootnoteDefinition(label.into_string()),
            Tag::DefinitionList => Container::DefinitionList,
            Tag::DefinitionListTitle => Container::DefinitionTitle,
            Tag::DefinitionListDefinition => Container::Definition,
            Tag::Table(alignments) => Container::Table(
                alignments
                    .into_iter()
                    .map(alignment_name)
                    .map(str::to_string)
                    .collect(),
            ),
            Tag::TableHead => Container::TableHead,
            Tag::TableRow => Container::TableRow,
            Tag::TableCell => Container::TableCell,
            Tag::Emphasis => Container::Emphasis,
            Tag::Strong => Container::Strong,
            Tag::Strikethrough => Container::Strikethrough,
            Tag::Superscript => Container::Superscript,
            Tag::Subscript => Container::Subscript,
            Tag::Link {
                dest_url, title, ..
            } => Container::Link {
                url: sanitize_link(&dest_url, false),
                title: title.into_string(),
            },
            Tag::Image {
                dest_url, title, ..
            } => Container::Image {
                url: sanitize_link(&dest_url, true),
                title: title.into_string(),
            },
            Tag::MetadataBlock(_) | Tag::HtmlBlock => Container::Metadata,
        };
        Self {
            range,
            container,
            children: Vec::new(),
        }
    }

    fn finish(self, source: &str) -> MarkdownNode {
        let kind_name = self.container.name();
        let kind = self.container.into_kind(self.children);
        make_node(source, self.range, kind_name, kind)
    }
}

enum Container {
    Paragraph,
    Heading(u8),
    BlockQuote,
    CodeBlock(Option<String>),
    List(Option<u64>),
    ListItem,
    FootnoteDefinition(String),
    DefinitionList,
    DefinitionTitle,
    Definition,
    Table(Vec<String>),
    TableHead,
    TableRow,
    TableCell,
    Emphasis,
    Strong,
    Strikethrough,
    Superscript,
    Subscript,
    Link { url: String, title: String },
    Image { url: String, title: String },
    Metadata,
}

impl Container {
    fn name(&self) -> &'static str {
        match self {
            Self::Paragraph => "paragraph",
            Self::Heading(_) => "heading",
            Self::BlockQuote => "blockquote",
            Self::CodeBlock(_) => "code-block",
            Self::List(_) => "list",
            Self::ListItem => "list-item",
            Self::FootnoteDefinition(_) => "footnote-definition",
            Self::DefinitionList => "definition-list",
            Self::DefinitionTitle => "definition-title",
            Self::Definition => "definition",
            Self::Table(_) => "table",
            Self::TableHead => "table-head",
            Self::TableRow => "table-row",
            Self::TableCell => "table-cell",
            Self::Emphasis => "emphasis",
            Self::Strong => "strong",
            Self::Strikethrough => "strikethrough",
            Self::Superscript => "superscript",
            Self::Subscript => "subscript",
            Self::Link { .. } => "link",
            Self::Image { .. } => "image",
            Self::Metadata => "metadata",
        }
    }

    fn into_kind(self, children: Vec<MarkdownNode>) -> MarkdownNodeKind {
        match self {
            Self::Paragraph => MarkdownNodeKind::Paragraph { children },
            Self::Heading(level) => MarkdownNodeKind::Heading { level, children },
            Self::BlockQuote => MarkdownNodeKind::BlockQuote { children },
            Self::CodeBlock(language) => MarkdownNodeKind::CodeBlock { language, children },
            Self::List(start) => MarkdownNodeKind::List { start, children },
            Self::ListItem => MarkdownNodeKind::ListItem { children },
            Self::FootnoteDefinition(label) => {
                MarkdownNodeKind::FootnoteDefinition { label, children }
            }
            Self::DefinitionList => MarkdownNodeKind::DefinitionList { children },
            Self::DefinitionTitle => MarkdownNodeKind::DefinitionTitle { children },
            Self::Definition => MarkdownNodeKind::Definition { children },
            Self::Table(alignments) => MarkdownNodeKind::Table {
                alignments,
                children,
            },
            Self::TableHead => MarkdownNodeKind::TableHead { children },
            Self::TableRow => MarkdownNodeKind::TableRow { children },
            Self::TableCell => MarkdownNodeKind::TableCell { children },
            Self::Emphasis => MarkdownNodeKind::Emphasis { children },
            Self::Strong => MarkdownNodeKind::Strong { children },
            Self::Strikethrough => MarkdownNodeKind::Strikethrough { children },
            Self::Superscript => MarkdownNodeKind::Superscript { children },
            Self::Subscript => MarkdownNodeKind::Subscript { children },
            Self::Link { url, title } => MarkdownNodeKind::Link {
                url,
                title,
                children,
            },
            Self::Image { url, title } => MarkdownNodeKind::Image {
                url,
                title,
                children,
            },
            Self::Metadata => MarkdownNodeKind::Metadata { children },
        }
    }
}

fn append_leaf(
    roots: &mut Vec<MarkdownNode>,
    stack: &mut [Frame],
    source: &str,
    range: Range<usize>,
    kind: MarkdownNodeKind,
) {
    let kind_name = leaf_name(&kind);
    append_node(roots, stack, make_node(source, range, kind_name, kind));
}

fn append_node(roots: &mut Vec<MarkdownNode>, stack: &mut [Frame], node: MarkdownNode) {
    if let Some(parent) = stack.last_mut() {
        parent.children.push(node);
    } else {
        roots.push(node);
    }
}

fn make_node(
    source: &str,
    range: Range<usize>,
    kind_name: &str,
    kind: MarkdownNodeKind,
) -> MarkdownNode {
    let mut hasher = DefaultHasher::new();
    kind_name.hash(&mut hasher);
    range.start.hash(&mut hasher);
    range.end.hash(&mut hasher);
    source
        .get(range.clone())
        .unwrap_or_default()
        .hash(&mut hasher);
    MarkdownNode {
        id: format!("md-{:016x}", hasher.finish()),
        range: SourceRange {
            start: range.start,
            end: range.end,
        },
        kind,
    }
}

fn leaf_name(kind: &MarkdownNodeKind) -> &'static str {
    match kind {
        MarkdownNodeKind::Text { .. } => "text",
        MarkdownNodeKind::InlineCode { .. } => "inline-code",
        MarkdownNodeKind::Html { .. } => "html",
        MarkdownNodeKind::Math { display, .. } if *display => "display-math",
        MarkdownNodeKind::Math { .. } => "inline-math",
        MarkdownNodeKind::FootnoteReference { .. } => "footnote-reference",
        MarkdownNodeKind::SoftBreak => "soft-break",
        MarkdownNodeKind::HardBreak => "hard-break",
        MarkdownNodeKind::Rule => "rule",
        MarkdownNodeKind::TaskListMarker { .. } => "task-list-marker",
        _ => "node",
    }
}

fn heading_level(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn alignment_name(alignment: Alignment) -> &'static str {
    match alignment {
        Alignment::None => "none",
        Alignment::Left => "left",
        Alignment::Center => "center",
        Alignment::Right => "right",
    }
}

fn collect_headings(nodes: &[MarkdownNode]) -> Vec<MarkdownOutlineNode> {
    let mut headings = Vec::new();
    for node in nodes {
        if let MarkdownNodeKind::Heading { level, children } = &node.kind {
            let title = plain_text(children).trim().to_string();
            if !title.is_empty() {
                headings.push(MarkdownOutlineNode {
                    id: format!("outline-{}", node.id),
                    node_id: node.id.clone(),
                    level: *level,
                    title,
                    children: Vec::new(),
                });
            }
        }
        if let Some(children) = node_children(&node.kind) {
            headings.extend(collect_headings(children));
        }
    }
    headings
}

fn nest_outline(
    flat: &[MarkdownOutlineNode],
    index: &mut usize,
    parent_level: u8,
) -> Vec<MarkdownOutlineNode> {
    let mut nodes = Vec::new();
    while *index < flat.len() && flat[*index].level > parent_level {
        let mut node = flat[*index].clone();
        *index += 1;
        node.children = nest_outline(flat, index, node.level);
        nodes.push(node);
    }
    nodes
}

fn plain_text(nodes: &[MarkdownNode]) -> String {
    let mut value = String::new();
    for node in nodes {
        match &node.kind {
            MarkdownNodeKind::Text { text }
            | MarkdownNodeKind::InlineCode { text }
            | MarkdownNodeKind::Math { text, .. } => value.push_str(text),
            MarkdownNodeKind::SoftBreak | MarkdownNodeKind::HardBreak => value.push(' '),
            kind => {
                if let Some(children) = node_children(kind) {
                    value.push_str(&plain_text(children));
                }
            }
        }
    }
    value
}

fn node_children(kind: &MarkdownNodeKind) -> Option<&[MarkdownNode]> {
    match kind {
        MarkdownNodeKind::Paragraph { children }
        | MarkdownNodeKind::Heading { children, .. }
        | MarkdownNodeKind::BlockQuote { children }
        | MarkdownNodeKind::CodeBlock { children, .. }
        | MarkdownNodeKind::List { children, .. }
        | MarkdownNodeKind::ListItem { children }
        | MarkdownNodeKind::FootnoteDefinition { children, .. }
        | MarkdownNodeKind::DefinitionList { children }
        | MarkdownNodeKind::DefinitionTitle { children }
        | MarkdownNodeKind::Definition { children }
        | MarkdownNodeKind::Table { children, .. }
        | MarkdownNodeKind::TableHead { children }
        | MarkdownNodeKind::TableRow { children }
        | MarkdownNodeKind::TableCell { children }
        | MarkdownNodeKind::Emphasis { children }
        | MarkdownNodeKind::Strong { children }
        | MarkdownNodeKind::Strikethrough { children }
        | MarkdownNodeKind::Superscript { children }
        | MarkdownNodeKind::Subscript { children }
        | MarkdownNodeKind::Link { children, .. }
        | MarkdownNodeKind::Image { children, .. }
        | MarkdownNodeKind::Metadata { children } => Some(children),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::{application::ports::MarkdownParser, domain::markdown_ast::MarkdownNodeKind};

    use super::PulldownMarkdownParser;

    #[test]
    fn parses_ast_source_ranges_and_nested_outline() {
        let parsed = PulldownMarkdownParser
            .parse("# Title\n\nText **bold**.\n\n## Child\n")
            .unwrap();
        assert_eq!(parsed.outline.len(), 1);
        assert_eq!(parsed.outline[0].title, "Title");
        assert_eq!(parsed.outline[0].children[0].title, "Child");
        assert!(matches!(
            parsed.nodes[0].kind,
            MarkdownNodeKind::Heading { level: 1, .. }
        ));
        assert!(parsed.nodes[0].range.end > parsed.nodes[0].range.start);
    }

    #[test]
    fn strips_unsafe_link_schemes_from_ast() {
        let parsed = PulldownMarkdownParser
            .parse("[unsafe](javascript:alert(1))")
            .unwrap();
        let MarkdownNodeKind::Paragraph { children } = &parsed.nodes[0].kind else {
            panic!("expected paragraph");
        };
        let MarkdownNodeKind::Link { url, .. } = &children[0].kind else {
            panic!("expected link");
        };
        assert!(url.is_empty());
    }
}

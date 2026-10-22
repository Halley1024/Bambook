import { Fragment, type ReactNode } from "react";
import type { MarkdownSearchLocator, SearchMatch } from "../../search";
import type { MarkdownNode } from "../types/markdownAst";

type MarkdownMatch = SearchMatch<MarkdownSearchLocator>;
type RenderSearch = { matches: MarkdownMatch[]; activeMatchId: string | null };

export function MarkdownAstRenderer({ nodes, matches, activeMatchId }: {
  nodes: MarkdownNode[];
  matches: MarkdownMatch[];
  activeMatchId: string | null;
}) {
  const search = { matches, activeMatchId };
  return <>{nodes.map((node) => <MarkdownAstNode key={node.id} node={node} search={search} />)}</>;
}

function MarkdownAstNode({ node, search }: { node: MarkdownNode; search: RenderSearch }): ReactNode {
  switch (node.type) {
    case "paragraph": return <p>{children(node, search)}</p>;
    case "heading": {
      const content = children(node, search);
      if (node.level === 1) return <h1 id={node.id}>{content}</h1>;
      if (node.level === 2) return <h2 id={node.id}>{content}</h2>;
      if (node.level === 3) return <h3 id={node.id}>{content}</h3>;
      if (node.level === 4) return <h4 id={node.id}>{content}</h4>;
      if (node.level === 5) return <h5 id={node.id}>{content}</h5>;
      return <h6 id={node.id}>{content}</h6>;
    }
    case "blockQuote": return <blockquote>{children(node, search)}</blockquote>;
    case "codeBlock": return <pre><code data-language={node.language}>{plainText(node.children)}</code></pre>;
    case "list": return node.start === undefined ? <ul>{children(node, search)}</ul> : <ol start={node.start}>{children(node, search)}</ol>;
    case "listItem": return <li>{children(node, search)}</li>;
    case "definitionList": return <dl>{children(node, search)}</dl>;
    case "definitionTitle": return <dt>{children(node, search)}</dt>;
    case "definition": return <dd>{children(node, search)}</dd>;
    case "table": return <table>{children(node, search)}</table>;
    case "tableHead": return <thead>{children(node, search)}</thead>;
    case "tableRow": return <tr>{children(node, search)}</tr>;
    case "tableCell": return <td>{children(node, search)}</td>;
    case "emphasis": return <em>{children(node, search)}</em>;
    case "strong": return <strong>{children(node, search)}</strong>;
    case "strikethrough": return <del>{children(node, search)}</del>;
    case "superscript": return <sup>{children(node, search)}</sup>;
    case "subscript": return <sub>{children(node, search)}</sub>;
    case "link": return node.url ? <a href={node.url} title={node.title || undefined}>{children(node, search)}</a> : <span>{children(node, search)}</span>;
    case "image": return node.url ? <img src={node.url} title={node.title || undefined} alt={plainText(node.children)} /> : <span>{plainText(node.children)}</span>;
    case "metadata": return null;
    case "text": return <HighlightedText node={node} search={search} />;
    case "inlineCode": return <code><HighlightedText node={node} search={search} /></code>;
    case "html": return <code className="markdown-raw-html"><HighlightedText node={node} search={search} /></code>;
    case "math": return <code className={node.display ? "markdown-math display" : "markdown-math"}><HighlightedText node={node} search={search} /></code>;
    case "footnoteDefinition": return <aside id={node.id} className="markdown-footnote"><sup>{node.label}</sup>{children(node, search)}</aside>;
    case "footnoteReference": return <sup>{node.label}</sup>;
    case "taskListMarker": return <input type="checkbox" checked={node.checked} readOnly />;
    case "softBreak": return "\n";
    case "hardBreak": return <br />;
    case "rule": return <hr />;
  }
}

function children(node: MarkdownNode & { children: MarkdownNode[] }, search: RenderSearch) {
  return node.children.map((child) => <MarkdownAstNode key={child.id} node={child} search={search} />);
}

function plainText(nodes: MarkdownNode[]): string {
  return nodes.map((node) => "text" in node ? node.text : "children" in node ? plainText(node.children) : "").join("");
}

function HighlightedText({ node, search }: {
  node: Extract<MarkdownNode, { text: string }>;
  search: RenderSearch;
}) {
  const segments = search.matches.flatMap((match) => {
    const start = Math.max(node.range.start, match.locator.startUtf16);
    const end = Math.min(node.range.end, match.locator.endUtf16);
    return end > start ? [{ start: start - node.range.start, end: end - node.range.start, match }] : [];
  }).sort((left, right) => left.start - right.start);
  if (!segments.length) return node.text;

  const output: ReactNode[] = [];
  let cursor = 0;
  segments.forEach(({ start, end, match }, index) => {
    if (start > cursor) output.push(<Fragment key={`text-${index}`}>{node.text.slice(cursor, start)}</Fragment>);
    const segmentEnd = Math.max(cursor, end);
    output.push(<mark key={`${match.id}-${index}`} data-search-match={match.id}
      className={match.id === search.activeMatchId ? "active" : ""}>
      {node.text.slice(Math.max(cursor, start), segmentEnd)}
    </mark>);
    cursor = segmentEnd;
  });
  if (cursor < node.text.length) output.push(<Fragment key="tail">{node.text.slice(cursor)}</Fragment>);
  return <>{output}</>;
}

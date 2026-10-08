import { Fragment, type ReactNode } from "react";

export function MarkdownContent({ content, className }: { content: string; className?: string }) {
  const lines = content.replace(/\r\n?/g, "\n").split("\n");
  const blocks: ReactNode[] = [];
  let index = 0;

  while (index < lines.length) {
    const line = lines[index];
    if (!line.trim()) {
      index += 1;
      continue;
    }

    const codeFence = line.match(/^```([^`]*)$/);
    if (codeFence) {
      const language = codeFence[1].trim();
      const codeLines: string[] = [];
      index += 1;
      while (index < lines.length && !lines[index].startsWith("```")) codeLines.push(lines[index++]);
      if (index < lines.length) index += 1;
      blocks.push(<pre key={`code-${index}`}><code data-language={language || undefined}>{codeLines.join("\n")}</code></pre>);
      continue;
    }

    const heading = line.match(/^(#{1,6})\s+(.+)$/);
    if (heading) {
      const Tag = `h${heading[1].length}` as "h1" | "h2" | "h3" | "h4" | "h5" | "h6";
      blocks.push(<Tag key={`heading-${index}`}>{renderInline(heading[2])}</Tag>);
      index += 1;
      continue;
    }

    if (/^\s{0,3}([-*_])(?:\s*\1){2,}\s*$/.test(line)) {
      blocks.push(<hr key={`rule-${index}`} />);
      index += 1;
      continue;
    }

    if (line.startsWith(">")) {
      const quote: string[] = [];
      while (index < lines.length && lines[index].startsWith(">")) quote.push(lines[index++].replace(/^>\s?/, ""));
      blocks.push(<blockquote key={`quote-${index}`}>{renderLines(quote)}</blockquote>);
      continue;
    }

    const listMatch = line.match(/^\s*([-*+])\s+(.+)$/);
    const orderedMatch = line.match(/^\s*(\d+)\.\s+(.+)$/);
    if (listMatch || orderedMatch) {
      const ordered = Boolean(orderedMatch);
      const items: ReactNode[] = [];
      const start = orderedMatch ? Number(orderedMatch[1]) : undefined;
      while (index < lines.length) {
        const match = ordered ? lines[index].match(/^\s*\d+\.\s+(.+)$/) : lines[index].match(/^\s*[-*+]\s+(.+)$/);
        if (!match) break;
        items.push(<li key={`item-${index}`}>{renderInline(match[1])}</li>);
        index += 1;
      }
      blocks.push(ordered ? <ol key={`list-${index}`} start={start}>{items}</ol> : <ul key={`list-${index}`}>{items}</ul>);
      continue;
    }

    const paragraph: string[] = [];
    while (index < lines.length && lines[index].trim() && !startsBlock(lines[index])) paragraph.push(lines[index++]);
    blocks.push(<p key={`paragraph-${index}`}>{renderLines(paragraph)}</p>);
  }

  return <div className={className}>{blocks}</div>;
}

function startsBlock(line: string) {
  return /^```|^(#{1,6})\s+|^>|^\s*([-*+])\s+|^\s*\d+\.\s+|^\s{0,3}([-*_])(?:\s*\2){2,}\s*$/.test(line);
}

function renderLines(lines: string[]) {
  return lines.flatMap((line, index) => index === lines.length - 1
    ? renderInline(line)
    : [...renderInline(line), <br key={`break-${index}`} />]);
}

function renderInline(text: string) {
  const parts: ReactNode[] = [];
  const pattern = /(\*\*[\s\S]+?\*\*|__[\s\S]+?__|`[^`]+`|\[[^\]]+\]\([^\s)]+\)|\*[^*\n]+\*|_[^_\n]+_)/g;
  let cursor = 0;
  let match: RegExpExecArray | null;
  let key = 0;

  while ((match = pattern.exec(text)) !== null) {
    if (match.index > cursor) parts.push(<Fragment key={`text-${key++}`}>{text.slice(cursor, match.index)}</Fragment>);
    const token = match[0];
    if (token.startsWith("**") || token.startsWith("__")) {
      parts.push(<strong key={`strong-${key++}`}>{renderInline(token.slice(2, -2))}</strong>);
    } else if (token.startsWith("`")) {
      parts.push(<code key={`code-${key++}`}>{token.slice(1, -1)}</code>);
    } else if (token.startsWith("[")) {
      const link = token.match(/^\[([^\]]+)\]\(([^\s)]+)\)$/);
      const href = link?.[2];
      parts.push(href && /^https?:\/\//i.test(href)
        ? <a key={`link-${key++}`} href={href} target="_blank" rel="noreferrer">{renderInline(link[1])}</a>
        : <Fragment key={`link-${key++}`}>{link?.[1] ?? token}</Fragment>);
    } else {
      parts.push(<em key={`emphasis-${key++}`}>{renderInline(token.slice(1, -1))}</em>);
    }
    cursor = match.index + token.length;
  }
  if (cursor < text.length) parts.push(<Fragment key={`tail-${key}`}>{text.slice(cursor)}</Fragment>);
  return parts;
}

import type { EditorSelection, MarkdownCommand } from "../types/markdownWorkspace";

type CommandResult = {
  content: string;
  selection: EditorSelection;
};

export function applyMarkdownCommand(
  content: string,
  selection: EditorSelection,
  command: MarkdownCommand,
  payload?: string,
): CommandResult {
  if (isHeadingCommand(command)) return applyHeadingCommand(content, selection, command);
  if (command === "clearFormatting") return clearFormatting(content, selection);
  const selected = content.slice(selection.start, selection.end);
  const template = commandTemplate(command, selected, payload);
  const contentAfter = content.slice(selection.end);
  const nextContent = `${content.slice(0, selection.start)}${template.text}${contentAfter}`;
  const cursorStart = selection.start + template.selectionStart;

  return {
    content: nextContent,
    selection: {
      start: cursorStart,
      end: selection.start + template.selectionEnd,
    },
  };
}

function commandTemplate(command: MarkdownCommand, selected: string, payload?: string) {
  const value = selected || fallbackText(command);
  switch (command) {
    case "bold":
      return wrap(`**${value}**`, 2, 2 + value.length);
    case "italic":
      return wrap(`*${value}*`, 1, 1 + value.length);
    case "underline":
      return wrap(`<u>${value}</u>`, 3, 3 + value.length);
    case "strikethrough":
      return wrap(`~~${value}~~`, 2, 2 + value.length);
    case "highlight":
      return wrap(`==${value}==`, 2, 2 + value.length);
    case "superscript":
      return wrap(`^${value}^`, 1, 1 + value.length);
    case "subscript":
      return wrap(`~${value}~`, 1, 1 + value.length);
    case "unorderedList":
      return prefixedLines(value, "- ");
    case "orderedList":
      return prefixedLines(value, (_, index) => `${index + 1}. `);
    case "taskList":
      return prefixedLines(value, "- [ ] ");
    case "inlineCode":
      return wrap(`\`${value}\``, 1, 1 + value.length);
    case "link":
      return wrap(`[${value}](https://)`, 1, 1 + value.length);
    case "image":
      return wrap(`![${value}](${payload || "image-path"})`, 2, 2 + value.length);
    case "inlineFormula":
      return wrap(`$${value}$`, 1, 1 + value.length);
    case "blockFormula":
      return wrap(`\n$$\n${value}\n$$\n`, 4, 4 + value.length);
    case "code":
      return wrap(`\n\`\`\`\n${value}\n\`\`\`\n`, 5, 5 + value.length);
    case "quote":
      return wrap(`> ${value}`, 2, 2 + value.length);
    case "table": {
      const table = "\n| 列 1 | 列 2 |\n| --- | --- |\n| 内容 | 内容 |\n";
      return wrap(table, table.length, table.length);
    }
    case "rule": {
      const rule = "\n---\n";
      return wrap(rule, rule.length, rule.length);
    }
    default:
      return wrap(value, 0, value.length);
  }
}

function fallbackText(command: MarkdownCommand) {
  if (command === "bold") return "粗体文字";
  if (command === "italic") return "斜体文字";
  if (command === "underline") return "下划线文字";
  if (command === "strikethrough") return "删除线文字";
  if (command === "highlight") return "高亮文字";
  if (command === "superscript") return "上标";
  if (command === "subscript") return "下标";
  if (command === "unorderedList" || command === "orderedList") return "列表项";
  if (command === "taskList") return "待办事项";
  if (command === "inlineCode" || command === "code") return "代码";
  if (command === "link") return "链接文字";
  if (command === "image") return "图片说明";
  if (command === "inlineFormula" || command === "blockFormula") return "E = mc^2";
  if (command === "quote") return "引用内容";
  return "";
}

function wrap(text: string, selectionStart: number, selectionEnd: number) {
  return { text, selectionStart, selectionEnd };
}

function prefixedLines(value: string, prefix: string | ((line: string, index: number) => string)) {
  const lines = value.split(/\r?\n/);
  const text = lines.map((line, index) => `${typeof prefix === "string" ? prefix : prefix(line, index)}${line}`).join("\n");
  return wrap(text, 0, text.length);
}

type HeadingCommand = Extract<MarkdownCommand,
  "heading1" | "heading2" | "heading3" | "heading4" | "heading5" | "promoteHeading" | "demoteHeading">;

function isHeadingCommand(command: MarkdownCommand): command is HeadingCommand {
  return command.startsWith("heading") || command === "promoteHeading" || command === "demoteHeading";
}

function applyHeadingCommand(content: string, selection: EditorSelection, command: HeadingCommand): CommandResult {
  const range = selectedLineRange(content, selection);
  const source = content.slice(range.start, range.end);
  const requestedLevel = command.startsWith("heading") ? Number(command.slice(-1)) : null;
  const text = source.split(/\r?\n/).map((line) => {
    const match = line.match(/^(\s{0,3})(#{1,5})\s+(.*)$/);
    const currentLevel = match?.[2].length ?? 0;
    const body = match?.[3] ?? line.replace(/^\s+/, "");
    const level = requestedLevel ?? (command === "promoteHeading"
      ? Math.max(1, currentLevel ? currentLevel - 1 : 1)
      : Math.min(5, currentLevel ? currentLevel + 1 : 1));
    return `${"#".repeat(level)} ${body}`;
  }).join("\n");
  return replaceRange(content, range, text);
}

function clearFormatting(content: string, selection: EditorSelection): CommandResult {
  const range = selection.start === selection.end ? selectedLineRange(content, selection) : selection;
  const source = content.slice(range.start, range.end);
  const text = source
    .replace(/^(\s{0,3})(#{1,5}|>|[-+*]|\d+\.|- \[[ xX]\])\s+/gm, "$1")
    .replace(/<\/?u>/g, "")
    .replace(/(\*\*|__|~~|==|`|\^|~)/g, "")
    .replace(/^\$\$?\s*|\s*\$\$?$/g, "");
  return replaceRange(content, range, text);
}

function selectedLineRange(content: string, selection: EditorSelection): EditorSelection {
  const start = content.lastIndexOf("\n", Math.max(0, selection.start - 1)) + 1;
  const nextBreak = content.indexOf("\n", selection.end);
  return { start, end: nextBreak === -1 ? content.length : nextBreak };
}

function replaceRange(content: string, range: EditorSelection, text: string): CommandResult {
  return {
    content: `${content.slice(0, range.start)}${text}${content.slice(range.end)}`,
    selection: { start: range.start, end: range.start + text.length },
  };
}

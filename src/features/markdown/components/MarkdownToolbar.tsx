import { Bold, Code2, Columns2, Eye, FileCode2, Heading, Heading1, Heading2, Heading3, Heading4, Heading5,
  Highlighter, Image, Italic, Link, List, ListChecks, ListOrdered, Minus, Quote, Redo2, RemoveFormatting,
  Sigma, Strikethrough, Subscript, Superscript, Table2, Type, Underline, Undo2 } from "lucide-react";
import type { ReactNode } from "react";
import { DropdownMenu, type DropdownMenuGroup } from "../../../components/menus/DropdownMenu";
import { selectFilePath } from "../../../platform/fileDialog";
import { useNotification } from "../../notifications";
import { DocumentSearchTools } from "../../search";
import { useMarkdownWorkspace } from "../hooks/useMarkdownWorkspace";
import { useMarkdownSearch } from "../search";
import type { MarkdownCommand, MarkdownViewMode } from "../types/markdownWorkspace";

const viewModes: Array<{ mode: MarkdownViewMode; label: string; icon: typeof Eye }> = [
  { mode: "realtime", label: "实时", icon: Eye },
  { mode: "source", label: "源码", icon: FileCode2 },
  { mode: "split", label: "分栏", icon: Columns2 },
];

export function MarkdownToolbar() {
  const workspace = useMarkdownWorkspace();
  const search = useMarkdownSearch();
  const notification = useNotification();
  const command = (value: MarkdownCommand) => () => workspace.executeCommand(value);
  const headingGroups: DropdownMenuGroup[] = [
    { id: "levels", items: [
      { id: "heading-1", label: "一级标题", icon: Heading1, shortcut: "Ctrl+Alt+1", onSelect: command("heading1") },
      { id: "heading-2", label: "二级标题", icon: Heading2, shortcut: "Ctrl+Alt+2", onSelect: command("heading2") },
      { id: "heading-3", label: "三级标题", icon: Heading3, shortcut: "Ctrl+Alt+3", onSelect: command("heading3") },
      { id: "heading-4", label: "四级标题", icon: Heading4, shortcut: "Ctrl+Alt+4", onSelect: command("heading4") },
      { id: "heading-5", label: "五级标题", icon: Heading5, shortcut: "Ctrl+Alt+5", onSelect: command("heading5") },
    ] },
    { id: "level-change", items: [
      { id: "promote-heading", label: "提升标题级别", icon: Heading1, shortcut: "Alt+Shift+←", onSelect: command("promoteHeading") },
      { id: "demote-heading", label: "降低标题级别", icon: Heading5, shortcut: "Alt+Shift+→", onSelect: command("demoteHeading") },
    ] },
  ];
  const formatGroups: DropdownMenuGroup[] = [
    { id: "emphasis", items: [
      { id: "bold", label: "加粗", icon: Bold, shortcut: "Ctrl+B", onSelect: command("bold") },
      { id: "italic", label: "斜体", icon: Italic, shortcut: "Ctrl+I", onSelect: command("italic") },
    ] },
    { id: "decoration", items: [
      { id: "underline", label: "下划线", icon: Underline, shortcut: "Ctrl+U", onSelect: command("underline") },
      { id: "strikethrough", label: "删除线", icon: Strikethrough, shortcut: "Alt+Shift+5", onSelect: command("strikethrough") },
      { id: "highlight", label: "高亮", icon: Highlighter, shortcut: "Ctrl+Shift+H", onSelect: command("highlight") },
    ] },
    { id: "script", items: [
      { id: "superscript", label: "上标", icon: Superscript, shortcut: "Ctrl+Shift+=", onSelect: command("superscript") },
      { id: "subscript", label: "下标", icon: Subscript, shortcut: "Ctrl+=", onSelect: command("subscript") },
    ] },
    { id: "clear", items: [
      { id: "clear-formatting", label: "删除样式", icon: RemoveFormatting, shortcut: "Ctrl+\\", onSelect: command("clearFormatting") },
    ] },
  ];
  const listGroups: DropdownMenuGroup[] = [{ id: "lists", items: [
    { id: "ordered-list", label: "有序列表", icon: ListOrdered, shortcut: "Ctrl+Shift+7", onSelect: command("orderedList") },
    { id: "unordered-list", label: "无序列表", icon: List, shortcut: "Ctrl+Shift+8", onSelect: command("unorderedList") },
    { id: "task-list", label: "任务列表", icon: ListChecks, shortcut: "Ctrl+Shift+9", onSelect: command("taskList") },
  ] }];
  const formulaGroups: DropdownMenuGroup[] = [{ id: "formula", items: [
    { id: "inline-formula", label: "内联公式", icon: Sigma, shortcut: "Ctrl+Shift+M", onSelect: command("inlineFormula") },
    { id: "block-formula", label: "外联公式", icon: Sigma, shortcut: "Alt+Shift+M", onSelect: command("blockFormula") },
  ] }];
  const insertImage = async () => {
    try {
      const path = await selectFilePath([{ name: "图片", extensions: ["png", "jpg", "jpeg", "gif", "webp", "svg", "bmp"] }]);
      if (path) workspace.executeCommand("image", path.replace(/\\/g, "/"));
    } catch (error) {
      notification.error(`选择图片失败：${String(error)}`);
    }
  };

  return <div className="mode-toolbar markdown-mode-toolbar" aria-label="Markdown 工具">
    <div className="toolbar-group markdown-history-tools" aria-label="编辑历史">
      <button className="icon-button" onClick={workspace.undo} disabled={!workspace.canUndo}
        data-tooltip="撤销：恢复上一次编辑（Ctrl+Z）" aria-label="撤销"><Undo2 size={18} /></button>
      <button className="icon-button" onClick={workspace.redo} disabled={!workspace.canRedo}
        data-tooltip="重做：恢复被撤销的编辑（Ctrl+Y）" aria-label="重做"><Redo2 size={18} /></button>
    </div>
    <div className="toolbar-group markdown-dropdown-group" aria-label="标题与格式">
      <DropdownMenu label="标题" icon={Heading} groups={headingGroups} />
      <DropdownMenu label="格式" icon={Type} groups={formatGroups} />
    </div>
    <div className="toolbar-group markdown-block-tools" aria-label="列表与段落">
      <DropdownMenu label="列表" icon={List} groups={listGroups} />
      <ToolbarAction label="引用" description="将当前段落转换为引用" icon={<Quote size={17} />} shortcut="Ctrl+Shift+Q" onClick={command("quote")} />
      <ToolbarAction label="代码块" description="插入或切换当前代码块" icon={<Code2 size={17} />} shortcut="Ctrl+Shift+`" onClick={command("code")} />
    </div>
    <div className="toolbar-group markdown-insert-tools" aria-label="插入内容">
      <ToolbarAction label="链接" description="为选中文字添加链接" icon={<Link size={17} />} shortcut="Ctrl+K" onClick={command("link")} />
      <ToolbarAction label="图片" description="从本地选择并插入图片" icon={<Image size={17} />} onClick={() => void insertImage()} />
      <ToolbarAction label="表格" description="在当前位置插入表格" icon={<Table2 size={17} />} onClick={command("table")} />
      <DropdownMenu label="公式" icon={Sigma} groups={formulaGroups} />
      <ToolbarAction label="分割线" description="在当前位置插入水平分割线" icon={<Minus size={17} />} onClick={command("rule")} />
    </div>
    <DocumentSearchTools search={search} placeholder="搜索 Markdown 文档" />
    <div className="toolbar-group markdown-view-tools" aria-label="Markdown 模式">
      {viewModes.map((item) => {
        const Icon = item.icon;
        return <button key={item.mode} className={`tool-button markdown-view-button ${workspace.viewMode === item.mode ? "active" : ""}`}
          onClick={() => workspace.setViewMode(item.mode)} data-tooltip={`${item.label}模式：切换 Markdown 显示方式`}
          aria-label={`${item.label}模式`}><Icon size={17} /><span>{item.label}</span></button>;
      })}
    </div>
  </div>;
}

function ToolbarAction({ label, description, icon, shortcut, onClick }: {
  label: string;
  description: string;
  icon: ReactNode;
  shortcut?: string;
  onClick: () => void;
}) {
  return <button className="tool-button markdown-command-button" onClick={onClick}
    data-tooltip={`${label}：${description}${shortcut ? `（${shortcut}）` : ""}`} aria-label={label}>
    {icon}<span className="markdown-tool-label">{label}</span>
  </button>;
}

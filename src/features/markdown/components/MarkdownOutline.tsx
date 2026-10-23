import { Hash } from "lucide-react";
import type { MarkdownOutlineItem } from "../types/markdownWorkspace";

export function MarkdownOutline({ items, activeId, onNavigate }: {
  items: MarkdownOutlineItem[];
  activeId?: string | null;
  onNavigate: (item: MarkdownOutlineItem) => void;
}) {
  if (items.length === 0) return <div className="markdown-sidebar-empty">当前文档没有标题</div>;

  return (
    <div className="markdown-outline" role="tree" aria-label="Markdown 标题目录">
      {items.map((item) => (
        <button
          key={item.id}
          className={`markdown-outline-item ${activeId === item.id ? "active" : ""}`}
          style={{ paddingLeft: 10 + (item.level - 1) * 14 }}
          onClick={() => onNavigate(item)}
          data-tooltip={`跳转标题：${item.title}`}
        >
          <Hash size={14} />
          <span>{item.title}</span>
        </button>
      ))}
    </div>
  );
}

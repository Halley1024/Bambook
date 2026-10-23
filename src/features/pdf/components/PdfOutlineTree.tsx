import { ChevronRight } from "lucide-react";
import type { MutableRefObject } from "react";
import type { OutlineItem } from "../hooks/usePdfOutline";

export function PdfOutlineTree({ items, loading, expanded, activeId, itemRefs, onToggle, onNavigate }: {
  items: OutlineItem[];
  loading: boolean;
  expanded: Set<string>;
  activeId: string | null;
  itemRefs: MutableRefObject<Map<string, HTMLButtonElement>>;
  onToggle: (id: string) => void;
  onNavigate: (page: number, title: string) => void;
}) {
  return (
    <div className="outline-list" role="tree" aria-label="PDF 目录">
      {loading && <div className="sidebar-empty">正在读取目录…</div>}
      {!loading && items.length === 0 && <div className="sidebar-empty">此 PDF 没有内置目录</div>}
      {items.map((item) => (
        <PdfOutlineNode
          key={item.id}
          item={item}
          depth={0}
          expanded={expanded}
          activeId={activeId}
          itemRefs={itemRefs}
          onToggle={onToggle}
          onNavigate={onNavigate}
        />
      ))}
    </div>
  );
}

function PdfOutlineNode({ item, depth, expanded, activeId, itemRefs, onToggle, onNavigate }: {
  item: OutlineItem;
  depth: number;
  expanded: Set<string>;
  activeId: string | null;
  itemRefs: MutableRefObject<Map<string, HTMLButtonElement>>;
  onToggle: (id: string) => void;
  onNavigate: (page: number, title: string) => void;
}) {
  const hasChildren = item.children.length > 0;
  const isExpanded = expanded.has(item.id);
  const isActive = activeId === item.id;

  return (
    <div className="outline-node" role="treeitem" aria-expanded={hasChildren ? isExpanded : undefined}>
      <div className="outline-row" style={{ paddingLeft: 10 + Math.min(depth, 8) * 15 }}>
        {hasChildren ? (
          <button
            className={`outline-toggle ${isExpanded ? "expanded" : ""}`}
            data-tooltip={`${isExpanded ? "折叠" : "展开"}目录：${item.title}`}
            onClick={() => onToggle(item.id)}
            aria-label={isExpanded ? `折叠 ${item.title}` : `展开 ${item.title}`}
          >
            <ChevronRight size={15} />
          </button>
        ) : (
          <span className="outline-toggle-spacer" />
        )}
        <button
          ref={(element) => {
            if (element) itemRefs.current.set(item.id, element);
            else itemRefs.current.delete(item.id);
          }}
          className={`outline-title ${isActive ? "active" : ""} ${item.page ? "" : "unavailable"}`}
          onClick={() => item.page && onNavigate(item.page, item.title)}
          data-tooltip={item.page ? `目录跳转：${item.title} · 第 ${item.page} 页` : `目录项：${item.title}`}
        >
          {item.title || "未命名标题"}
        </button>
      </div>
      {hasChildren && (
        <div className={`outline-children ${isExpanded ? "expanded" : ""}`}>
          <div>
            {item.children.map((child) => (
              <PdfOutlineNode
                key={child.id}
                item={child}
                depth={depth + 1}
                expanded={expanded}
                activeId={activeId}
                itemRefs={itemRefs}
                onToggle={onToggle}
                onNavigate={onNavigate}
              />
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

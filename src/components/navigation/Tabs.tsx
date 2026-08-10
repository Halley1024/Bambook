import { useCallback, useEffect, useRef, useState } from "react";
import { BookOpen, Copy, ExternalLink, FileText, Files, FolderTree, X } from "lucide-react";

export type TabItem = {
  id: string;
  label: string;
  description?: string;
  kind?: "pdf" | "markdown";
  dirty?: boolean;
};

type TabsProps = {
  items: TabItem[];
  activeId: string | null;
  emptyText: string;
  ariaLabel: string;
  onActivate: (id: string) => void;
  onClose: (id: string) => void;
  onCloseOthers: (id: string) => void;
  onCloseAll: () => void;
  onCopyLabel: (id: string) => void;
  onCopyRelativePath: (id: string) => void;
  onCopyAbsolutePath: (id: string) => void;
  onReorder: (sourceId: string, targetId: string) => void;
};

export function Tabs({ items, activeId, emptyText, ariaLabel, onActivate, onClose, onCloseOthers, onCloseAll,
  onCopyLabel, onCopyRelativePath, onCopyAbsolutePath, onReorder }: TabsProps) {
  const [draggedId, setDraggedId] = useState<string | null>(null);
  const [dropTargetId, setDropTargetId] = useState<string | null>(null);
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number; item: TabItem } | null>(null);
  const scrollRef = useRef<HTMLDivElement>(null);
  const dragRef = useRef<{ sourceId: string; startX: number; startY: number; moved: boolean } | null>(null);
  const dropTargetRef = useRef<string | null>(null);
  const onReorderRef = useRef(onReorder);
  onReorderRef.current = onReorder;

  const findTabAtPoint = useCallback((x: number, y: number) => {
    const el = document.elementFromPoint(x, y);
    if (!el) return null;
    const tab = (el as HTMLElement).closest("[data-tab-id]");
    return tab ? tab.getAttribute("data-tab-id") : null;
  }, []);

  useEffect(() => {
    const onMove = (event: PointerEvent) => {
      const drag = dragRef.current;
      if (!drag) return;
      const dx = Math.abs(event.clientX - drag.startX);
      const dy = Math.abs(event.clientY - drag.startY);
      if (dx < 6 && dy < 6) return;
      if (!drag.moved) {
        drag.moved = true;
        setDraggedId(drag.sourceId);
      }
      const targetId = findTabAtPoint(event.clientX, event.clientY);
      dropTargetRef.current = targetId !== drag.sourceId ? targetId : null;
      setDropTargetId(dropTargetRef.current);
    };

    const onUp = () => {
      const drag = dragRef.current;
      if (!drag) return;
      if (drag.moved) {
        const target = dropTargetRef.current;
        if (target && target !== drag.sourceId) {
          onReorderRef.current(drag.sourceId, target);
        }
      }
      dragRef.current = null;
      dropTargetRef.current = null;
      setDraggedId(null);
      setDropTargetId(null);
    };

    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
    return () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
    };
  }, [findTabAtPoint]);

  useEffect(() => {
    if (!contextMenu) return;
    const close = () => setContextMenu(null);
    const closeOnEscape = (event: KeyboardEvent) => event.key === "Escape" && close();
    window.addEventListener("pointerdown", close);
    window.addEventListener("keydown", closeOnEscape);
    return () => {
      window.removeEventListener("pointerdown", close);
      window.removeEventListener("keydown", closeOnEscape);
    };
  }, [contextMenu]);

  const handlePointerDown = (event: React.PointerEvent, item: TabItem) => {
    if ((event.target as HTMLElement).closest(".document-tab-close")) return;
    if (event.button !== 0) return;
    dragRef.current = { sourceId: item.id, startX: event.clientX, startY: event.clientY, moved: false };
  };

  return (
    <div className="document-tabs" role="tablist" aria-label={ariaLabel}>
      <div className="document-tabs-scroll" ref={scrollRef}>
        {items.length === 0 && <span className="document-tabs-empty">{emptyText}</span>}
        {items.map((item) => {
          const active = item.id === activeId;
          const DocumentIcon = item.kind === "pdf" ? BookOpen : FileText;
          return (
            <div
              key={item.id}
              data-tab-id={item.id}
              className={`document-tab ${active ? "active" : ""} ${draggedId === item.id ? "dragging" : ""} ${
                dropTargetId === item.id ? "drop-target" : ""
              }`}
              role="tab"
              aria-selected={active}
              data-tooltip={item.description ? `${item.label}：${item.description}` : `切换文档：${item.label}`}
              onPointerDown={(event) => handlePointerDown(event, item)}
              onPointerUp={() => {
                const drag = dragRef.current;
                if (drag && !drag.moved) {
                  dragRef.current = null;
                  onActivate(item.id);
                }
              }}
              onContextMenu={(event) => {
                event.preventDefault();
                onActivate(item.id);
                setContextMenu({
                  x: Math.min(event.clientX, window.innerWidth - 230),
                  y: Math.max(8, Math.min(event.clientY, window.innerHeight - 548)),
                  item,
                });
              }}
            >
              <DocumentIcon size={14} className="document-tab-icon" />
              <span className="document-tab-title">{item.label}</span>
              {item.dirty && <span className="document-tab-dirty" aria-label="未保存" />}
              <button
                className="document-tab-close"
                onClick={(event) => {
                  event.stopPropagation();
                  onClose(item.id);
                }}
                onPointerDown={(event) => event.stopPropagation()}
                data-tooltip={`关闭文档：${item.label}`}
                aria-label={`关闭 ${item.label}`}
              >
                <X size={14} />
              </button>
            </div>
          );
        })}
      </div>

      {contextMenu && (
        <div className="tab-context-menu" style={{ left: contextMenu.x, top: contextMenu.y }} onPointerDown={(event) => event.stopPropagation()}>
          <button onClick={() => { onCopyLabel(contextMenu.item.id); setContextMenu(null); }}><Copy size={15} />复制标签</button>
          <button onClick={() => { onCopyRelativePath(contextMenu.item.id); setContextMenu(null); }}><FolderTree size={15} />复制相对路径</button>
          <button onClick={() => { onCopyAbsolutePath(contextMenu.item.id); setContextMenu(null); }}><FileText size={15} />复制绝对路径</button>
          <div className="tab-context-menu-separator" role="separator" />
          <button disabled title="将在新建窗口功能完成后接入"><ExternalLink size={15} />复制到新窗口</button>
          <button disabled title="将在新建窗口功能完成后接入"><ExternalLink size={15} />移动到新窗口</button>
          <div className="tab-context-menu-separator" role="separator" />
          <button onClick={() => { onClose(contextMenu.item.id); setContextMenu(null); }}><X size={15} />关闭标签</button>
          <button disabled={items.length <= 1} onClick={() => { onCloseOthers(contextMenu.item.id); setContextMenu(null); }}>
            <Files size={15} />关闭其他标签
          </button>
          <button className="danger" onClick={() => { onCloseAll(); setContextMenu(null); }}><X size={15} />关闭所有标签</button>
        </div>
      )}
    </div>
  );
}

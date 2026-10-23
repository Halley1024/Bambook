import { Bell, BookOpen, FileText, X } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { UnifiedDocumentTabs, useDocumentWorkspace } from "../../features/documents";
import { TitlebarIcon } from "./TitlebarIcons";

export function WindowTitleBar() {
  const workspace = useDocumentWorkspace();
  const [popover, setPopover] = useState<"notifications" | "tabs" | null>(null);
  const actionsRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (!popover) return;
    const closeOutside = (event: PointerEvent) => {
      if (!actionsRef.current?.contains(event.target as Node)) setPopover(null);
    };
    const closeOnEscape = (event: KeyboardEvent) => { if (event.key === "Escape") setPopover(null); };
    window.addEventListener("pointerdown", closeOutside);
    window.addEventListener("keydown", closeOnEscape);
    return () => {
      window.removeEventListener("pointerdown", closeOutside);
      window.removeEventListener("keydown", closeOnEscape);
    };
  }, [popover]);

  return (
    <header className="window-titlebar">
      <button type="button" className={`window-brand ${workspace.homeVisible ? "active" : ""}`}
        onClick={workspace.showHome} aria-label="Bambook 主页" data-tooltip="主页">
        <span className="window-brand-icon"><TitlebarIcon slot="navigation.home" size={18} /></span>
        <span className="window-brand-wordmark" aria-hidden="true">Bambook</span>
      </button>

      <div className="window-document-tabs"><UnifiedDocumentTabs /></div>

      <div className="window-titlebar-spacer" aria-hidden="true" />

      <div className="window-titlebar-actions" ref={actionsRef}>
        <div className="titlebar-popover-host">
          <button type="button" className={`titlebar-icon-button ${popover === "tabs" ? "active" : ""}`}
            onClick={() => setPopover((value) => value === "tabs" ? null : "tabs")}
            aria-label="所有打开的标签页" data-tooltip="所有打开的标签页"><TitlebarIcon slot="tabs.list" size={18} /></button>
          {popover === "tabs" && <div className="titlebar-popover open-tabs-popover">
            <header><strong>打开的标签页</strong><small>{workspace.documents.length}</small></header>
            <div className="open-tabs-list">
              {workspace.documents.length === 0 ? <div className="titlebar-empty-message"><span>没有打开的标签页</span></div>
                : workspace.documents.map((document) => {
                  const Icon = document.kind === "pdf" ? BookOpen : FileText;
                  return <div className={`open-tabs-item ${document.id === workspace.activeDocumentId ? "active" : ""}`} key={document.id}>
                    <button type="button" className="open-tabs-activate"
                      onClick={() => { workspace.activateDocument(document.id); setPopover(null); }}>
                      <Icon size={15} /><span>{document.title}</span>
                      {workspace.isDocumentDirty?.(document.id) && <i aria-label="未保存" />}
                    </button>
                    <button type="button" className="open-tabs-close" aria-label={`关闭 ${document.title}`}
                      data-tooltip={`关闭标签页：${document.title}`}
                      onClick={() => void workspace.closeDocument(document.id)}><X size={15} /></button>
                  </div>;
                })}
            </div>
            <footer><button type="button" className="danger" disabled={!workspace.documents.length}
              onClick={() => { setPopover(null); void workspace.closeAllDocuments(); }}><X size={15} />关闭所有标签页</button></footer>
          </div>}
        </div>

        <div className="titlebar-popover-host">
          <button type="button" className={`titlebar-icon-button ${popover === "notifications" ? "active" : ""}`}
            onClick={() => setPopover((value) => value === "notifications" ? null : "notifications")}
            aria-label="通知" data-tooltip="通知"><TitlebarIcon slot="notifications.updates" size={17} /></button>
          {popover === "notifications" && <div className="titlebar-popover notification-popover">
            <header><strong>通知</strong><small>版本更新</small></header>
            <div className="titlebar-empty-message"><Bell size={22} /><span>暂无新通知</span><small>检测到新版本时会在这里提醒。</small></div>
          </div>}
        </div>

      </div>
    </header>
  );
}

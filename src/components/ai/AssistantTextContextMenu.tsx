import { Bot, Copy } from "lucide-react";
import { useEffect, useState, type ReactNode } from "react";
import { createPortal } from "react-dom";
import { copyText } from "../../platform/clipboard";
import { useNotification } from "../../features/notifications";
import { AssistantMenuActions } from "./AssistantMenuActions";
import type { AssistantTextAction } from "./assistantActions";

export function AssistantTextContextMenu({ children, onRequest }: {
  children: ReactNode;
  onRequest: (text: string, action?: AssistantTextAction) => void;
}) {
  const [menu, setMenu] = useState<{ x: number; y: number; text: string } | null>(null);
  const notification = useNotification();
  useEffect(() => {
    if (!menu) return;
    const close = () => setMenu(null);
    const key = (event: KeyboardEvent) => { if (event.key === "Escape") close(); };
    window.addEventListener("pointerdown", close);
    window.addEventListener("keydown", key);
    window.addEventListener("resize", close);
    window.addEventListener("scroll", close, true);
    return () => {
      window.removeEventListener("pointerdown", close);
      window.removeEventListener("keydown", key);
      window.removeEventListener("resize", close);
      window.removeEventListener("scroll", close, true);
    };
  }, [menu]);
  const request = (action?: AssistantTextAction) => {
    if (!menu) return;
    onRequest(menu.text, action);
    setMenu(null);
  };
  return <div className="assistant-text-menu-surface" onContextMenu={(event) => {
    const target = event.target;
    let text = "";
    if (target instanceof HTMLTextAreaElement) text = target.value.slice(target.selectionStart, target.selectionEnd).trim();
    else {
      const selection = window.getSelection();
      if (selection?.anchorNode && selection.focusNode && event.currentTarget.contains(selection.anchorNode) && event.currentTarget.contains(selection.focusNode)) text = selection.toString().trim();
    }
    if (!text) { setMenu(null); return; }
    event.preventDefault();
    setMenu({ text, x: Math.max(8, Math.min(event.clientX, window.innerWidth - 208)), y: Math.max(8, Math.min(event.clientY, window.innerHeight - 196)) });
  }}>{children}{menu && createPortal(<div className="selection-context-menu assistant-text-context-menu" style={{ left: menu.x, top: menu.y }} aria-label="文字操作"
    onPointerDown={(event) => { event.preventDefault(); event.stopPropagation(); }}>
    <button type="button" onClick={() => { void copyText(menu.text).catch(() => notification.error("复制失败，请手动复制文字。")); setMenu(null); }}><Copy size={17} />复制文字</button>
    <button type="button" onClick={() => request()}><Bot size={17} />询问大模型</button>
    <div className="context-menu-separator" />
    <AssistantMenuActions onAction={request} />
  </div>, document.body)}</div>;
}

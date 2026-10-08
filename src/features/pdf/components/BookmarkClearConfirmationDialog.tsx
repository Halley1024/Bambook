import { useEffect, useRef } from "react";
import { createPortal } from "react-dom";
import { AlertTriangle, Trash2 } from "lucide-react";

export function BookmarkClearConfirmationDialog({ description, onCancel, onConfirm }: {
  description: string;
  onCancel: () => void;
  onConfirm: () => void;
}) {
  const confirmRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    confirmRef.current?.focus();
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      event.preventDefault();
      onCancel();
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [onCancel]);

  return createPortal(
    <div className="close-dialog-backdrop" onMouseDown={onCancel}>
      <section className="close-dialog bookmark-clear-dialog" role="alertdialog" aria-modal="true"
        aria-labelledby="bookmark-clear-title" aria-describedby="bookmark-clear-description"
        onMouseDown={(event) => event.stopPropagation()}>
        <header className="close-dialog-header">
          <span className="close-dialog-icon bookmark-clear-dialog-icon" aria-hidden="true"><AlertTriangle size={20} /></span>
          <div>
            <h2 id="bookmark-clear-title">清除书签</h2>
            <p id="bookmark-clear-description">{description}</p>
          </div>
        </header>
        <footer className="close-dialog-actions">
          <button type="button" className="close-dialog-button" onClick={onCancel}>取消</button>
          <button ref={confirmRef} type="button" className="close-dialog-button discard" onClick={onConfirm}>
            <Trash2 size={15} aria-hidden="true" />继续清除
          </button>
        </footer>
      </section>
    </div>,
    window.document.body,
  );
}

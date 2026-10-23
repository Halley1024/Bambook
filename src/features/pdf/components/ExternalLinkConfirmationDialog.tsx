import { useEffect, useRef } from "react";
import { createPortal } from "react-dom";
import { ExternalLink, Link2 } from "lucide-react";

export function ExternalLinkConfirmationDialog({ uri, onCancel, onConfirm }: {
  uri: string;
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

  return createPortal(<div className="close-dialog-backdrop" onMouseDown={onCancel}>
    <section className="close-dialog external-link-dialog" role="alertdialog" aria-modal="true"
      aria-labelledby="external-link-title" aria-describedby="external-link-description"
      onMouseDown={(event) => event.stopPropagation()}>
      <header className="close-dialog-header">
        <span className="close-dialog-icon" aria-hidden="true"><ExternalLink size={20} /></span>
        <div><h2 id="external-link-title">打开外部链接</h2>
          <p id="external-link-description">该链接将使用系统默认应用打开，是否继续？</p></div>
      </header>
      <div className="close-dialog-files external-link-address"><div className="close-dialog-file">
        <Link2 size={15} aria-hidden="true" /><span title={uri}>{uri}</span>
      </div></div>
      <footer className="close-dialog-actions">
        <button type="button" className="close-dialog-button" onClick={onCancel}>取消</button>
        <button ref={confirmRef} type="button" className="close-dialog-button primary" onClick={onConfirm}>
          <ExternalLink size={15} aria-hidden="true" />打开链接
        </button>
      </footer>
    </section>
  </div>, window.document.body);
}

import { useEffect, useRef } from "react";
import { AlertTriangle, FileText, Save } from "lucide-react";

export type CloseDialogDecision = "save" | "discard" | "cancel";

type CloseConfirmationDialogProps = {
  title: string;
  description: string;
  fileNames: string[];
  saveLabel: string;
  onDecision: (decision: CloseDialogDecision) => void;
};

export function CloseConfirmationDialog({
  title,
  description,
  fileNames,
  saveLabel,
  onDecision,
}: CloseConfirmationDialogProps) {
  const saveButtonRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    saveButtonRef.current?.focus();
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      event.preventDefault();
      onDecision("cancel");
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [onDecision]);

  return (
    <div className="close-dialog-backdrop" onMouseDown={() => onDecision("cancel")}>
      <section
        className="close-dialog"
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="close-dialog-title"
        aria-describedby="close-dialog-description"
        onMouseDown={(event) => event.stopPropagation()}
      >
        <header className="close-dialog-header">
          <span className="close-dialog-icon" aria-hidden="true"><AlertTriangle size={20} /></span>
          <div>
            <h2 id="close-dialog-title">{title}</h2>
            <p id="close-dialog-description">{description}</p>
          </div>
        </header>

        <div className="close-dialog-files" aria-label="未保存的文件">
          {fileNames.map((fileName, index) => (
            <div className="close-dialog-file" key={`${fileName}-${index}`}>
              <FileText size={15} aria-hidden="true" />
              <span title={fileName}>{fileName}</span>
            </div>
          ))}
        </div>

        <footer className="close-dialog-actions">
          <button type="button" className="close-dialog-button" data-tooltip="取消：返回并继续编辑文档" onClick={() => onDecision("cancel")}>
            取消
          </button>
          <button type="button" className="close-dialog-button discard" data-tooltip="不保存：放弃未保存的修改" onClick={() => onDecision("discard")}>
            不保存
          </button>
          <button
            ref={saveButtonRef}
            type="button"
            className="close-dialog-button primary"
            data-tooltip={`${saveLabel}：保存修改后继续关闭`}
            onClick={() => onDecision("save")}
          >
            <Save size={15} aria-hidden="true" />
            {saveLabel}
          </button>
        </footer>
      </section>
    </div>
  );
}

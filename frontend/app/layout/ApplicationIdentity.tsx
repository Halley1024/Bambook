import { BookOpen, FileText, Library } from "lucide-react";
import { useDocumentWorkspace } from "../../features/documents";

export function ApplicationIdentity() {
  const { activeDocument } = useDocumentWorkspace();
  const Icon = activeDocument?.kind === "pdf" ? BookOpen : activeDocument?.kind === "markdown" ? FileText : Library;
  const label = activeDocument?.kind === "pdf" ? "PDF 阅读" : activeDocument?.kind === "markdown" ? "Markdown 编辑" : "Light Reader";
  return (
    <div className="application-identity" aria-label={label}>
      <span className="application-identity-icon"><Icon size={18} /></span>
      <span>{label}</span>
    </div>
  );
}

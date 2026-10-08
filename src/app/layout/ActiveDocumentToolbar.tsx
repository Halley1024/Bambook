import { useDocumentWorkspace } from "../../features/documents";
import { MarkdownToolbar } from "../../features/markdown";
import { PdfToolbar } from "../../features/pdf";

export function ActiveDocumentToolbar() {
  const { activeDocument } = useDocumentWorkspace();
  if (!activeDocument) return <div className="empty-document-toolbar">打开文件后显示文档工具</div>;
  return activeDocument.kind === "pdf" ? <PdfToolbar /> : <MarkdownToolbar />;
}

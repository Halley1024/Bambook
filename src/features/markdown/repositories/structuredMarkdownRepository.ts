import { invokeCommand } from "../../../platform/tauriClient";
import type { MarkdownEdit, StructuredMarkdownDocument } from "../types/markdownAst";

export const structuredMarkdownRepository = {
  open(path: string) {
    return invokeCommand<StructuredMarkdownDocument>("open_markdown_document", { path });
  },
  get(sessionId: string) {
    return invokeCommand<StructuredMarkdownDocument>("get_markdown_structure", { sessionId });
  },
  applyEdits(sessionId: string, baseRevision: number, edits: MarkdownEdit[]) {
    return invokeCommand<StructuredMarkdownDocument>("apply_markdown_edits", { sessionId, baseRevision, edits });
  },
  save(sessionId: string, revision: number) {
    return invokeCommand<void>("save_markdown_document", { sessionId, revision });
  },
  saveAs(sessionId: string, revision: number, path: string) {
    return invokeCommand<StructuredMarkdownDocument>("save_markdown_document_as", { sessionId, revision, path });
  },
  exportHtml(sessionId: string, revision: number, path: string) {
    return invokeCommand<void>("export_markdown_html", { sessionId, revision, path });
  },
  exportPdf(sessionId: string, revision: number, path: string) {
    return invokeCommand<void>("export_markdown_pdf", { sessionId, revision, path });
  },
  close(sessionId: string) {
    return invokeCommand<void>("close_markdown_document", { sessionId });
  },
};

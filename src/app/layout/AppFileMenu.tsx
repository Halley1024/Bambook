import { WorkspaceFileMenu, useDocumentWorkspace } from "../../features/documents";
import { MarkdownFileMenu } from "../../features/markdown";
import { PdfFileMenu } from "../../features/pdf";

export function AppFileMenu() {
  const { activeDocument } = useDocumentWorkspace();
  return !activeDocument
        ? <WorkspaceFileMenu />
        : activeDocument.kind === "pdf" ? <PdfFileMenu /> : <MarkdownFileMenu />;
}

import { HelpMenu } from "../../features/help";
import { WorkspaceFileMenu, useDocumentWorkspace } from "../../features/documents";
import { MarkdownFileMenu } from "../../features/markdown";
import { PdfFileMenu } from "../../features/pdf";
import { SettingsMenu } from "../../features/settings";

export function AppMenuActions() {
  const { activeDocument } = useDocumentWorkspace();
  return (
    <nav className="toolbar-group toolbar-file-actions" aria-label="文件、设置与帮助">
      {!activeDocument
        ? <WorkspaceFileMenu />
        : activeDocument.kind === "pdf" ? <PdfFileMenu /> : <MarkdownFileMenu />}
      <SettingsMenu />
      <HelpMenu />
    </nav>
  );
}

import { useEffect, useRef, useState, type CSSProperties } from "react";
import { StartWorkspace, useDocumentWorkspace } from "../../features/documents";
import { WorkspaceNavigation, type LibrarySection } from "../../features/documents/components/WorkspaceNavigation";
import { MarkdownWorkspace } from "../../features/markdown";
import { PdfWorkspace } from "../../features/pdf";
import { useSettings } from "../../features/settings";
import { TopToolbar } from "./TopToolbar";

export function WorkspaceHost() {
  const workspace = useDocumentWorkspace();
  const { settings, ready } = useSettings();
  const [section, setSection] = useState<LibrarySection>("all");
  const [navigationRatio, setNavigationRatio] = useState<number | null>(null);
  const shellRef = useRef<HTMLElement>(null);
  const { activeDocument } = workspace;
  const kind = activeDocument?.kind ?? "pdf";
  const leftWidth = (kind === "pdf" ? settings.pdfWorkspaceLayout : settings.markdownWorkspaceLayout).leftWidth;
  useEffect(() => {
    if (!ready || navigationRatio !== null || !shellRef.current) return;
    const shell = shellRef.current;
    const navigation = shell.querySelector(".workspace-navigation");
    if (!navigation || !shell.clientWidth) return;
    setNavigationRatio(navigation.getBoundingClientRect().width / shell.clientWidth * 100);
  }, [ready, navigationRatio]);
  const style = { "--navigation-width": navigationRatio === null ? `${leftWidth}px` : `${navigationRatio}%` } as CSSProperties;
  return <section ref={shellRef} className="document-workspace-host workspace-shell" style={style}>
    <WorkspaceNavigation section={activeDocument ? null : section} onNavigate={(next) => { setSection(next); workspace.showHome(); }} />
    <div className="workspace-document-area">
      <TopToolbar />
      {!activeDocument ? <StartWorkspace section={section} /> : activeDocument.kind === "pdf" ? <PdfWorkspace /> : <MarkdownWorkspace />}
    </div>
  </section>;
}

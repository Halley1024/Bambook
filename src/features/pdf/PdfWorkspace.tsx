import { useEffect, type CSSProperties, type ReactNode } from "react";
import { AnnotationPanel } from "../annotations";
import { PdfReader } from "./components/PdfReader";
import { PdfSidebar } from "./components/PdfSidebar";
import { usePdfWorkspace } from "./hooks/usePdfWorkspace";
import { PdfSessionProvider } from "./state/PdfSessionContext";
import { useWorkspacePanelWidths } from "../../shared/useWorkspacePanelWidths";
import { useSettings } from "../settings";

type PdfWorkspaceProps = {
  documentTabs: ReactNode;
};

export function PdfWorkspace({ documentTabs }: PdfWorkspaceProps) {
  const workspace = usePdfWorkspace();
  const { settings, updateWorkspaceLayout } = useSettings();
  const panels = useWorkspacePanelWidths(
    settings.pdfWorkspaceLayout,
    (layout) => updateWorkspaceLayout("pdf", layout),
  );
  const panelStyle = {
    "--workspace-left-width": `${panels.leftWidth}px`,
    "--workspace-right-width": `${panels.rightWidth}px`,
  } as CSSProperties;

  useEffect(() => {
    const blockContextMenuOutsideDocument = (event: MouseEvent) => {
      const target = event.target instanceof Element ? event.target : null;
      if (target?.closest(".pdf-paper, .document-tab, .tab-context-menu")) return;
      event.preventDefault();
      event.stopPropagation();
    };
    window.addEventListener("contextmenu", blockContextMenuOutsideDocument, true);
    return () => window.removeEventListener("contextmenu", blockContextMenuOutsideDocument, true);
  }, []);

  const content = <section className="workspace-content">
      <div className="document-bar">{documentTabs}</div>
      <section className="reader-area">
        <PdfReader
          scale={workspace.scale}
          zoomMode={workspace.zoomMode}
          interactionMode={workspace.interactionMode}
          activeMarkupType={workspace.activeMarkupType}
          annotationColor={workspace.activeAnnotationColor}
          annotations={workspace.annotations}
          bookmarks={workspace.bookmarks}
          navigationTarget={workspace.navigationTarget}
          selectedAnnotationId={workspace.selectedAnnotationId}
          onCurrentPageChange={workspace.setCurrentPage}
          onFitScaleResolved={workspace.resolveFitScale}
          onScaleChange={workspace.setScale}
          onSelectionCapture={workspace.captureSelection}
          onClearSelection={workspace.clearSelection}
          onApplyMarkup={workspace.applyMarkup}
          onAnnotationSelect={workspace.selectAnnotation}
          onAnnotationColor={workspace.setAnnotationColor}
          onAnnotationColorPreview={workspace.previewAnnotationColor}
          onAnnotationConvert={workspace.convertAnnotation}
          onAnnotationDelete={workspace.deleteAnnotation}
          onBookmarkAdd={workspace.addBookmark}
          onBookmarkNavigate={workspace.navigateToBookmark}
          onBookmarkUpdate={workspace.updateBookmark}
          onBookmarkDelete={workspace.deleteBookmark}
          movingBookmarkId={workspace.movingBookmarkId}
          onBeginBookmarkMove={workspace.beginBookmarkMove}
          transformingAreaId={workspace.transformingAreaId}
          onBeginAreaTransform={workspace.beginAreaTransform}
          onAreaCreate={workspace.addAreaAnnotation}
          onAnnotationUpdate={workspace.updateAnnotation}
          onLinkNavigate={workspace.navigateToLink}
          onExternalLinkOpen={workspace.openExternalLink}
        />
      </section>
    </section>;

  return (
    <PdfSessionProvider document={workspace.activeDocument}>
      <main
        className={`workspace ${workspace.sidebarCollapsed ? "sidebar-collapsed" : ""} ${
          workspace.annotationPanelCollapsed ? "annotation-collapsed" : ""
        } ${panels.resizing ? "resizing-panels" : ""}`}
        style={panelStyle}
      >
        <PdfSidebar
          currentPage={workspace.currentPage}
          bookmarks={workspace.bookmarks}
          onNavigatePage={workspace.navigateToPage}
          onNavigateOutline={workspace.navigateToOutline}
          collapsed={workspace.sidebarCollapsed}
          onToggleCollapsed={workspace.toggleSidebar}
          onResizeStart={panels.beginLeftResize}
        />

        {content}

        <AnnotationPanel
          annotations={workspace.annotations}
          onUpdate={workspace.updateAnnotation}
          onDelete={workspace.deleteAnnotation}
          onMerge={workspace.mergeAnnotations}
          selectedAnnotationId={workspace.selectedAnnotationId}
          onSelect={workspace.navigateToAnnotation}
          collapsed={workspace.annotationPanelCollapsed}
          onToggleCollapsed={workspace.toggleAnnotationPanel}
          onResizeStart={panels.beginRightResize}
        />
      </main>
    </PdfSessionProvider>
  );
}

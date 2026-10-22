import { useEffect, useState, type CSSProperties } from "react";
import { AssistantSidebar } from "../../components/ai/AssistantSidebar";
import { PdfReader } from "./components/PdfReader";
import { PdfSidebar } from "./components/PdfSidebar";
import { usePdfWorkspace } from "./hooks/usePdfWorkspace";
import { PdfSessionProvider } from "./state/PdfSessionContext";
import { useWorkspacePanelWidths } from "../../shared/useWorkspacePanelWidths";
import { useSettings } from "../settings";

export function PdfWorkspace() {
  const workspace = usePdfWorkspace();
  const { settings, updateWorkspaceLayout } = useSettings();
  const panels = useWorkspacePanelWidths(
    settings.pdfWorkspaceLayout,
    (layout) => updateWorkspaceLayout("pdf", layout),
    { onLeftCollapse: workspace.sidebarCollapsed ? undefined : workspace.toggleSidebar },
  );
  const [llmText, setLlmText] = useState("");
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
          onTranslateSelection={() => undefined}
          onAskLlmSelection={(selection) => {
            setLlmText(selection.text);
            if (workspace.annotationPanelCollapsed) workspace.toggleAnnotationPanel();
          }}
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

        <AssistantSidebar
          contextText={llmText}
          contextLabel="PDF 选中文本"
          emptyContextHint="在 PDF 中选中文字，右键选择“询问大模型”以带入上下文。"
          collapsed={workspace.annotationPanelCollapsed}
          onToggleCollapsed={workspace.toggleAnnotationPanel}
          onResizeStart={panels.beginRightResize}
        />
      </main>
    </PdfSessionProvider>
  );
}

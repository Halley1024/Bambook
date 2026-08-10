import { FolderWorkspace, StartWorkspace, UnifiedDocumentTabs, useDocumentWorkspace } from "../../features/documents";
import { MarkdownWorkspace } from "../../features/markdown";
import { PdfWorkspace } from "../../features/pdf";

export function WorkspaceHost() {
  const workspace = useDocumentWorkspace();
  const { activeDocument } = workspace;
  const documentTabs = <UnifiedDocumentTabs />;

  return (
    <section className="document-workspace-host">
      {!activeDocument
        ? (workspace.folderPath ? <FolderWorkspace /> : <StartWorkspace />)
        : activeDocument.kind === "pdf"
          ? <PdfWorkspace documentTabs={documentTabs} />
          : <MarkdownWorkspace documentTabs={documentTabs} />}
    </section>
  );
}

/* The retired multi-pane implementation is intentionally left below only until the
   current patch is compacted; it is excluded from compilation and runtime. */
/*

function WorkspaceChrome({ children }: { children: ReactNode }) {
  const documents = useDocumentWorkspace();
  const pdf = usePdfWorkspace();
  const markdown = useMarkdownWorkspace();
  const pdfActive = documents.activeDocument?.kind === "pdf";
  const panels = useWorkspacePanelWidths(280, pdfActive ? 340 : 310);
  const panelStyle = {
    "--workspace-left-width": `${panels.leftWidth}px`,
    "--workspace-right-width": `${panels.rightWidth}px`,
  } as CSSProperties;

  useEffect(() => {
    if (!pdfActive) return;
    const block = (event: MouseEvent) => {
      const target = event.target instanceof Element ? event.target : null;
      if (target?.closest(".pdf-paper, .document-tab, .tab-context-menu")) return;
      event.preventDefault();
      event.stopPropagation();
    };
    window.addEventListener("contextmenu", block, true);
    return () => window.removeEventListener("contextmenu", block, true);
  }, [pdfActive]);

  if (!documents.activeDocument) {
    return <main className="workspace editor-empty-chrome" style={panelStyle}>{children}</main>;
  }

  const className = pdfActive
    ? `workspace ${pdf.sidebarCollapsed ? "sidebar-collapsed" : ""} ${pdf.annotationPanelCollapsed ? "annotation-collapsed" : ""} ${panels.resizing ? "resizing-panels" : ""}`
    : `workspace markdown-workspace ${markdown.sidebarCollapsed ? "sidebar-collapsed" : ""} ${markdown.helpPanelCollapsed ? "help-collapsed" : ""} ${panels.resizing ? "resizing-panels" : ""}`;

  return <main className={className} style={panelStyle}>
    {pdfActive
      ? <PdfSessionProvider document={pdf.activeDocument}><PdfSidebar
          currentPage={pdf.currentPage}
          bookmarks={pdf.bookmarks}
          onNavigatePage={pdf.navigateToPage}
          onNavigateOutline={pdf.navigateToOutline}
          collapsed={pdf.sidebarCollapsed}
          onToggleCollapsed={pdf.toggleSidebar}
          onResizeStart={panels.beginLeftResize}
        /></PdfSessionProvider>
      : <MarkdownSidebar onResizeStart={panels.beginLeftResize} />}
    {children}
    {pdfActive
      ? <AnnotationPanel
          annotations={pdf.annotations}
          onUpdate={pdf.updateAnnotation}
          onDelete={pdf.deleteAnnotation}
          onMerge={pdf.mergeAnnotations}
          selectedAnnotationId={pdf.selectedAnnotationId}
          onSelect={pdf.navigateToAnnotation}
          collapsed={pdf.annotationPanelCollapsed}
          onToggleCollapsed={pdf.toggleAnnotationPanel}
          onResizeStart={panels.beginRightResize}
        />
      : <MarkdownInspectorPanel onResizeStart={panels.beginRightResize} />}
  </main>;
}

type EditorPaneStore = {
  pdfContexts: Record<string, PdfWorkspaceContextValue>;
  markdownContexts: Record<string, MarkdownWorkspaceContextValue>;
  pdfSnapshots: Record<string, PdfPaneViewState>;
  markdownSnapshots: Record<string, MarkdownPaneViewState>;
  restoredDocuments: Record<string, string | null>;
};

function EditorLayout({ node, store }: { node: EditorLayoutNode; store: EditorPaneStore }) {
  if (node.type === "group") return <EditorPane groupId={node.groupId} store={store} />;
  return <div className={`editor-split editor-split-${node.axis}`}>
    <EditorLayout node={node.first} store={store} />
    <EditorLayout node={node.second} store={store} />
  </div>;
}

function EditorPane({ groupId, store }: { groupId: string; store: EditorPaneStore }) {
  const documents = useDocumentWorkspace();
  const livePdfWorkspace = usePdfWorkspace();
  const liveMarkdownWorkspace = useMarkdownWorkspace();
  const group = documents.editorGroups.find((item) => item.id === groupId);
  const document = group?.activeDocumentId
    ? documents.documents.find((item) => item.id === group.activeDocumentId) ?? null
    : null;
  const focused = documents.focusedGroupId === groupId;
  const cacheKey = document ? `${groupId}::${document.id}` : null;

  if (cacheKey && focused && document?.kind === "pdf" && livePdfWorkspace.activeDocumentId === document.id) store.pdfContexts[cacheKey] = livePdfWorkspace;
  if (cacheKey && focused && document?.kind === "markdown" && liveMarkdownWorkspace.activeDocumentId === document.id) store.markdownContexts[cacheKey] = liveMarkdownWorkspace;

  useEffect(() => {
    if (!focused) {
      store.restoredDocuments[groupId] = null;
      return;
    }
    if (!document || !cacheKey || store.restoredDocuments[groupId] === document.id) return;
    if (document.kind === "pdf" && livePdfWorkspace.activeDocumentId === document.id) {
      const snapshot = store.pdfSnapshots[cacheKey];
      if (snapshot) livePdfWorkspace.restorePaneViewState(snapshot);
      store.restoredDocuments[groupId] = document.id;
    } else if (document.kind === "markdown" && liveMarkdownWorkspace.activeDocumentId === document.id) {
      const snapshot = store.markdownSnapshots[cacheKey];
      if (snapshot) liveMarkdownWorkspace.restorePaneViewState(snapshot);
      store.restoredDocuments[groupId] = document.id;
    }
  }, [cacheKey, document, focused, groupId, liveMarkdownWorkspace, livePdfWorkspace, store]);

  useEffect(() => () => {
    if (!document || !cacheKey) return;
    if (document.kind === "pdf") {
      const context = store.pdfContexts[cacheKey];
      if (context) store.pdfSnapshots[cacheKey] = context.capturePaneViewState();
    } else {
      const context = store.markdownContexts[cacheKey];
      if (context) store.markdownSnapshots[cacheKey] = context.capturePaneViewState();
    }
  }, [cacheKey, document?.id, focused, store]);

  const pdfWorkspace = document?.kind === "pdf" && cacheKey ? store.pdfContexts[cacheKey] ?? livePdfWorkspace : livePdfWorkspace;
  const markdownWorkspace = document?.kind === "markdown" && cacheKey ? store.markdownContexts[cacheKey] ?? liveMarkdownWorkspace : liveMarkdownWorkspace;

  return <section
    className={`editor-pane ${focused ? "focused" : "inactive"}`}
    data-editor-group-id={groupId}
    onPointerDownCapture={(event) => {
      if (focused) return;
      event.preventDefault();
      event.stopPropagation();
      documents.focusEditorGroup(groupId);
    }}
  >
    {document?.kind === "pdf" && pdfWorkspace.activeDocumentId === document.id
      ? <PdfWorkspaceContext.Provider value={pdfWorkspace}>
          <PdfWorkspace contentOnly documentTabs={<UnifiedDocumentTabs groupId={groupId} />} />
        </PdfWorkspaceContext.Provider>
      : document?.kind === "markdown" && markdownWorkspace.activeDocumentId === document.id
        ? <MarkdownWorkspaceContext.Provider value={markdownWorkspace}>
            <MarkdownWorkspace contentOnly documentTabs={<UnifiedDocumentTabs groupId={groupId} />} />
          </MarkdownWorkspaceContext.Provider>
        : <EmptyEditorGroup group={group} />}
  </section>;
}

function EmptyEditorGroup({ group }: { group?: EditorGroup }) {
  return <section className="workspace-content editor-group-empty">
    <div className="document-bar"><UnifiedDocumentTabs groupId={group?.id} /></div>
    <div className="reader-placeholder">将标签移动到此窗格，或从其他窗格拆分文档。</div>
  </section>;
}
*/

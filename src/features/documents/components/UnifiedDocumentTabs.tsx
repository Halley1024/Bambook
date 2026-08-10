import { Tabs } from "../../../components/navigation/Tabs";
import { useDocumentWorkspace } from "../hooks/useDocumentWorkspace";

export function UnifiedDocumentTabs() {
  const workspace = useDocumentWorkspace();
  return (
    <Tabs
      items={workspace.documents.map((document) => ({
        id: document.id,
        label: document.title,
        description: document.kind === "pdf" ? document.sourcePath : (document.path ?? document.title),
        kind: document.kind,
        dirty: workspace.isDocumentDirty?.(document.id) ?? false,
      }))}
      activeId={workspace.activeDocumentId}
      emptyText="尚未打开文档"
      ariaLabel="已打开的文档"
      onActivate={workspace.activateDocument}
      onClose={(id) => void workspace.closeDocument(id)}
      onCloseOthers={(id) => void workspace.closeOtherDocuments(id)}
      onCloseAll={() => void workspace.closeAllDocuments()}
      onCopyLabel={workspace.copyDocumentLabel}
      onCopyRelativePath={workspace.copyRelativeDocumentPath}
      onCopyAbsolutePath={workspace.copyAbsoluteDocumentPath}
      onReorder={workspace.reorderDocuments}
    />
  );
}

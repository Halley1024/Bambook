import { useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { useDocumentWorkspace, type MarkdownDocument } from "../documents";
import { useNotification } from "../notifications";
import { selectDirectoryPath, selectSavePath } from "../../platform/fileDialog";
import { message } from "@tauri-apps/plugin-dialog";
import { markdownRepository } from "./repositories/markdownRepository";
import { structuredMarkdownRepository } from "./repositories/structuredMarkdownRepository";
import { clearUntitledRecovery, loadUntitledRecovery, saveUntitledRecovery } from "../documents/repositories/documentRepository";
import { applyMarkdownCommand } from "./services/markdownCommands";
import { MarkdownWorkspaceContext } from "./state/MarkdownWorkspaceContext";
import type { MarkdownNode, MarkdownOutlineNode, StructuredMarkdownDocument } from "./types/markdownAst";
import type { EditorSelection, EditorSelectionRequest, MarkdownCommand, MarkdownFileEntry, MarkdownOutlineItem, MarkdownOutlineNavigationRequest, MarkdownViewMode } from "./types/markdownWorkspace";
import { documentLibraryRepository, notifyDocumentLibraryChanged } from "../documents/repositories/documentLibraryRepository";
import { applyRelinkedDocument, ensureDocumentSource } from "../documents/repositories/documentRelink";
import { useSettings } from "../settings";

const emptySelection: EditorSelection = { start: 0, end: 0 };
type HistoryEntry = { content: string; selection: EditorSelection };
export function MarkdownWorkspaceProvider({ children }: { children: ReactNode }) {
  const selectionRequestIdRef = useRef(0);
  const structuresRef = useRef<Record<string, StructuredMarkdownDocument>>({});
  const syncChainsRef = useRef<Record<string, Promise<void>>>({});
  const syncTimersRef = useRef<Record<string, number>>({});
  const undoStacksRef = useRef<Record<string, HistoryEntry[]>>({});
  const redoStacksRef = useRef<Record<string, HistoryEntry[]>>({});
  const documentWorkspace = useDocumentWorkspace();
  const documents = useMemo(
    () => documentWorkspace.documents.filter(
      (document): document is MarkdownDocument => document.kind === "markdown",
    ),
    [documentWorkspace.documents],
  );
  const activeDocument = documentWorkspace.activeDocument?.kind === "markdown"
    ? documentWorkspace.activeDocument
    : null;
  const activeDocumentId = activeDocument?.id ?? null;
  const notification = useNotification();
  const { persistSettings } = useSettings();
  const [buffers, setBuffers] = useState<Record<string, string>>({});
  const [savedBuffers, setSavedBuffers] = useState<Record<string, string>>({});
  const [structures, setStructures] = useState<Record<string, StructuredMarkdownDocument>>({});
  const [selections, setSelections] = useState<Record<string, EditorSelection>>({});
  const [selectionRequest, setSelectionRequest] = useState<EditorSelectionRequest | null>(null);
  const [outlineNavigationRequest, setOutlineNavigationRequest] = useState<MarkdownOutlineNavigationRequest | null>(null);
  const [viewMode, setViewMode] = useState<MarkdownViewMode>("realtime");
  const [fileTree, setFileTree] = useState<MarkdownFileEntry[]>([]);
  const [fileTreeLoading, setFileTreeLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
  const [helpPanelCollapsed, setHelpPanelCollapsed] = useState(true);
  const [, setHistoryRevision] = useState(0);

  const activeStructure = activeDocument ? structures[activeDocument.id] ?? toStructure(activeDocument) : null;
  const content = activeDocument ? buffers[activeDocument.id] ?? activeDocument.markdown : "";
  const savedContent = activeDocument ? savedBuffers[activeDocument.id] ?? activeDocument.markdown : "";
  const dirty = Boolean(activeDocument && (content !== savedContent || !activeDocument.path));

  useEffect(() => {
    if (activeDocument) documentWorkspace.setDocumentDirty(activeDocument.id, dirty);
  }, [activeDocument, dirty, documentWorkspace.setDocumentDirty]);

  useEffect(() => {
    if (!documentWorkspace.saveActiveRequest.revision || !activeDocument
      || documentWorkspace.saveActiveRequest.documentId !== activeDocument.id) return;
    void saveActiveDocument();
  }, [documentWorkspace.saveActiveRequest.revision]);

  useEffect(() => {
    if (!documentWorkspace.saveAsActiveRequest || !activeDocument) return;
    void saveActiveDocumentAs();
  }, [documentWorkspace.saveAsActiveRequest]);

  useEffect(() => {
    let cancelled = false;
    void loadUntitledRecovery().then(async (recovery) => {
      if (cancelled || !recovery) return;
      void clearUntitledRecovery();
      const decision = await message("检测到上次未保存的 Markdown 文档，是否恢复？", {
        title: "恢复未保存文档",
        kind: "warning",
        buttons: { yes: "恢复", no: "丢弃", cancel: "取消" } as const,
      });
      if (cancelled || decision === "取消") return;
      if (decision === "恢复") {
        try {
          const document = await documentWorkspace.createMarkdownAndReturn();
          if (cancelled || !document) return;
          setBuffers((current) => ({ ...current, [document.id]: recovery }));
          setSavedBuffers((current) => ({ ...current, [document.id]: "" }));
        } catch (error) {
          notification.error(`恢复文档失败：${String(error)}`);
        }
      }
    }).catch(() => {});
    return () => { cancelled = true; };
  }, []);

  const lastUntitledContentRef = useRef<string>("");
  useEffect(() => {
    if (!activeDocument || activeDocument.path || !content) {
      if (lastUntitledContentRef.current) {
        void clearUntitledRecovery();
        lastUntitledContentRef.current = "";
      }
      return;
    }
    if (content === lastUntitledContentRef.current) return;
    lastUntitledContentRef.current = content;
    const timer = window.setTimeout(() => {
      void saveUntitledRecovery(content).catch(() => {});
    }, 500);
    return () => window.clearTimeout(timer);
  }, [activeDocument, content]);

  const outline = useMemo(() => buildOutline(activeStructure?.nodes ?? [], activeStructure?.outline ?? []), [activeStructure]);
  const cursorOffset = activeDocumentId ? selections[activeDocumentId]?.start ?? 0 : 0;
  const selection = activeDocumentId ? selections[activeDocumentId] ?? emptySelection : emptySelection;
  const canUndo = Boolean(activeDocumentId && undoStacksRef.current[activeDocumentId]?.length);
  const canRedo = Boolean(activeDocumentId && redoStacksRef.current[activeDocumentId]?.length);
  const activeOutlineId = [...outline].reverse().find((item) => item.offset <= cursorOffset)?.id ?? null;

  useEffect(() => {
    setSelectionRequest(null);
    setOutlineNavigationRequest(null);
  }, [activeDocumentId]);

  useEffect(() => {
    documents.forEach((document) => {
      setBuffers((current) => document.id in current ? current : { ...current, [document.id]: document.markdown });
      setSavedBuffers((current) => document.id in current ? current : { ...current, [document.id]: document.markdown });
      setSelections((current) => document.id in current ? current : { ...current, [document.id]: emptySelection });
      if (!(document.id in undoStacksRef.current)) undoStacksRef.current[document.id] = [];
      if (!(document.id in redoStacksRef.current)) redoStacksRef.current[document.id] = [];
      if (!(document.id in structuresRef.current)) commitStructure(document.id, toStructure(document));
    });
  }, [documents]);

  useEffect(() => {
    const document = activeDocument;
    if (!document) { setFileTree([]); return; }
    if (!document.path) { setFileTree([]); setFileTreeLoading(false); return; }
    let cancelled = false;
    setFileTreeLoading(true);
    markdownRepository.listDirectory(document.path)
      .then((entries) => { if (!cancelled) setFileTree(entries); })
      .catch((error) => { if (!cancelled) { setFileTree([]); notification.error(`读取 Markdown 文件目录失败：${String(error)}`); } })
      .finally(() => { if (!cancelled) setFileTreeLoading(false); });
    return () => { cancelled = true; };
  }, [activeDocument?.path, notification.error]);

  function commitStructure(id: string, structure: StructuredMarkdownDocument) {
    structuresRef.current = { ...structuresRef.current, [id]: structure };
    setStructures(structuresRef.current);
  }

  async function syncDocument(document: MarkdownDocument, target: string) {
    const previous = syncChainsRef.current[document.id] ?? Promise.resolve();
    const next = previous.catch(() => undefined).then(async () => {
      const current = structuresRef.current[document.id] ?? toStructure(document);
      if (current.source === target) return;
      const parsed = await structuredMarkdownRepository.applyEdits(current.sessionId, current.revision, [{
        startUtf16: 0, endUtf16: current.source.length, text: target,
      }]);
      commitStructure(document.id, parsed);
    });
    syncChainsRef.current[document.id] = next;
    await next;
    return structuresRef.current[document.id] ?? toStructure(document);
  }

  async function openFileFromPath(path: string) {
    await documentWorkspace.openPath(path);
  }

  async function saveActiveDocument() {
    const document = activeDocument;
    if (!document) { notification.warning("请先打开 Markdown 文档。"); return; }
    if (!document.path) {
      await saveDocumentAs(document, true);
      return;
    }
    if (!await ensureSourceBeforeSave(document)) return;
    setSaving(true);
    try {
      window.clearTimeout(syncTimersRef.current[document.id]);
      const latest = buffers[document.id] ?? document.markdown;
      const current = await syncDocument(document, latest);
      await structuredMarkdownRepository.save(current.sessionId, current.revision);
      notifyDocumentLibraryChanged();
      setSavedBuffers((buffers) => ({ ...buffers, [document.id]: current.source }));
      notification.success(`已保存：${document.title}`);
    } catch (error) { notification.error(`保存 Markdown 失败：${String(error)}`); }
    finally { setSaving(false); }
  }

  async function saveActiveDocumentAs() {
    const document = activeDocument;
    if (!document) { notification.warning("请先打开 Markdown 文档。"); return; }
    if (!document.path) { await saveDocumentAs(document, true); return; }
    if (!await ensureSourceBeforeSave(document)) return;
    const destination = await selectDirectoryPath();
    if (!destination) return;
    setSaving(true);
    try {
      window.clearTimeout(syncTimersRef.current[document.id]);
      const latest = buffers[document.id] ?? document.markdown;
      const current = await syncDocument(document, latest);
      await structuredMarkdownRepository.save(current.sessionId, current.revision);
      notifyDocumentLibraryChanged();
      const packagePath = await documentLibraryRepository.exportPackage(document.id, destination);
      setSavedBuffers((items) => ({ ...items, [document.id]: current.source }));
      notification.success(`文档包已保存到：${packagePath}`);
    } catch (error) { notification.error(`另存 Markdown 文档包失败：${String(error)}`); }
    finally { setSaving(false); }
  }

  async function saveDocumentAs(document: MarkdownDocument, notify: boolean) {
    const selectedPath = await selectSavePath({
      title: "Markdown 另存为",
      defaultPath: withExtension(document.title, "md"),
      filters: [{ name: "Markdown", extensions: ["md"] }],
    });
    if (!selectedPath) return false;
    const path = ensureExtension(selectedPath, "md");
    return saveDocumentAsToPath(document, path, notify);
  }

  async function saveDocumentAsToPath(document: MarkdownDocument, path: string, notify: boolean) {
    setSaving(true);
    try {
      window.clearTimeout(syncTimersRef.current[document.id]);
      const latest = buffers[document.id] ?? document.markdown;
      const current = await syncDocument(document, latest);
      const saved = await structuredMarkdownRepository.saveAs(current.sessionId, current.revision, path);
      notifyDocumentLibraryChanged();
      commitStructure(document.id, saved);
      setBuffers((buffers) => ({ ...buffers, [document.id]: saved.source }));
      setSavedBuffers((buffers) => ({ ...buffers, [document.id]: saved.source }));
      documentWorkspace.replaceDocument({
        ...document,
        path: saved.path,
        title: saved.title,
        revision: saved.revision,
        markdown: saved.source,
        nodes: saved.nodes,
        outline: saved.outline,
        diagnostics: saved.diagnostics,
      });
      if (notify) notification.success(`已另存为：${saved.path}`);
      return true;
    } catch (error) { notification.error(`Markdown 另存为失败：${String(error)}`); return false; }
    finally { setSaving(false); }
  }

  async function saveDocumentBeforeClose(document: MarkdownDocument) {
    const latest = buffers[document.id] ?? document.markdown;
    window.clearTimeout(syncTimersRef.current[document.id]);
    const current = await syncDocument(document, latest);
    if (!document.path) return false;
    await structuredMarkdownRepository.save(current.sessionId, current.revision);
    notifyDocumentLibraryChanged();
    setSavedBuffers((values) => ({ ...values, [document.id]: current.source }));
    return true;
  }

  async function ensureSourceBeforeSave(document: MarkdownDocument) {
    const result = await ensureDocumentSource(document);
    if (result.status === "cancelled") return false;
    if (result.status === "relinked") {
      documentWorkspace.replaceDocument(applyRelinkedDocument(document, result.document));
    }
    return true;
  }

  async function exportDocument(format: "html" | "pdf") {
    const document = activeDocument;
    if (!document) { notification.warning("请先打开 Markdown 文档。"); return; }
    const label = format === "html" ? "HTML" : "PDF";
    const selectedPath = await selectSavePath({
      title: `导出 Markdown 为 ${label}`,
      defaultPath: withExtension(document.title, format),
      filters: [{ name: label, extensions: [format] }],
    });
    if (!selectedPath) return;
    const path = ensureExtension(selectedPath, format);
    try {
      window.clearTimeout(syncTimersRef.current[document.id]);
      const current = await syncDocument(document, content);
      if (format === "html") await structuredMarkdownRepository.exportHtml(current.sessionId, current.revision, path);
      else await structuredMarkdownRepository.exportPdf(current.sessionId, current.revision, path);
      notification.success(`已导出 ${label}：${path}`);
    } catch (error) { notification.error(`导出 ${label} 失败：${String(error)}`); }
  }

  function requiresCloseDecision(document: MarkdownDocument) {
    const latest = buffers[document.id] ?? document.markdown;
    const saved = savedBuffers[document.id] ?? document.markdown;
    return latest !== saved || !document.path;
  }

  useEffect(() => documentWorkspace.registerLifecycle("markdown", {
    requiresCloseDecision: (document) =>
      document.kind === "markdown" && requiresCloseDecision(document),
    prepareSave: async (document) => {
      if (document.kind !== "markdown") return { status: "ready" };
      if (document.path) {
        if (!await ensureSourceBeforeSave(document)) return { status: "cancelled" };
        return { status: "ready", commit: () => saveDocumentBeforeClose(document) };
      }
      const selectedPath = await selectSavePath({
        title: "保存 Markdown 文档",
        defaultPath: withExtension(document.title, "md"),
        filters: [{ name: "Markdown", extensions: ["md"] }],
      });
      if (!selectedPath) return { status: "cancelled" };
      const path = ensureExtension(selectedPath, "md");
      return { status: "ready", commit: () => saveDocumentAsToPath(document, path, false) };
    },
    close: async (document) => {
      if (document.kind !== "markdown") return;
      const latest = buffers[document.id] ?? document.markdown;
      window.clearTimeout(syncTimersRef.current[document.id]);
      await syncDocument(document, latest);
      await documentLibraryRepository.reconcileStorage(document.id);
      notifyDocumentLibraryChanged();
      await persistSettings();
      await structuredMarkdownRepository.close(document.sessionId);
    },
  }), [buffers, documentWorkspace.registerLifecycle, persistSettings, savedBuffers]);
  function commitContent(document: MarkdownDocument, nextContent: string) {
    setBuffers((current) => ({ ...current, [document.id]: nextContent }));
    window.clearTimeout(syncTimersRef.current[document.id]);
    syncTimersRef.current[document.id] = window.setTimeout(() => {
      void syncDocument(document, nextContent)
        .catch((error) => notification.error(`解析 Markdown 失败：${String(error)}`));
    }, 180);
  }
  function updateContent(nextContent: string) {
    const document = activeDocument;
    if (!document) return;
    if (nextContent === content) return;
    const undoStack = undoStacksRef.current[document.id] ?? [];
    undoStacksRef.current[document.id] = [...undoStack.slice(-99), { content, selection }];
    redoStacksRef.current[document.id] = [];
    setHistoryRevision((value) => value + 1);
    commitContent(document, nextContent);
  }
  function updateSelection(selection: EditorSelection) { if (activeDocumentId) setSelections((current) => ({ ...current, [activeDocumentId]: selection })); }
  function requestSelection(selection: EditorSelection) {
    updateSelection(selection); selectionRequestIdRef.current += 1;
    const requestId = selectionRequestIdRef.current;
    setSelectionRequest({ ...selection, requestId });
    return requestId;
  }
  function navigateToOutline(item: MarkdownOutlineItem) {
    const requestId = requestSelection({ start: item.offset, end: item.offset });
    setOutlineNavigationRequest({ nodeId: item.nodeId, offset: item.offset, requestId });
  }
  function executeCommand(command: MarkdownCommand, payload?: string) {
    if (!activeDocumentId) { notification.warning("请先打开 Markdown 文档。"); return; }
    const result = applyMarkdownCommand(content, selections[activeDocumentId] ?? emptySelection, command, payload);
    updateContent(result.content); requestSelection(result.selection);
  }
  function undo() {
    const document = activeDocument;
    if (!document) return;
    const stack = undoStacksRef.current[document.id] ?? [];
    const previous = stack[stack.length - 1];
    if (!previous) return;
    undoStacksRef.current[document.id] = stack.slice(0, -1);
    redoStacksRef.current[document.id] = [...(redoStacksRef.current[document.id] ?? []), { content, selection }];
    commitContent(document, previous.content);
    requestSelection(previous.selection);
    setHistoryRevision((value) => value + 1);
  }
  function redo() {
    const document = activeDocument;
    if (!document) return;
    const stack = redoStacksRef.current[document.id] ?? [];
    const next = stack[stack.length - 1];
    if (!next) return;
    redoStacksRef.current[document.id] = stack.slice(0, -1);
    undoStacksRef.current[document.id] = [...(undoStacksRef.current[document.id] ?? []), { content, selection }];
    commitContent(document, next.content);
    requestSelection(next.selection);
    setHistoryRevision((value) => value + 1);
  }

  return <MarkdownWorkspaceContext.Provider value={{
    documents, activeDocument, activeDocumentId,
    content, selection, revision: activeStructure?.revision ?? 0,
    nodes: activeStructure?.nodes ?? [], diagnostics: activeStructure?.diagnostics ?? [], dirty, saving, canUndo, canRedo,
    viewMode, fileTree, fileTreeLoading, outline, activeOutlineId, selectionRequest, outlineNavigationRequest,
    sidebarCollapsed, helpPanelCollapsed, openFile: documentWorkspace.openFile, openFileFromPath,
    saveActiveDocument, saveActiveDocumentAs, exportHtml: () => exportDocument("html"), exportPdf: () => exportDocument("pdf"),
    activateDocument: documentWorkspace.activateDocument,
    closeDocument: (id: string) => void documentWorkspace.closeDocument(id),
    closeOtherDocuments: (id: string) => void documentWorkspace.closeOtherDocuments(id),
    reorderDocuments: documentWorkspace.reorderDocuments,
    copyDocumentPath: documentWorkspace.copyDocumentPath,
    setViewMode, updateContent, updateSelection, executeCommand, undo, redo,
    navigateToOutline,
    navigateToSearchResult: (startUtf16, endUtf16) => requestSelection({ start: startUtf16, end: endUtf16 }),
    toggleSidebar: () => setSidebarCollapsed((value) => !value),
    toggleHelpPanel: () => setHelpPanelCollapsed((value) => !value),
  }}>{children}</MarkdownWorkspaceContext.Provider>;
}

function toStructure(document: MarkdownDocument): StructuredMarkdownDocument {
  return { documentId: document.id, sessionId: document.sessionId, revision: document.revision, path: document.path, title: document.title,
    source: document.markdown, nodes: document.nodes, outline: document.outline, diagnostics: document.diagnostics };
}

function withExtension(title: string, extension: string) {
  const base = title.replace(/\.[^.]+$/, "") || "untitled";
  return `${base}.${extension}`;
}

function ensureExtension(path: string, extension: string) {
  return path.toLocaleLowerCase().endsWith(`.${extension.toLocaleLowerCase()}`) ? path : `${path}.${extension}`;
}

function buildOutline(nodes: MarkdownNode[], outline: MarkdownOutlineNode[]): MarkdownOutlineItem[] {
  const offsets = new Map<string, number>();
  const visit = (items: MarkdownNode[]) => items.forEach((node) => {
    offsets.set(node.id, node.range.start);
    if ("children" in node) visit(node.children);
  });
  visit(nodes);
  const flatten = (items: MarkdownOutlineNode[]): MarkdownOutlineItem[] => items.flatMap((item) => [
    { id: item.id, nodeId: item.nodeId, title: item.title, level: item.level, offset: offsets.get(item.nodeId) ?? 0 },
    ...flatten(item.children),
  ]);
  return flatten(outline);
}

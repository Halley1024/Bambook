import { useCallback, useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { copyText } from "../../platform/clipboard";
import { selectDirectoryPath } from "../../platform/fileDialog";
import { useNotification } from "../notifications";
import { useSettings } from "../settings";
import { useWorkspaceTabs } from "./hooks/useWorkspaceTabs";
import { useDocumentHistory } from "./hooks/useDocumentHistory";
import {
  authenticatePdfDocument,
  closeReaderDocument,
  createUntitledMarkdownDocument,
  openAnyDocument,
  openDocumentPath,
  listSupportedDirectory,
} from "./repositories/documentRepository";
import { workspaceStateRepository } from "./repositories/workspaceStateRepository";
import {
  DocumentWorkspaceContext,
  type DocumentLifecycleController,
  type FileNavigationMode,
} from "./state/DocumentWorkspaceContext";
import {
  runCloseTransaction,
  type CloseCandidate,
  type ClosePreparation,
} from "./close/closeCoordinator";
import type { DocumentKind, FolderEntry, ReaderDocument } from "./types";
import {
  CloseConfirmationDialog,
  type CloseDialogDecision,
} from "./components/CloseConfirmationDialog";

type CloseDialogRequest = {
  title: string;
  description: string;
  fileNames: string[];
  saveLabel: string;
};

export function DocumentWorkspaceProvider({ children }: { children: ReactNode }) {
  const tabs = useWorkspaceTabs<ReaderDocument>();
  const notification = useNotification();
  const notifyError = notification.error;
  const history = useDocumentHistory();
  const { settings, ready: settingsReady } = useSettings();
  const lifecyclesRef = useRef<Partial<Record<DocumentKind, DocumentLifecycleController>>>({});
  const restorationStartedRef = useRef(false);
  const folderPathRef = useRef<string | undefined>();
  const [folderPath, setFolderPath] = useState<string | null>(null);
  const [folderEntries, setFolderEntries] = useState<FolderEntry[]>([]);
  const [folderLoading, setFolderLoading] = useState(false);
  const [fileNavigationMode, setFileNavigationMode] = useState<FileNavigationMode>("directory");
  const [workspaceReady, setWorkspaceReady] = useState(false);
  const [dirtyMap, setDirtyMap] = useState<Record<string, boolean>>({});
  const [saveRequest, setSaveRequest] = useState<{ revision: number; documentId: string | null }>({
    revision: 0,
    documentId: null,
  });
  const [saveAsRequestCounter, setSaveAsRequestCounter] = useState(0);
  const [closeDialog, setCloseDialog] = useState<CloseDialogRequest | null>(null);
  const closeDialogResolverRef = useRef<((decision: CloseDialogDecision) => void) | null>(null);

  const showCloseDialog = useCallback((request: CloseDialogRequest) => {
    if (closeDialogResolverRef.current) return Promise.resolve<CloseDialogDecision>("cancel");
    return new Promise<CloseDialogDecision>((resolve) => {
      closeDialogResolverRef.current = resolve;
      setCloseDialog(request);
    });
  }, []);

  const resolveCloseDialog = useCallback((decision: CloseDialogDecision) => {
    const resolve = closeDialogResolverRef.current;
    if (!resolve) return;
    closeDialogResolverRef.current = null;
    setCloseDialog(null);
    resolve(decision);
  }, []);

  useEffect(() => () => {
    closeDialogResolverRef.current?.("cancel");
    closeDialogResolverRef.current = null;
  }, []);

  const registerLifecycle = useCallback((kind: DocumentKind, controller: DocumentLifecycleController) => {
    lifecyclesRef.current[kind] = controller;
    return () => {
      if (lifecyclesRef.current[kind] === controller) delete lifecyclesRef.current[kind];
    };
  }, []);

  const prepareDocumentSave = useCallback(async (document: ReaderDocument) => {
    const controller = lifecyclesRef.current[document.kind];
    return controller?.prepareSave
      ? await controller.prepareSave(document)
      : { status: "ready" as const };
  }, []);

  const requiresCloseDecision = useCallback((document: ReaderDocument) => {
    return Boolean(dirtyMap[document.id])
      || (lifecyclesRef.current[document.kind]?.requiresCloseDecision?.(document) ?? false);
  }, [dirtyMap]);

  const releaseDocument = useCallback(async (document: ReaderDocument) => {
    const controller = lifecyclesRef.current[document.kind];
    if (controller) await controller.close(document);
    else await closeReaderDocument(document);
  }, []);

  const deactivateCurrent = useCallback(() => {
    const current = tabs.activeDocument;
    if (current) lifecyclesRef.current[current.kind]?.deactivate?.(current);
  }, [tabs.activeDocument]);

  const activateDocument = useCallback((id: string) => {
    if (id === tabs.activeId) return;
    deactivateCurrent();
    tabs.activate(id);
  }, [deactivateCurrent, tabs]);

  const prepareIndividualDocumentClose = useCallback(async (document: ReaderDocument): Promise<ClosePreparation> => {
    if (!requiresCloseDecision(document)) return { status: "ready" };
    activateDocument(document.id);
    await new Promise<void>((resolve) => window.requestAnimationFrame(() => resolve()));
    const decision = await showCloseDialog({
      title: "保存文档",
      description: "此文档包含未保存的更改。",
      fileNames: [document.title],
      saveLabel: "保存",
    });
    switch (decision) {
      case "cancel": return { status: "cancelled" };
      case "discard": return { status: "ready" };
      case "save": return prepareDocumentSave(document);
    }
  }, [activateDocument, prepareDocumentSave, requiresCloseDecision, showCloseDialog]);

  const registerOpenedDocument = useCallback(async (opened: ReaderDocument) => {
    const existing = opened.path ? tabs.documents.find((document) => document.path === opened.path) : undefined;
    if (existing) {
      await closeReaderDocument(opened);
      activateDocument(existing.id);
      return existing;
    }
    let document = opened;
    if (document.kind === "pdf" && document.needsPassword) {
      const password = window.prompt("该 PDF 需要密码：");
      if (password === null) {
        await closeReaderDocument(document);
        return null;
      }
      document = await authenticatePdfDocument(document, password);
    }
    deactivateCurrent();
    tabs.open(document);
    await history.recordAccessed(document);
    notification.success(`已打开：${document.title}`);
    return document;
  }, [activateDocument, deactivateCurrent, history, notification, tabs]);

  const openFile = useCallback(async () => {
    try {
      const opened = await openAnyDocument();
      if (opened) {
        setFileNavigationMode("directory");
        await registerOpenedDocument(opened);
      }
    } catch (error) {
      notification.error(`打开文档失败：${String(error)}`);
    }
  }, [notification, registerOpenedDocument]);

  const loadFolder = useCallback(async (path: string) => {
    setFolderLoading(true);
    try {
      const entries = await listSupportedDirectory(path);
      folderPathRef.current = path;
      setFolderPath(path);
      setFolderEntries(entries);
      return true;
    } catch (error) {
      notification.error(`读取文件夹失败：${String(error)}`);
      return false;
    } finally {
      setFolderLoading(false);
    }
  }, [notification]);

  const openFolder = useCallback(async () => {
    const path = await selectDirectoryPath();
    if (!path || !(await loadFolder(path))) return;
    setFileNavigationMode("directory");
    deactivateCurrent();
    tabs.deactivate();
  }, [deactivateCurrent, loadFolder, tabs]);

  const openPath = useCallback(async (path: string) => {
    const existing = tabs.documents.find((document) => document.path === path);
    if (existing) {
      activateDocument(existing.id);
      return true;
    }
    try {
      return Boolean(await registerOpenedDocument(await openDocumentPath(path)));
    } catch (error) {
      notification.error(`打开文档失败：${String(error)}`);
      return false;
    }
  }, [activateDocument, notification, registerOpenedDocument, tabs]);

  const createMarkdown = useCallback(async () => {
    try {
      setFileNavigationMode("directory");
      await registerOpenedDocument(await createUntitledMarkdownDocument());
    } catch (error) {
      notification.error(`新建 Markdown 文档失败：${String(error)}`);
    }
  }, [notification, registerOpenedDocument]);

  const createMarkdownAndReturn = useCallback(async () => {
    try {
      setFileNavigationMode("directory");
      return await registerOpenedDocument(await createUntitledMarkdownDocument());
    } catch (error) {
      notification.error(`新建 Markdown 文档失败：${String(error)}`);
      return null;
    }
  }, [notification, registerOpenedDocument]);

  const closeContext = useMemo(() => ({
    prepare: (candidate: CloseCandidate) => prepareIndividualDocumentClose(
      tabs.documents.find((item) => item.id === candidate.id) as ReaderDocument,
    ),
    release: (candidate: CloseCandidate) =>
      releaseDocument(tabs.documents.find((item) => item.id === candidate.id) as ReaderDocument),
    recordClosed: (candidate: CloseCandidate) =>
      history.recordClosed(tabs.documents.find((item) => item.id === candidate.id) as ReaderDocument),
  }), [history, prepareIndividualDocumentClose, releaseDocument, tabs.documents]);

  const closeDocument = useCallback(async (id: string) => {
    const document = tabs.documents.find((item) => item.id === id);
    if (!document) return;
    try {
      const result = await runCloseTransaction([document], closeContext);
      if (!result.completed) return;
      tabs.close(id);
    } catch (error) {
      notification.error(`关闭文档失败：${String(error)}`);
    }
  }, [closeContext, notification, tabs]);

  const closeOtherDocuments = useCallback(async (id: string) => {
    const documents = tabs.documents.filter((document) => document.id !== id);
    try {
      const result = await runCloseTransaction(documents, closeContext);
      if (!result.completed) return;
      tabs.closeOthers(id);
    } catch (error) {
      notification.error(`关闭文档失败：${String(error)}`);
    }
  }, [closeContext, notification, tabs]);

  const closeAllDocuments = useCallback(async () => {
    const documents = [...tabs.documents];
    if (!documents.length) return;
    try {
      const result = await runCloseTransaction(documents, closeContext);
      if (!result.completed) return;
      tabs.closeAll();
    } catch (error) {
      notification.error(`关闭所有文档失败：${String(error)}`);
    }
  }, [closeContext, notification, tabs]);

  const prepareWindowClose = useCallback(async () => {
    const documents = [...tabs.documents];
    try {
      const dirtyDocuments = documents.filter(requiresCloseDecision);
      const preparations = new Map<string, ClosePreparation>();
      if (dirtyDocuments.length > 0) {
        const decision = await showCloseDialog({
          title: "关闭 Bambook",
          description: "以下文件包含未保存的更改。是否全部保存？",
          fileNames: dirtyDocuments.map((document) => document.title),
          saveLabel: "全部保存",
        });
        switch (decision) {
          case "cancel": return false;
          case "discard":
            dirtyDocuments.forEach((document) => preparations.set(document.id, { status: "ready" }));
            break;
          case "save":
            for (const document of dirtyDocuments) {
              const preparation = await prepareDocumentSave(document);
              if (preparation.status === "cancelled") return false;
              preparations.set(document.id, preparation);
            }
            break;
        }
      }
      const result = await runCloseTransaction(documents, {
        ...closeContext,
        prepare: async (candidate) => preparations.get(candidate.id) ?? { status: "ready" },
      });
      return result.completed;
    } catch (error) {
      notification.error(`退出保存确认失败：${String(error)}`);
      return false;
    }
  }, [closeContext, notification, prepareDocumentSave, requiresCloseDecision, showCloseDialog, tabs.documents]);

  useEffect(() => {
    if (!settingsReady || restorationStartedRef.current) return;
    restorationStartedRef.current = true;
    if (settings.startupBehavior === "startPage") {
      setWorkspaceReady(true);
      return;
    }
    if (settings.startupBehavior === "newMarkdown") {
      void createUntitledMarkdownDocument()
        .then(registerOpenedDocument)
        .catch((error) => notifyError(`新建 Markdown 文档失败：${String(error)}`))
        .finally(() => setWorkspaceReady(true));
      return;
    }
    void workspaceStateRepository.load()
      .then(async (workspace) => {
        folderPathRef.current = workspace.folderPath;
        if (workspace.folderPath) {
          setFolderPath(workspace.folderPath);
          await loadFolder(workspace.folderPath);
        }
        let requestedActiveId: string | null = null;
        for (const persisted of workspace.openDocuments) {
          try {
            const opened = await registerOpenedDocument(await openDocumentPath(persisted.path));
            if (opened && persisted.path === workspace.activeDocumentPath) requestedActiveId = opened.id;
          } catch (error) {
            notifyError(`恢复文档失败：${persisted.path}：${String(error)}`);
          }
        }
        if (requestedActiveId) tabs.activate(requestedActiveId);
        else if (workspace.folderPath) tabs.deactivate();
      })
      .catch((error) => notifyError(`读取工作区状态失败：${String(error)}`))
      .finally(() => setWorkspaceReady(true));
  }, [settings.startupBehavior, settingsReady]);

  useEffect(() => {
    if (!workspaceReady) return;
    const timer = window.setTimeout(() => {
      void workspaceStateRepository.save({
        openDocuments: tabs.documents
          .filter((document) => Boolean(document.path))
          .map((document) => ({ path: document.path, kind: document.kind })),
        activeDocumentPath: tabs.activeDocument?.path || undefined,
        folderPath: folderPath ?? undefined,
      }).catch((error) => notifyError(`保存工作区状态失败：${String(error)}`));
    }, 180);
    return () => window.clearTimeout(timer);
  }, [notifyError, tabs.activeDocument?.path, tabs.documents, folderPath, workspaceReady]);

  const visibleDocumentPath = useCallback((id: string) => {
    const document = tabs.documents.find((item) => item.id === id);
    if (!document) return null;
    return document.kind === "pdf" ? document.sourcePath : document.path;
  }, [tabs.documents]);

  const copyValue = useCallback((value: string | null, successMessage: string) => {
    if (!value) {
      notification.warning("当前标签还没有可复制的文件路径。");
      return;
    }
    void copyText(value)
      .then(() => notification.success(successMessage))
      .catch((error) => notification.error(`复制失败：${String(error)}`));
  }, [notification]);

  const copyDocumentLabel = useCallback((id: string) => {
    const document = tabs.documents.find((item) => item.id === id);
    if (document) copyValue(document.title, `已复制标签：${document.title}`);
  }, [copyValue, tabs.documents]);

  const copyAbsoluteDocumentPath = useCallback((id: string) => {
    const path = visibleDocumentPath(id);
    copyValue(path, path ? `已复制绝对路径：${path}` : "");
  }, [copyValue, visibleDocumentPath]);

  const copyRelativeDocumentPath = useCallback((id: string) => {
    const path = visibleDocumentPath(id);
    if (!path) {
      copyValue(null, "");
      return;
    }
    const relativePath = toWorkspaceRelativePath(path, folderPath);
    copyValue(relativePath, `已复制相对路径：${relativePath}`);
  }, [copyValue, folderPath, visibleDocumentPath]);

  const setDocumentDirty = useCallback((id: string, dirty: boolean) => {
    setDirtyMap((current) => current[id] === dirty ? current : { ...current, [id]: dirty });
  }, []);

  const isDocumentDirty = useCallback((id: string) => dirtyMap[id] ?? false, [dirtyMap]);

  const requestSaveActive = useCallback((id: string) => {
    setSaveRequest((current) => ({ revision: current.revision + 1, documentId: id }));
  }, []);

  const requestSaveActiveAs = useCallback((_id: string) => {
    setSaveAsRequestCounter((c) => c + 1);
  }, []);

  return (
    <DocumentWorkspaceContext.Provider value={{
      documents: tabs.documents,
      activeDocument: tabs.activeDocument,
      activeDocumentId: tabs.activeId,
      folderPath,
      folderEntries,
      folderLoading,
      fileNavigationMode,
      setFileNavigationMode,
      openFile,
      openFolder,
      openPath,
      createMarkdown,
      activateDocument,
      closeDocument,
      closeOtherDocuments,
      closeAllDocuments,
      prepareWindowClose,
      replaceDocument: tabs.replace,
      reorderDocuments: tabs.reorder,
      copyDocumentLabel,
      copyRelativeDocumentPath,
      copyAbsoluteDocumentPath,
      copyDocumentPath: copyAbsoluteDocumentPath,
      registerLifecycle,
      setDocumentDirty,
      isDocumentDirty,
      createMarkdownAndReturn,
      requestSaveActive,
      requestSaveActiveAs,
      saveActiveRequest: saveRequest,
      saveAsActiveRequest: saveAsRequestCounter,
    }}>
      {children}
      {closeDialog && (
        <CloseConfirmationDialog
          {...closeDialog}
          onDecision={resolveCloseDialog}
        />
      )}
    </DocumentWorkspaceContext.Provider>
  );
}

function toWorkspaceRelativePath(path: string, folderPath: string | null) {
  const normalizedPath = path.replace(/\//g, "\\");
  const normalizedFolder = folderPath?.replace(/[\\/]+$/, "").replace(/\//g, "\\");
  if (normalizedFolder) {
    const prefix = `${normalizedFolder}\\`;
    if (normalizedPath.toLocaleLowerCase().startsWith(prefix.toLocaleLowerCase())) {
      return normalizedPath.slice(prefix.length);
    }
  }
  return normalizedPath.split("\\").filter(Boolean).pop() ?? normalizedPath;
}

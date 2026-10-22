import { ChevronDown, FileText, FolderTree, Library, Link2Off, RefreshCw } from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { createPortal } from "react-dom";
import { selectFilePath } from "../../../platform/fileDialog";
import { useNotification } from "../../notifications";
import { useDocumentWorkspace } from "../hooks/useDocumentWorkspace";
import { listSupportedDirectory } from "../repositories/documentRepository";
import {
  DOCUMENT_LIBRARY_CHANGED_EVENT,
  documentLibraryRepository,
  notifyDocumentLibraryChanged,
  type StoredDocument,
} from "../repositories/documentLibraryRepository";
import type { DocumentKind, FolderEntry, ReaderDocument } from "../types";
import { DocumentFolderTree } from "./DocumentFolderTree";

type RelinkConfirmation = {
  document: StoredDocument;
  selectedPath: string;
  recordedName: string;
  selectedName: string;
};

export function DocumentFileNavigator() {
  const workspace = useDocumentWorkspace();
  const notification = useNotification();
  const document = workspace.activeDocument;
  const untitled = Boolean(document?.kind === "markdown" && !document.path);
  const [library, setLibrary] = useState<StoredDocument[]>([]);
  const [libraryLoading, setLibraryLoading] = useState(false);
  const [directoryEntries, setDirectoryEntries] = useState<FolderEntry[]>([]);
  const [directoryLoading, setDirectoryLoading] = useState(false);
  const [expandedKinds, setExpandedKinds] = useState<Record<DocumentKind, boolean>>({ pdf: true, markdown: true });
  const [relinkingId, setRelinkingId] = useState<string | null>(null);
  const [confirmation, setConfirmation] = useState<RelinkConfirmation | null>(null);

  const loadLibrary = useCallback(async (quiet = false) => {
    if (!quiet) setLibraryLoading(true);
    try {
      const items = await documentLibraryRepository.load();
      setLibrary([...items].sort((left, right) => right.updatedAt - left.updatedAt || left.title.localeCompare(right.title, "zh-CN")));
    } catch (error) {
      if (!quiet) notification.error(`读取文档库失败：${String(error)}`);
    } finally {
      if (!quiet) setLibraryLoading(false);
    }
  }, [notification]);

  useEffect(() => {
    if (untitled) {
      setLibrary([]);
      setLibraryLoading(false);
      return;
    }
    void loadLibrary();
    const refresh = () => void loadLibrary(true);
    const timer = window.setInterval(refresh, 5000);
    window.addEventListener(DOCUMENT_LIBRARY_CHANGED_EVENT, refresh);
    return () => {
      window.clearInterval(timer);
      window.removeEventListener(DOCUMENT_LIBRARY_CHANGED_EVENT, refresh);
    };
  }, [loadLibrary, untitled]);

  const storedDocument = useMemo(() => document ? findStoredDocument(library, document) : undefined, [document, library]);
  const sourcePath = document ? (storedDocument?.sourcePath || visibleSourcePath(document)) : "";
  const directoryRoot = workspace.folderPath || parentDirectory(sourcePath);
  const sourceUnavailable = Boolean(storedDocument && !storedDocument.sourceAvailable && !workspace.folderPath);
  const requiresRelink = Boolean(sourceUnavailable && !storedDocument?.pathAvailable);

  useEffect(() => {
    if (document && requiresRelink) workspace.setDocumentDirty(document.id, true);
  }, [document, requiresRelink, workspace.setDocumentDirty]);

  useEffect(() => {
    if ((!document && !workspace.folderPath) || untitled || workspace.fileNavigationMode !== "directory" || workspace.folderPath || sourceUnavailable || !directoryRoot) {
      setDirectoryEntries([]);
      setDirectoryLoading(false);
      return;
    }
    let cancelled = false;
    setDirectoryLoading(true);
    void listSupportedDirectory(directoryRoot)
      .then((entries) => { if (!cancelled) setDirectoryEntries(entries); })
      .catch(() => { if (!cancelled) setDirectoryEntries([]); })
      .finally(() => { if (!cancelled) setDirectoryLoading(false); });
    return () => { cancelled = true; };
  }, [directoryRoot, document, sourceUnavailable, untitled, workspace.fileNavigationMode, workspace.folderPath]);

  if (!document && !workspace.folderPath) return <NavigatorEmpty message="打开文档后显示文件导航" />;
  if (untitled) return <NavigatorEmpty message="当前文档尚未保存，保存后可浏览所在目录或 Bambook 文档库。" />;

  const beginRelink = async (target: StoredDocument) => {
    if (relinkingId) return;
    setRelinkingId(target.id);
    try {
      const selectedPath = await selectFilePath([{
        name: target.kind === "pdf" ? "PDF 文档" : "Markdown 文档",
        extensions: target.kind === "pdf" ? ["pdf"] : ["md", "markdown", "mdown", "mkd"],
      }]);
      if (!selectedPath) return;
      const inspection = await documentLibraryRepository.inspectRelink(target.id, selectedPath);
      if (!inspection.nameMatches) {
        setConfirmation({ document: target, selectedPath, ...inspection });
        return;
      }
      await finishRelink(target, selectedPath);
    } catch (error) {
      notification.error(`重新关联文件失败：${String(error)}`);
    } finally {
      setRelinkingId(null);
    }
  };

  const finishRelink = async (target: StoredDocument, selectedPath: string) => {
    const updated = await documentLibraryRepository.relink(target.id, selectedPath);
    setLibrary((items) => items.map((item) => item.id === updated.id ? updated : item));
    const current = workspace.activeDocument;
    if (current && findStoredDocument([target], current)) {
      workspace.replaceDocument(current.kind === "pdf"
        ? { ...current, title: updated.title, path: updated.path, sourcePath: updated.sourcePath }
        : { ...current, title: updated.title, path: updated.path });
    }
    notifyDocumentLibraryChanged();
    workspace.setFileNavigationMode("directory");
    notification.success(`已重新关联：${updated.title}`);
  };

  const confirmRelink = async () => {
    if (!confirmation) return;
    const pending = confirmation;
    setConfirmation(null);
    setRelinkingId(pending.document.id);
    try {
      await finishRelink(pending.document, pending.selectedPath);
    } catch (error) {
      notification.error(`重新关联文件失败：${String(error)}`);
    } finally {
      setRelinkingId(null);
    }
  };

  return <div className="document-file-navigator">
    <div className="document-file-content">
      {workspace.fileNavigationMode === "directory" ? (
        sourceUnavailable && storedDocument ? (
          <InvalidSourceState document={storedDocument} relinking={relinkingId === storedDocument.id} onRelink={() => void beginRelink(storedDocument)} />
        ) : directoryLoading || workspace.folderLoading ? (
          <NavigatorEmpty message="正在读取文件目录…" />
        ) : (
          <DocumentFolderTree entries={workspace.folderPath ? workspace.folderEntries : directoryEntries}
            activePath={sourcePath} onOpenFile={(path) => void workspace.openPath(path)} />
        )
      ) : (
        <div className="document-library-groups">
          {libraryLoading && !library.length ? <NavigatorEmpty message="正在读取 Bambook 文档库…" /> : null}
          {!libraryLoading && !library.length ? <NavigatorEmpty message="文档库中还没有已保存的文档" /> : null}
          {(["pdf", "markdown"] as const).map((kind) => {
            const items = library.filter((item) => item.kind === kind);
            return <section className="document-library-group" key={kind}>
              <button type="button" className="document-library-group-heading"
                aria-expanded={expandedKinds[kind]}
                onClick={() => setExpandedKinds((value) => ({ ...value, [kind]: !value[kind] }))}>
                <ChevronDown size={15} className={expandedKinds[kind] ? "expanded" : ""} />
                <strong>{kind === "pdf" ? "PDF" : "Markdown"}</strong><span>{items.length}</span>
              </button>
              {expandedKinds[kind] && <div className="document-library-items">
                {items.length ? items.map((item) => <LibraryDocumentRow key={item.id} document={item}
                  active={storedDocument?.id === item.id} relinking={relinkingId === item.id}
                  onOpen={() => item.pathAvailable ? void workspace.openPath(item.path)
                    : item.sourceAvailable ? void workspace.openPath(item.sourcePath) : void beginRelink(item)}
                  onRelink={() => void beginRelink(item)} />) : <p>暂无文档</p>}
              </div>}
            </section>;
          })}
        </div>
      )}
    </div>

    <div className="document-file-viewbar">
      <div>
        <strong>{workspace.fileNavigationMode === "directory" ? "文件夹视图" : "Bambook 文档库"}</strong>
        <small title={workspace.fileNavigationMode === "directory" ? directoryRoot : "Bambook 托管文档"}>
          {workspace.fileNavigationMode === "directory" ? (directoryRoot || "来源目录") : `${library.length} 个文档`}
        </small>
      </div>
      <button type="button" className="document-file-view-toggle"
        data-tooltip={workspace.fileNavigationMode === "directory" ? "切换到文档库视图" : "切换到文件夹视图"}
        aria-label={workspace.fileNavigationMode === "directory" ? "切换到文档库视图" : "切换到文件夹视图"}
        onClick={() => workspace.setFileNavigationMode(workspace.fileNavigationMode === "directory" ? "library" : "directory")}>
        {workspace.fileNavigationMode === "directory" ? <Library size={17} /> : <FolderTree size={17} />}
      </button>
    </div>

    {confirmation && createPortal(<RelinkConfirmationDialog confirmation={confirmation}
      onCancel={() => setConfirmation(null)} onConfirm={() => void confirmRelink()} />, window.document.body)}
  </div>;
}

function LibraryDocumentRow({ document, active, relinking, onOpen, onRelink }: {
  document: StoredDocument; active: boolean; relinking: boolean; onOpen: () => void; onRelink: () => void;
}) {
  const linkedMissing = document.storageMode === "linked-file" && !document.pathAvailable;
  const sourceMissing = !document.sourceAvailable;
  return <div className={`document-library-row ${active ? "active" : ""} ${linkedMissing ? "invalid" : ""}`}>
    <button type="button" className="document-library-open" onClick={onOpen}
      title={`${document.title}\n${displayPath(document.path)}${sourceMissing ? "\n源文件位置已失效" : ""}`}>
      <FileText size={17} />
      <span><strong>{document.title}</strong><small>{displayPath(document.path)}</small></span>
    </button>
    {sourceMissing && <button type="button" className="document-library-relink" disabled={relinking}
      data-tooltip={linkedMissing ? "链接失效：重新定位文件" : "源文件已移动：重新定位源文件"}
      aria-label="重新定位源文件" onClick={onRelink}>
      {relinking ? <RefreshCw className="spinning" size={14} /> : <Link2Off size={14} />}
    </button>}
  </div>;
}

function InvalidSourceState({ document, relinking, onRelink }: { document: StoredDocument; relinking: boolean; onRelink: () => void }) {
  const linked = document.storageMode === "linked-file";
  return <div className="document-source-invalid">
    <span><Link2Off size={22} /></span>
    <strong>{linked ? "文件链接已失效" : "原始文件位置已发生变化"}</strong>
    <p>{linked ? "请重新定位文件后继续阅读或编辑。" : "托管副本仍可正常使用，请重新定位源文件以恢复目录导航。"}</p>
    <button type="button" disabled={relinking} onClick={onRelink}>{relinking ? "正在检索…" : "重新检索"}</button>
  </div>;
}

function RelinkConfirmationDialog({ confirmation, onCancel, onConfirm }: {
  confirmation: RelinkConfirmation; onCancel: () => void; onConfirm: () => void;
}) {
  return <div className="document-relink-backdrop" onMouseDown={onCancel}>
    <section className="document-relink-dialog" role="alertdialog" aria-modal="true" aria-labelledby="document-relink-title"
      onMouseDown={(event) => event.stopPropagation()}>
      <header><span><Link2Off size={19} /></span><div><h2 id="document-relink-title">文件名可能不一致</h2><p>确认是否重新建立来源映射。</p></div></header>
      <div className="document-relink-comparison">
        <div><small>Bambook 记录</small><strong title={confirmation.recordedName}>{confirmation.recordedName}</strong></div>
        <div><small>所选文件</small><strong title={confirmation.selectedName}>{confirmation.selectedName}</strong></div>
      </div>
      <p className="document-relink-warning">继续后将更新文档库和 metadata.json 中的源文件地址。</p>
      <footer><button type="button" onClick={onCancel}>取消</button><button type="button" className="primary" onClick={onConfirm}>继续</button></footer>
    </section>
  </div>;
}

function NavigatorEmpty({ message }: { message: string }) {
  return <div className="document-navigator-empty"><FolderTree size={22} /><span>{message}</span></div>;
}

function findStoredDocument(items: StoredDocument[], document: ReaderDocument) {
  const paths = document.kind === "pdf" ? [document.path, document.sourcePath] : [document.path];
  return items.find((item) => item.id === document.id || [item.path, item.sourcePath].some((path) => paths.some((candidate) => samePath(path, candidate))));
}

function visibleSourcePath(document: ReaderDocument) {
  return document.kind === "pdf" ? document.sourcePath : document.path;
}

function samePath(left: string, right: string) {
  return normalizePath(left).toLocaleLowerCase() === normalizePath(right).toLocaleLowerCase();
}

function normalizePath(path: string) {
  return path.replace(/^\\\\\?\\/, "").replace(/\//g, "\\").replace(/\\+$/, "");
}

function displayPath(path: string) {
  return path.replace(/^\\\\\?\\/, "");
}

function parentDirectory(path: string) {
  const normalized = normalizePath(path);
  const separator = normalized.lastIndexOf("\\");
  if (separator < 0) return "";
  return separator === 2 && normalized[1] === ":" ? normalized.slice(0, 3) : normalized.slice(0, separator);
}

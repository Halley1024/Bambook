import {
  BookOpen,
  Clock3,
  FilePlus2,
  FileText,
  FolderOpen,
  Library,
  Search,
} from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { message } from "@tauri-apps/plugin-dialog";
import { selectFilePath } from "../../../platform/fileDialog";
import { useNotification } from "../../notifications";
import { useDocumentHistory } from "../hooks/useDocumentHistory";
import { useDocumentWorkspace } from "../hooks/useDocumentWorkspace";
import {
  documentLibraryRepository,
  type StoredDocument,
} from "../repositories/documentLibraryRepository";
import type { RecentDocument } from "../repositories/documentHistoryRepository";
import type { DocumentKind } from "../types";

type LibraryFilter = "all" | DocumentKind;

export function StartWorkspace() {
  const workspace = useDocumentWorkspace();
  const history = useDocumentHistory();
  const notification = useNotification();
  const [documents, setDocuments] = useState<StoredDocument[]>([]);
  const [loadingLibrary, setLoadingLibrary] = useState(true);
  const [openingPath, setOpeningPath] = useState<string | null>(null);
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState<LibraryFilter>("all");

  useEffect(() => {
    let cancelled = false;
    setLoadingLibrary(true);
    void documentLibraryRepository.load()
      .then((items) => { if (!cancelled) setDocuments(items); })
      .catch((error) => notification.error(`读取文档库失败：${String(error)}`))
      .finally(() => { if (!cancelled) setLoadingLibrary(false); });
    void history.refresh();
    return () => { cancelled = true; };
  }, [history.refresh, notification.error]);

  const filteredDocuments = useMemo(() => {
    const normalizedQuery = query.trim().toLocaleLowerCase();
    return documents.filter((document) => {
      if (filter !== "all" && document.kind !== filter) return false;
      if (!normalizedQuery) return true;
      return document.title.toLocaleLowerCase().includes(normalizedQuery)
        || document.sourcePath.toLocaleLowerCase().includes(normalizedQuery);
    });
  }, [documents, filter, query]);

  const openPath = async (path: string) => {
    if (openingPath) return;
    setOpeningPath(path);
    try {
      await workspace.openPath(path);
    } finally {
      setOpeningPath(null);
    }
  };

  const openStoredDocument = async (document: StoredDocument) => {
    if (openingPath) return;
    setOpeningPath(document.path);
    try {
      if (await workspace.openPath(document.path)) return;
      if (document.sourceAvailable && document.sourcePath !== document.path
        && await workspace.openPath(document.sourcePath)) return;
      const decision = await message("Bambook 无法访问该链接文件。是否重新选择文件并恢复链接？", {
        title: "文件链接已失效", kind: "warning", buttons: { ok: "重新检索", cancel: "取消" } as const,
      });
      if (decision !== "重新检索") return;
      const selected = await selectFilePath([{ name: document.kind === "pdf" ? "PDF 文档" : "Markdown 文档", extensions: document.kind === "pdf" ? ["pdf"] : ["md", "markdown", "mdown", "mkd"] }]);
      if (!selected) return;
      const inspection = await documentLibraryRepository.inspectRelink(document.id, selected);
      if (!inspection.nameMatches) {
        const confirm = await message(`所选文件“${inspection.selectedName}”与 Bambook 记录“${inspection.recordedName}”不一致。仍要重新建立链接映射吗？`, {
        title: "文件名可能不一致", kind: "warning", buttons: { ok: "继续", cancel: "取消" } as const,
        });
        if (confirm !== "继续") return;
      }
      const updated = await documentLibraryRepository.relink(document.id, selected);
      setDocuments((current) => current.map((item) => item.id === updated.id ? updated : item));
      await workspace.openPath(updated.pathAvailable ? updated.path : updated.sourcePath);
    } catch (error) {
      notification.error(`重新链接文件失败：${String(error)}`);
    } finally {
      setOpeningPath(null);
    }
  };

  const recent = history.recent.slice(0, 12);
  return (
    <main className="start-workspace">
      <div className="start-page-shell">
        <header className="start-hero">
          <div className="start-hero-copy">
            <span className="start-eyebrow"><BookOpen size={16} />Bambook 文档工作区</span>
            <h1>继续阅读，或者开始一份新文档</h1>
            <p>最近访问记录与 Bambook 托管文档集中显示在这里。</p>
          </div>
          <div className="start-workspace-actions">
            <button className="tool-button primary" onClick={() => void workspace.openFile()}>
              <FolderOpen size={18} />打开文件
            </button>
            <button className="tool-button" onClick={() => void workspace.createMarkdown()}>
              <FilePlus2 size={18} />新建 Markdown
            </button>
          </div>
        </header>

        <div className="start-content-grid">
          <section className="start-section start-recent-section">
            <div className="start-section-heading">
              <div><Clock3 size={18} /><h2>最近访问</h2></div>
              <span>{history.recent.length}</span>
            </div>
            <div className="start-recent-list">
              {history.loading && !recent.length ? <StartEmpty label="正在读取最近文件…" /> : null}
              {!history.loading && !recent.length ? <StartEmpty label="最近还没有访问过文档" /> : null}
              {recent.map((document) => {
                const stored = documents.find((item) => item.path === document.path || item.sourcePath === document.path);
                return <RecentDocumentButton
                  key={document.path}
                  document={document}
                  opening={openingPath === document.path}
                  canRelink={stored?.storageMode === "linked-file"}
                  onOpen={stored ? async () => openStoredDocument(stored) : openPath}
                />;
              })}
            </div>
          </section>

          <section className="start-section start-library-section">
            <div className="start-library-heading">
              <div className="start-section-heading">
                <div><Library size={18} /><h2>全部文档</h2></div>
                <span>{documents.length}</span>
              </div>
              <div className="start-library-controls">
                <label className="start-library-search">
                  <Search size={16} />
                  <input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="搜索标题或原始路径" />
                </label>
                <div className="start-library-filters" aria-label="文档类型筛选">
                  <FilterButton active={filter === "all"} onClick={() => setFilter("all")}>全部</FilterButton>
                  <FilterButton active={filter === "pdf"} onClick={() => setFilter("pdf")}>PDF</FilterButton>
                  <FilterButton active={filter === "markdown"} onClick={() => setFilter("markdown")}>Markdown</FilterButton>
                </div>
              </div>
            </div>
            <div className="start-library-list">
              {loadingLibrary ? <StartEmpty label="正在读取文档库…" /> : null}
              {!loadingLibrary && !filteredDocuments.length ? (
                <StartEmpty label={documents.length ? "没有符合条件的文档" : "文档库还是空的，打开文件后会自动加入"} />
              ) : null}
              {filteredDocuments.map((document) => (
                <button
                  className="start-library-item"
                  key={document.id}
                  disabled={Boolean(openingPath)}
                  onClick={() => void openStoredDocument(document)}
                  title={document.sourcePath}
                >
                  <DocumentIcon kind={document.kind} />
                  <span className="start-document-copy">
                    <strong>{document.title}</strong>
                    <small>{document.sourcePath}</small>
                  </span>
                  <span className="start-document-meta">
                    <small>{document.kind === "pdf" ? "PDF" : "Markdown"}</small>
                    <time>{formatAccessTime(document.lastOpenedAt)}</time>
                  </span>
                </button>
              ))}
            </div>
          </section>
        </div>
      </div>
    </main>
  );
}

function RecentDocumentButton({ document, opening, canRelink, onOpen }: {
  document: RecentDocument;
  opening: boolean;
  canRelink: boolean;
  onOpen: (path: string) => Promise<void>;
}) {
  const available = document.availability === "available";
  const status = availabilityLabel(document);
  return (
    <button className="start-recent-item" disabled={(!available && !canRelink) || opening} onClick={() => void onOpen(document.path)}
      title={`${document.title}\n${document.path}${available ? "" : `\n${status}`}`}>
      <DocumentIcon kind={document.kind} />
      <span className="start-document-copy">
        <strong>{document.title}</strong>
        <small>{document.path}</small>
      </span>
      <span className="start-recent-meta">
        <time>{formatAccessTime(document.lastClosedAt)}</time>
        {!available && <small>{status}</small>}
      </span>
    </button>
  );
}

function DocumentIcon({ kind }: { kind: DocumentKind }) {
  return <span className={`start-document-icon ${kind}`}><FileText size={19} /></span>;
}

function FilterButton({ active, onClick, children }: {
  active: boolean;
  onClick: () => void;
  children: string;
}) {
  return <button className={active ? "active" : ""} aria-pressed={active} onClick={onClick}>{children}</button>;
}

function StartEmpty({ label }: { label: string }) {
  return <div className="start-empty-state"><FileText size={22} /><span>{label}</span></div>;
}

function formatAccessTime(timestamp: number) {
  if (!timestamp) return "时间未知";
  return new Intl.DateTimeFormat("zh-CN", { month: "numeric", day: "numeric" }).format(timestamp);
}

function availabilityLabel(document: RecentDocument) {
  switch (document.availability) {
    case "missing": return "文件不存在";
    case "inaccessible": return "文件无法访问";
    case "notFile": return "路径不是文件";
    case "unknown": return "文件状态未知";
    case "available": return document.path;
  }
}

import { Check, FileText, ShieldCheck, Star } from "lucide-react";
import { useDocumentWorkspace } from "../../features/documents";
import { useDocumentFavorites } from "../../features/documents/hooks/useDocumentFavorites";
import { useMarkdownWorkspace } from "../../features/markdown/hooks/useMarkdownWorkspace";
import { usePdfWorkspace } from "../../features/pdf/hooks/usePdfWorkspace";
import { PdfPageNavigation } from "../../features/pdf/components/PdfPageNavigation";
import { PdfZoomControls } from "../../features/pdf/components/PdfZoomControls";

export function WorkspaceStatusBar() {
  const documents = useDocumentWorkspace();
  const pdf = usePdfWorkspace();
  const markdown = useMarkdownWorkspace();
  const favorites = useDocumentFavorites();
  const document = documents.activeDocument;
  const path = document?.path || "";
  const pdfActive = document?.kind === "pdf";
  const markdownActive = document?.kind === "markdown";
  const pageCount = pdfActive ? document.pageCount : 0;
  const page = Math.min(pageCount, Math.max(1, pdf.currentPage));
  const progress = pageCount > 0 ? Math.round(page / pageCount * 100) : 0;
  const beforeCursor = markdown.content.slice(0, markdown.selection.start).split(/\r?\n/);
  const characters = Array.from(markdown.content.replace(/\s/g, "")).length;
  const dirty = markdownActive ? markdown.dirty : Boolean(document && documents.isDocumentDirty?.(document.id));
  return <footer className={`workspace-status-bar ${pdfActive ? "pdf-status-bar" : ""}`} aria-label="文档状态栏">
    <span className="workspace-status-local"><ShieldCheck size={14} />本地工作区</span>
    {document && <span className="workspace-status-document" title={document.path}><FileText size={13} />{document.title}</span>}
    <span className="workspace-status-spacer" />
    {document && path && <button type="button" aria-label={favorites.isFavorite(path) ? "取消收藏当前文档" : "收藏当前文档"} aria-pressed={favorites.isFavorite(path)} data-tooltip={favorites.isFavorite(path) ? "取消收藏" : "收藏当前文档"} onClick={() => favorites.toggle(path)}><Star size={14} fill={favorites.isFavorite(path) ? "currentColor" : "none"} /></button>}
    {document && <span className={dirty ? "dirty" : "saved"}><Check size={13} />{markdownActive && markdown.saving ? "保存中…" : dirty ? "未保存" : markdownActive && !path ? "未保存到文件" : "已保存"}</span>}
    {pdfActive && <><PdfPageNavigation /><span className="workspace-reading-progress">阅读进度 {progress}%<progress value={progress} max={100} aria-label="阅读进度" /></span><PdfZoomControls /></>}
    {markdownActive && <><span>{characters} 字</span><span className="workspace-status-cursor">第 {beforeCursor.length} 行，第 {(beforeCursor[beforeCursor.length - 1]?.length ?? 0) + 1} 列</span>{markdown.diagnostics.length > 0 && <span>{markdown.diagnostics.length} 个问题</span>}<span>UTF-8</span><span>Markdown</span></>}
    {!document && <span>{documents.documents.length} 份已打开文档</span>}
  </footer>;
}

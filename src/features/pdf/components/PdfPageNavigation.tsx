import { useEffect, useRef, useState } from "react";
import { ChevronLeft, ChevronRight, Redo2, Undo2 } from "lucide-react";
import { usePdfWorkspace } from "../hooks/usePdfWorkspace";

export function PdfPageNavigation() {
  const workspace = usePdfWorkspace();
  const pageCount = workspace.activeDocument?.pageCount ?? 0;
  const [pageValue, setPageValue] = useState(String(workspace.currentPage));
  const cancelCommit = useRef(false);

  useEffect(() => {
    setPageValue(String(workspace.currentPage));
    cancelCommit.current = false;
  }, [workspace.currentPage, workspace.activeDocumentId]);

  function commitPage() {
    if (cancelCommit.current) {
      cancelCommit.current = false;
      return;
    }
    const requested = Number(pageValue);
    if (!Number.isInteger(requested) || !pageValue || pageCount === 0) {
      setPageValue(String(workspace.currentPage));
      return;
    }
    const page = Math.max(1, Math.min(pageCount, requested));
    setPageValue(String(page));
    if (page !== workspace.currentPage) workspace.navigateToPage(page);
  }

  return <nav className="pdf-status-navigation" aria-label="PDF 页面与阅读历史">
    <button type="button" data-tooltip="上一页：跳转到前一页" aria-label="上一页"
      disabled={!pageCount || workspace.currentPage <= 1}
      onClick={() => workspace.navigateToPage(workspace.currentPage - 1)}><ChevronLeft size={15} /></button>
    <label className="pdf-status-page-input" title="输入页码并按回车跳转">
      <input value={pageValue} inputMode="numeric" aria-label="当前页码" disabled={!pageCount}
        onFocus={(event) => event.currentTarget.select()}
        onChange={(event) => setPageValue(event.target.value.replace(/[^0-9]/g, ""))}
        onBlur={commitPage}
        onKeyDown={(event) => {
          if (event.key === "Enter") { event.preventDefault(); event.currentTarget.blur(); }
          if (event.key === "Escape") {
            event.preventDefault();
            cancelCommit.current = true;
            setPageValue(String(workspace.currentPage));
            event.currentTarget.blur();
          }
        }} />
      <span>/ {pageCount} 页</span>
    </label>
    <button type="button" data-tooltip="下一页：跳转到后一页" aria-label="下一页"
      disabled={!pageCount || workspace.currentPage >= pageCount}
      onClick={() => workspace.navigateToPage(workspace.currentPage + 1)}><ChevronRight size={15} /></button>
    <span className="pdf-status-navigation-divider" aria-hidden="true" />
    <button type="button" data-tooltip="历史后退：返回上一个阅读位置" aria-label="阅读历史后退"
      disabled={!workspace.canNavigateHistoryBack} onClick={workspace.navigateHistoryBack}><Undo2 size={14} /></button>
    <button type="button" data-tooltip="历史前进：前往下一个阅读位置" aria-label="阅读历史前进"
      disabled={!workspace.canNavigateHistoryForward} onClick={workspace.navigateHistoryForward}><Redo2 size={14} /></button>
  </nav>;
}

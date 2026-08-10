import { CircleAlert, FileText, RotateCcw, Trash2 } from "lucide-react";
import { useEffect, useState } from "react";
import { useDocumentHistory } from "../hooks/useDocumentHistory";
import { useDocumentWorkspace } from "../hooks/useDocumentWorkspace";
import type { RecentDocument } from "../repositories/documentHistoryRepository";

export function RecentDocumentsPanel() {
  const workspace = useDocumentWorkspace();
  const history = useDocumentHistory();
  const [openingClosed, setOpeningClosed] = useState(false);
  const recent = history.recent;
  const closed = history.closed;
  const lastClosedStatus = closed[0] ? unavailableLabel(closed[0]) : null;
  const pdf = recent.filter((item) => item.kind === "pdf");
  const markdown = recent.filter((item) => item.kind === "markdown");
  useEffect(() => { void history.refresh(); }, [history.refresh]);
  const open = (item: RecentDocument) => {
    if (item.availability === "available") void workspace.openPath(item.path);
  };
  const reopenLastClosed = async () => {
    const item = closed[0];
    if (!item || item.availability !== "available" || openingClosed) return;
    setOpeningClosed(true);
    try {
      if (await workspace.openPath(item.path)) await history.confirmReopened(item.path);
    } finally {
      setOpeningClosed(false);
    }
  };
  return <section className="recent-documents-panel" aria-label="最近文件">
    <button className="recent-primary" data-tooltip="重新打开：恢复最近关闭的文档"
      disabled={!closed.length || Boolean(lastClosedStatus) || openingClosed} onClick={() => void reopenLastClosed()}>
      <RotateCcw size={16} />重新打开关闭的文件{lastClosedStatus && <span className="recent-status">（{lastClosedStatus}）</span>}
    </button>
    <div className="recent-groups">
      <RecentGroup title="PDF" items={pdf} onOpen={open} />
      <RecentGroup title="Markdown" items={markdown} onOpen={open} />
      {!recent.length && <p className="recent-empty">暂无最近文件</p>}
    </div>
    <button className="recent-clear" data-tooltip="清除记录：移除最近文件与关闭历史，不删除磁盘文件"
      disabled={history.loading || (!recent.length && !closed.length)} onClick={() => void history.clear()}>
      <Trash2 size={15} />清除所有最近文件
    </button>
  </section>;
}

function RecentGroup({ title, items, onOpen }: { title: string; items: RecentDocument[]; onOpen: (item: RecentDocument) => void }) {
  if (!items.length) return null;
  return <div className="recent-group"><h4>{title}</h4>{items.map((item) => {
    const status = unavailableLabel(item);
    return <button
      key={item.path}
      className={status ? "recent-document-unavailable" : undefined}
      data-tooltip={status ? `无法打开：${status}；${item.path}` : `打开最近文件：${item.path}`}
      disabled={Boolean(status)}
      onClick={() => onOpen(item)}
    >
      {status ? <CircleAlert size={15} /> : <FileText size={15} />}
      <span className="recent-document-copy">
        <strong>{item.title}</strong>
        <small>{item.path}</small>
      </span>
      {status && <span className="recent-status">{status}</span>}
    </button>;
  })}</div>;
}

function unavailableLabel(item: RecentDocument) {
  switch (item.availability) {
    case "available": return null;
    case "missing": return "文件不存在";
    case "inaccessible": return "无法访问";
    case "notFile": return "不是文件";
    case "unknown": return "状态未知";
  }
}

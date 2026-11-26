import { assistantRequestByteLimit } from "./assistantRequest";

export function AssistantContextBudget({ bytes, exceeded, messageLimitReached, compact = false }: { bytes: number; exceeded: boolean; messageLimitReached: boolean; compact?: boolean }) {
  const progress = Math.max(0, Math.min(100, bytes / assistantRequestByteLimit * 100));
  const used = bytes >= 1000 ? `${(bytes / 1000).toFixed(1)} KB` : `${bytes} B`;
  const warning = exceeded ? (messageLimitReached ? "对话已达上限，请新建对话" : "内容过长，请缩短引用或新建对话") : "";
  return <div className={`assistant-request-budget ${compact ? "compact" : ""} ${exceeded ? "exceeded" : ""}`} title={`上下文长度：${used} / 96 KB${warning ? `；${warning}` : ""}`}>
    <div className="assistant-context-ring" role="progressbar" aria-label="上下文长度" aria-valuemin={0} aria-valuemax={assistantRequestByteLimit}
      aria-valuenow={Math.min(bytes, assistantRequestByteLimit)} aria-valuetext={`${used} / 96 KB，${Math.round(progress)}%${warning ? `，${warning}` : ""}`}>
      <svg viewBox="0 0 44 44" aria-hidden="true"><circle className="assistant-ring-track" cx="22" cy="22" r="18" /><circle className="assistant-ring-value" cx="22" cy="22" r="18" pathLength="100" strokeDasharray="100" strokeDashoffset={100 - progress} transform="rotate(-90 22 22)" /></svg>
      <span>{Math.round(progress)}%</span>
    </div>
    {!compact && <div className="assistant-budget-text"><span>上下文长度 · UTF-8</span><span>{used} / 96 KB</span>{exceeded && <small>{warning}</small>}</div>}
  </div>;
}

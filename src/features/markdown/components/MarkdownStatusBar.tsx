import { useMarkdownWorkspace } from "../hooks/useMarkdownWorkspace";

export function MarkdownStatusBar() {
  const workspace = useMarkdownWorkspace();
  const lineCount = workspace.content ? workspace.content.split(/\r?\n/).length : 0;
  const beforeCursor = workspace.content.slice(0, workspace.selection.start);
  const cursorLines = beforeCursor.split(/\r?\n/);
  const cursorLine = cursorLines.length;
  const cursorColumn = (cursorLines[cursorLines.length - 1]?.length ?? 0) + 1;
  return (
    <footer className="markdown-status-bar">
      <div><span className={workspace.dirty ? "dirty" : "saved"}>{workspace.saving ? "保存中…" : workspace.dirty ? "● 未保存" : "已保存"}</span>
        {workspace.diagnostics.length > 0 && <span>{workspace.diagnostics.length} 个问题</span>}</div>
      <div><span>第 {cursorLine} 行，第 {cursorColumn} 列</span><span>{lineCount} 行</span><span>{workspace.content.length} 字符</span></div>
    </footer>
  );
}

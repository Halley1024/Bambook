import { AlertCircle, CheckCircle2, Copy, FileText, Image, Link, PanelRightClose, PanelRightOpen } from "lucide-react";
import { useMemo, useState, type PointerEvent, type ReactNode } from "react";
import { useMarkdownWorkspace } from "../hooks/useMarkdownWorkspace";
import type { MarkdownNode } from "../types/markdownAst";

type InspectorTab = "document" | "problems" | "resources";
type MarkdownResource = {
  id: string;
  kind: "link" | "image";
  label: string;
  url: string;
  start: number;
  end: number;
};

const tabs = [
  { id: "document", label: "文档", icon: FileText },
  { id: "problems", label: "问题", icon: AlertCircle },
  { id: "resources", label: "资源", icon: Image },
] as const;

export function MarkdownInspectorPanel({ onResizeStart }: { onResizeStart?: (event: PointerEvent) => void }) {
  const workspace = useMarkdownWorkspace();
  const [tab, setTab] = useState<InspectorTab>("document");
  const resources = useMemo(() => collectResources(workspace.nodes), [workspace.nodes]);
  const documentStats = useMemo(() => collectDocumentStats(workspace.content, workspace.nodes), [workspace.content, workspace.nodes]);
  const navigateToRange = (start: number, end: number) => {
    if (workspace.viewMode === "realtime") workspace.setViewMode("split");
    workspace.navigateToSearchResult(start, end);
  };

  return <aside className={`markdown-help-panel markdown-inspector-panel ${workspace.helpPanelCollapsed ? "collapsed" : ""}`}
    aria-label="Markdown 文档检查器">
    {workspace.helpPanelCollapsed ? (
      <button className="markdown-help-expand markdown-inspector-expand" onClick={workspace.toggleHelpPanel}
        data-tooltip="展开文档检查器：查看 Markdown 文档信息、问题与资源">
        <PanelRightOpen size={19} /><span>检查器</span>
      </button>
    ) : <>
      <header className="markdown-inspector-header">
        <div>
          <button onClick={workspace.toggleHelpPanel} data-tooltip="收起文档检查器" aria-label="收起文档检查器">
            <PanelRightClose size={18} />
          </button>
          <FileText size={17} /><h2>文档检查器</h2>
        </div>
      </header>
      <nav className="markdown-inspector-tabs" role="tablist" aria-label="检查器分类">
        {tabs.map((item) => {
          const Icon = item.icon;
          const count = item.id === "problems" ? workspace.diagnostics.length : item.id === "resources" ? resources.length : null;
          return <button key={item.id} className={tab === item.id ? "active" : ""} role="tab"
            aria-selected={tab === item.id} onClick={() => setTab(item.id)}
            data-tooltip={`${item.label}：${inspectorTabDescription(item.id)}`}>
            <Icon size={15} /><span>{item.label}</span>{count !== null && <small>{count}</small>}
          </button>;
        })}
      </nav>
      <div className="markdown-inspector-content">
        {!workspace.activeDocument ? <InspectorEmpty text="打开 Markdown 文档后显示检查信息" /> : <>
          {tab === "document" && <section className="markdown-inspector-section">
            <div className="markdown-document-title">
              <div><strong>{workspace.activeDocument.title}</strong><small>{workspace.activeDocument.path || "尚未保存到磁盘"}</small></div>
              {workspace.activeDocument.path && <button onClick={() => workspace.copyDocumentPath(workspace.activeDocument!.id)}
                data-tooltip="复制路径：复制当前 Markdown 文件位置" aria-label="复制文件路径"><Copy size={15} /></button>}
            </div>
            <div className="markdown-document-state">
              {workspace.dirty ? <><AlertCircle size={15} />未保存更改</> : <><CheckCircle2 size={15} />已保存</>}
            </div>
            <dl className="markdown-stat-grid">
              <Stat label="字词" value={documentStats.words} />
              <Stat label="字符" value={workspace.content.length} />
              <Stat label="行数" value={documentStats.lines} />
              <Stat label="标题" value={workspace.outline.length} />
              <Stat label="链接" value={documentStats.links} />
              <Stat label="图片" value={documentStats.images} />
            </dl>
          </section>}

          {tab === "problems" && <section className="markdown-inspector-section">
            {!workspace.diagnostics.length ? <InspectorEmpty icon={<CheckCircle2 size={22} />} text="未发现 Markdown 解析问题" />
              : <div className="markdown-inspector-list">{workspace.diagnostics.map((diagnostic, index) =>
                <button key={`${diagnostic.message}-${index}`} disabled={!diagnostic.range}
                  onClick={() => diagnostic.range && navigateToRange(diagnostic.range.start, diagnostic.range.end)}
                  data-tooltip={diagnostic.range ? "跳转：定位到对应 Markdown 源码" : diagnostic.message}>
                  <AlertCircle size={16} /><span><strong>{severityLabel(diagnostic.severity)}</strong>
                    <small>{diagnostic.message}</small>{diagnostic.range && <em>{positionLabel(workspace.content, diagnostic.range.start)}</em>}</span>
                </button>)}</div>}
          </section>}

          {tab === "resources" && <section className="markdown-inspector-section">
            {!resources.length ? <InspectorEmpty text="当前文档没有链接或图片资源" />
              : <div className="markdown-inspector-list">{resources.map((resource) =>
                <button key={resource.id} onClick={() => navigateToRange(resource.start, resource.end)}
                  data-tooltip="跳转：定位到该资源的 Markdown 源码">
                  {resource.kind === "image" ? <Image size={16} /> : <Link size={16} />}
                  <span><strong>{resource.label}</strong><small>{resource.url || "未填写路径"}</small>
                    <em>{resource.kind === "image" ? "图片" : "链接"}</em></span>
                </button>)}</div>}
            <p className="markdown-inspector-interface-note">本地资源有效性将在文件检查接口接入后显示。</p>
          </section>}
        </>}
      </div>
    </>}
    {!workspace.helpPanelCollapsed && onResizeStart && <div className="panel-resize-handle left-edge" role="separator"
      aria-orientation="vertical" aria-label="调整文档检查器宽度" onPointerDown={onResizeStart} />}
  </aside>;
}

function Stat({ label, value }: { label: string; value: number }) {
  return <div><dt>{label}</dt><dd>{value}</dd></div>;
}

function InspectorEmpty({ text, icon }: { text: string; icon?: ReactNode }) {
  return <div className="markdown-inspector-empty">{icon}{text}</div>;
}

function collectResources(nodes: MarkdownNode[]): MarkdownResource[] {
  return nodes.flatMap((node): MarkdownResource[] => {
    const own = node.type === "link" || node.type === "image" ? [{
      id: node.id,
      kind: node.type,
      label: plainText(node.children) || node.title || node.url || (node.type === "image" ? "未命名图片" : "未命名链接"),
      url: node.url,
      start: node.range.start,
      end: node.range.end,
    }] : [];
    return "children" in node ? [...own, ...collectResources(node.children)] : own;
  });
}

function collectDocumentStats(content: string, nodes: MarkdownNode[]) {
  const allNodes = flattenNodes(nodes);
  return {
    words: (content.match(/[\u3400-\u9fff]|[A-Za-z0-9]+(?:['’-][A-Za-z0-9]+)*/g) ?? []).length,
    lines: content ? content.split(/\r?\n/).length : 0,
    links: allNodes.filter((node) => node.type === "link").length,
    images: allNodes.filter((node) => node.type === "image").length,
  };
}

function flattenNodes(nodes: MarkdownNode[]): MarkdownNode[] {
  return nodes.flatMap((node) => [node, ...("children" in node ? flattenNodes(node.children) : [])]);
}

function plainText(nodes: MarkdownNode[]): string {
  return nodes.map((node) => "text" in node ? node.text : "children" in node ? plainText(node.children) : "").join("");
}

function severityLabel(severity: string) {
  if (severity.toLowerCase() === "error") return "错误";
  if (severity.toLowerCase() === "warning") return "警告";
  return "提示";
}

function positionLabel(content: string, offset: number) {
  const before = content.slice(0, offset);
  const lines = before.split(/\r?\n/);
  return `第 ${lines.length} 行，第 ${(lines[lines.length - 1]?.length ?? 0) + 1} 列`;
}

function inspectorTabDescription(tab: InspectorTab) {
  if (tab === "document") return "查看统计与保存状态";
  if (tab === "problems") return "查看解析诊断并跳转定位";
  return "查看文档中的链接和图片";
}

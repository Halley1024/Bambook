import { ChevronRight, FileText, Folder } from "lucide-react";
import { useState } from "react";
import type { FolderEntry } from "../types";

export function DocumentFolderTree({ entries, activePath, onOpenFile }: {
  entries: FolderEntry[];
  activePath?: string;
  onOpenFile: (path: string) => void;
}) {
  const [expanded, setExpanded] = useState<Set<string>>(() => new Set(entries.filter((entry) => entry.isDirectory).map((entry) => entry.path)));
  return <div className="markdown-file-tree" role="tree" aria-label="文件夹文档">
    {entries.map((entry) => <FolderNode key={entry.path} entry={entry} depth={0} expanded={expanded}
      activePath={activePath} onOpenFile={onOpenFile} onToggle={(path) => setExpanded((current) => {
        const next = new Set(current);
        if (next.has(path)) next.delete(path); else next.add(path);
        return next;
      })} />)}
  </div>;
}

function FolderNode({ entry, depth, expanded, activePath, onOpenFile, onToggle }: {
  entry: FolderEntry;
  depth: number;
  expanded: Set<string>;
  activePath?: string;
  onOpenFile: (path: string) => void;
  onToggle: (path: string) => void;
}) {
  const isExpanded = expanded.has(entry.path);
  const active = !entry.isDirectory && entry.path.toLocaleLowerCase() === activePath?.toLocaleLowerCase();
  return <div className="markdown-file-node" role="treeitem" aria-expanded={entry.isDirectory ? isExpanded : undefined}>
    <button className={`markdown-file-row ${active ? "active" : ""}`}
      style={{ paddingLeft: 10 + Math.min(depth, 7) * 15 }}
      onClick={() => entry.isDirectory ? onToggle(entry.path) : onOpenFile(entry.path)} title={entry.path}>
      {entry.isDirectory ? <><ChevronRight className={`markdown-tree-chevron ${isExpanded ? "expanded" : ""}`} size={14} /><Folder size={15} /></>
        : <><span className="markdown-tree-spacer" /><FileText size={15} /></>}
      <span>{entry.name}</span>
    </button>
    {entry.isDirectory && <div className={`markdown-file-children ${isExpanded ? "expanded" : ""}`}><div>
      {entry.children.map((child) => <FolderNode key={child.path} entry={child} depth={depth + 1}
        expanded={expanded} activePath={activePath} onOpenFile={onOpenFile} onToggle={onToggle} />)}
    </div></div>}
  </div>;
}

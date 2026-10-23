import { ChevronRight, FileText, Folder } from "lucide-react";
import { useState } from "react";
import type { MarkdownFileEntry } from "../types/markdownWorkspace";

type MarkdownFileTreeProps = {
  entries: MarkdownFileEntry[];
  activePath?: string;
  onOpenFile: (path: string) => void;
};

export function MarkdownFileTree({ entries, activePath, onOpenFile }: MarkdownFileTreeProps) {
  const [expanded, setExpanded] = useState<Set<string>>(new Set());

  return (
    <div className="markdown-file-tree" role="tree" aria-label="Markdown 文件结构">
      {entries.map((entry) => (
        <MarkdownFileNode
          key={entry.path}
          entry={entry}
          depth={0}
          expanded={expanded}
          activePath={activePath}
          onToggle={(path) =>
            setExpanded((current) => {
              const next = new Set(current);
              if (next.has(path)) next.delete(path);
              else next.add(path);
              return next;
            })
          }
          onOpenFile={onOpenFile}
        />
      ))}
    </div>
  );
}

function MarkdownFileNode({ entry, depth, expanded, activePath, onToggle, onOpenFile }: {
  entry: MarkdownFileEntry;
  depth: number;
  expanded: Set<string>;
  activePath?: string;
  onToggle: (path: string) => void;
  onOpenFile: (path: string) => void;
}) {
  const isExpanded = expanded.has(entry.path);
  const isActive = !entry.isDirectory && entry.path.toLowerCase() === activePath?.toLowerCase();
  return (
    <div className="markdown-file-node" role="treeitem" aria-expanded={entry.isDirectory ? isExpanded : undefined}>
      <button
        className={`markdown-file-row ${isActive ? "active" : ""}`}
        style={{ paddingLeft: 10 + Math.min(depth, 7) * 15 }}
        onClick={() => (entry.isDirectory ? onToggle(entry.path) : onOpenFile(entry.path))}
        data-tooltip={`${entry.isDirectory ? "展开目录" : "打开文件"}：${entry.path}`}
      >
        {entry.isDirectory ? (
          <>
            <ChevronRight className={`markdown-tree-chevron ${isExpanded ? "expanded" : ""}`} size={14} />
            <Folder size={15} />
          </>
        ) : (
          <>
            <span className="markdown-tree-spacer" />
            <FileText size={15} />
          </>
        )}
        <span>{entry.name}</span>
      </button>
      {entry.isDirectory && (
        <div className={`markdown-file-children ${isExpanded ? "expanded" : ""}`}>
          <div>
            {entry.children.map((child) => (
              <MarkdownFileNode
                key={child.path}
                entry={child}
                depth={depth + 1}
                expanded={expanded}
                activePath={activePath}
                onToggle={onToggle}
                onOpenFile={onOpenFile}
              />
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

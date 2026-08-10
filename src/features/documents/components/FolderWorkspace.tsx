import { FileSearch, FolderOpen } from "lucide-react";
import { useDocumentWorkspace } from "../hooks/useDocumentWorkspace";
import { DocumentFileNavigator } from "./DocumentFileNavigator";
import { UnifiedDocumentTabs } from "./UnifiedDocumentTabs";

export function FolderWorkspace() {
  const workspace = useDocumentWorkspace();
  return <main className="folder-workspace">
    <aside className="folder-workspace-sidebar">
      <header><FolderOpen size={17} /><div><strong>文件夹</strong><small title={workspace.folderPath ?? ""}>{folderName(workspace.folderPath)}</small></div></header>
      <div className="folder-workspace-tree document-navigation-host">
        <DocumentFileNavigator />
      </div>
    </aside>
    <section className="workspace-content">
      <div className="document-bar"><UnifiedDocumentTabs /></div>
      <div className="folder-workspace-empty">
        <span><FileSearch size={30} /></span>
        <h1>从左侧选择文档</h1>
        <p>选择 PDF 开始阅读，或者打开 Markdown 文档进行编辑。</p>
      </div>
    </section>
  </main>;
}

function folderName(path: string | null) {
  if (!path) return "未选择文件夹";
  return path.replace(/[\\/]+$/, "").split(/[\\/]/).pop() || path;
}

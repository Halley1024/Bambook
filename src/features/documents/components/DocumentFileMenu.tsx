import { ExternalLink, File, FilePlus2, FolderOpen, History, SquarePlus, X } from "lucide-react";
import { DropdownMenu, type DropdownMenuGroup } from "../../../components/menus/DropdownMenu";
import { openReaderWindow } from "../../../platform/window";
import { useNotification } from "../../notifications";
import { useDocumentWorkspace } from "../hooks/useDocumentWorkspace";
import { revealInFileManager } from "../repositories/documentRepository";
import { RecentDocumentsPanel } from "./RecentDocumentsPanel";

export function DocumentFileMenu({ documentGroups = [] }: { documentGroups?: DropdownMenuGroup[] }) {
  const workspace = useDocumentWorkspace();
  const notification = useNotification();
  const hasDocument = Boolean(workspace.activeDocument);
  const revealActiveDocument = async () => {
    if (!workspace.activeDocument) return;
    try {
      const path = workspace.activeDocument.kind === "pdf"
        ? workspace.activeDocument.sourcePath
        : workspace.activeDocument.path;
      await revealInFileManager(path);
    } catch (error) { notification.error(`无法在文件管理器中显示文件：${String(error)}`); }
  };
  const createWindow = async () => {
    try {
      await openReaderWindow();
    } catch (error) { notification.error(`新建窗口失败：${String(error)}`); }
  };
  const groups: DropdownMenuGroup[] = [
    {
      id: "window",
      items: [
        { id: "new-markdown", label: "新建 Markdown", icon: FilePlus2, title: "创建一个空白 Markdown 标签页", onSelect: () => void workspace.createMarkdown() },
        { id: "new-window", label: "新建窗口", icon: SquarePlus, title: "打开一个新的 Bambook 窗口", onSelect: () => void createWindow() },
        ...(hasDocument ? [{ id: "reveal", label: "在文件中显示", icon: ExternalLink, title: "在系统文件管理器中定位当前文档", onSelect: () => void revealActiveDocument() }] : []),
      ],
    },
    {
      id: "open",
      items: [
        { id: "open-file", label: "打开文件", icon: File, title: "选择一个 PDF 或 Markdown 文档", onSelect: () => void workspace.openFile() },
        { id: "open-folder", label: "打开文件夹", icon: FolderOpen, title: "在工作区中浏览指定文件夹", onSelect: () => void workspace.openFolder() },
        {
          id: "recent",
          label: "打开最近文件",
          icon: History,
          title: "查看最近访问和关闭的文档",
          submenu: <RecentDocumentsPanel />,
        },
      ],
    },
    ...documentGroups,
    {
      id: "close",
      items: [{ id: "close-document", label: "关闭", icon: X, danger: true, disabled: !hasDocument,
        title: hasDocument ? "关闭当前标签页；未保存内容将进入统一关闭事务" : "当前没有可关闭的标签页",
        onSelect: () => { if (workspace.activeDocument) void workspace.closeDocument(workspace.activeDocument.id); } }],
    },
  ];
  return <DropdownMenu label="文件" icon={File} groups={groups} />;
}

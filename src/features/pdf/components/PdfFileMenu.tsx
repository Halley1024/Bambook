import {
  Download,
  Save,
  SaveAll,
} from "lucide-react";
import type { DropdownMenuGroup } from "../../../components/menus/DropdownMenu";
import { DocumentFileMenu } from "../../documents";
import { usePdfWorkspace } from "../hooks/usePdfWorkspace";

export function PdfFileMenu() {
  const workspace = usePdfWorkspace();
  const hasDocument = Boolean(workspace.activeDocument);
  const groups: DropdownMenuGroup[] = [
    {
      id: "notes",
      items: [
        {
          id: "export-notes",
          label: "导出笔记",
          icon: Download,
          disabled: !hasDocument,
          title: hasDocument ? "将当前 PDF 笔记整理为 Markdown 文件" : "请先打开 PDF 文档",
          onSelect: () => void workspace.exportNotes(),
        },
      ],
    },
    {
      id: "save",
      items: [
        {
          id: "save",
          label: "保存",
          icon: Save,
          shortcut: "Ctrl+S",
          disabled: !hasDocument,
          title: hasDocument ? "写入当前批注、书签和阅读数据" : "请先打开 PDF 文档",
          onSelect: () => void workspace.saveActiveDocument(),
        },
        {
          id: "save-as",
          label: "另存为...",
          icon: SaveAll,
          disabled: !hasDocument,
          title: "保留PDF原文及批注、书签、阅读状态",
          onSelect: () => void workspace.saveActiveDocumentAs(),
        },
      ],
    },
  ];

  return <DocumentFileMenu documentGroups={groups} />;
}

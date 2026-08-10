import {
  FileCode2,
  FileDown,
  Save,
  SaveAll,
} from "lucide-react";
import type { DropdownMenuGroup } from "../../../components/menus/DropdownMenu";
import { DocumentFileMenu } from "../../documents";
import { useMarkdownWorkspace } from "../hooks/useMarkdownWorkspace";

export function MarkdownFileMenu() {
  const workspace = useMarkdownWorkspace();
  const hasDocument = Boolean(workspace.activeDocument);
  const groups: DropdownMenuGroup[] = [
    {
      id: "export",
      items: [
        {
          id: "export-html",
          label: "导出为 HTML",
          icon: FileCode2,
          disabled: !hasDocument,
          title: hasDocument ? "生成可独立打开的 HTML 文件" : "请先打开 Markdown 文档",
          onSelect: () => void workspace.exportHtml(),
        },
        {
          id: "export-pdf",
          label: "导出为 PDF",
          icon: FileDown,
          disabled: !hasDocument,
          title: hasDocument ? "生成经过排版的 PDF 文件" : "请先打开 Markdown 文档",
          onSelect: () => void workspace.exportPdf(),
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
          disabled: !hasDocument || workspace.saving,
          title: hasDocument ? "将当前编辑内容写入文档" : "请先打开 Markdown 文档",
          onSelect: () => void workspace.saveActiveDocument(),
        },
        {
          id: "save-as",
          label: "另存为文档包",
          icon: SaveAll,
          disabled: !hasDocument || workspace.saving,
          title: hasDocument ? "将文档、资源文件和元数据打包到指定目录" : "请先打开 Markdown 文档",
          onSelect: () => void workspace.saveActiveDocumentAs(),
        },
      ],
    },
  ];

  return <DocumentFileMenu documentGroups={groups} />;
}

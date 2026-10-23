import { message } from "@tauri-apps/plugin-dialog";
import { selectFilePath } from "../../../platform/fileDialog";
import {
  documentLibraryRepository,
  notifyDocumentLibraryChanged,
  type StoredDocument,
} from "./documentLibraryRepository";
import type { ReaderDocument } from "../types";

export type EnsureDocumentSourceResult =
  | { status: "unchanged" }
  | { status: "cancelled" }
  | { status: "relinked"; document: StoredDocument };

export async function ensureDocumentSource(document: ReaderDocument): Promise<EnsureDocumentSourceResult> {
  const library = await documentLibraryRepository.load();
  const stored = findStoredDocument(library, document);
  if (!stored || stored.sourceAvailable || (stored.storageMode === "managed-copy" && stored.pathAvailable)) {
    return { status: "unchanged" };
  }

  const selectedPath = await selectFilePath([{
    name: stored.kind === "pdf" ? "PDF 文档" : "Markdown 文档",
    extensions: stored.kind === "pdf" ? ["pdf"] : ["md", "markdown", "mdown", "mkd"],
  }]);
  if (!selectedPath) return { status: "cancelled" };

  const inspection = await documentLibraryRepository.inspectRelink(stored.id, selectedPath);
  if (!inspection.nameMatches) {
    const decision = await message(
      `所选文件“${inspection.selectedName}”与 Bambook 记录“${inspection.recordedName}”不一致。是否继续重新建立来源映射？`,
      { title: "文件名可能不一致", kind: "warning", buttons: { ok: "继续", cancel: "取消" } as const },
    );
    if (decision !== "继续") return { status: "cancelled" };
  }

  const updated = await documentLibraryRepository.relink(stored.id, selectedPath);
  notifyDocumentLibraryChanged();
  return { status: "relinked", document: updated };
}

export function applyRelinkedDocument(document: ReaderDocument, stored: StoredDocument): ReaderDocument {
  return document.kind === "pdf"
    ? { ...document, title: stored.title, path: stored.path, sourcePath: stored.sourcePath }
    : { ...document, title: stored.title, path: stored.path };
}

function findStoredDocument(items: StoredDocument[], document: ReaderDocument) {
  const paths = document.kind === "pdf" ? [document.path, document.sourcePath] : [document.path];
  return items.find((item) => item.id === document.id || [item.path, item.sourcePath]
    .some((path) => paths.some((candidate) => normalizePath(path) === normalizePath(candidate))));
}

function normalizePath(path: string) {
  return path.replace(/^\\\\\?\\/, "").replace(/\//g, "\\").replace(/\\+$/, "").toLocaleLowerCase();
}

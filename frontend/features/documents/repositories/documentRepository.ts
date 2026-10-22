import { selectFilePath } from "../../../platform/fileDialog";
import { invokeCommand } from "../../../platform/tauriClient";
import type { DocumentKind, FolderEntry, MarkdownDocument, PdfDocument, ReaderDocument } from "../types";

type OpenedPdf = Omit<PdfDocument, "id" | "kind"> & { documentId: string };
type OpenedMarkdown = {
  documentId: string;
  sessionId: string;
  revision: number;
  path: string;
  title: string;
  source: string;
  nodes: MarkdownDocument["nodes"];
  outline: MarkdownDocument["outline"];
  diagnostics: MarkdownDocument["diagnostics"];
};

export async function openAnyDocument(): Promise<ReaderDocument | null> {
  const path = await selectFilePath([
    { name: "支持的文档", extensions: ["pdf", "md", "markdown", "mdown", "mkd"] },
    { name: "PDF", extensions: ["pdf"] },
    { name: "Markdown", extensions: ["md", "markdown", "mdown", "mkd"] },
  ]);
  if (!path) return null;
  return openDocumentPath(path);
}

export function createMarkdownDocument(path: string) {
  return invokeCommand<void>("create_markdown_document", { path });
}

export async function createUntitledMarkdownDocument(): Promise<MarkdownDocument> {
  return mapMarkdown(await invokeCommand<OpenedMarkdown>("create_untitled_markdown_document"));
}

export function revealInFileManager(path: string) {
  return invokeCommand<void>("reveal_in_file_manager", { path });
}

export function openDirectoryInFileManager(path: string) {
  return invokeCommand<void>("open_directory_in_file_manager", { path });
}

export function listSupportedDirectory(path: string) {
  return invokeCommand<FolderEntry[]>("list_supported_directory", { path });
}

export async function openDocumentPath(path: string): Promise<ReaderDocument> {
  const { kind } = await invokeCommand<{ kind: DocumentKind }>("detect_document_kind", { path });
  return readDocumentPath(kind, path);
}

export function readDocumentPath(kind: "pdf", path: string): Promise<PdfDocument>;
export function readDocumentPath(kind: "markdown", path: string): Promise<MarkdownDocument>;
export function readDocumentPath(kind: DocumentKind, path: string): Promise<ReaderDocument>;
export async function readDocumentPath(kind: DocumentKind, path: string): Promise<ReaderDocument> {
  if (kind === "pdf") {
    const file = await invokeCommand<OpenedPdf>("open_pdf_document", { path });
    return { ...file, id: file.documentId, kind: "pdf" };
  }
  return mapMarkdown(await invokeCommand<OpenedMarkdown>("open_markdown_document", { path }));
}

function mapMarkdown(file: OpenedMarkdown): MarkdownDocument {
  return {
    id: file.documentId,
    kind: "markdown",
    sessionId: file.sessionId,
    revision: file.revision,
    path: file.path,
    title: file.title,
    markdown: file.source,
    nodes: file.nodes,
    outline: file.outline,
    diagnostics: file.diagnostics,
  };
}

export async function authenticatePdfDocument(document: PdfDocument, password: string): Promise<PdfDocument> {
  const authenticated = await invokeCommand<OpenedPdf>("authenticate_pdf_document", {
    sessionId: document.sessionId,
    password,
  });
  return { ...authenticated, id: document.id, kind: "pdf" };
}

export async function closeReaderDocument(document: ReaderDocument) {
  if (document.kind === "pdf") {
    await invokeCommand<void>("close_pdf_document", { sessionId: document.sessionId });
  } else {
    await invokeCommand<void>("close_markdown_document", { sessionId: document.sessionId });
  }
}

export async function renameDocumentFile(document: ReaderDocument, newPath: string): Promise<boolean> {
  if (document.kind === "markdown") {
    await invokeCommand<void>("rename_markdown_document", { path: document.path, newPath });
    return true;
  }
  if (document.kind === "pdf") {
    await invokeCommand<void>("rename_pdf_document", { path: document.path, newPath });
    return true;
  }
  return false;
}

async function fingerprint(value: string) {
  const data = new TextEncoder().encode(value);
  const digest = await crypto.subtle.digest("SHA-256", data);
  return Array.from(new Uint8Array(digest))
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}

export { fingerprint };

export function saveUntitledRecovery(content: string) {
  return invokeCommand<void>("save_untitled_markdown_recovery", { content });
}

export async function loadUntitledRecovery(): Promise<string | null> {
  const result = await invokeCommand<string | null>("load_untitled_markdown_recovery");
  return result ?? null;
}

export function clearUntitledRecovery() {
  return invokeCommand<void>("clear_untitled_markdown_recovery");
}

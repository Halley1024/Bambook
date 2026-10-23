import { invokeCommand } from "../../../platform/tauriClient";
import type { MarkdownFileEntry } from "../types/markdownWorkspace";

export const markdownRepository = {
  listDirectory(documentPath: string) {
    return invokeCommand<MarkdownFileEntry[]>("list_markdown_directory", { path: documentPath });
  },
};

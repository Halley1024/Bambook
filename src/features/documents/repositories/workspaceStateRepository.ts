import { invokeCommand } from "../../../platform/tauriClient";
import type { DocumentKind } from "../types";

export type PersistedWorkspaceState = {
  openDocuments: Array<{ path: string; kind: DocumentKind }>;
  activeDocumentPath?: string;
  folderPath?: string;
};

export const workspaceStateRepository = {
  load() {
    return invokeCommand<PersistedWorkspaceState>("load_workspace_state");
  },
  async save(workspace: PersistedWorkspaceState) {
    await invokeCommand<void>("save_workspace_state", { workspace });
  },
};

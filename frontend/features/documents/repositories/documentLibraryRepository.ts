import { invokeCommand } from "../../../platform/tauriClient";
import type { DocumentKind } from "../types";

export type StoredDocument = {
  id: string;
  title: string;
  kind: DocumentKind;
  path: string;
  sourcePath: string;
  dataPath: string;
  storageMode: "managed-copy" | "linked-file";
  createdAt: number;
  updatedAt: number;
  lastOpenedAt: number;
  pathAvailable: boolean;
  sourceAvailable: boolean;
};

export const DOCUMENT_LIBRARY_CHANGED_EVENT = "bambook:document-library-changed";

export function notifyDocumentLibraryChanged() {
  window.dispatchEvent(new Event(DOCUMENT_LIBRARY_CHANGED_EVENT));
}

export const documentLibraryRepository = {
  load() {
    return invokeCommand<StoredDocument[]>("load_document_library");
  },
  inspectRelink(documentId: string, newPath: string) {
    return invokeCommand<{ recordedName: string; selectedName: string; nameMatches: boolean }>("inspect_document_relink", { documentId, newPath });
  },
  relink(documentId: string, newPath: string) {
    return invokeCommand<StoredDocument>("relink_document", { documentId, newPath });
  },
  reconcileStorage(documentId: string) {
    return invokeCommand<void>("reconcile_document_storage", { documentId });
  },
  exportPackage(documentId: string, destination: string) {
    return invokeCommand<string>("export_document_package", { documentId, destination });
  },
};

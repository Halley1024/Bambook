import { createContext } from "react";
import type { DocumentKind, FolderEntry, ReaderDocument } from "../types";
import type { ClosePreparation } from "../close/closeCoordinator";

export type DocumentLifecycleController = {
  deactivate?: (document: ReaderDocument) => void;
  requiresCloseDecision?: (document: ReaderDocument) => boolean;
  prepareSave?: (document: ReaderDocument) => Promise<ClosePreparation> | ClosePreparation;
  close: (document: ReaderDocument) => Promise<void> | void;
};

export type FileNavigationMode = "directory" | "library";

export type DocumentWorkspaceContextValue = {
  documents: ReaderDocument[];
  activeDocument: ReaderDocument | null;
  activeDocumentId: string | null;
  folderPath: string | null;
  folderEntries: FolderEntry[];
  folderLoading: boolean;
  fileNavigationMode: FileNavigationMode;
  setFileNavigationMode: (mode: FileNavigationMode) => void;
  openFile: () => Promise<void>;
  openFolder: () => Promise<void>;
  openPath: (path: string) => Promise<boolean>;
  createMarkdown: () => Promise<void>;
  activateDocument: (id: string) => void;
  closeDocument: (id: string) => Promise<void>;
  closeOtherDocuments: (id: string) => Promise<void>;
  closeAllDocuments: () => Promise<void>;
  prepareWindowClose: () => Promise<boolean>;
  replaceDocument: (document: ReaderDocument) => void;
  reorderDocuments: (sourceId: string, targetId: string) => void;
  copyDocumentLabel: (id: string) => void;
  copyRelativeDocumentPath: (id: string) => void;
  copyAbsoluteDocumentPath: (id: string) => void;
  copyDocumentPath: (id: string) => void;
  registerLifecycle: (kind: DocumentKind, controller: DocumentLifecycleController) => () => void;
  setDocumentDirty: (id: string, dirty: boolean) => void;
  isDocumentDirty?: (id: string) => boolean;
  createMarkdownAndReturn: () => Promise<ReaderDocument | null>;
  requestSaveActive?: (id: string) => void;
  requestSaveActiveAs?: (id: string) => void;
  saveActiveRequest: { revision: number; documentId: string | null };
  saveAsActiveRequest: number;
};

export const DocumentWorkspaceContext = createContext<DocumentWorkspaceContextValue | null>(null);

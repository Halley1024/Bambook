import { createContext } from "react";
import type { ReaderDocument } from "../types";
import type { DocumentHistory } from "../repositories/documentHistoryRepository";

export type DocumentHistoryContextValue = DocumentHistory & {
  loading: boolean;
  refresh: () => Promise<void>;
  recordClosed: (document: ReaderDocument) => Promise<void>;
  recordAccessed: (document: ReaderDocument) => Promise<void>;
  confirmReopened: (path: string) => Promise<void>;
  clear: () => Promise<void>;
};

export const DocumentHistoryContext = createContext<DocumentHistoryContextValue | null>(null);
